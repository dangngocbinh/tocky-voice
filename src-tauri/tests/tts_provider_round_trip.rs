//! Live round-trip against the real text-to-speech providers.
//!
//! These exercise the actual request bodies and response decoding — the part most
//! likely to be wrong when a wire format was inferred from documentation rather than
//! observed, and the part unit tests cannot cover. They need network access and a real
//! API key, so they are `#[ignore]`d by default:
//!
//! ```sh
//! SONIOX_API_KEY=... OPENAI_API_KEY=... \
//! cargo test --test tts_provider_round_trip -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Set `FVT_TTS_OUT=/tmp/tts` to also write each provider's audio out as a `.wav` you
//! can actually listen to — "the request succeeded" and "it produced speech" are not
//! the same claim, and only one of them is worth shipping on.

use tockyvoice_lib::settings::{defaults, TtsProviderKind, TtsSettings};
use tockyvoice_lib::tts::{self, SpeechRequest};

/// Vietnamese with full diacritics: the thing this app exists to read, and the first
/// thing a provider with shaky Vietnamese support mangles.
const SAMPLE: &str = "Xin chào, đây là giọng đọc thử tiếng Việt có dấu đầy đủ.";

fn settings_for(provider: TtsProviderKind, model: &str, voice: &str) -> TtsSettings {
    TtsSettings {
        provider,
        model: model.into(),
        voice: voice.into(),
        speed: 1.0,
        ..defaults::default_tts()
    }
}

/// Fails loudly on digital silence. A provider that returns a well-formed buffer of
/// zeros passes every "did the HTTP call work" check while producing nothing audible,
/// which is the exact failure this suite exists to catch.
fn assert_audible(audio: &tts::Audio, label: &str) {
    assert!(!audio.pcm.is_empty(), "{label}: no samples at all");
    let peak = audio.pcm.iter().map(|s| s.unsigned_abs() as u32).max().unwrap_or(0);
    let duration = audio.pcm.len() as f32 / audio.sample_rate as f32;
    println!(
        "{label}: {} samples @ {} Hz = {duration:.2}s, peak {peak}",
        audio.pcm.len(),
        audio.sample_rate
    );
    assert!(duration > 0.5, "{label}: {duration:.2}s is too short to be that sentence");
    assert!(peak > 1000, "{label}: peak {peak} is silence, not speech");

    if let Ok(dir) = std::env::var("FVT_TTS_OUT") {
        let _ = std::fs::create_dir_all(&dir);
        let path = format!("{dir}/{label}.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: audio.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).expect("creating wav");
        for sample in &audio.pcm {
            writer.write_sample(*sample).expect("writing sample");
        }
        writer.finalize().expect("finalizing wav");
        println!("{label}: wrote {path}");
    }
}

#[tokio::test]
#[ignore = "needs SONIOX_API_KEY and network"]
async fn soniox_speaks_vietnamese() {
    let key = std::env::var("SONIOX_API_KEY").expect("set SONIOX_API_KEY");

    // The catalogue drives the settings UI's dropdowns, so a shape change here is a
    // change to what the user can pick — assert the parse, not just the HTTP status.
    let settings = settings_for(TtsProviderKind::Soniox, "tts-rt-v1", "");
    let models = tts::catalog(&settings, &key).await.expect("listing models");
    assert!(!models.is_empty(), "no models returned");
    let model = models.iter().find(|m| m.id == "tts-rt-v1").expect("tts-rt-v1 missing");
    assert!(!model.voices.is_empty(), "model reported no voices");
    assert!(
        !model.voices.iter().any(|v| v.id == model.id),
        "the model id leaked into the voice list — see soniox::parse_catalog"
    );
    println!(
        "soniox: {} models, {} voices on {}, speed {}-{}",
        models.len(),
        model.voices.len(),
        model.id,
        model.speed_min,
        model.speed_max
    );

    let voice = model.voices[0].id.clone();
    let settings = settings_for(TtsProviderKind::Soniox, &model.id, &voice);
    let engine = tts::build_engine(&settings, key);
    let audio = tts::synthesize(
        &engine,
        &SpeechRequest { text: SAMPLE.into(), voice, speed: 1.0 },
    )
    .await
    .expect("synthesizing");
    assert_audible(&audio, "soniox");
}

#[tokio::test]
#[ignore = "needs OPENAI_API_KEY and network"]
async fn openai_speaks_vietnamese() {
    let key = std::env::var("OPENAI_API_KEY").expect("set OPENAI_API_KEY");
    let settings = settings_for(TtsProviderKind::OpenAi, "gpt-4o-mini-tts", "alloy");
    let engine = tts::build_engine(&settings, key);
    let audio = tts::synthesize(
        &engine,
        &SpeechRequest { text: SAMPLE.into(), voice: "alloy".into(), speed: 1.0 },
    )
    .await
    .expect("synthesizing");
    assert_audible(&audio, "openai");
}

#[tokio::test]
#[ignore = "needs GEMINI_API_KEY and network"]
async fn gemini_speaks_vietnamese() {
    let key = std::env::var("GEMINI_API_KEY").expect("set GEMINI_API_KEY");
    let settings = settings_for(TtsProviderKind::Gemini, "gemini-2.5-flash-preview-tts", "Kore");
    let engine = tts::build_engine(&settings, key);
    let audio = tts::synthesize(
        &engine,
        &SpeechRequest { text: SAMPLE.into(), voice: "Kore".into(), speed: 1.0 },
    )
    .await
    .expect("synthesizing");
    assert_audible(&audio, "gemini");
}

#[tokio::test]
#[ignore = "needs ELEVENLABS_API_KEY and network"]
async fn elevenlabs_speaks_vietnamese() {
    let key = std::env::var("ELEVENLABS_API_KEY").expect("set ELEVENLABS_API_KEY");
    let settings = settings_for(TtsProviderKind::ElevenLabs, "eleven_flash_v2_5", "");
    let models = tts::catalog(&settings, &key).await.expect("listing voices");
    let voice = models
        .first()
        .and_then(|m| m.voices.first())
        .map(|v| v.id.clone())
        .expect("no voices on the account");
    let engine = tts::build_engine(&settings, key);
    let audio = tts::synthesize(
        &engine,
        &SpeechRequest { text: SAMPLE.into(), voice, speed: 1.0 },
    )
    .await
    .expect("synthesizing");
    assert_audible(&audio, "elevenlabs");
}

/// Vbee needs both halves of its credential; `VBEE_APP_ID` is the header, the env key
/// is the bearer token.
#[tokio::test]
#[ignore = "needs VBEE_APP_ID + VBEE_TOKEN and network"]
async fn vbee_speaks_vietnamese() {
    let app_id = std::env::var("VBEE_APP_ID").expect("set VBEE_APP_ID");
    let token = std::env::var("VBEE_TOKEN").expect("set VBEE_TOKEN");
    let mut settings = settings_for(TtsProviderKind::Vbee, "", "");
    settings.vbee_app_id = app_id;

    let models = tts::catalog(&settings, &token).await.expect("listing voices");
    let voice = models
        .first()
        .and_then(|m| m.voices.first())
        .map(|v| v.id.clone())
        .expect("no voices returned");
    println!("vbee: {} voices, first = {voice}", models[0].voices.len());

    let engine = tts::build_engine(&settings, token);
    let audio = tts::synthesize(
        &engine,
        &SpeechRequest { text: SAMPLE.into(), voice, speed: 1.0 },
    )
    .await
    .expect("synthesizing");
    assert_audible(&audio, "vbee");
}

/// The chunker feeds every provider, so a chunk it produces has to be something the
/// provider accepts — including the short first chunk and any hard-wrapped long one.
#[tokio::test]
#[ignore = "needs SONIOX_API_KEY and network"]
async fn every_chunk_of_a_real_article_synthesizes() {
    let key = std::env::var("SONIOX_API_KEY").expect("set SONIOX_API_KEY");
    let article = "Trí tuệ nhân tạo đang thay đổi cách chúng ta làm việc mỗi ngày. \
        Các công cụ như ChatGPT, Claude hay Gemini giúp tự động hoá nhiều tác vụ lặp đi lặp lại. \
        Tuy nhiên, việc áp dụng cần cân nhắc kỹ về bảo mật dữ liệu và chi phí vận hành.";
    let chunks = tts::chunker::split(article, 120, 280);
    assert!(chunks.len() >= 2, "expected the article to split: {chunks:?}");

    let settings = settings_for(TtsProviderKind::Soniox, "tts-rt-v1", "Maya");
    let engine = tts::build_engine(&settings, key);
    for (i, chunk) in chunks.iter().enumerate() {
        let audio = tts::synthesize(
            &engine,
            &SpeechRequest { text: chunk.clone(), voice: "Maya".into(), speed: 1.0 },
        )
        .await
        .unwrap_or_else(|e| panic!("chunk {i} ({} chars) failed: {e:#}", chunk.chars().count()));
        assert_audible(&audio, &format!("soniox-chunk-{i}"));
    }
}

/// Vbee's voices disagree about which sample rates and modes they support, and the
/// catalogue exposes neither. This walks a spread of them through the real code path,
/// which is the only way to know the sync→batch fallback actually covers the gaps.
#[tokio::test]
#[ignore = "needs VBEE_APP_ID + VBEE_TOKEN and network"]
async fn vbee_handles_every_kind_of_voice() {
    let app_id = std::env::var("VBEE_APP_ID").expect("set VBEE_APP_ID");
    let token = std::env::var("VBEE_TOKEN").expect("set VBEE_TOKEN");
    let mut settings = settings_for(TtsProviderKind::Vbee, "", "");
    settings.vbee_app_id = app_id;

    let models = tts::catalog(&settings, &token).await.expect("listing voices");
    let all = &models[0].voices;
    // A spread rather than the whole 400+: enough to hit both the Vietnamese voices and
    // the imported English ones, without a test that costs a fortune to run.
    let sample: Vec<_> = all.iter().step_by(all.len() / 6).take(6).collect();

    let engine = tts::build_engine(&settings, token);
    let mut failures = Vec::new();
    for voice in sample {
        let result = tts::synthesize(
            &engine,
            &SpeechRequest { text: "Xin chào.".into(), voice: voice.id.clone(), speed: 1.0 },
        )
        .await;
        match result {
            Ok(audio) => println!(
                "  ok   {} -> {} Hz, {:.2}s",
                voice.id,
                audio.sample_rate,
                audio.pcm.len() as f32 / audio.sample_rate as f32
            ),
            Err(e) => {
                println!("  FAIL {} -> {e:#}", voice.id);
                failures.push(format!("{}: {e:#}", voice.id));
            }
        }
    }
    assert!(failures.is_empty(), "some voices could not be synthesized:\n{}", failures.join("\n"));
}
