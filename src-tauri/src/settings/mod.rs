//! Persisted user configuration: STT provider, LLM provider, modes, hotkeys, history policy.
//!
//! Stored as plain JSON in the Tauri app-config dir. API keys never live here — they go
//! to the OS keychain via [`secrets`]. Keeping them apart means the settings file stays
//! safe to inspect, diff, or back up.

pub mod defaults;
pub mod secrets;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SttProviderKind {
    Soniox,
    Deepgram,
    AssemblyAi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttSettings {
    pub provider: SttProviderKind,
    /// Soniox realtime model; `stt-rt-preview` handles Vietnamese/English code-switching.
    pub soniox_model: String,
    /// Deepgram model. `nova-2` is the one with Vietnamese support (`nova-3` is English-centric).
    pub deepgram_model: String,
    /// Primary language code sent to providers that want a single language (Deepgram).
    pub language: String,
    /// Hint list for providers that accept several (Soniox). Order matters — most likely first.
    pub language_hints: Vec<String>,
}

/// Which wire format an LLM endpoint speaks. Nearly every vendor besides Anthropic
/// exposes an OpenAI-compatible `/chat/completions`, so one client covers them all.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LlmWire {
    Anthropic,
    OpenAiCompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    /// Preset id from [`defaults::llm_presets`], or `custom`.
    pub preset: String,
    pub model: String,
    /// Only meaningful when `preset == "custom"`; otherwise the preset's URL wins.
    pub base_url: Option<String>,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputAction {
    /// Copy to clipboard, then synthesize Cmd/Ctrl+V into the focused app.
    Paste,
    /// Copy only — useful for modes whose output you want to place by hand.
    CopyOnly,
}

/// A named recipe: what the AI should do with the raw transcript, and where the result goes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode {
    pub id: String,
    pub name: String,
    /// Optional dedicated shortcut that both switches to this mode and starts recording.
    pub hotkey: Option<String>,
    /// When false the raw transcript is pasted untouched — fastest path, no LLM call.
    pub ai_cleanup: bool,
    /// System prompt handed to the LLM. Ignored when `ai_cleanup` is false.
    pub prompt: String,
    /// Per-mode LLM override, e.g. a stronger model for the Email mode.
    pub llm_override: Option<LlmSettings>,
    pub output: OutputAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    /// Press once to start, again to stop-and-transcribe. The only way in.
    pub toggle: Option<String>,
    /// Abort the take: stops capture and throws the audio away.
    pub cancel: Option<String>,
    /// Cycle to the next mode in the list.
    pub next_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// cpal device name; `None` follows the system default input.
    pub input_device: Option<String>,
    /// Short tones on start / stop / done / cancel so you can dictate without looking.
    pub feedback_sounds: bool,
    pub feedback_volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySettings {
    pub enabled: bool,
    /// Keep the captured WAV alongside the text so you can replay a bad transcription.
    pub keep_audio: bool,
    pub max_entries: usize,
    /// Audio older than this is pruned even if the text entry survives.
    pub audio_retention_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub stt: SttSettings,
    pub llm: LlmSettings,
    pub modes: Vec<Mode>,
    pub active_mode_id: String,
    pub hotkeys: HotkeySettings,
    pub audio: AudioSettings,
    pub history: HistorySettings,
    pub autostart: bool,
    /// Interface language: `system`, `en` or `vi`. `system` follows the OS locale,
    /// resolved in the frontend where `navigator.language` is available.
    /// `serde(default)` keeps older settings files loading.
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    /// Whether the first-run walkthrough has been finished. Defaults to false so an
    /// existing install that predates onboarding still gets shown it once.
    #[serde(default)]
    pub onboarding_completed: bool,
    /// Store API keys in the OS keychain instead of the local `0600` vault file.
    /// Stronger, but only worth it on a code-signed build — see [`secrets`].
    /// `serde(default)` so settings files written before this existed still load.
    #[serde(default)]
    pub use_os_keychain: bool,
    /// Check GitHub for a newer release once per launch. The only outbound call this
    /// setting controls is a GET of `latest.json`, carrying the app version and the
    /// user's IP — no other telemetry. Defaults to on; the toggle lives in Settings.
    /// `serde(default = "default_true")` so settings files written before this existed
    /// still load.
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,
}

fn default_ui_language() -> String {
    "system".to_string()
}

fn default_true() -> bool {
    true
}

/// Every credential name the app can store, for backend migration.
pub fn all_secret_accounts() -> Vec<&'static str> {
    let mut accounts = vec!["soniox", "deepgram", "assemblyai"];
    accounts.extend(defaults::llm_presets().iter().map(|p| p.secret_key));
    accounts
}

impl AppSettings {
    pub fn active_mode(&self) -> &Mode {
        self.modes
            .iter()
            .find(|m| m.id == self.active_mode_id)
            .unwrap_or_else(|| &self.modes[0])
    }

    pub fn mode(&self, id: &str) -> Option<&Mode> {
        self.modes.iter().find(|m| m.id == id)
    }

    /// LLM config for a mode, falling back to the global one.
    pub fn llm_for(&self, mode: &Mode) -> LlmSettings {
        mode.llm_override.clone().unwrap_or_else(|| self.llm.clone())
    }
}

pub fn settings_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .context("no app config dir available")?;
    crate::private_file::create_dir(&dir)?;
    Ok(dir.join(SETTINGS_FILE))
}

/// Reads settings from disk, falling back to defaults when the file is missing or corrupt.
/// A corrupt file is preserved as `settings.json.bak` rather than silently overwritten.
pub fn load(app: &AppHandle) -> AppSettings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(e) => {
            log::error!("settings path unavailable: {e:#}");
            return defaults::default_settings();
        }
    };
    let raw = match std::fs::read_to_string(&path) {
        Ok(r) => r,
        Err(_) => return defaults::default_settings(),
    };
    match serde_json::from_str::<AppSettings>(&raw) {
        Ok(mut s) => {
            restore_missing_dictation_hotkey(&mut s);
            s
        }
        Err(e) => {
            log::error!("settings.json unreadable ({e}); backing up and using defaults");
            let _ = std::fs::write(path.with_extension("json.bak"), &raw);
            defaults::default_settings()
        }
    }
}

/// Guarantees there is a key that starts a dictation.
///
/// Earlier builds offered a second, hold-to-talk binding, and someone who dictated that
/// way could reasonably have cleared the press-once key they never used. That settings
/// file now describes an app with no way to start a take at all, so the factory key is
/// put back rather than leaving the hotkey silently dead.
fn restore_missing_dictation_hotkey(settings: &mut AppSettings) {
    if settings.hotkeys.toggle.is_none() {
        let replacement = defaults::default_hotkeys().toggle;
        log::info!("no dictation hotkey was set; restoring the default {replacement:?}");
        settings.hotkeys.toggle = replacement;
    }
}

#[cfg(test)]
mod default_tests {
    use super::*;

    /// Setup asks for a speech key and nothing else, so the mode a fresh install lands
    /// in must not need an AI key. Picking one that does meant every first dictation
    /// raised "no AI provider key saved" and then pasted the raw transcript regardless —
    /// an error message attached to an outcome that was already correct.
    #[test]
    fn the_out_of_the_box_mode_works_without_an_ai_key() {
        let settings = defaults::default_settings();
        let mode = settings.active_mode();
        assert!(
            !mode.ai_cleanup,
            "default mode {:?} needs an AI key that setup never asks for",
            mode.id
        );
    }

    #[test]
    fn the_default_active_mode_actually_exists() {
        let settings = defaults::default_settings();
        assert!(settings.mode(&settings.active_mode_id).is_some());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A settings file written by a build that still had hold-to-talk keeps loading:
    /// the retired `push_to_talk` key is simply not read any more.
    #[test]
    fn a_settings_file_from_the_hold_to_talk_era_still_loads() {
        let mut raw = serde_json::to_value(defaults::default_settings()).unwrap();
        raw["hotkeys"]["push_to_talk"] =
            serde_json::json!({ "kind": "modifier", "key": "right_option" });

        let parsed: AppSettings = serde_json::from_value(raw).unwrap();

        assert!(parsed.hotkeys.toggle.is_some());
    }

    #[test]
    fn a_settings_file_with_no_dictation_key_gets_the_default_back() {
        let mut settings = defaults::default_settings();
        settings.hotkeys.toggle = None;

        restore_missing_dictation_hotkey(&mut settings);

        assert_eq!(settings.hotkeys.toggle, defaults::default_hotkeys().toggle);
    }

    #[test]
    fn a_dictation_key_the_user_chose_themselves_is_left_alone() {
        let mut settings = defaults::default_settings();
        settings.hotkeys.toggle = Some("Control+Shift+Space".into());

        restore_missing_dictation_hotkey(&mut settings);

        assert_eq!(settings.hotkeys.toggle.as_deref(), Some("Control+Shift+Space"));
    }
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<()> {
    let path = settings_path(app)?;
    let json = serde_json::to_string_pretty(settings)?;
    crate::private_file::write(&path, &json)
}
