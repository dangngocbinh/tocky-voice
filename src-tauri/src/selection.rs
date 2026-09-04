//! Grabbing whatever text is highlighted in whatever app currently has focus.
//!
//! There is no cross-platform "give me the selection" API that survives contact with
//! Chrome, Electron and VS Code (their accessibility trees don't expose `AXSelectedText`
//! reliably), so this goes through the clipboard instead — the one channel every app on
//! every platform already supports.
//!
//! The trick is telling "nothing is selected" apart from "the target app ignored our
//! Cmd/Ctrl+C". Firing the shortcut and reading the clipboard back is not enough on its
//! own: if the app never copies anything, the clipboard still holds whatever the user
//! copied yesterday, and reading that back would read someone else's text out loud.
//! So the clipboard is cleared first — an empty clipboard afterwards can only mean the
//! copy never landed, never "the user selected nothing new".

use crate::inject;
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

const POLL_INTERVAL: Duration = Duration::from_millis(20);
/// Terminals and Electron apps are slower to service a copy than a native text field;
/// the poll returns as soon as content lands, so a longer ceiling costs nothing in the
/// common case and rescues the slow ones.
const POLL_TIMEOUT: Duration = Duration::from_millis(600);

/// Matches `inject::CLIPBOARD_SETTLE` — the gap an app needs to notice the pasteboard
/// changed underneath it.
const CLIPBOARD_SETTLE: Duration = Duration::from_millis(90);

/// Same delay `inject::paste` uses before restoring the clipboard — long enough for a
/// slow target app to finish reading it, short enough not to be noticeable.
const RESTORE_DELAY: Duration = Duration::from_millis(400);

#[derive(Debug)]
pub enum SelectionError {
    /// Nothing was highlighted anywhere, or the target app ignored the copy shortcut.
    ///
    /// Carries whatever was already on the clipboard when the attempt began. Some apps
    /// (iTerm2 and other terminals most of all) simply will not service a synthesized
    /// copy, and for those the user's own Cmd/Ctrl+C is the only way text ever gets out
    /// — so the caller can offer to read that instead of dead-ending. `None` when the
    /// clipboard was empty or unreadable, i.e. when there is nothing to offer.
    NothingSelected { clipboard: Option<String> },
    /// macOS will not let us synthesize the copy keystroke.
    NeedsAccessibility,
    Clipboard(anyhow::Error),
}

/// The text currently highlighted in whichever app has focus.
///
/// Never passes stale clipboard content off as a selection: the clipboard is blanked
/// before the copy shortcut fires, so a clipboard that is still empty afterwards can
/// only mean nothing was selected. The old contents do come back on the failure path —
/// see [`SelectionError::NothingSelected`] — but as something to be offered and
/// consented to, never as the `Ok` the caller would speak without asking. The user's
/// previous clipboard contents are always restored, on every return path, including the
/// error ones.
pub fn capture(app: &AppHandle) -> Result<String, SelectionError> {
    if !inject::can_synthesize_input() {
        return Err(SelectionError::NeedsAccessibility);
    }

    // The trigger for this is a hotkey, and a hotkey fires on key-*down* — the user is
    // very likely still holding ⌘⇧ when we get here. Those held modifiers combine with
    // the keystroke synthesized below, so what the target app receives is ⌘⇧C, not ⌘C,
    // and nothing is ever copied. `inject::paste` never hit this because a paste
    // happens seconds later, after a whole dictation.
    //
    // Waiting is not enough on its own: a user who simply keeps the hotkey down (or a
    // key that repeats) still has it held when the wait gives up, and the copy is then
    // lost for exactly the reason above. Measured against a real iTerm2 selection:
    // plain ⌘C copied 1410 bytes, the same ⌘C under a held ⌥⇧ copied nothing. So on
    // timeout the modifiers are cleared outright rather than hoped away.
    if !wait_for_modifier_release() {
        if let Err(e) = inject::release_modifiers(app) {
            log::debug!("could not clear the held modifiers before copying: {e:#}");
        }
        std::thread::sleep(MODIFIER_SETTLE);
    }

    let previous = app.clipboard().read_text().ok();

    // Clearing first is what stops `poll_clipboard` from reading back whatever was
    // already there and treating it as the selection — but only ever when there is
    // text there to be mistaken for one. A clipboard that would not read as text is
    // holding an image or a file, in flavours `write_text` destroys and nothing here
    // can put back, and no amount of it can be confused for a copy that never landed.
    // So it is left alone: taking someone's screenshot away because they pressed the
    // read hotkey is a far worse trade than the discrimination it buys.
    //
    // An empty string, not an unset clipboard: some platforms report "empty" as `Err`
    // rather than `Ok("")`, and normalizing to an explicit write means the poll below
    // only has to check one shape either way.
    if previous.is_some() {
        if let Err(e) = app.clipboard().write_text(String::new()) {
            return Err(SelectionError::Clipboard(anyhow::anyhow!(
                "could not clear the clipboard before copying: {e}"
            )));
        }

        // Same reason `inject::paste` waits after writing the clipboard: apps observe
        // the pasteboard asynchronously, and a copy fired the instant after a clear can
        // race the clear itself.
        std::thread::sleep(CLIPBOARD_SETTLE);
    }

    // The pasteboard's changeCount rises whenever *anything* writes to it, so comparing
    // it across the keystroke separates "the target app ignored the copy" from "it
    // copied and we failed to read it back" — a distinction the clipboard contents
    // alone cannot make, and the one that decides where to look when this fails.
    let before_copy = pasteboard_change_count();
    log::debug!("copying from {}", frontmost_app_name());

    if let Err(e) = inject::send_shortcut(app, 'c') {
        restore_clipboard(app, previous);
        return Err(SelectionError::Clipboard(e));
    }

    let after_copy = poll_clipboard(app);
    log::debug!(
        "pasteboard changeCount {before_copy} -> {} ({})",
        pasteboard_change_count(),
        if pasteboard_change_count() > before_copy { "the app copied" } else { "no copy landed" },
    );
    // Length only, never content: this can be a password or a private message. The
    // counts are what distinguishes "the copy never landed" from "it landed and the
    // decision below rejected it", which is otherwise unknowable from a bug report.
    log::info!(
        "selection capture: {} chars after copy (previous clipboard: {} chars)",
        after_copy.as_deref().map(str::len).unwrap_or(0),
        previous.as_deref().map(str::len).unwrap_or(0),
    );
    // Cloned before `previous` is handed to the restore thread: on the failure path it
    // becomes the fallback the user is offered, and the restore has to keep happening
    // exactly as before either way.
    let fallback = previous.clone();
    restore_clipboard(app, previous);

    decide(after_copy, fallback)
}

#[cfg(target_os = "macos")]
fn pasteboard_change_count() -> isize {
    use objc2_app_kit::NSPasteboard;
    unsafe { NSPasteboard::generalPasteboard().changeCount() }
}

#[cfg(not(target_os = "macos"))]
fn pasteboard_change_count() -> isize {
    -1
}

/// Whichever app the copy keystroke will actually reach — not necessarily the one
/// captured as the paste target, and the difference is worth seeing when a copy comes
/// back empty.
#[cfg(target_os = "macos")]
fn frontmost_app_name() -> String {
    use objc2_app_kit::NSWorkspace;
    unsafe { NSWorkspace::sharedWorkspace().frontmostApplication() }
        .and_then(|app| unsafe { app.localizedName() })
        .map(|name| name.to_string())
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(not(target_os = "macos"))]
fn frontmost_app_name() -> String {
    "unknown".into()
}

/// How long to wait for the user to let go of the hotkey before synthesizing the copy.
/// Generous: a slow release is common, and the cost of waiting is invisible next to the
/// cost of the copy silently not happening.
const MODIFIER_RELEASE_TIMEOUT: Duration = Duration::from_millis(600);
const MODIFIER_POLL: Duration = Duration::from_millis(15);

/// Blocks until no modifier key is physically held, or [`MODIFIER_RELEASE_TIMEOUT`]
/// passes. See the call site for why this matters.
///
/// Returns whether the modifiers actually came up — `false` means the caller has to
/// clear them itself before synthesizing anything.
#[cfg(target_os = "macos")]
fn wait_for_modifier_release() -> bool {
    use objc2_app_kit::{NSEvent, NSEventModifierFlags};

    // Only the four that change what a letter key means. Caps Lock and the numeric
    // keypad flag are ignored: they are states a user can sit in indefinitely, and
    // waiting for those to clear would just burn the whole timeout every time.
    let blocking = NSEventModifierFlags::NSEventModifierFlagCommand
        | NSEventModifierFlags::NSEventModifierFlagShift
        | NSEventModifierFlags::NSEventModifierFlagControl
        | NSEventModifierFlags::NSEventModifierFlagOption;

    let deadline = std::time::Instant::now() + MODIFIER_RELEASE_TIMEOUT;
    let started = std::time::Instant::now();
    loop {
        let held = unsafe { NSEvent::modifierFlags_class() };
        if !held.intersects(blocking) {
            log::debug!("modifiers clear after {:?}", started.elapsed());
            // A beat past the release: the window server delivers the key-up
            // asynchronously, and a copy synthesized in the same instant can still be
            // seen by the target app as modified.
            std::thread::sleep(MODIFIER_SETTLE);
            return true;
        }
        if std::time::Instant::now() >= deadline {
            log::debug!("modifiers still held after {MODIFIER_RELEASE_TIMEOUT:?}; clearing them");
            return false;
        }
        std::thread::sleep(MODIFIER_POLL);
    }
}

/// Elsewhere there is no cheap way to read the live modifier state, so this is a plain
/// pause — long enough to cover an ordinary key release. Reported as a success either
/// way: without a reading there is nothing to say the modifiers are still down, and
/// synthesizing a blind release of every modifier has its own cost on Windows, where a
/// stray Meta key-up pops the Start menu.
#[cfg(not(target_os = "macos"))]
fn wait_for_modifier_release() -> bool {
    std::thread::sleep(Duration::from_millis(120));
    true
}

const MODIFIER_SETTLE: Duration = Duration::from_millis(40);

/// Reads the clipboard every [`POLL_INTERVAL`] until it holds non-empty text or
/// [`POLL_TIMEOUT`] elapses. Most apps land the copy well under the deadline, so this
/// returns as soon as it sees content rather than always waiting the full window.
fn poll_clipboard(app: &AppHandle) -> Option<String> {
    let deadline = std::time::Instant::now() + POLL_TIMEOUT;
    loop {
        // `Ok("")` and `Err` both mean "nothing there yet" — see the comment on `decide`.
        if let Ok(text) = app.clipboard().read_text() {
            if !text.trim().is_empty() {
                return Some(text);
            }
        }
        if std::time::Instant::now() >= deadline {
            return app.clipboard().read_text().ok();
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

/// Puts the user's previous clipboard content back on a detached thread, mirroring
/// `inject::paste` — the caller must not block on the target app finishing its read,
/// and this must run on every path out of `capture`, including the error ones.
fn restore_clipboard(app: &AppHandle, previous: Option<String>) {
    let Some(previous) = previous else { return };
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(RESTORE_DELAY);
        if let Err(e) = app.clipboard().write_text(previous) {
            log::debug!("could not restore the previous clipboard after selection capture: {e}");
        }
    });
}

/// The decision logic, pulled out of the I/O so it can be tested without a real
/// clipboard: `None` or all-whitespace both mean the copy never produced anything.
///
/// `clipboard` is what was on the clipboard before the attempt; it rides along on the
/// failure so the caller can offer it, and is dropped when it holds nothing worth
/// reading.
fn decide(
    after_copy: Option<String>,
    clipboard: Option<String>,
) -> Result<String, SelectionError> {
    match after_copy {
        Some(text) if !text.trim().is_empty() => Ok(text.trim().to_string()),
        _ => Err(SelectionError::NothingSelected {
            clipboard: clipboard
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_means_nothing_selected() {
        assert!(matches!(
            decide(None, None),
            Err(SelectionError::NothingSelected { clipboard: None })
        ));
    }

    #[test]
    fn empty_string_means_nothing_selected() {
        assert!(matches!(
            decide(Some(String::new()), None),
            Err(SelectionError::NothingSelected { .. })
        ));
    }

    #[test]
    fn whitespace_only_means_nothing_selected() {
        assert!(matches!(
            decide(Some("   \n\t  ".to_string()), None),
            Err(SelectionError::NothingSelected { .. })
        ));
    }

    #[test]
    fn real_text_is_trimmed_and_returned() {
        assert_eq!(decide(Some("  xin chào  ".to_string()), None).unwrap(), "xin chào");
    }

    #[test]
    fn real_text_with_no_surrounding_whitespace_is_unchanged() {
        assert_eq!(
            decide(Some("hello world".to_string()), None).unwrap(),
            "hello world"
        );
    }

    #[test]
    fn a_failed_copy_carries_the_existing_clipboard_as_the_fallback() {
        let Err(SelectionError::NothingSelected { clipboard }) =
            decide(None, Some("  đã copy trước đó  ".to_string()))
        else {
            panic!("expected NothingSelected");
        };
        assert_eq!(clipboard.as_deref(), Some("đã copy trước đó"));
    }

    #[test]
    fn a_blank_clipboard_is_not_offered_as_a_fallback() {
        let Err(SelectionError::NothingSelected { clipboard }) =
            decide(None, Some("   \n ".to_string()))
        else {
            panic!("expected NothingSelected");
        };
        assert!(clipboard.is_none());
    }
}
