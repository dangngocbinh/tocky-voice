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
    /// Reads the current selection aloud in the active read mode.
    ///
    /// Defaults to the real factory accelerator, not `None` — a bare
    /// `#[serde(default)]` on an `Option<String>` resolves to `None`, which would leave
    /// every settings file written before this feature existed with no read hotkey
    /// bound at all (and no way to notice short of reading `hotkeys::apply`'s debug
    /// log). See `restore_missing_dictation_hotkey` for the same class of bug on the
    /// dictation hotkey, caught once already.
    #[serde(default = "defaults::default_read_hotkey")]
    pub read: Option<String>,
    /// Flow C: capture the selection, then take a spoken instruction for what to do
    /// with it before reading the result aloud.
    #[serde(default = "defaults::default_read_with_voice_hotkey")]
    pub read_with_voice: Option<String>,
}

/// Which text-to-speech vendor synthesizes the read-aloud audio.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TtsProviderKind {
    Soniox,
    Gemini,
    OpenAi,
    ElevenLabs,
    Vbee,
}

/// Read-aloud configuration. Off by default — see [`AppSettings::tts`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsSettings {
    pub enabled: bool,
    pub provider: TtsProviderKind,
    pub voice: String,
    pub model: String,
    pub speed: f32,
    /// Selections longer than this are refused rather than sent to a paid API by
    /// accident — the default is generous enough for a long article, not a whole page.
    pub max_chars: usize,
    /// Where the mini player was last **dragged** to, so it reopens in the same spot.
    ///
    /// Deliberately not the old `player_position`: that one was also written whenever
    /// the backend positioned the window itself, so it captured the default placement
    /// as though the user had chosen it — which then permanently outranked the default,
    /// making later changes to the default do nothing for anyone who had run the app
    /// once. Renaming is the migration: the polluted values simply stop being read.
    #[serde(default)]
    pub player_drag_position: Option<(f64, f64)>,
    /// Vbee is the one provider whose auth is two parts (`App-Id` header + bearer
    /// token) instead of a single key. The token goes through the normal credential
    /// vault under the `"vbee"` account like every other provider; the app id is not
    /// the sensitive half of the pair — Vbee's own docs describe it as identifying
    /// *which application* is calling, with the token being the actual bearer secret —
    /// so it lives here as a plain settings field instead of a second vault entry.
    #[serde(default)]
    pub vbee_app_id: String,
}

/// A named recipe for reading selected text aloud: an optional AI rewrite pass
/// (summarize, explain, translate) followed by speech synthesis. Deliberately not
/// [`Mode`] — a read mode has a voice and speed that mean nothing for dictation, and
/// `Mode::output` means nothing for something that is only ever heard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadMode {
    pub id: String,
    pub name: String,
    pub hotkey: Option<String>,
    /// `false` reads the selection verbatim, with no LLM call.
    pub ai: bool,
    pub prompt: String,
    pub llm_override: Option<LlmSettings>,
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

/// Outbound HTTP proxy for the speech-recognition websocket.
///
/// Off by default. Turn it on when the direct route to the speech provider is lossy
/// enough that the transcript arrives after `stt::DRAIN_TIMEOUT` has given up on it —
/// see `crate::proxy` for why that loses the end of the sentence rather than merely
/// being slow.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxySettings {
    pub enabled: bool,
    /// `host:port`. An `http://` prefix is accepted and ignored. Credentials never
    /// live here: the `username:password` pair goes to the vault under `"proxy"`, so
    /// the settings file stays safe to inspect, diff or back up like the rest of it.
    pub url: String,
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
    /// Interface theme: `system`, `light` or `dark`. `system` follows the OS setting,
    /// resolved in the frontend where `prefers-color-scheme` is available.
    /// `serde(default)` keeps older settings files loading.
    #[serde(default = "default_theme")]
    pub theme: String,
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
    /// Read-aloud configuration. `serde(default)` so every settings file written before
    /// this feature existed still loads, landing on the off-by-default factory config.
    #[serde(default = "defaults::default_tts")]
    pub tts: TtsSettings,
    /// The read-mode catalogue (verbatim / summary / explain / translate), separate
    /// from `modes` — see [`ReadMode`] for why.
    #[serde(default = "defaults::default_read_modes")]
    pub read_modes: Vec<ReadMode>,
    #[serde(default = "default_active_read_mode_id")]
    pub active_read_mode_id: String,
    /// Route the speech websocket through an HTTP proxy. `serde(default)` so settings
    /// files written before this existed still load, landing on off.
    #[serde(default)]
    pub proxy: ProxySettings,
}

fn default_ui_language() -> String {
    "system".to_string()
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_true() -> bool {
    true
}

fn default_active_read_mode_id() -> String {
    "verbatim".to_string()
}

/// Every credential name the app can store, for backend migration.
pub fn all_secret_accounts() -> Vec<&'static str> {
    let mut accounts = vec![
        "soniox",
        "deepgram",
        "assemblyai",
        "elevenlabs",
        "vbee",
        crate::proxy::SECRET_ACCOUNT,
    ];
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

    /// The read mode the current `active_read_mode_id` names, or the first one if a
    /// hand-edited settings file points at an id that no longer exists — mirrors
    /// `active_mode`'s `unwrap_or(&modes[0])` rather than panicking.
    pub fn active_read_mode(&self) -> &ReadMode {
        self.read_modes
            .iter()
            .find(|m| m.id == self.active_read_mode_id)
            .unwrap_or_else(|| &self.read_modes[0])
    }

    pub fn read_mode(&self, id: &str) -> Option<&ReadMode> {
        self.read_modes.iter().find(|m| m.id == id)
    }

    /// LLM config for a read mode, falling back to the global one — same rule as
    /// [`Self::llm_for`].
    pub fn llm_for_read_mode(&self, mode: &ReadMode) -> LlmSettings {
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
            restore_missing_read_hotkey(&mut s);
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

/// Same repair as [`restore_missing_dictation_hotkey`], for the same reason: a
/// settings file written by a build where `HotkeySettings::read` had a bare
/// `#[serde(default)]` (resolving to `None` instead of the real factory accelerator)
/// persisted that `None` to disk the first time it was saved. `#[serde(default = ...)]`
/// only fills in a field that is *missing*, not one written out as an explicit `null`,
/// so those already-saved files need this repair on top of the field-level fix.
fn restore_missing_read_hotkey(settings: &mut AppSettings) {
    if settings.hotkeys.read.is_none() {
        let replacement = defaults::default_read_hotkey();
        log::info!("no read-aloud hotkey was set; restoring the default {replacement:?}");
        settings.hotkeys.read = replacement;
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

    /// The whole read-aloud feature has to start invisible — see plan.md's "Bất biến
    /// phải giữ" #1. This is the config-level half of that guarantee; `hotkeys` tests
    /// cover the registration half.
    #[test]
    fn read_aloud_defaults_to_off() {
        assert!(!defaults::default_settings().tts.enabled);
    }

    #[test]
    fn the_default_active_read_mode_actually_exists() {
        let settings = defaults::default_settings();
        assert!(settings.read_mode(&settings.active_read_mode_id).is_some());
    }

    /// ElevenLabs is the one TTS provider with its own credential rather than reusing
    /// an existing one; missing it here means switching vault ⇄ keychain silently
    /// drops that key instead of migrating it.
    #[test]
    fn elevenlabs_is_included_in_credential_migration() {
        assert!(all_secret_accounts().contains(&"elevenlabs"));
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

    /// Regression: a settings file saved by the build with the `#[serde(default)]` bug
    /// has `"read": null` written out explicitly, which the field-level fix alone does
    /// not repair (that only fires when the key is absent, not when it's `null`).
    #[test]
    fn a_settings_file_with_an_explicit_null_read_hotkey_gets_the_default_back() {
        let mut settings = defaults::default_settings();
        settings.hotkeys.read = None;

        restore_missing_read_hotkey(&mut settings);

        assert_eq!(settings.hotkeys.read, defaults::default_read_hotkey());
    }

    #[test]
    fn a_dictation_key_the_user_chose_themselves_is_left_alone() {
        let mut settings = defaults::default_settings();
        settings.hotkeys.toggle = Some("Control+Shift+Space".into());

        restore_missing_dictation_hotkey(&mut settings);

        assert_eq!(settings.hotkeys.toggle.as_deref(), Some("Control+Shift+Space"));
    }

    /// A 0.4.0 settings file predates `tts`, `read_modes`, `active_read_mode_id` and the
    /// two new hotkey fields entirely. Every one of them needs `#[serde(default)]` or
    /// this fails to deserialize and every existing user's settings get replaced.
    #[test]
    fn a_settings_file_from_before_read_aloud_still_loads() {
        let mut raw = serde_json::to_value(defaults::default_settings()).unwrap();
        let obj = raw.as_object_mut().unwrap();
        obj.remove("tts");
        obj.remove("read_modes");
        obj.remove("active_read_mode_id");
        obj["hotkeys"].as_object_mut().unwrap().remove("read");
        obj["hotkeys"].as_object_mut().unwrap().remove("read_with_voice");

        let parsed: AppSettings = serde_json::from_value(raw).unwrap();

        assert!(!parsed.tts.enabled);
        assert!(!parsed.read_modes.is_empty());
        // Regression: these two used to fall back to a bare `#[serde(default)]`, which
        // for `Option<String>` resolves to `None` — an upgrading user got no read
        // hotkey at all rather than the real factory binding, discoverable only by
        // noticing `hotkeys::apply` never logged a "-> Read" registration line.
        assert_eq!(parsed.hotkeys.read, defaults::default_read_hotkey());
        assert_eq!(parsed.hotkeys.read_with_voice, defaults::default_read_with_voice_hotkey());
    }
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<()> {
    let path = settings_path(app)?;
    let json = serde_json::to_string_pretty(settings)?;
    crate::private_file::write(&path, &json)
}
