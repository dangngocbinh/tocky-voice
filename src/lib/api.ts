/** Thin typed wrappers over the Rust commands, so components never touch `invoke` strings. */

import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  HistoryEntry,
  LlmPreset,
  SttSettings,
  ReadStatusPayload,
  TtsModel,
  TtsSettings,
} from "./types";
import { RELEASES_URL } from "./update-policy";

export const EVENTS = {
  status: "fvt://status",
  partial: "fvt://partial",
  transcript: "fvt://transcript",
  level: "fvt://level",
  historyChanged: "fvt://history-changed",
  settingsChanged: "fvt://settings-changed",
  error: "fvt://error",
  readStatus: "fvt://read-status",
} as const;

export const getSettings = () => invoke<AppSettings>("get_settings");
export const saveSettings = (settings: AppSettings) =>
  invoke<void>("save_settings", { settings });
export const resetSettings = () => invoke<AppSettings>("reset_settings");

export const listLlmPresets = () => invoke<LlmPreset[]>("list_llm_presets");
export const listInputDevices = () => invoke<string[]>("list_input_devices");
/** Opens a microphone and streams `EVENTS.level` from it, with no session behind it —
 *  the setup wizard's proof that the chosen input actually carries sound. */
export const startMicTest = (device: string | null) =>
  invoke<void>("start_mic_test", { device });
export const stopMicTest = () => invoke<void>("stop_mic_test");

export const setApiKey = (account: string, value: string) =>
  invoke<void>("set_api_key", { account, value });
export const deleteApiKey = (account: string) =>
  invoke<void>("delete_api_key", { account });
/** Which credentials are configured — the plaintext never crosses this boundary. */
export const keyStatus = () => invoke<Record<string, boolean>>("key_status");

export const startRecording = (modeId?: string) =>
  invoke<void>("start_recording", { modeId: modeId ?? null });
export const stopRecording = () => invoke<void>("stop_recording");
export const cancelRecording = () => invoke<void>("cancel_recording");
/** While onboarding's "try it" step is mounted, a take started there renders in its
 *  own preview instead of popping the floating overlay on top of it. */
export const setOverlaySuppressed = (suppressed: boolean) =>
  invoke<void>("set_overlay_suppressed", { suppressed });
export const toggleRecording = () => invoke<void>("toggle_recording");
export const setActiveMode = (modeId: string) =>
  invoke<void>("set_active_mode", { modeId });

export const getHistory = () => invoke<HistoryEntry[]>("get_history");
export const deleteHistoryEntry = (id: string) =>
  invoke<void>("delete_history_entry", { id });
export const clearHistory = () => invoke<void>("clear_history");

export const copyText = (text: string) => invoke<void>("copy_text", { text });
export const permissionStatus = () =>
  invoke<{ accessibility: boolean }>("permission_status");
/** Whether this build may update itself in place — see `code_signature.rs`. */
export const canSelfInstall = () => invoke<boolean>("can_self_install");
export const openAccessibilitySettings = () =>
  invoke<void>("open_accessibility_settings");
/** Opens an https link in the default browser. */
export const openUrl = (url: string) => invoke<void>("open_url", { url });

/** Manual-download fallback for every update path. */
export const openReleasesPage = () => openUrl(RELEASES_URL);
export const testLlm = () => invoke<string>("test_llm");
/** Opens a real speech stream with the saved key, so a rejected credential is caught
 *  where it is entered instead of as a dictation that produces nothing. */
export const testSttKey = (stt: SttSettings) => invoke<void>("test_stt_key", { stt });
/** Live model list straight from the configured provider. */
export const listModels = () => invoke<string[]>("list_models");

/** Released while the settings UI captures a key combination, so the app doesn't
 *  react to the very shortcut you are trying to assign. */
export const suspendHotkeys = () => invoke<void>("suspend_hotkeys");
export const resumeHotkeys = () => invoke<void>("resume_hotkeys");

// ---------------------------------------------------------------- read-aloud

export const startReading = (modeId?: string) =>
  invoke<void>("start_reading", { modeId: modeId ?? null });
export const toggleReading = () => invoke<void>("toggle_reading");
export const pauseReading = () => invoke<void>("pause_reading");
export const resumeReading = () => invoke<void>("resume_reading");
export const stopReading = () => invoke<void>("stop_reading");
/** Takes up the player's offer to read what was already on the clipboard, after a
 *  capture that came back empty. */
export const readFromClipboard = () => invoke<void>("read_from_clipboard");
/** Current status, pulled by the player as it mounts — its webview subscribes to the
 *  event stream only after booting, so on the first read it misses the `preparing`
 *  event that opened it and would otherwise sit blank until audio started. */
export const getReadStatus = () => invoke<ReadStatusPayload | null>("get_read_status");
/** Changes reading speed mid-session; applies from the next chunk. */
export const setReadSpeed = (speed: number) => invoke<void>("set_read_speed", { speed });
/** Switches read mode and re-reads the original selection under it. */
export const setReadMode = (modeId: string) => invoke<void>("set_read_mode", { modeId });
export const savePlayerPosition = (x: number, y: number) =>
  invoke<void>("save_player_position", { x, y });
/** Live model catalogue (each with its voices and speed range) for the provider the
 *  "Đọc" tab currently has selected. Takes `tts` from the caller (not the saved
 *  snapshot) — same reason `testSttKey` does: settings save on a debounce, so a
 *  provider picked a moment ago has not reached disk yet. */
export const listTtsVoices = (tts: TtsSettings) => invoke<TtsModel[]>("list_tts_voices", { tts });
/** Synthesizes and plays a fixed sample sentence in the current voice/speed. */
export const previewVoice = (tts: TtsSettings) => invoke<void>("preview_voice", { tts });
