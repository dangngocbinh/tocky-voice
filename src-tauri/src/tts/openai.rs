//! OpenAI text-to-speech (`gpt-4o-mini-tts` and siblings). Reuses the same key already
//! saved for AI cleanup — the OpenAI preset's `secret_key` in `settings/defaults.rs`.

use super::{Audio, SpeechRequest};
use anyhow::{anyhow, Context, Result};
use std::time::Duration;

const ENDPOINT: &str = "https://api.openai.com/v1/audio/speech";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// PCM output is 24 kHz, 16-bit, mono, little-endian, with no header — documented
/// behaviour of `response_format: "pcm"`, not something the response states per call.
const SAMPLE_RATE: u32 = 24_000;

pub async fn synthesize(api_key: &str, model: &str, req: &SpeechRequest) -> Result<Audio> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    let body = serde_json::json!({
        "model": model,
        "input": req.text,
        "voice": if req.voice.is_empty() { "alloy" } else { req.voice.as_str() },
        "response_format": "pcm",
        // OpenAI's documented range is 0.25-4.0; the settings UI already keeps the
        // shared speed slider inside that window.
        "speed": req.speed,
    });

    let response = client
        .post(ENDPOINT)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .context("calling OpenAI text-to-speech")?;

    let status = response.status();
    if !status.is_success() {
        let payload: serde_json::Value = response.json().await.unwrap_or_default();
        let message = payload
            .pointer("/error/message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error")
            .to_string();
        return Err(anyhow!("OpenAI TTS {status}: {message}"));
    }

    let bytes = response.bytes().await.context("reading OpenAI audio response")?;
    let pcm = bytes
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]))
        .collect();
    Ok(Audio { pcm, sample_rate: SAMPLE_RATE })
}

/// OpenAI's TTS models and voices are a small fixed catalogue rather than a queryable
/// endpoint — listed by name in their docs, not fetched live. Verified working against
/// the live `/v1/audio/speech` endpoint with `gpt-4o-mini-tts` + `alloy` + `pcm`.
pub fn catalog() -> Vec<super::TtsModel> {
    const VOICES: &[&str] = &[
        "alloy", "ash", "ballad", "coral", "echo", "fable", "onyx", "nova", "sage", "shimmer",
    ];
    vec![
        super::TtsModel::fixed("gpt-4o-mini-tts", "GPT-4o mini TTS", VOICES, Some((0.25, 4.0))),
        super::TtsModel::fixed("tts-1", "TTS-1 (fast)", VOICES, Some((0.25, 4.0))),
        super::TtsModel::fixed("tts-1-hd", "TTS-1 HD", VOICES, Some((0.25, 4.0))),
    ]
}
