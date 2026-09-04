//! Vbee text-to-speech — Vietnamese-focused, Vietnamese company (VTI). Verified against
//! their public docs at `api-docs.vbee.vn` (the "Realtime API", their sync mode meant
//! for short text — a good match for the already-short chunks `tts/chunker.rs` hands
//! every provider: Vbee's own 300-character request limit sits above `chunker`'s
//! 280-character rest-of-article ceiling).
//!
//! **Auth is two parts**, unlike every other provider here: an `App-Id` header plus a
//! bearer `token`. The token is the vault secret (`secrets::get_key("vbee")`); the app
//! id is a plain settings field, `TtsSettings.vbee_app_id` — see the comment there for
//! why it doesn't need vault storage.

use super::{Audio, SpeechRequest, TtsModel, Voice};
use anyhow::{anyhow, Context, Result};
use std::time::Duration;

const ENDPOINT: &str = "https://api.vbee.vn/v1/tts";
/// A different host from the synthesis endpoint — confirmed against Vbee's docs, not a
/// typo.
const VOICES_ENDPOINT: &str = "https://vbee.vn/api/public/v1/voices";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// One of Vbee's documented valid rates (`8000, 16000, 22050, 24000, 32000, 44100,
/// 48000`), matching every other provider here so chunks mix cleanly through one sink.
const SAMPLE_RATE: u32 = 24_000;

/// Set once the account is seen to reject `mode: "sync"`. Vbee gates its Realtime API
/// behind a paid tier ("This feature is not supported in user package"), and there is
/// no capability endpoint to ask up front — so the first synthesis discovers it, and
/// every one after goes straight to the batch path rather than paying for a doomed
/// round trip per chunk.
static SYNC_UNSUPPORTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub async fn synthesize(app_id: &str, token: &str, req: &SpeechRequest) -> Result<Audio> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    if !SYNC_UNSUPPORTED.load(std::sync::atomic::Ordering::Relaxed) {
        match synthesize_sync(&client, app_id, token, req).await {
            Ok(audio) => return Ok(audio),
            Err(e) if is_package_limit(&e) => {
                log::info!("Vbee: realtime mode is not in this account's package; using batch mode");
                SYNC_UNSUPPORTED.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            // Not remembered, unlike the package limit: this one is a property of the
            // chosen voice, not the account, so another voice may well take the fast
            // path. Batch handles it because it names no rate at all.
            Err(e) if is_sample_rate_rejection(&e) => {
                log::info!("Vbee: this voice does not offer {SAMPLE_RATE} Hz; using batch mode");
            }
            Err(e) => return Err(e),
        }
    }
    synthesize_batch(&client, app_id, token, req).await
}

fn is_package_limit(error: &anyhow::Error) -> bool {
    error.to_string().contains("not supported in user package")
}

fn is_sample_rate_rejection(error: &anyhow::Error) -> bool {
    error.to_string().contains("sample rate")
}

/// A real `code`, not the "HN - Ngọc Huyền" display name the TTS docs use in their
/// example — the voices endpoint returns codes in this shape, and that is what
/// `voiceCode` actually wants. Only a fallback: the UI populates this from the
/// catalogue as soon as credentials are saved.
fn voice_code(req: &SpeechRequest) -> &str {
    if req.voice.is_empty() {
        "hn_female_ngochuyen_full_48k-fhg"
    } else {
        req.voice.as_str()
    }
}

/// Vbee's documented range is 0.25–1.9, narrower than the shared slider's at the top
/// end — clamp rather than send a value it would reject.
fn speed(req: &SpeechRequest) -> f32 {
    req.speed.clamp(0.25, 1.9)
}

/// Realtime API: one request, raw audio straight back. Only available on packages that
/// include it — see [`SYNC_UNSUPPORTED`].
async fn synthesize_sync(
    client: &reqwest::Client,
    app_id: &str,
    token: &str,
    req: &SpeechRequest,
) -> Result<Audio> {
    // PCM, not WAV: this account's package rejects `wav` in sync mode outright
    // ("This feature is not supported in user package") while accepting `pcm`. Raw PCM
    // has no header, so unlike the batch path this one must name a sample rate — and a
    // voice that does not offer this rate is exactly what pushes the call over to
    // batch below, where the rate is left to the vendor.
    let body = serde_json::json!({
        "text": req.text,
        "mode": "sync",
        "voiceCode": voice_code(req),
        "outputFormat": "pcm",
        "sampleRate": SAMPLE_RATE,
        "speed": speed(req),
    });

    let response = client
        .post(ENDPOINT)
        .bearer_auth(token)
        .header("App-Id", app_id)
        .json(&body)
        .send()
        .await
        .context("calling Vbee text-to-speech")?;

    let status = response.status();
    if !status.is_success() {
        return Err(anyhow!("Vbee TTS {status}: {}", error_message(response).await));
    }

    let bytes = response.bytes().await.context("reading Vbee audio response")?;
    let pcm = bytes
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]))
        .collect();
    Ok(Audio { pcm, sample_rate: SAMPLE_RATE })
}

/// How long to wait for a batch job before giving up, and how often to ask. Batch is
/// meant for whole documents, but the chunks fed to it here are a couple of sentences,
/// so in practice it finishes in a second or two.
const BATCH_TIMEOUT: Duration = Duration::from_secs(60);
const BATCH_POLL_INTERVAL: Duration = Duration::from_millis(400);

/// Batch API: submit, poll for completion, download the finished file.
///
/// `webhookUrl` is a required field even though nothing here can receive a webhook —
/// a desktop app has no public URL — so a deliberately unreachable one is sent and the
/// result is polled from `/v1/tts/requests/{id}` instead.
///
/// Asks for WAV rather than the default MP3: `hound` is already a dependency and can
/// decode WAV, whereas MP3 would mean pulling in a decoder for one provider's fallback
/// path.
async fn synthesize_batch(
    client: &reqwest::Client,
    app_id: &str,
    token: &str,
    req: &SpeechRequest,
) -> Result<Audio> {
    // No `sampleRate`: voices differ in which rates they accept ("This voice only
    // support sample rate of [8000,16000,22050]"), and there is no per-voice field in
    // the catalogue to look it up from. Naming none lets each voice use its own, and
    // the WAV header then says what that was — strictly better than any table.
    let body = serde_json::json!({
        "text": req.text,
        "mode": "async",
        "voiceCode": voice_code(req),
        "outputFormat": "wav",
        "speed": speed(req),
        "webhookUrl": "https://localhost/unused",
    });

    let response = client
        .post(ENDPOINT)
        .bearer_auth(token)
        .header("App-Id", app_id)
        .json(&body)
        .send()
        .await
        .context("submitting Vbee batch job")?;

    let status = response.status();
    if !status.is_success() {
        return Err(anyhow!("Vbee TTS {status}: {}", error_message(response).await));
    }

    let payload: serde_json::Value = response.json().await.context("reading Vbee job id")?;
    let request_id = payload
        .get("requestId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Vbee batch response carried no requestId"))?
        .to_string();

    let audio_link = poll_for_audio(client, app_id, token, &request_id).await?;
    let bytes = client
        .get(&audio_link)
        .send()
        .await
        .context("downloading Vbee audio")?
        .bytes()
        .await
        .context("reading Vbee audio")?;
    decode_wav(&bytes)
}

async fn poll_for_audio(
    client: &reqwest::Client,
    app_id: &str,
    token: &str,
    request_id: &str,
) -> Result<String> {
    let deadline = std::time::Instant::now() + BATCH_TIMEOUT;
    loop {
        let response = client
            .get(format!("{ENDPOINT}/requests/{request_id}"))
            .bearer_auth(token)
            .header("App-Id", app_id)
            .send()
            .await
            .context("checking the Vbee job")?;

        let status = response.status();
        if !status.is_success() {
            return Err(anyhow!("Vbee job status {status}: {}", error_message(response).await));
        }
        let payload: serde_json::Value =
            response.json().await.context("reading the Vbee job status")?;

        match payload.get("status").and_then(|s| s.as_str()) {
            Some("COMPLETED") => {
                return payload
                    .get("audioLink")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .ok_or_else(|| anyhow!("Vbee reported the job complete but sent no audioLink"))
            }
            Some("FAILED") => {
                let detail = payload
                    .get("error_message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("no reason given");
                return Err(anyhow!("Vbee could not synthesize this text: {detail}"));
            }
            _ => {}
        }

        if std::time::Instant::now() >= deadline {
            return Err(anyhow!(
                "Vbee did not finish within {}s",
                BATCH_TIMEOUT.as_secs()
            ));
        }
        tokio::time::sleep(BATCH_POLL_INTERVAL).await;
    }
}

/// Decodes a WAV file into the 16-bit mono PCM the player expects, downmixing stereo
/// and carrying the file's own sample rate through rather than assuming one.
fn decode_wav(bytes: &[u8]) -> Result<Audio> {
    let mut reader = hound::WavReader::new(std::io::Cursor::new(bytes))
        .context("Vbee returned something that is not a WAV file")?;
    let spec = reader.spec();

    let samples: Vec<i16> = match spec.sample_format {
        hound::SampleFormat::Int if spec.bits_per_sample == 16 => {
            reader.samples::<i16>().filter_map(Result::ok).collect()
        }
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .filter_map(Result::ok)
            .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .collect(),
        _ => {
            return Err(anyhow!(
                "unsupported WAV format from Vbee: {} bit {:?}",
                spec.bits_per_sample,
                spec.sample_format
            ))
        }
    };

    let pcm = if spec.channels > 1 {
        let channels = spec.channels as usize;
        samples
            .chunks(channels)
            .map(|frame| (frame.iter().map(|s| *s as i32).sum::<i32>() / channels as i32) as i16)
            .collect()
    } else {
        samples
    };

    Ok(Audio { pcm, sample_rate: spec.sample_rate })
}

/// The vendor's own words for a failed response, tried across the shapes Vbee uses.
async fn error_message(response: reqwest::Response) -> String {
    let text = response.text().await.unwrap_or_default();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return text;
    };
    for pointer in ["/error/message", "/error_message", "/message"] {
        if let Some(message) = value.pointer(pointer).and_then(|v| v.as_str()) {
            return message.to_string();
        }
    }
    text
}

/// Vbee has no model selector — one synthesis endpoint, voices chosen by `voiceCode` —
/// so this reports a single synthetic entry carrying the account's voice list.
pub async fn catalog(app_id: &str, token: &str) -> Result<Vec<TtsModel>> {
    Ok(vec![TtsModel {
        id: String::new(),
        label: "Vbee".to_string(),
        voices: list_voices(app_id, token).await?,
        // Vbee's documented `speed` range.
        speed_min: 0.25,
        speed_max: 1.9,
        supports_speed: true,
    }])
}

/// Largest page the endpoint allows; the default is 20, and Vbee ships far more voices
/// than that, so without this (and the cursor loop below) most of them are invisible.
const VOICES_PAGE_SIZE: u32 = 100;
/// Backstop against a cursor that never reports the last page — 20 pages is 2000
/// voices, well past any real catalogue.
const MAX_VOICE_PAGES: usize = 20;

async fn list_voices(app_id: &str, token: &str) -> Result<Vec<Voice>> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    let mut voices = Vec::new();
    let mut cursor: Option<String> = None;

    for _ in 0..MAX_VOICE_PAGES {
        // `voiceOwnership` is documented as optional but the live endpoint rejects the
        // request without it (`"voiceOwnership" is required`). VBEE = the vendor's own
        // catalogue, which is what a user without cloned voices of their own wants.
        let mut request = client
            .get(VOICES_ENDPOINT)
            .query(&[("voiceOwnership", "VBEE")])
            .query(&[("limit", VOICES_PAGE_SIZE)])
            .bearer_auth(token)
            .header("App-Id", app_id);
        if let Some(cursor) = &cursor {
            request = request.query(&[("cursor", cursor)]);
        }

        let response = request.send().await.context("listing Vbee voices")?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Vbee voice list {status}: {text}"));
        }

        let payload: serde_json::Value =
            response.json().await.context("reading Vbee voice list")?;
        voices.extend(parse_voices(&payload));

        let has_next = payload
            .pointer("/result/pagination/has_next_page")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let next = payload
            .pointer("/result/pagination/next_cursor")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        match (has_next, next) {
            (true, Some(next)) if !next.is_empty() => cursor = Some(next),
            _ => break,
        }
    }

    Ok(voices)
}

fn parse_voices(payload: &serde_json::Value) -> Vec<Voice> {
    payload
        .pointer("/result/voices")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| {
                    let code = v.get("code")?.as_str()?;
                    let name = v.get("name")?.as_str()?;
                    let language = v
                        .get("language_code")
                        .and_then(|l| l.as_str())
                        .map(str::to_string);
                    let label = match v.get("gender").and_then(|g| g.as_str()) {
                        Some(gender) => format!("{name} ({gender})"),
                        None => name.to_string(),
                    };
                    Some(Voice { id: code.to_string(), label, language })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shape taken from Vbee's documented response: the voice id is `code`, and real
    /// codes look like `hn_female_ngochuyen_full_48k-fhg` rather than the display name.
    #[test]
    fn reads_the_voice_code_as_the_id_not_the_display_name() {
        let payload = serde_json::json!({
            "result": {
                "voices": [{
                    "code": "hn_female_ngochuyen_full_48k-fhg",
                    "name": "Ngọc Huyền",
                    "gender": "female",
                    "language_code": "vi-VN"
                }]
            },
            "status": 1
        });
        let voices = parse_voices(&payload);
        assert_eq!(voices[0].id, "hn_female_ngochuyen_full_48k-fhg");
        assert_eq!(voices[0].label, "Ngọc Huyền (female)");
        assert_eq!(voices[0].language.as_deref(), Some("vi-VN"));
    }

    #[test]
    fn an_unrecognised_shape_degrades_to_an_empty_list() {
        assert!(parse_voices(&serde_json::json!({ "status": 0 })).is_empty());
    }
}
