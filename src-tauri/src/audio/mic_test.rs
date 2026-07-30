//! Microphone preview for the setup wizard.
//!
//! Opening the microphone is the one step of setup that cannot be verified by reading
//! a checkbox: the device list happily shows an input that is muted at the OS level,
//! disabled in Windows privacy settings, or simply not the one being spoken into. This
//! runs capture on its own, with no speech provider and no session, purely so the level
//! meter can prove sound is arriving before the user spends a key on finding out.

use super::capture::{self, CaptureChunk, CaptureHandle};
use crate::state::events;
use anyhow::Result;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

/// The preview stream, if one is running. At most one at a time.
#[derive(Default)]
pub struct MicTest {
    handle: Mutex<Option<CaptureHandle>>,
}

/// Opens `device` and starts emitting level events. Replaces any preview already
/// running, so switching device in the picker is a single call.
pub fn start(app: &AppHandle, device: Option<String>) -> Result<()> {
    stop(app);

    let (tx, mut rx) = mpsc::unbounded_channel::<CaptureChunk>();
    let handle = capture::start(device, tx)?;

    let emitter = app.clone();
    tauri::async_runtime::spawn(async move {
        // Same throttle as a real take: ~10 updates/sec is plenty for a meter.
        let mut since_last_level = 0u8;
        while let Some(chunk) = rx.recv().await {
            since_last_level += 1;
            if since_last_level >= 3 {
                since_last_level = 0;
                let _ = emitter.emit(events::LEVEL, chunk.peak);
            }
        }
    });

    if let Ok(mut slot) = app_state(app).handle.lock() {
        *slot = Some(handle);
    }
    Ok(())
}

/// Closes the preview. Safe to call when nothing is running — which is why every path
/// that opens the microphone for real can call it unconditionally first.
pub fn stop(app: &AppHandle) {
    if let Ok(mut slot) = app_state(app).handle.lock() {
        // Dropping the handle stops the capture thread and closes the device.
        slot.take();
    }
}

fn app_state(app: &AppHandle) -> tauri::State<'_, MicTest> {
    app.state::<MicTest>()
}
