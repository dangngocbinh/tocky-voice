/** Mirrors the serde representation of the Rust settings types. */

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

export type ModifierKey =
  | "right_option"
  | "left_option"
  | "right_command"
  | "fn";

export type PushToTalk =
  | { kind: "disabled" }
  | { kind: "modifier"; key: ModifierKey }
  | { kind: "shortcut"; accelerator: string };

export interface HotkeySettings {
  toggle: string | null;
  cancel: string | null;
  next_mode: string | null;
  push_to_talk: PushToTalk;
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
  onboarding_completed: boolean;
  use_os_keychain: boolean;
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
  | "transcription_failed";

export interface ErrorPayload {
  kind: ErrorKind;
  detail: string | null;
}
