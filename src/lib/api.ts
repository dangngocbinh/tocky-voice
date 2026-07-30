/** Thin typed wrappers over the Rust commands, so components never touch `invoke` strings. */

import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  HistoryEntry,
  LlmPreset,
  SttSettings,
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
