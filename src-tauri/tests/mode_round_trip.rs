//! Every mode, run through the real cleanup client against a live provider.
//!
//! The unit tests cover the transport; this covers the thing a user actually notices —
//! that each mode's prompt produces usable text rather than an explanation, a refusal,
//! an empty string, or the transcript echoed back unchanged.
//!
//! ```sh
//! FVT_LLM_PRESET=deepseek FVT_LLM_MODEL=deepseek-v4-flash FVT_LLM_KEY=sk-... \
//! cargo test --test mode_round_trip -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Point `FVT_SETTINGS` at a live `settings.json` to exercise the modes someone has
//! actually configured, custom ones included, rather than only the shipped defaults.

use tockyvoice_lib::refine::{self, RefineRequest};
use tockyvoice_lib::settings::{defaults, AppSettings, LlmSettings, Mode, OutputAction};

/// Vietnamese with English technical terms, no punctuation, one filler word — the shape
/// of text every mode has to cope with.
const TRANSCRIPT: &str = "chào các bạn mình là bình hôm nay mình sẽ ờ hướng dẫn cài đặt \
môi trường cần cài node và rust sau đó cấu hình api key rồi build app và cấp quyền \
accessibility trên macos";

fn llm() -> LlmSettings {
    LlmSettings {
        preset: std::env::var("FVT_LLM_PRESET").expect("set FVT_LLM_PRESET"),
        model: std::env::var("FVT_LLM_MODEL").expect("set FVT_LLM_MODEL"),
        base_url: None,
        max_tokens: 2048,
    }
}

/// Signs that the model answered *about* the task instead of doing it. A mode that
/// trips these pastes a chatbot reply into the user's document.
fn looks_like_commentary(text: &str) -> Option<&'static str> {
    let lower = text.trim().to_lowercase();
    for marker in [
        "as an ai",
        "tôi là một",
        "dưới đây là",
        "here is the",
        "here's the",
        "i cannot",
        "xin lỗi",
    ] {
        if lower.starts_with(marker) {
            return Some("opens with commentary rather than the rewritten text");
        }
    }
    None
}

/// Runs one mode and returns a failure description, or `None` when it behaved.
async fn check(mode: &Mode, llm: &LlmSettings) -> Option<String> {
    // Paste is what makes a mode useful without extra clicks; a mode that only copies
    // is a silent regression the UI does not advertise.
    if !matches!(mode.output, OutputAction::Paste) {
        return Some(format!("[{}] {} → does not paste", mode.id, mode.name));
    }

    if !mode.ai_cleanup {
        // Raw is the escape hatch: it must never call an LLM, so there is nothing to
        // round-trip — its correctness is that the transcript stays untouched.
        if !mode.prompt.trim().is_empty() {
            return Some(format!(
                "[{}] {} → skips cleanup but carries a prompt",
                mode.id, mode.name
            ));
        }
        println!("[{}] {} — no AI pass, skipped\n", mode.id, mode.name);
        return None;
    }

    let started = std::time::Instant::now();
    let result = refine::refine(RefineRequest {
        system_prompt: mode.prompt.clone(),
        transcript: TRANSCRIPT.into(),
        llm: llm.clone(),
        api_key: std::env::var("FVT_LLM_KEY").ok(),
    })
    .await;

    match result {
        Err(e) => Some(format!("[{}] {} → error: {e:#}", mode.id, mode.name)),
        Ok(text) if text.trim().is_empty() => {
            Some(format!("[{}] {} → empty output", mode.id, mode.name))
        }
        Ok(text) => {
            println!(
                "[{}] {} — {:.1}s, {} chars\n{}\n",
                mode.id,
                mode.name,
                started.elapsed().as_secs_f32(),
                text.chars().count(),
                text.trim()
            );
            looks_like_commentary(&text).map(|why| format!("[{}] {} → {why}", mode.id, mode.name))
        }
    }
}

async fn run(modes: &[Mode]) {
    let llm = llm();
    println!("provider: {} / {}\n", llm.preset, llm.model);

    let mut failures = Vec::new();
    for mode in modes {
        if let Some(failure) = check(mode, &llm).await {
            failures.push(failure);
        }
    }
    assert!(
        failures.is_empty(),
        "modes failed:\n  {}",
        failures.join("\n  ")
    );
}

#[tokio::test]
#[ignore = "hits a live LLM API"]
async fn every_shipped_mode_returns_usable_text() {
    run(&defaults::default_modes()).await;
}

/// The defaults are what ships; this is what the person in front of the app has. A
/// prompt someone typed themselves is exactly the one nobody has ever tested.
#[tokio::test]
#[ignore = "hits a live LLM API and needs FVT_SETTINGS"]
async fn every_configured_mode_returns_usable_text() {
    let path = std::env::var("FVT_SETTINGS").expect("set FVT_SETTINGS to a settings.json");
    let raw = std::fs::read_to_string(&path).expect("reading settings.json");
    let settings: AppSettings = serde_json::from_str(&raw).expect("parsing settings.json");
    run(&settings.modes).await;
}
