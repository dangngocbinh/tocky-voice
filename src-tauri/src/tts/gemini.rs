//! Gemini text-to-speech.
//!
//! **Deliberately does not go through `refine::resolve_endpoint`.** The `gemini` LLM
//! preset in `settings/defaults.rs` points at
//! `https://generativelanguage.googleapis.com/v1beta/openai` — the OpenAI-compatible
//! chat layer, which has no TTS route at all. Text-to-speech only exists on Google's
//! native endpoint, `https://generativelanguage.googleapis.com/v1beta`. Reusing the
//! chat preset's base URL here would 404 in a way that gives no hint why, so this file
//! keeps its own endpoint constant instead.

use super::{Audio, SpeechRequest, TtsModel};
use anyhow::{anyhow, Context, Result};
use base64::Engine as _;
use std::time::Duration;

/// Native Gemini endpoint — see the module doc for why this is not `resolve_endpoint`.
const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// `inlineData` comes back as 24 kHz, 16-bit, mono PCM — documented Gemini TTS output,
/// not something the response states per call.
const SAMPLE_RATE: u32 = 24_000;

pub async fn synthesize(api_key: &str, model: &str, req: &SpeechRequest) -> Result<Audio> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    // Gemini's TTS request has no speed field at all — unlike Soniox/OpenAI/ElevenLabs,
    // which all take it as a synthesis parameter. The only lever here is asking in
    // words, which is unreliable enough not to promise a specific outcome; the settings
    // UI dims the speed slider for this provider instead of pretending it does anything
    // (`tts::supports_speed`).
    let text = if (req.speed - 1.0).abs() > 0.05 {
        let pace = if req.speed < 1.0 { "chậm rãi" } else { "nhanh" };
        format!("Đọc với tốc độ {pace}: {}", req.text)
    } else {
        req.text.clone()
    };

    let voice = if req.voice.is_empty() { "Kore" } else { req.voice.as_str() };
    let body = serde_json::json!({
        "contents": [{ "parts": [{ "text": text }] }],
        "generationConfig": {
            "responseModalities": ["AUDIO"],
            "speechConfig": {
                "voiceConfig": { "prebuiltVoiceConfig": { "voiceName": voice } }
            }
        }
    });

    // The key goes in a header, not the `?key=` query string the docs lead with —
    // `x-goog-api-key` is Google's documented alternative, and a header never ends up
    // inside the URL a failed `reqwest` call embeds in its own error message. A
    // request-construction failure (bad URL, TLS error, timeout) surfacing that URL to
    // the UI/log would otherwise leak the key through `anyhow::Context` + `to_err`.
    let url = format!("{BASE_URL}/models/{model}:generateContent");
    let response = client
        .post(&url)
        .header("x-goog-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("calling Gemini text-to-speech")?;

    let status = response.status();
    let payload: serde_json::Value = response.json().await.context("reading Gemini response")?;
    if !status.is_success() {
        let message = payload
            .pointer("/error/message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        return Err(anyhow!("Gemini TTS {status}: {message}"));
    }

    let base64_audio = payload
        .pointer("/candidates/0/content/parts/0/inlineData/data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Gemini TTS response had no audio data"))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_audio)
        .context("decoding Gemini audio")?;
    let pcm = bytes
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]))
        .collect();
    Ok(Audio { pcm, sample_rate: SAMPLE_RATE })
}

/// Gemini's prebuilt voice set — a small fixed catalogue documented by Google, not a
/// queryable endpoint. `None` for the speed range: this endpoint takes no speed
/// parameter at all (see `synthesize`), so the UI hides the slider rather than
/// offering one that does nothing.
pub fn catalog() -> Vec<TtsModel> {
    const VOICES: &[&str] =
        &["Kore", "Puck", "Charon", "Fenrir", "Aoede", "Leda", "Orus", "Zephyr"];
    vec![
        TtsModel::fixed("gemini-2.5-flash-preview-tts", "Gemini 2.5 Flash TTS", VOICES, None),
        TtsModel::fixed("gemini-2.5-pro-preview-tts", "Gemini 2.5 Pro TTS", VOICES, None),
    ]
}
