//! Delivering text into whatever app has focus.
//!
//! Pasting via the clipboard is the only approach that is fast and reliable across
//! apps (typing character-by-character breaks on Vietnamese diacritics and is slow),
//! so the previous clipboard contents are saved and restored afterwards to keep the
//! user's own copy buffer intact.

use anyhow::{Context, Result};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Give the target app time to observe the new clipboard before the keystroke, and
/// again before restoring — pasting reads the clipboard asynchronously in most apps.
const CLIPBOARD_SETTLE: std::time::Duration = std::time::Duration::from_millis(90);
const RESTORE_DELAY: std::time::Duration = std::time::Duration::from_millis(400);

/// The system pasteboard is a single shared resource, and a write fails outright while
/// another process holds it — clipboard managers and browsers grab it briefly and often.
/// The failure is transient and the retry is cheap, whereas losing the write means the
/// dictated text never reaches the user at all.
const COPY_ATTEMPTS: usize = 3;
const COPY_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(60);

pub fn copy(app: &AppHandle, text: &str) -> Result<()> {
    let mut last = None;
    for attempt in 1..=COPY_ATTEMPTS {
        match app.clipboard().write_text(text.to_string()) {
            Ok(()) => return Ok(()),
            Err(e) => {
                log::warn!("clipboard write failed (attempt {attempt}/{COPY_ATTEMPTS}): {e}");
                last = Some(e);
                if attempt < COPY_ATTEMPTS {
                    std::thread::sleep(COPY_RETRY_DELAY);
                }
            }
        }
    }
    Err(last.expect("at least one attempt was made")).context("writing to clipboard")
}

/// Copies `text` and synthesizes the paste shortcut. Requires Accessibility
/// permission on macOS; without it the keystroke is silently swallowed by the OS,
/// which is why the caller surfaces a permission hint when nothing appears.
pub fn paste(app: &AppHandle, text: &str) -> Result<()> {
    let previous = app.clipboard().read_text().ok();

    copy(app, text)?;
    std::thread::sleep(CLIPBOARD_SETTLE);
    send_paste_keystroke(app)?;

    // Restore on a detached thread so the caller isn't blocked waiting for the target
    // app to finish reading the clipboard.
    if let Some(previous) = previous {
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(RESTORE_DELAY);
            if let Err(e) = app.clipboard().write_text(previous) {
                log::debug!("could not restore previous clipboard: {e}");
            }
        });
    }
    Ok(())
}

/// How long to wait for the main thread to run the keystroke before giving up.
const MAIN_THREAD_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Synthesizes the paste shortcut **on the main thread**.
///
/// This must not run on a worker thread. Resolving the key for 'v' goes through
/// HIToolbox's Text Services Manager to honour the active keyboard layout, and that
/// API calls `dispatch_assert_queue(main)` — off the main thread macOS raises
/// SIGTRAP and the whole app dies rather than returning an error.
fn send_paste_keystroke(app: &AppHandle) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = tx.send(synthesize_paste());
    })
    .context("dispatching the paste keystroke to the main thread")?;

    rx.recv_timeout(MAIN_THREAD_TIMEOUT)
        .context("timed out waiting for the main thread to send the paste keystroke")?
}

fn synthesize_paste() -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("initializing input synthesis")?;

    #[cfg(target_os = "macos")]
    let modifier = Key::Meta; // Command
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    enigo
        .key(modifier, Direction::Press)
        .context("pressing paste modifier")?;
    let result = enigo.key(Key::Unicode('v'), Direction::Click);
    // Always release the modifier, even if the 'v' press failed — a stuck modifier
    // would leave the whole system in a bad state.
    let release = enigo.key(modifier, Direction::Release);
    result.context("sending paste keystroke")?;
    release.context("releasing paste modifier")?;
    Ok(())
}

/// Whether the OS will let us synthesize keystrokes. On macOS this is the
/// Accessibility permission; elsewhere it is always available.
///
/// The answer is the same on any thread — measured, not assumed. What it is *not* the
/// same across is how the app was launched: a build started from a terminal inherits
/// the terminal's trust, so it reports `true` while the same bundle opened from the
/// Dock reports `false`. Only the Dock answer is the real one.
pub fn can_synthesize_input() -> bool {
    #[cfg(target_os = "macos")]
    {
        crate::hotkeys::macos_ptt::has_accessibility_permission()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}
