/** Mirrors the serde representation of the Rust settings types. */

import type { UiTheme } from "./theme";

export type SttProviderKind = "soniox" | "deepgram" | "assembly_ai";

export interface SttSettings {
  provider: SttProviderKind;
  soniox_model: string;
  deepgram_model: string;
  language: string;
  language_hints: string[];
}

export interface LlmSettings {
  preset: string;
  model: string;
  base_url: string | null;
  max_tokens: number;
}

export type OutputAction = "paste" | "copy_only";

export interface Mode {
  id: string;
  name: string;
  hotkey: string | null;
  ai_cleanup: boolean;
  prompt: string;
  llm_override: LlmSettings | null;
  output: OutputAction;
}

export interface HotkeySettings {
  toggle: string | null;
  cancel: string | null;
  next_mode: string | null;
  read: string | null;
  read_with_voice: string | null;
}

export type TtsProviderKind = "soniox" | "gemini" | "open_ai" | "eleven_labs" | "vbee";

export interface TtsSettings {
  enabled: boolean;
  provider: TtsProviderKind;
  voice: string;
  model: string;
  speed: number;
  max_chars: number;
  player_drag_position: [number, number] | null;
  /** Vbee's second auth field — its `App-Id` header. Not the sensitive half of the
   *  pair (the bearer token, which lives in the credential vault like every other
   *  provider's key), so it lives here as a plain field rather than a vault entry. */
  vbee_app_id: string;
}

export interface ReadMode {
  id: string;
  name: string;
  hotkey: string | null;
  ai: boolean;
  prompt: string;
  llm_override: LlmSettings | null;
}

export interface TtsVoice {
  id: string;
  label: string;
  language: string | null;
}

/** One selectable model plus everything needed to render its controls. Speed bounds
 *  are per-model and come from the provider (Soniox reports 0.7–1.3), so the slider
 *  can never offer a value the vendor would reject. */
export interface TtsModel {
  id: string;
  label: string;
  voices: TtsVoice[];
  speed_min: number;
  speed_max: number;
  supports_speed: boolean;
}

export interface AudioSettings {
  input_device: string | null;
  feedback_sounds: boolean;
  feedback_volume: number;
}

export interface HistorySettings {
  enabled: boolean;
  keep_audio: boolean;
  max_entries: number;
  audio_retention_days: number;
}

export interface AppSettings {
  stt: SttSettings;
  llm: LlmSettings;
  modes: Mode[];
  active_mode_id: string;
  hotkeys: HotkeySettings;
  audio: AudioSettings;
  history: HistorySettings;
  autostart: boolean;
  ui_language: "system" | "en" | "vi";
  theme: UiTheme;
  onboarding_completed: boolean;
  use_os_keychain: boolean;
  auto_check_updates: boolean;
  tts: TtsSettings;
  read_modes: ReadMode[];
  active_read_mode_id: string;
}

export interface LlmPreset {
  id: string;
  label: string;
  default_model: string;
  models: string[];
  secret_key: string;
  needs_key: boolean;
  base_url: string;
  signup_url: string;
}

export interface HistoryEntry {
  id: string;
  created_at: string;
  mode_id: string;
  mode_name: string;
  raw_text: string;
  final_text: string;
  duration_secs: number;
  stt_provider: string;
  audio_path: string | null;
}

export type Phase = "idle" | "recording" | "transcribing" | "refining" | "pasting";

export interface StatusPayload {
  phase: Phase;
  mode_id: string;
  mode_name: string;
}

/**
 * The speech providers, described by *properties* rather than prose.
 *
 * The wording lives in the interface dictionary so it follows the user's language —
 * a hardcoded English sentence here would show up untranslated everywhere it is used.
 * Prices and credits verified against each vendor's public pricing page.
 */
export const STT_PROVIDERS: {
  id: SttProviderKind;
  label: string;
  secret: string;
  signupUrl: string;
  /**
   * The single thing worth knowing about this provider, shown as one badge.
   *
   * One badge, not a row of them: the badge answers "why would I pick this one",
   * and a provider with four chips answers nothing.
   */
  badge: "best_vietnamese" | "free_credit";
  /** What a new account gets, or null when the vendor bills from the first minute. */
  freeCredit: string | null;
  /** Streaming price in USD per hour, so the trade-off is comparable at a glance. */
  hourlyUsd: number;
}[] = [
  {
    id: "soniox",
    label: "Soniox",
    secret: "soniox",
    signupUrl: "https://console.soniox.com/",
    badge: "best_vietnamese",
    freeCredit: null,
    hourlyUsd: 0.12,
  },
  {
    id: "deepgram",
    label: "Deepgram",
    secret: "deepgram",
    signupUrl: "https://console.deepgram.com/signup",
    badge: "free_credit",
    freeCredit: "$200",
    hourlyUsd: 0.29,
  },
  {
    id: "assembly_ai",
    label: "AssemblyAI",
    secret: "assemblyai",
    signupUrl: "https://www.assemblyai.com/dashboard/signup",
    badge: "free_credit",
    freeCredit: "$50",
    hourlyUsd: 0.15,
  },
];

/**
 * A failure from the backend. The backend sends a *kind*, not a sentence, so the
 * wording comes from the interface dictionary and follows the user's language.
 * `detail` is the vendor's or the OS's own words and is shown verbatim.
 */
export type ErrorKind =
  | "needs_accessibility"
  | "no_paste_target"
  | "delivery_failed"
  | "no_stt_key"
  | "no_llm_key"
  | "cleanup_failed"
  | "mic_unavailable"
  | "no_audio_captured"
  | "transcription_failed"
  | "nothing_selected"
  | "selection_too_long"
  | "tts_failed"
  | "no_tts_key";

export interface ErrorPayload {
  kind: ErrorKind;
  detail: string | null;
}

export type ReadPhase = "preparing" | "speaking" | "paused" | "done" | "failed";

export interface ReadStatusPayload {
  phase: ReadPhase;
  mode_name: string;
  provider: string;
  speed: number;
  /** The active provider's accepted range, so the player's slider stays inside it. */
  speed_min: number;
  speed_max: number;
  played_ms: number;
  total_ms: number;
  error: ErrorPayload | null;
  /** Opening of the clipboard text being offered in place of a selection the app could
   *  not capture — non-null exactly while that offer stands. Shown so the button is an
   *  informed yes: this text is about to reach a speech vendor and be said out loud. */
  clipboard_preview: string | null;
}

/** The read-aloud providers, described the same way `STT_PROVIDERS` is: properties,
 *  not prose, so the wording stays in the interface dictionary and follows language. */
export const TTS_PROVIDERS: {
  id: TtsProviderKind;
  label: string;
  secret: string;
  signupUrl: string;
  /** Whether picking this provider needs a brand-new key, or reuses one already saved
   *  for speech recognition / AI cleanup. */
  needsNewKey: boolean;
  supportsSpeed: boolean;
  /** Each provider's model ids live in a completely different namespace — switching
   *  provider without resetting this would send provider A's model name to provider B
   *  and 404. Picked here so the switch in `read-voice-section.tsx` is one line. */
  defaultModel: string;
  /** Vbee is the only provider whose auth is two parts (`App-Id` + bearer token)
   *  instead of one key — `read-voice-section.tsx` shows a second field only when
   *  this is set. */
  needsAppId: boolean;
}[] = [
  {
    id: "soniox",
    label: "Soniox",
    secret: "soniox",
    signupUrl: "https://console.soniox.com/",
    needsNewKey: false,
    supportsSpeed: true,
    defaultModel: "tts-rt-v1",
    needsAppId: false,
  },
  {
    id: "gemini",
    label: "Google Gemini",
    secret: "gemini",
    signupUrl: "https://aistudio.google.com/apikey",
    needsNewKey: false,
    supportsSpeed: false,
    defaultModel: "gemini-2.5-flash-preview-tts",
    needsAppId: false,
  },
  {
    id: "open_ai",
    label: "OpenAI",
    secret: "openai",
    signupUrl: "https://platform.openai.com/api-keys",
    needsNewKey: false,
    supportsSpeed: true,
    defaultModel: "gpt-4o-mini-tts",
    needsAppId: false,
  },
  {
    id: "eleven_labs",
    label: "ElevenLabs",
    secret: "elevenlabs",
    signupUrl: "https://elevenlabs.io/app/settings/api-keys",
    needsNewKey: true,
    supportsSpeed: true,
    defaultModel: "eleven_flash_v2_5",
    needsAppId: false,
  },
  {
    id: "vbee",
    label: "Vbee",
    secret: "vbee",
    signupUrl: "https://studio.vbee.vn/apps",
    needsNewKey: true,
    supportsSpeed: true,
    defaultModel: "",
    needsAppId: true,
  },
];
