//! Factory settings and the LLM endpoint catalogue.

use super::*;

/// A known LLM endpoint. Everything except Anthropic speaks the OpenAI chat-completions
/// shape, so one HTTP client serves the whole list — only the base URL and model differ.
#[derive(Debug, Clone, Serialize)]
pub struct LlmPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub wire: LlmWire,
    pub base_url: &'static str,
    /// Fast, cheap model suitable for transcript cleanup.
    pub default_model: &'static str,
    /// Extra models worth offering in the picker.
    pub models: &'static [&'static str],
    /// Keychain account name holding this vendor's API key.
    pub secret_key: &'static str,
    /// Where a new user goes to create an account and get a key.
    pub signup_url: &'static str,
    /// Local endpoints need no credential.
    pub needs_key: bool,
    /// Vendor-specific request fields merged into the chat-completions body, as a JSON
    /// object literal. Cleanup is a rewrite, not a puzzle: vendors that think before
    /// answering spend seconds producing hidden reasoning nobody reads, so this is where
    /// a preset turns that off. Kept as a string because a preset table is `const`.
    pub extra_body: Option<&'static str>,
}

pub fn llm_presets() -> &'static [LlmPreset] {
    &[
        LlmPreset {
            id: "anthropic",
            label: "Anthropic Claude",
            wire: LlmWire::Anthropic,
            base_url: "https://api.anthropic.com/v1",
            default_model: "claude-haiku-4-5",
            models: &["claude-haiku-4-5", "claude-sonnet-5", "claude-opus-5"],
            secret_key: "anthropic",
            signup_url: "https://console.anthropic.com/settings/keys",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "openai",
            label: "OpenAI",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://api.openai.com/v1",
            default_model: "gpt-4.1-mini",
            models: &["gpt-4.1-mini", "gpt-4.1", "gpt-4o-mini"],
            secret_key: "openai",
            signup_url: "https://platform.openai.com/api-keys",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "gemini",
            label: "Google Gemini",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
            default_model: "gemini-2.5-flash",
            models: &["gemini-2.5-flash", "gemini-2.5-flash-lite", "gemini-2.5-pro"],
            secret_key: "gemini",
            signup_url: "https://aistudio.google.com/apikey",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "deepseek",
            label: "DeepSeek",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://api.deepseek.com/v1",
            default_model: "deepseek-v4-flash",
            models: &["deepseek-v4-flash", "deepseek-v4-pro"],
            secret_key: "deepseek",
            signup_url: "https://platform.deepseek.com/api_keys",
            needs_key: true,
            // V4 thinks before it answers. Measured on a "reformat this as markdown"
            // prompt: 10.8s with thinking on (one run 28.7s), 1.9s with it off, for
            // output of the same quality — the reasoning is wasted on a rewrite task.
            extra_body: Some(r#"{"thinking":{"type":"disabled"}}"#),
        },
        LlmPreset {
            id: "qwen",
            label: "Qwen (Alibaba DashScope)",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
            default_model: "qwen-flash",
            models: &["qwen-flash", "qwen-plus", "qwen-max"],
            secret_key: "qwen",
            signup_url: "https://modelstudio.console.alibabacloud.com/",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "moonshot",
            label: "Moonshot Kimi",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://api.moonshot.ai/v1",
            default_model: "kimi-k2-turbo-preview",
            models: &["kimi-k2-turbo-preview", "kimi-k2-0905-preview"],
            secret_key: "moonshot",
            signup_url: "https://platform.moonshot.ai/console/api-keys",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "zhipu",
            label: "Zhipu GLM",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://open.bigmodel.cn/api/paas/v4",
            default_model: "glm-4-flash",
            models: &["glm-4-flash", "glm-4-air", "glm-4-plus"],
            secret_key: "zhipu",
            signup_url: "https://open.bigmodel.cn/usercenter/apikeys",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "minimax",
            label: "MiniMax",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://api.minimax.io/v1",
            default_model: "MiniMax-Text-01",
            models: &["MiniMax-Text-01"],
            secret_key: "minimax",
            signup_url: "https://platform.minimax.io/user-center/basic-information/interface-key",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "groq",
            label: "Groq",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://api.groq.com/openai/v1",
            default_model: "llama-3.3-70b-versatile",
            models: &["llama-3.3-70b-versatile", "llama-3.1-8b-instant"],
            secret_key: "groq",
            signup_url: "https://console.groq.com/keys",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "xai",
            label: "xAI Grok",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://api.x.ai/v1",
            default_model: "grok-4-fast",
            models: &["grok-4-fast", "grok-4"],
            secret_key: "xai",
            signup_url: "https://console.x.ai/",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "openrouter",
            label: "OpenRouter (any model)",
            wire: LlmWire::OpenAiCompatible,
            base_url: "https://openrouter.ai/api/v1",
            default_model: "google/gemini-2.5-flash",
            models: &["google/gemini-2.5-flash", "anthropic/claude-haiku-4.5"],
            secret_key: "openrouter",
            signup_url: "https://openrouter.ai/settings/keys",
            needs_key: true,
            extra_body: None,
        },
        LlmPreset {
            id: "ollama",
            label: "Ollama (local)",
            wire: LlmWire::OpenAiCompatible,
            base_url: "http://localhost:11434/v1",
            default_model: "qwen2.5:7b",
            models: &["qwen2.5:7b", "llama3.1:8b"],
            secret_key: "ollama",
            signup_url: "https://ollama.com/download",
            needs_key: false,
            extra_body: None,
        },
        LlmPreset {
            id: "custom",
            label: "Custom OpenAI-compatible endpoint",
            wire: LlmWire::OpenAiCompatible,
            base_url: "",
            default_model: "",
            models: &[],
            secret_key: "custom",
            signup_url: "",
            needs_key: true,
            extra_body: None,
        },
    ]
}

pub fn preset(id: &str) -> Option<&'static LlmPreset> {
    llm_presets().iter().find(|p| p.id == id)
}

const CLEAN_PROMPT: &str = "Bạn là bộ hiệu đính cho phần mềm nhập liệu bằng giọng nói. \
Người dùng nói tiếng Việt có xen thuật ngữ tiếng Anh.

Sửa lại đoạn văn bản nhận dạng được: thêm dấu câu và viết hoa cho đúng, sửa lỗi chính tả và \
lỗi nhận dạng sai, bỏ các từ đệm lặp (ừ, à, ờ, kiểu như) và các đoạn nói lắp. \
Giữ nguyên thuật ngữ kỹ thuật tiếng Anh (API, deploy, function, commit...) — không dịch chúng.

Giữ nguyên ý và giọng văn của người nói. Không thêm nội dung, không trả lời câu hỏi trong văn bản, \
không giải thích. Chỉ trả về đúng đoạn văn bản đã sửa.";

const PROMPT_MODE_PROMPT: &str = "Bạn chuyển lời nói thành một prompt rõ ràng cho AI coding agent.

Viết lại nội dung người dùng vừa đọc thành một yêu cầu mạch lạc, đúng chính tả, có cấu trúc: \
nêu rõ mục tiêu, ràng buộc và tiêu chí hoàn thành nếu người nói có đề cập. \
Giữ nguyên tên file, tên hàm, thuật ngữ kỹ thuật. Dùng gạch đầu dòng khi có nhiều yêu cầu.

Không tự thêm yêu cầu người dùng không nói. Chỉ trả về prompt, không giải thích.";

const EMAIL_PROMPT: &str = "Bạn viết lại lời nói thành email/tin nhắn công việc trang trọng.

Sửa chính tả, dấu câu, và diễn đạt lại cho lịch sự, chuyên nghiệp, ngắn gọn. \
Giữ nguyên toàn bộ thông tin và ý định của người nói — không thêm, không bịa chi tiết. \
Không thêm dòng chào/ký tên trừ khi người nói có đọc.

Chỉ trả về nội dung email, không giải thích.";

/// The modes a fresh install ships with. Public so the integration tests can
/// exercise the same prompts users actually get.
pub fn default_modes() -> Vec<Mode> {
    vec![
        Mode {
            id: "raw".into(),
            name: "Raw".into(),
            hotkey: None,
            ai_cleanup: false,
            prompt: String::new(),
            llm_override: None,
            output: OutputAction::Paste,
        },
        Mode {
            id: "clean".into(),
            name: "Clean".into(),
            hotkey: None,
            ai_cleanup: true,
            prompt: CLEAN_PROMPT.into(),
            llm_override: None,
            output: OutputAction::Paste,
        },
        Mode {
            id: "prompt".into(),
            name: "Prompt".into(),
            hotkey: None,
            ai_cleanup: true,
            prompt: PROMPT_MODE_PROMPT.into(),
            llm_override: None,
            output: OutputAction::Paste,
        },
        Mode {
            id: "email".into(),
            name: "Email".into(),
            hotkey: None,
            ai_cleanup: true,
            prompt: EMAIL_PROMPT.into(),
            llm_override: None,
            output: OutputAction::Paste,
        },
    ]
}

pub fn default_settings() -> AppSettings {
    AppSettings {
        stt: SttSettings {
            provider: SttProviderKind::Soniox,
            soniox_model: "stt-rt-preview".into(),
            deepgram_model: "nova-2".into(),
            language: "vi".into(),
            language_hints: vec!["vi".into(), "en".into()],
        },
        llm: LlmSettings {
            preset: "deepseek".into(),
            model: "deepseek-v4-flash".into(),
            base_url: None,
            // Reasoning models spend this budget thinking before they answer; too low
            // and the response comes back empty. 2048 leaves room for both.
            max_tokens: 2048,
        },
        modes: default_modes(),
        active_mode_id: "clean".into(),
        hotkeys: HotkeySettings {
            toggle: Some("Control+Alt+D".into()),
            cancel: Some("Control+Alt+X".into()),
            next_mode: Some("Control+Alt+M".into()),
            push_to_talk: PushToTalk::Modifier {
                key: ModifierKey::RightOption,
            },
        },
        audio: AudioSettings {
            input_device: None,
            feedback_sounds: true,
            feedback_volume: 0.15,
        },
        history: HistorySettings {
            enabled: true,
            keep_audio: true,
            max_entries: 500,
            audio_retention_days: 14,
        },
        autostart: false,
        // Follow the OS locale; the frontend maps anything non-Vietnamese to English.
        ui_language: "system".into(),
        onboarding_completed: false,
        // The local vault, not the keychain. An unsigned macOS build makes the keychain
        // prompt for the login password on every read; see `secrets` for the reasoning.
        use_os_keychain: false,
    }
}
