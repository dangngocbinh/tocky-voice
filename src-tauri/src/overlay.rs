//! The floating recording indicator.
//!
//! This is the window you see while dictating from another app: a small borderless
//! panel showing the level meter and live transcript. The hard requirement is that it
//! must never take keyboard focus — the whole point is that the app you were typing in
//! stays frontmost, so the paste keystroke lands there and not here.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow};

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

/// Places the overlay near the bottom of the monitor containing the pointer on
/// Windows and macOS. Other platforms keep the existing current-monitor behavior.
fn position_bottom_center(window: &WebviewWindow) {
    let monitor = monitor_for_pointer(window).or_else(|| window.current_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };

    let scale = monitor.scale_factor();
    let current_scale = window.scale_factor().unwrap_or(scale);
    let size = size_on_monitor(&size, current_scale, scale);
    let area = monitor.size();
    let origin = monitor.position();

    let x = origin.x + ((area.width as i32 - size.width as i32) / 2);
    let y =
        origin.y + area.height as i32 - size.height as i32 - (BOTTOM_MARGIN_LOGICAL * scale) as i32;

    if let Err(e) = window.set_position(PhysicalPosition::new(x, y)) {
        log::debug!("could not position the overlay: {e}");
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn monitor_for_pointer(window: &WebviewWindow) -> Option<Monitor> {
    window.cursor_position().ok().and_then(|cursor| {
        window.available_monitors().ok().and_then(|monitors| {
            monitors
                .into_iter()
                .find(|monitor| monitor_contains_point(monitor.position(), monitor.size(), &cursor))
        })
    })
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn monitor_for_pointer(_window: &WebviewWindow) -> Option<Monitor> {
    None
}

fn size_on_monitor(
    size: &PhysicalSize<u32>,
    current_scale: f64,
    target_scale: f64,
) -> PhysicalSize<u32> {
    size.to_logical::<f64>(current_scale)
        .to_physical::<u32>(target_scale)
}

fn monitor_contains_point(
    origin: &PhysicalPosition<i32>,
    size: &PhysicalSize<u32>,
    point: &PhysicalPosition<f64>,
) -> bool {
    point.x >= origin.x as f64
        && point.x < origin.x as f64 + size.width as f64
        && point.y >= origin.y as f64
        && point.y < origin.y as f64 + size.height as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_monitor_above_the_primary_display() {
        let primary = PhysicalPosition::new(0, 0);
        let upper = PhysicalPosition::new(0, -1620);
        let primary_size = PhysicalSize::new(2560, 1600);
        let upper_size = PhysicalSize::new(2880, 1620);
        let pointer = PhysicalPosition::new(1554.0, -342.0);

        assert!(!monitor_contains_point(&primary, &primary_size, &pointer));
        assert!(monitor_contains_point(&upper, &upper_size, &pointer));
    }

    #[test]
    fn preserves_logical_size_across_monitor_scales() {
        let size = PhysicalSize::new(480, 212);
        let target_size = size_on_monitor(&size, 1.0, 2.0);

        assert_eq!(target_size, PhysicalSize::new(960, 424));
    }
}
