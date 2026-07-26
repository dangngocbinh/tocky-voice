//! Remembering and restoring the frontmost application.
//!
//! Pasting sends ⌘V/^V to whatever app is frontmost. Anything that gives our own windows
//! focus mid-dictation — clicking Stop on the overlay, opening Settings — would send
//! the text to us instead. So the target app is captured when recording starts and
//! reactivated just before the keystroke.

/// Where the pasted text should go.
///
/// Deliberately not an `Option<pid>`: "we know there is nowhere to paste" and "this
/// platform does not track focus" need opposite handling, and collapsing them into
/// `None` silently disables pasting everywhere focus tracking is not implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetApp {
    /// Another app was frontmost; bring it back before the keystroke.
    App(i32),
    /// One of our own windows was frontmost, so the text has nowhere to land.
    OurOwnWindow,
    /// Focus is not tracked here; paste into whatever happens to be frontmost.
    Untracked,
}

impl TargetApp {
    /// Whether pasting can deliver the text somewhere the user will see it.
    pub fn can_receive_paste(self) -> bool {
        !matches!(self, TargetApp::OurOwnWindow)
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::TargetApp;
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};

    /// Frontmost app at the moment of the call.
    pub fn capture() -> TargetApp {
        let ours = std::process::id() as i32;
        let Some(frontmost) = (unsafe { NSWorkspace::sharedWorkspace().frontmostApplication() })
        else {
            return TargetApp::Untracked;
        };
        let pid = unsafe { frontmost.processIdentifier() };
        let name = unsafe { frontmost.localizedName() }
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".into());

        if pid == ours {
            // Starting a take from our own window means there is nowhere to paste into.
            // Naming it in the log turns "the paste silently did nothing" into a
            // diagnosable event.
            log::info!("take started from our own window; no paste target");
            return TargetApp::OurOwnWindow;
        }
        log::info!("paste target: {name} (pid {pid})");
        TargetApp::App(pid)
    }

    /// Brings the captured app back to the front. Returns false when it has since quit.
    pub fn restore(pid: i32) -> bool {
        let Some(app) =
            (unsafe { NSRunningApplication::runningApplicationWithProcessIdentifier(pid) })
        else {
            return false;
        };
        // `ignoringOtherApps` is deprecated and inert from macOS 14 on; `allWindows`
        // is the supported way to bring the target app forward.
        unsafe {
            app.activateWithOptions(NSApplicationActivationOptions::NSApplicationActivateAllWindows)
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::TargetApp;

    /// Windows and Linux have no focus tracking here yet. Hiding the overlay before the
    /// keystroke already returns focus to the previous window in practice, so the paste
    /// lands correctly — it is only the "started from our own window" case that goes
    /// undetected.
    pub fn capture() -> TargetApp {
        TargetApp::Untracked
    }

    pub fn restore(_pid: i32) -> bool {
        false
    }
}

pub use platform::capture;

/// Reactivates the target and gives the window server a moment to finish the switch —
/// a keystroke sent immediately after activation can still land on the old app.
pub fn restore_and_settle(target: TargetApp) {
    match target {
        TargetApp::App(pid) => {
            if platform::restore(pid) {
                std::thread::sleep(std::time::Duration::from_millis(120));
                log::info!("reactivated pid {pid} for paste");
            } else {
                log::warn!("target app {pid} is gone; pasting into whatever is frontmost");
            }
        }
        TargetApp::Untracked => {}
        TargetApp::OurOwnWindow => {
            log::warn!("restore called with no paste target; this should not happen")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TargetApp;

    /// Regression: `Untracked` once shared a representation with "nowhere to paste",
    /// which disabled pasting entirely on every platform without focus tracking.
    #[test]
    fn only_our_own_window_blocks_pasting() {
        assert!(TargetApp::App(42).can_receive_paste());
        assert!(TargetApp::Untracked.can_receive_paste());
        assert!(!TargetApp::OurOwnWindow.can_receive_paste());
    }
}
