//! The floating read-aloud control bar: 360x56, draggable, shown only while a read
//! session is active. Same non-focus-stealing shape as `overlay.rs`, but **top**-right
//! of whichever monitor holds the mouse rather than bottom-center. The bottom of the
//! screen is taken twice over: the dictation overlay sits bottom-center, and the Dock
//! covers bottom-right on a default macOS setup — a control bar hidden behind the Dock
//! is a control bar you cannot press.

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

pub const LABEL: &str = "player";

/// Gap between the visible *panel* and the top/right edges of the screen, logical px.
const MARGIN: f64 = 24.0;

/// Transparent margins the window carries around the panel so its shadow can fade out
/// rather than end at the window edge. Mirror `--panel-gutter-x` / `--panel-gutter-top`
/// in `styles.css`, and are subtracted from [`MARGIN`] so the panel keeps its place.
const SHADOW_GUTTER_X: f64 = 26.0;
const SHADOW_GUTTER_TOP: f64 = 16.0;

/// Height to leave clear at the top of the screen. Covers the macOS menu bar (and the
/// notch on the machines that have one) with room to spare; harmless elsewhere, where
/// it just reads as a slightly larger top margin.
#[cfg(target_os = "macos")]
const MENU_BAR_ALLOWANCE: f64 = 32.0;
#[cfg(not(target_os = "macos"))]
const MENU_BAR_ALLOWANCE: f64 = 0.0;

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

/// Shows the player at `saved` if that position still lands on a connected monitor, or
/// at the default bottom-right spot otherwise. Never focuses the window — see
/// `overlay::show` for why that matters for a window that sits over an app you're
/// reading, not typing into, so there is nothing here to protect from a stray
/// keystroke, but the same "must not steal focus" rule applies for the same reason:
/// stealing focus here would break arrow-key scrolling in whatever the user was
/// reading.
pub fn show(app: &AppHandle, saved: Option<(f64, f64)>) {
    let Some(window) = window(app) else {
        log::warn!("player window is missing");
        return;
    };

    let position = saved
        .filter(|&(x, y)| is_position_visible(&window, x, y))
        .or_else(|| default_position(&window));

    if let Some((x, y)) = position {
        if let Err(e) = window.set_position(PhysicalPosition::new(x, y)) {
            log::debug!("could not position the read-aloud player: {e}");
        }
    }

    // Re-asserted on every show, not just trusted from tauri.conf.json. The config
    // flag only sets the level at creation; a window that has been sitting hidden
    // while other apps came and went can still end up behind them, which is fatal for
    // a control bar whose whole job is to be reachable over the thing you are reading.
    if let Err(e) = window.set_always_on_top(true) {
        log::debug!("could not pin the read-aloud player on top: {e}");
    }
    // Above full-screen apps and other utility windows too, not merely above normal
    // ones — the reading being narrated is often exactly a full-screen document.
    #[cfg(target_os = "macos")]
    if let Err(e) = window.set_visible_on_all_workspaces(true) {
        log::debug!("could not make the read-aloud player follow spaces: {e}");
    }

    if let Err(e) = window.show() {
        log::warn!("could not show the read-aloud player: {e}");
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = window(app) {
        if let Err(e) = window.hide() {
            log::warn!("could not hide the read-aloud player: {e}");
        }
    }
}

/// Top-right of the monitor currently holding the mouse cursor, clear of the menu bar.
/// Falls back to the primary monitor when the cursor position or its monitor can't be
/// resolved.
fn default_position(window: &WebviewWindow) -> Option<(f64, f64)> {
    let cursor = window.cursor_position().ok();
    let monitor = cursor
        .and_then(|c| {
            window
                .available_monitors()
                .ok()?
                .into_iter()
                .find(|m| point_in_monitor(m, c.x, c.y))
        })
        .or_else(|| window.primary_monitor().ok().flatten())?;

    let size = window.outer_size().ok()?;
    let scale = monitor.scale_factor();
    let area = monitor.size();
    let origin = monitor.position();
    let x = origin.x + area.width as i32
        - size.width as i32
        - ((MARGIN - SHADOW_GUTTER_X) * scale) as i32;
    // `monitor.size()` is the whole panel including the menu bar, so the top margin has
    // to clear it as well — otherwise the bar lands underneath and only its lower half
    // is clickable.
    let y = origin.y
        + ((MARGIN - SHADOW_GUTTER_TOP + MENU_BAR_ALLOWANCE) * scale) as i32;
    Some((x as f64, y as f64))
}

fn point_in_monitor(monitor: &tauri::Monitor, x: f64, y: f64) -> bool {
    let origin = monitor.position();
    let size = monitor.size();
    x >= origin.x as f64
        && x <= (origin.x as f64 + size.width as f64)
        && y >= origin.y as f64
        && y <= (origin.y as f64 + size.height as f64)
}

/// A position saved before a monitor was unplugged (or on a machine that has since
/// changed its display layout) can point at empty space — a window there is
/// technically "shown" and practically invisible. Checked against the current monitor
/// list rather than trusted blindly.
fn is_position_visible(window: &WebviewWindow, x: f64, y: f64) -> bool {
    let Ok(monitors) = window.available_monitors() else {
        return false;
    };
    monitors.iter().any(|m| point_in_monitor(m, x, y))
}
