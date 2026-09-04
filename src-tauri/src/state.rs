//! Shared application state and the event names the frontend subscribes to.

use crate::settings::AppSettings;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub struct AppState {
    pub settings: Mutex<AppSettings>,
}

impl AppState {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            settings: Mutex::new(settings),
        }
    }
}

/// Snapshot of the settings, cloned so callers never hold the lock across an await.
pub fn settings_snapshot(app: &AppHandle) -> AppSettings {
    app.state::<AppState>()
        .settings
        .lock()
        .expect("settings lock poisoned")
        .clone()
}

pub mod events {
    pub const STATUS: &str = "fvt://status";
    pub const PARTIAL: &str = "fvt://partial";
    pub const TRANSCRIPT: &str = "fvt://transcript";
    pub const LEVEL: &str = "fvt://level";
    pub const HISTORY_CHANGED: &str = "fvt://history-changed";
    pub const SETTINGS_CHANGED: &str = "fvt://settings-changed";
    pub const ERROR: &str = "fvt://error";
    /// Read-aloud session state, consumed by the mini player window.
    pub const READ_STATUS: &str = "fvt://read-status";
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Recording,
    Transcribing,
    Refining,
    Pasting,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusPayload {
    pub phase: Phase,
    pub mode_id: String,
    pub mode_name: String,
}

pub fn emit_status(app: &AppHandle, phase: Phase, mode_id: &str) {
    let settings = settings_snapshot(app);
    let mode_name = settings
        .mode(mode_id)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| mode_id.to_string());
    let _ = app.emit(
        events::STATUS,
        StatusPayload {
            phase,
            mode_id: mode_id.to_string(),
            mode_name,
        },
    );
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadPhase {
    /// Selection captured, first chunk still synthesizing — no audio yet.
    Preparing,
    Speaking,
    Paused,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadStatusPayload {
    pub phase: ReadPhase,
    pub mode_name: String,
    pub provider: String,
    pub speed: f32,
    /// The active provider's accepted speed range, so the player's slider cannot offer
    /// a value the vendor would reject (Soniox 0.7–1.3, Vbee up to 1.9).
    pub speed_min: f32,
    pub speed_max: f32,
    pub played_ms: u64,
    pub total_ms: u64,
    /// Set only on `ReadPhase::Failed`, translated client-side like every other error.
    pub error: Option<crate::errors::ErrorPayload>,
    /// The opening of the clipboard text the player is offering to read in place of a
    /// selection it could not capture — `Some` exactly when that offer is standing.
    /// Some apps, terminals above all, never service a synthesized copy, and this is
    /// the one route out of that dead end: the user copies by hand, presses the read
    /// key, and takes the offer.
    ///
    /// A preview rather than a bare flag because the clipboard is where passwords and
    /// private messages live, and this text is about to be sent to a speech vendor and
    /// said out loud. Seeing the first line of it is what makes the button an informed
    /// yes instead of a blind one.
    pub clipboard_preview: Option<String>,
}

pub fn emit_read_status(app: &AppHandle, payload: ReadStatusPayload) {
    let _ = app.emit(events::READ_STATUS, payload);
}

/// Sends a failure to the UI as a kind the frontend can translate.
///
/// The log keeps the English name plus the raw cause, so a bug report is readable no
/// matter which language the person was running the app in.
pub fn emit_error(app: &AppHandle, payload: crate::errors::ErrorPayload) {
    match &payload.detail {
        Some(detail) => log::error!("{:?}: {detail}", payload.kind),
        None => log::error!("{:?}", payload.kind),
    }
    let _ = app.emit(events::ERROR, payload);
}
