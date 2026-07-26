//! Live round-trip against an LLM cleanup provider, exercising the same client the
//! app uses rather than a hand-written curl equivalent.
//!
//! `#[ignore]`d because it needs credentials and network:
//!
//! ```sh
//! FVT_LLM_PRESET=deepseek FVT_LLM_MODEL=deepseek-v4-flash FVT_LLM_KEY=sk-... \
//! cargo test --test refine_round_trip -- --ignored --nocapture
//! ```

use tockyvoice_lib::refine::{self, RefineRequest};
use tockyvoice_lib::settings::LlmSettings;

const CLEANUP_PROMPT: &str = "Sửa chính tả và dấu câu, bỏ từ đệm lặp. \
Giữ nguyên thuật ngữ tiếng Anh. Chỉ trả về văn bản đã sửa, không giải thích.";

/// Deliberately messy: no punctuation, a filler word, and English terms that must survive.
const MESSY_TRANSCRIPT: &str = "chào các bạn mình là đặng ngọc bình hôm nay mình sẽ ờ \
viết một cái ứng dụng để gõ chữ bằng giọng nói rồi mình sẽ deploy nó lên server";

#[tokio::test]
#[ignore = "hits a live LLM API"]
async fn cleans_up_a_messy_vietnamese_transcript() {
    let preset = std::env::var("FVT_LLM_PRESET").expect("set FVT_LLM_PRESET");
    let model = std::env::var("FVT_LLM_MODEL").expect("set FVT_LLM_MODEL");
    let api_key = std::env::var("FVT_LLM_KEY").ok();

    let started = std::time::Instant::now();
    let cleaned = refine::refine(RefineRequest {
        system_prompt: CLEANUP_PROMPT.into(),
        transcript: MESSY_TRANSCRIPT.into(),
        llm: LlmSettings {
            preset,
            model,
            base_url: None,
            max_tokens: 2048,
        },
        api_key,
    })
    .await
    .expect("refine failed");

    println!("\nin  : {MESSY_TRANSCRIPT}");
    println!("out : {cleaned}");
    println!("took: {:.1}s\n", started.elapsed().as_secs_f32());

    assert!(!cleaned.trim().is_empty(), "cleanup returned nothing");
    // The point of the pass is punctuation and capitalisation, and English technical
    // terms must survive untranslated.
    assert!(cleaned.contains('.'), "no sentence punctuation was added");
    assert!(cleaned.contains("deploy"), "the term 'deploy' was not preserved");
    assert!(cleaned.contains("server"), "the term 'server' was not preserved");
}
