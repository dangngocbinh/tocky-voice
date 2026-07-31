//! The floating recording indicator.
//!
//! This is the window you see while dictating from another app: a small borderless
//! panel showing the level meter and live transcript. The hard requirement is that it
//! must never take keyboard focus — the whole point is that the app you were typing in
//! stays frontmost, so the paste keystroke lands there and not here.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

pub const LABEL: &str = "overlay";

/// Gap between the overlay and the bottom edge of the screen, in physical pixels
/// at 1x — scaled by the monitor's DPI factor below.
const BOTTOM_MARGIN_LOGICAL: f64 = 90.0;

/// Set while onboarding's "try it" step is on screen. That step drives a real take
/// from our own window on purpose, to show recognized text without sending anyone
/// to another app — and it already renders its own level meter and transcript from
/// the same event stream this window listens to. Showing the overlay on top of it
/// would just be the same take on screen twice.
static SUPPRESSED: AtomicBool = AtomicBool::new(false);

pub fn set_suppressed(suppressed: bool) {
    SUPPRESSED.store(suppressed, Ordering::Relaxed);
}

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

/// Shows the overlay without focusing it. Deliberately never calls `set_focus`.
pub fn show(app: &AppHandle) {
    if SUPPRESSED.load(Ordering::Relaxed) {
        return;
    }
    let Some(window) = window(app) else {
        log::warn!("overlay window is missing");
        return;
    };
    position_bottom_center(&window);
    if let Err(e) = window.show() {
        log::warn!("could not show the overlay: {e}");
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = window(app) {
        if let Err(e) = window.hide() {
            log::warn!("could not hide the overlay: {e}");
        }
    }
}

/// Places the overlay near the bottom of whichever monitor currently holds it,
/// so it stays out of the way of the text field being dictated into.
fn position_bottom_center(window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };

    let scale = monitor.scale_factor();
    let area = monitor.size();
    let origin = monitor.position();

    let x = origin.x + ((area.width as i32 - size.width as i32) / 2);
    let y = origin.y + area.height as i32
        - size.height as i32
        - (BOTTOM_MARGIN_LOGICAL * scale) as i32;

    if let Err(e) = window.set_position(PhysicalPosition::new(x, y)) {
        log::debug!("could not position the overlay: {e}");
    }
}
