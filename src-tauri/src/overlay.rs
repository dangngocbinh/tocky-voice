//! The floating recording indicator.
//!
//! This is the window you see while dictating from another app: a small borderless
//! panel showing the level meter and live transcript. The hard requirement is that it
//! must never take keyboard focus — the whole point is that the app you were typing in
//! stays frontmost, so the paste keystroke lands there and not here.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

pub const LABEL: &str = "overlay";

/// Gap between the visible *panel* and the bottom edge of the screen, in logical
/// pixels — scaled by the monitor's DPI factor below.
const PANEL_BOTTOM_GAP_LOGICAL: f64 = 97.0;

/// Transparent margin the window carries below the panel so the panel's shadow has
/// somewhere to fade out instead of being sliced off at the window edge. Mirrors
/// `--panel-gutter-bottom` in `styles.css`; subtracted from the gap above so growing
/// the window for the shadow's sake does not move the panel up the screen.
const SHADOW_GUTTER_BOTTOM_LOGICAL: f64 = 36.0;

/// Set while onboarding's "try it" step is on screen. That step drives a real take
/// from our own window on purpose, to show recognized text without sending anyone
/// to another app — and it already renders its own level meter and transcript from
/// the same event stream this window listens to. Showing the overlay on top of it
/// would just be the same take on screen twice.
static SUPPRESSED: AtomicBool = AtomicBool::new(false);

pub fn set_suppressed(suppressed: bool) {
    SUPPRESSED.store(suppressed, Ordering::Relaxed);
}

/// Bumped every time the panel is shown.
///
/// Whoever schedules a delayed `hide` captures this first and hides only if it still
/// matches, so a timer left over from a failure cannot blank the panel of the take the
/// user started in the meantime — the likeliest thing for them to do after being told
/// their last take was cut short.
static GENERATION: AtomicU64 = AtomicU64::new(0);

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

/// Hides the panel only if it is still the same appearance `generation` referred to.
pub fn hide_if_unchanged(app: &AppHandle, generation: u64) {
    if GENERATION.load(Ordering::Relaxed) == generation {
        hide(app);
    }
}

/// Shows the overlay without focusing it. Deliberately never calls `set_focus`.
///
/// Returns this appearance's [`GENERATION`], for callers that want to hide it later
/// without stepping on a panel that has since been shown again.
pub fn show(app: &AppHandle) -> u64 {
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    if SUPPRESSED.load(Ordering::Relaxed) {
        return generation;
    }
    let Some(window) = window(app) else {
        log::warn!("overlay window is missing");
        return generation;
    };
    position_bottom_center(&window);
    if let Err(e) = window.show() {
        log::warn!("could not show the overlay: {e}");
    }
    generation
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
    let window_margin = PANEL_BOTTOM_GAP_LOGICAL - SHADOW_GUTTER_BOTTOM_LOGICAL;
    let y = origin.y + area.height as i32 - size.height as i32 - (window_margin * scale) as i32;

    if let Err(e) = window.set_position(PhysicalPosition::new(x, y)) {
        log::debug!("could not position the overlay: {e}");
    }
}
