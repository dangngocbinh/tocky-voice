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

export const STT_PROVIDERS: {
  id: SttProviderKind;
  label: string;
  secret: string;
  note: string;
  signupUrl: string;
  /** Only stated where the vendor publishes it — see the About tab. */
  freeCredit: string | null;
}[] = [
  {
    id: "soniox",
    label: "Soniox",
    secret: "soniox",
    note: "Best for mixed Vietnamese + English in one sentence. $0.12/hour streaming.",
    signupUrl: "https://console.soniox.com/",
    freeCredit: "free trial credit on signup",
  },
  {
    id: "deepgram",
    label: "Deepgram",
    secret: "deepgram",
    note: "Lowest latency. Vietnamese needs the nova-2 model, and quality on mixed VI/EN is weaker.",
    signupUrl: "https://console.deepgram.com/signup",
    freeCredit: "$200 free credit",
  },
  {
    id: "assembly_ai",
    label: "AssemblyAI",
    secret: "assemblyai",
    note: "Very accurate on English; its streaming tier has no Vietnamese yet.",
    signupUrl: "https://www.assemblyai.com/dashboard/signup",
    freeCredit: "$50 free credit",
  },
];
