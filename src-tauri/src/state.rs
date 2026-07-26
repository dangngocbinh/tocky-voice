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
