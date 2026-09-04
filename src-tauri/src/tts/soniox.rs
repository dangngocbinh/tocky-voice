//! Soniox text-to-speech — the default provider, because it takes the exact API key
//! already saved for speech recognition and nobody has to sign up for anything new.
//!
//! **REST, not websocket.** `chunker.rs` already splits the text into short pieces
//! before any of this runs, so there is no long single request to stream partial audio
//! out of — each chunk is already a small, fast request. A websocket only pays off when
//! it can start returning audio before the *whole* input is synthesized, and here the
//! input per call is already ~120-280 characters. REST is simpler, has no connection
//! lifecycle to manage per chunk, and was confirmed against Soniox's own docs
//! (`https://soniox.com/docs/tts/rest-api/generate-speech`) rather than the realtime
//! websocket doc, which does not fully specify its wire format publicly.
//!
//! Verified against public documentation only — no live account was available to
//! `curl` against while writing this, which the plan calls for as the stronger check.
//! If Soniox rejects a request shape here, the vendor's own error message comes back
//! verbatim (see `synthesize`'s error path) rather than a silent wrong-format failure.

use super::{Audio, SpeechRequest, TtsModel, Voice};
use anyhow::{anyhow, Context, Result};
use std::time::Duration;

const ENDPOINT: &str = "https://tts-rt.soniox.com/tts";
const MODELS_ENDPOINT: &str = "https://api.soniox.com/v1/tts-models";

/// Same ceiling `refine::REQUEST_TIMEOUT` uses for the AI cleanup pass — past this the
/// person listening has already given up and closed the player.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// One of the sample rates Soniox's docs list as valid (`8000, 16000, 24000, 44100,
/// 48000`); 24 kHz matches what OpenAI/Gemini/ElevenLabs all hand back too, so chunks
/// from any provider mix cleanly through the same `rodio` sink.
const SAMPLE_RATE: u32 = 24_000;

pub async fn synthesize(api_key: &str, model: &str, req: &SpeechRequest) -> Result<Audio> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    // Soniox's own docs example always includes `voice` (`"Adrian"`); nothing public
    // says it is optional, so — like OpenAI's `alloy` and Gemini's `Kore` below —
    // default to it rather than omitting the field when no voice has been picked yet.
    let voice = if req.voice.is_empty() { "Adrian" } else { req.voice.as_str() };
    // Confirmed required by a live 400 ("Missing required field: language") — not
    // documented as optional anywhere public. This app is Vietnamese-first, so that is
    // the sane default rather than "en"; a mode's AI pass may itself translate into
    // English text, but the voice/language pairing is a `tts` setting, not per-request,
    // so there is nowhere better to source this without a larger redesign.
    let body = serde_json::json!({
        "text": req.text,
        "model": model,
        "voice": voice,
        "language": "vi",
        "audio_format": "pcm_s16le",
        "sample_rate": SAMPLE_RATE,
        // The live `/v1/tts-models` reports 0.7–1.3 for every current model. The UI
        // already bounds its slider to the selected model's range; this clamp is the
        // backstop for a settings file carrying a value from a different provider.
        "speed": req.speed.clamp(0.7, 1.3),
    });

    let response = client
        .post(ENDPOINT)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .context("calling Soniox text-to-speech")?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        let message = extract_vendor_message(&text).unwrap_or(text);
        return Err(anyhow!("Soniox TTS {status}: {message}"));
    }

    let bytes = response.bytes().await.context("reading Soniox audio response")?;
    let pcm = bytes_to_pcm16(&bytes);
    Ok(Audio { pcm, sample_rate: SAMPLE_RATE })
}

/// The full model catalogue, each model carrying its own voice list and speed range.
///
/// Shape verified against the live endpoint (see the fixture in this module's tests):
/// `{"models":[{"id","name","voices":[{"id","description","gender"}],"languages":[…],
/// "speed_min","speed_max","supports_speed_adjustment"}]}`. Note voices carry **no**
/// `name` field — the id (`"Maya"`, `"Adrian"`) is the display name — and the model
/// objects are the ones with `name`, which is what an earlier "walk the tree for
/// anything with id+name" implementation latched onto, offering the three *models* as
/// if they were voices and producing `Invalid voice 'tts-rt-v1' for model 'tts-rt-v1'`.
pub async fn catalog(api_key: &str) -> Result<Vec<TtsModel>> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    let response = client
        .get(MODELS_ENDPOINT)
        .bearer_auth(api_key)
        .send()
        .await
        .context("listing Soniox TTS models")?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        let message = extract_vendor_message(&text).unwrap_or(text);
        return Err(anyhow!("Soniox TTS models {status}: {message}"));
    }

    let payload: serde_json::Value = response.json().await.context("reading Soniox models list")?;
    Ok(parse_catalog(&payload))
}

fn parse_catalog(payload: &serde_json::Value) -> Vec<TtsModel> {
    payload
        .get("models")
        .and_then(|m| m.as_array())
        .map(|models| {
            models
                .iter()
                .filter_map(|m| {
                    let id = m.get("id")?.as_str()?.to_string();
                    let label = m.get("name").and_then(|n| n.as_str()).unwrap_or(&id).to_string();
                    let voices = m
                        .get("voices")
                        .and_then(|v| v.as_array())
                        .map(|list| {
                            list.iter()
                                .filter_map(|v| {
                                    let vid = v.get("id")?.as_str()?.to_string();
                                    // Voices have no `name`; the id is the display
                                    // name, with gender as the only extra hint worth
                                    // surfacing in a narrow dropdown.
                                    let label = match v.get("gender").and_then(|g| g.as_str()) {
                                        Some(g) => format!("{vid} ({g})"),
                                        None => vid.clone(),
                                    };
                                    Some(Voice { id: vid, label, language: None })
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(TtsModel {
                        id,
                        label,
                        voices,
                        // The API states the real range per model (0.7–1.3 at the time
                        // of writing) — taken from it rather than hardcoded, so the
                        // slider cannot offer a value the vendor rejects.
                        speed_min: m.get("speed_min").and_then(|s| s.as_f64()).unwrap_or(1.0) as f32,
                        speed_max: m.get("speed_max").and_then(|s| s.as_f64()).unwrap_or(1.0) as f32,
                        supports_speed: m
                            .get("supports_speed_adjustment")
                            .and_then(|s| s.as_bool())
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Soniox error responses are JSON with a message field in at least one of these
/// shapes elsewhere in their API (`stt/soniox.rs` sees `error_message`); tried in
/// order, falling back to the raw body so nothing is ever silently dropped.
fn extract_vendor_message(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    for pointer in ["/error_message", "/message", "/error/message", "/detail"] {
        if let Some(msg) = value.pointer(pointer).and_then(|v| v.as_str()) {
            return Some(msg.to_string());
        }
    }
    None
}

/// Little-endian 16-bit PCM bytes to samples, dropping a trailing odd byte rather than
/// panicking on it — better a clipped last sample than a crashed read-aloud session.
fn bytes_to_pcm16(bytes: &[u8]) -> Vec<i16> {
    bytes
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_the_vendor_error_message_when_present() {
        assert_eq!(
            extract_vendor_message(r#"{"error_message":"invalid api key"}"#),
            Some("invalid api key".to_string())
        );
        assert_eq!(
            extract_vendor_message(r#"{"message":"bad request"}"#),
            Some("bad request".to_string())
        );
    }

    #[test]
    fn falls_back_to_none_on_unparseable_bodies() {
        assert_eq!(extract_vendor_message("not json"), None);
    }

    #[test]
    fn converts_little_endian_pcm_bytes_to_samples() {
        let bytes = [0x00, 0x00, 0xFF, 0x7F, 0x00, 0x80];
        assert_eq!(bytes_to_pcm16(&bytes), vec![0, i16::MAX, i16::MIN]);
    }

    #[test]
    fn drops_a_trailing_odd_byte_instead_of_panicking() {
        assert_eq!(bytes_to_pcm16(&[0x01, 0x00, 0xFF]), vec![1]);
    }

    /// Trimmed copy of a real `GET /v1/tts-models` response — the field names here are
    /// captured from the live endpoint, not guessed.
    fn live_response_fixture() -> serde_json::Value {
        serde_json::json!({
            "models": [
                {
                    "id": "tts-rt-v1",
                    "aliased_model_id": null,
                    "name": "TTS RT v1",
                    "voices": [
                        { "id": "Maya", "description": "A steady, clear voice…", "gender": "female" },
                        { "id": "Adrian", "description": "A deep, focused male voice…", "gender": "male" }
                    ],
                    "languages": [{ "code": "vi", "name": "Vietnamese" }],
                    "supports_speed_adjustment": true,
                    "speed_min": 0.7,
                    "speed_max": 1.3
                }
            ]
        })
    }

    /// Regression: an earlier implementation walked the whole tree for any object with
    /// `id` + `name`, which matched the *model* objects (voices have no `name`) and
    /// offered "TTS RT v1" as a voice — producing a live
    /// `Invalid voice 'tts-rt-v1' for model 'tts-rt-v1'` on every read.
    #[test]
    fn voices_come_from_the_voices_array_not_the_model_objects() {
        let models = parse_catalog(&live_response_fixture());
        assert_eq!(models.len(), 1);
        let voice_ids: Vec<&str> = models[0].voices.iter().map(|v| v.id.as_str()).collect();
        assert_eq!(voice_ids, vec!["Maya", "Adrian"]);
        assert!(
            !voice_ids.contains(&"tts-rt-v1"),
            "the model id must never be offered as a voice"
        );
    }

    #[test]
    fn a_voice_label_falls_back_to_its_id_because_voices_have_no_name_field() {
        let models = parse_catalog(&live_response_fixture());
        assert_eq!(models[0].voices[0].label, "Maya (female)");
    }

    /// The slider must not be able to request a speed the vendor rejects, so the range
    /// is read from the API rather than hardcoded.
    #[test]
    fn the_speed_range_comes_from_the_model_not_a_hardcoded_guess() {
        let models = parse_catalog(&live_response_fixture());
        assert_eq!(models[0].speed_min, 0.7);
        assert_eq!(models[0].speed_max, 1.3);
        assert!(models[0].supports_speed);
    }

    #[test]
    fn an_unrecognised_shape_degrades_to_an_empty_list_not_an_error() {
        assert!(parse_catalog(&serde_json::json!({ "something_else": true })).is_empty());
    }
}
