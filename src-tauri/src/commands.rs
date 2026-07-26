//! The IPC surface exposed to the frontend.
//!
//! Commands return `Result<_, String>` because Tauri needs a serializable error;
//! anyhow's chain is flattened with `{:#}` so the UI can show the root cause.

use crate::audio::capture;
use crate::history::{self, HistoryEntry};
use crate::settings::{self, defaults, secrets, AppSettings};
use crate::state::{events, AppState};
use crate::{hotkeys, inject, session};
use serde::Serialize;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Manager};

fn to_err(e: anyhow::Error) -> String {
    format!("{e:#}")
}

#[tauri::command]
pub fn get_settings(state: tauri::State<AppState>) -> AppSettings {
    state.settings.lock().expect("settings lock").clone()
}

/// Persists settings and re-applies anything derived from them (hotkeys, autostart)
/// so a save takes effect immediately without a restart.
#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: tauri::State<AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    // Move any stored keys before the new preference takes effect, otherwise flipping
    // the backend looks like every key disappeared.
    let accounts = settings::all_secret_accounts();
    if let Err(e) = secrets::migrate_to(settings.use_os_keychain, &accounts) {
        log::warn!("credential migration failed: {e:#}");
    }

    settings::save(&app, &settings).map_err(to_err)?;
    apply_autostart(&app, settings.autostart);
    *state.settings.lock().expect("settings lock") = settings.clone();
    hotkeys::apply(&app, &settings);
    crate::tray::apply_language(&app, &settings);
    let _ = app.emit(events::SETTINGS_CHANGED, ());
    Ok(())
}

#[tauri::command]
pub fn reset_settings(app: AppHandle, state: tauri::State<AppState>) -> Result<AppSettings, String> {
    let fresh = defaults::default_settings();
    settings::save(&app, &fresh).map_err(to_err)?;
    *state.settings.lock().expect("settings lock") = fresh.clone();
    hotkeys::apply(&app, &fresh);
    let _ = app.emit(events::SETTINGS_CHANGED, ());
    Ok(fresh)
}

#[derive(Serialize)]
pub struct PresetDto {
    pub id: &'static str,
    pub label: &'static str,
    pub default_model: &'static str,
    pub models: &'static [&'static str],
    pub secret_key: &'static str,
    pub needs_key: bool,
    pub base_url: &'static str,
    pub signup_url: &'static str,
}

#[tauri::command]
pub fn list_llm_presets() -> Vec<PresetDto> {
    defaults::llm_presets()
        .iter()
        .map(|p| PresetDto {
            id: p.id,
            label: p.label,
            default_model: p.default_model,
            models: p.models,
            secret_key: p.secret_key,
            needs_key: p.needs_key,
            base_url: p.base_url,
            signup_url: p.signup_url,
        })
        .collect()
}

#[tauri::command]
pub fn list_input_devices() -> Vec<String> {
    capture::list_input_devices()
}

/// Writes a credential to the OS keychain. There is deliberately no "read key"
/// command — the UI only ever learns whether a key exists, via [`key_status`].
#[tauri::command]
pub fn set_api_key(account: String, value: String) -> Result<(), String> {
    secrets::set_key(&account, &value).map_err(to_err)
}

#[tauri::command]
pub fn delete_api_key(account: String) -> Result<(), String> {
    secrets::delete_key(&account).map_err(to_err)
}

/// Map of credential name → configured, for every provider the UI can show.
#[tauri::command]
pub fn key_status() -> HashMap<String, bool> {
    let mut status = HashMap::new();
    for account in ["soniox", "deepgram", "assemblyai"] {
        status.insert(account.to_string(), secrets::has_key(account));
    }
    for preset in defaults::llm_presets() {
        status.insert(
            preset.secret_key.to_string(),
            !preset.needs_key || secrets::has_key(preset.secret_key),
        );
    }
    status
}

#[tauri::command]
pub fn start_recording(app: AppHandle, mode_id: Option<String>) {
    session::start(&app, mode_id);
}

#[tauri::command]
pub fn stop_recording(app: AppHandle) {
    session::stop(&app);
}

#[tauri::command]
pub fn cancel_recording(app: AppHandle) {
    session::cancel(&app);
}

#[tauri::command]
pub fn toggle_recording(app: AppHandle) {
    session::toggle(&app);
}

#[tauri::command]
pub fn set_active_mode(app: AppHandle, mode_id: String) {
    session::set_active_mode(&app, &mode_id);
}

#[tauri::command]
pub fn get_history(app: AppHandle) -> Vec<HistoryEntry> {
    history::load(&app)
}

#[tauri::command]
pub fn delete_history_entry(app: AppHandle, id: String) -> Result<(), String> {
    history::delete(&app, &id).map_err(to_err)?;
    let _ = app.emit(events::HISTORY_CHANGED, ());
    Ok(())
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    history::clear(&app).map_err(to_err)?;
    let _ = app.emit(events::HISTORY_CHANGED, ());
    Ok(())
}

#[tauri::command]
pub fn copy_text(app: AppHandle, text: String) -> Result<(), String> {
    inject::copy(&app, &text).map_err(to_err)
}

#[derive(Serialize)]
pub struct PermissionStatus {
    /// Needed both for the paste keystroke and for hold-a-modifier push-to-talk.
    pub accessibility: bool,
}

#[tauri::command]
pub fn permission_status() -> PermissionStatus {
    PermissionStatus {
        accessibility: inject::can_synthesize_input(),
    }
}

/// Opens an external link in the default browser — provider signup pages, the author's
/// site. Restricted to https so a compromised webview cannot hand the OS a `file://`
/// or a custom scheme to launch.
#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    if !url.starts_with("https://") {
        return Err(format!("refusing to open a non-https link: {url}"));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Opens the OS pane where the user grants Accessibility access.
#[tauri::command]
pub fn open_accessibility_settings(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_opener::OpenerExt;
        app.opener()
            .open_url(
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
                None::<&str>,
            )
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        Ok(())
    }
}

/// Round-trips a short prompt through the configured LLM so key and model problems
/// surface in Settings rather than mid-dictation.
#[tauri::command]
pub async fn test_llm(app: AppHandle) -> Result<String, String> {
    let settings = crate::state::settings_snapshot(&app);
    let llm = settings.llm.clone();
    let api_key = defaults::preset(&llm.preset)
        .filter(|p| p.needs_key)
        .and_then(|p| secrets::get_key(p.secret_key));

    crate::refine::refine(crate::refine::RefineRequest {
        system_prompt: "Reply with exactly: OK".into(),
        transcript: "ping".into(),
        llm,
        api_key,
    })
    .await
    .map_err(to_err)
}

/// Releases the global hotkeys so the settings UI can capture a key combination
/// without the app reacting to it.
#[tauri::command]
pub fn suspend_hotkeys(app: AppHandle) {
    hotkeys::suspend(&app);
}

/// Re-binds the hotkeys from the current settings once recording is over.
#[tauri::command]
pub fn resume_hotkeys(app: AppHandle, state: tauri::State<AppState>) {
    let settings = state.settings.lock().expect("settings lock").clone();
    hotkeys::apply(&app, &settings);
}

/// Fetches the provider's current model list. Falls back to the preset's built-in list
/// on the frontend when this fails (no key yet, offline, provider without the endpoint).
#[tauri::command]
pub async fn list_models(app: AppHandle) -> Result<Vec<String>, String> {
    let settings = crate::state::settings_snapshot(&app);
    let llm = settings.llm.clone();
    let api_key = defaults::preset(&llm.preset)
        .filter(|p| p.needs_key)
        .and_then(|p| secrets::get_key(p.secret_key));

    crate::refine::list_models(&llm, api_key).await.map_err(to_err)
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn apply_autostart(app: &AppHandle, enabled: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    if let Err(e) = result {
        log::warn!("could not update the launch-at-login setting: {e}");
    }
}
