//! ElevenLabs text-to-speech — the one provider here that needs a credential nobody
//! already has saved. Priced roughly 20x Soniox; for anyone who wants the better voice
//! quality and doesn't mind paying for it.

use super::{Audio, SpeechRequest, TtsModel, Voice};
use anyhow::{anyhow, Context, Result};
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const SAMPLE_RATE: u32 = 24_000;

pub async fn synthesize(api_key: &str, model: &str, req: &SpeechRequest) -> Result<Audio> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    let voice_id = if req.voice.is_empty() {
        // Rachel — ElevenLabs' long-standing default demo voice, used only when the
        // user has not picked one yet in the settings UI.
        "21m00Tcm4TlvDq8ikWAM"
    } else {
        req.voice.as_str()
    };
    let model = if model.is_empty() { "eleven_flash_v2_5" } else { model };

    let url = format!(
        "https://api.elevenlabs.io/v1/text-to-speech/{voice_id}?output_format=pcm_{SAMPLE_RATE}"
    );
    let body = serde_json::json!({
        "text": req.text,
        "model_id": model,
        // ElevenLabs' documented speed range is 0.7-1.2, narrower than the shared
        // slider's 0.5-2.0 — clamp rather than send a value the vendor would reject.
        "voice_settings": { "speed": req.speed.clamp(0.7, 1.2) },
    });

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("calling ElevenLabs text-to-speech")?;

    let status = response.status();
    if !status.is_success() {
        let payload: serde_json::Value = response.json().await.unwrap_or_default();
        let message = payload
            .pointer("/detail/message")
            .and_then(|m| m.as_str())
            .or_else(|| payload.pointer("/detail").and_then(|d| d.as_str()))
            .unwrap_or("unknown error");
        return Err(anyhow!("ElevenLabs TTS {status}: {message}"));
    }

    let bytes = response.bytes().await.context("reading ElevenLabs audio response")?;
    let pcm = bytes
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]))
        .collect();
    Ok(Audio { pcm, sample_rate: SAMPLE_RATE })
}

/// ElevenLabs splits this across two sources: the model list is fixed documentation,
/// the voice list is per-account (custom and cloned voices), so the voices are fetched
/// once and attached to every model.
pub async fn catalog(api_key: &str) -> Result<Vec<TtsModel>> {
    let voices = list_voices(api_key).await?;
    let models = [
        ("eleven_flash_v2_5", "Flash v2.5 (fastest)"),
        ("eleven_turbo_v2_5", "Turbo v2.5"),
        ("eleven_multilingual_v2", "Multilingual v2 (best quality)"),
    ];
    Ok(models
        .into_iter()
        .map(|(id, label)| TtsModel {
            id: id.to_string(),
            label: label.to_string(),
            voices: voices.clone(),
            // ElevenLabs' documented `voice_settings.speed` range.
            speed_min: 0.7,
            speed_max: 1.2,
            supports_speed: true,
        })
        .collect())
}

async fn list_voices(api_key: &str) -> Result<Vec<Voice>> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    let response = client
        .get("https://api.elevenlabs.io/v2/voices")
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("listing ElevenLabs voices")?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(anyhow!("ElevenLabs voice list {status}: {text}"));
    }

    let payload: serde_json::Value = response.json().await.context("reading ElevenLabs voice list")?;
    let voices = payload
        .get("voices")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| {
                    let id = v.get("voice_id")?.as_str()?;
                    let name = v.get("name")?.as_str()?;
                    Some(Voice {
                        id: id.to_string(),
                        label: name.to_string(),
                        language: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(voices)
}
