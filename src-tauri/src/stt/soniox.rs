//! Soniox realtime. Best of the three for Vietnamese/English code-switching because
//! it accepts several language hints at once instead of one fixed language.
//!
//! Protocol: JSON config frame (carrying the API key — Soniox has no auth header),
//! then raw PCM frames, then an empty text frame to signal end of audio.

use super::{request_with_header, SttEvent, WsProtocol};
use crate::settings::SttSettings;
use anyhow::{anyhow, Result};
use serde_json::json;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::Message;

const ENDPOINT: &str = "wss://stt-rt.soniox.com/transcribe-websocket";

pub struct Soniox {
    api_key: String,
    model: String,
    language_hints: Vec<String>,
}

impl Soniox {
    pub fn new(settings: &SttSettings, api_key: String) -> Self {
        Self {
            api_key,
            model: settings.soniox_model.clone(),
            language_hints: settings.language_hints.clone(),
        }
    }
}

impl WsProtocol for Soniox {
    fn request(&self) -> Result<Request<()>> {
        // Soniox authenticates in the config frame, so no header is needed. Reuse the
        // shared builder with a harmless User-Agent to keep one request-construction path.
        request_with_header(ENDPOINT, "user-agent", "tockyvoice")
    }

    fn init_message(&self) -> Option<Message> {
        let config = json!({
            "api_key": self.api_key,
            "model": self.model,
            "audio_format": "pcm_s16le",
            "sample_rate": super::super::audio::capture::TARGET_SAMPLE_RATE,
            "num_channels": 1,
            "language_hints": self.language_hints,
            "enable_language_identification": true,
            "enable_endpoint_detection": true,
        });
        Some(Message::Text(config.to_string()))
    }

    fn finish_message(&self) -> Message {
        // An empty text frame is Soniox's "no more audio" signal.
        Message::Text(String::new())
    }

    fn parse(&mut self, text: &str) -> Result<Vec<SttEvent>> {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
            return Ok(Vec::new());
        };
        // Soniox reports a bad key, an unknown model or an exhausted balance as an
        // ordinary text frame and then hangs up. Swallowing it left the user with a
        // recording panel that closed itself two seconds in and said nothing.
        if let Some(err) = value.get("error_message").and_then(|v| v.as_str()) {
            let code = value
                .get("error_code")
                .map(|c| format!(" ({c})"))
                .unwrap_or_default();
            return Err(anyhow!("Soniox: {err}{code}"));
        }

        let mut committed = String::new();
        let mut interim = String::new();
        for token in value
            .get("tokens")
            .and_then(|t| t.as_array())
            .map(|v| v.as_slice())
            .unwrap_or_default()
        {
            let Some(text) = token.get("text").and_then(|t| t.as_str()) else {
                continue;
            };
            // Soniox emits control tokens such as `<end>` and `<fin>` inline.
            if text.starts_with('<') && text.ends_with('>') {
                continue;
            }
            if token.get("is_final").and_then(|f| f.as_bool()).unwrap_or(false) {
                committed.push_str(text);
            } else {
                interim.push_str(text);
            }
        }

        let mut events = Vec::new();
        if !committed.trim().is_empty() {
            events.push(SttEvent::Final(committed));
        }
        if !interim.trim().is_empty() {
            events.push(SttEvent::Partial(interim));
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{SttProviderKind, SttSettings};

    fn soniox() -> Soniox {
        Soniox::new(
            &SttSettings {
                provider: SttProviderKind::Soniox,
                soniox_model: "stt-rt-preview".into(),
                deepgram_model: "nova-2".into(),
                language: "vi".into(),
                language_hints: vec!["vi".into()],
            },
            "test-key".into(),
        )
    }

    /// Regression: this frame used to be logged and dropped, so a rejected key ended
    /// the take with an empty transcript and an overlay that closed itself a couple of
    /// seconds in, telling the user nothing.
    #[test]
    fn an_error_frame_fails_the_stream_instead_of_being_swallowed() {
        let err = soniox()
            .parse(r#"{"error_code":401,"error_message":"invalid api key"}"#)
            .expect_err("an error frame must not parse as zero events");
        let message = format!("{err}");
        assert!(message.contains("invalid api key"), "{message}");
        assert!(message.contains("401"), "{message}");
    }

    #[test]
    fn a_frame_that_is_not_json_is_ignored_rather_than_treated_as_a_failure() {
        assert!(soniox().parse("keepalive").unwrap().is_empty());
    }

    #[test]
    fn splits_tokens_into_committed_and_interim() {
        let events = soniox()
            .parse(
                r#"{"tokens":[
                    {"text":"xin chào","is_final":true},
                    {"text":"<end>","is_final":true},
                    {"text":" mọi người","is_final":false}
                ]}"#,
            )
            .unwrap();
        assert!(matches!(&events[0], SttEvent::Final(t) if t == "xin chào"));
        assert!(matches!(&events[1], SttEvent::Partial(t) if t == " mọi người"));
    }
}
