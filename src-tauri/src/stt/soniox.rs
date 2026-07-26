//! Soniox realtime. Best of the three for Vietnamese/English code-switching because
//! it accepts several language hints at once instead of one fixed language.
//!
//! Protocol: JSON config frame (carrying the API key — Soniox has no auth header),
//! then raw PCM frames, then an empty text frame to signal end of audio.

use super::{request_with_header, SttEvent, WsProtocol};
use crate::settings::SttSettings;
use anyhow::Result;
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

    fn parse(&mut self, text: &str) -> Vec<SttEvent> {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
            return Vec::new();
        };
        if let Some(err) = value.get("error_message").and_then(|v| v.as_str()) {
            log::error!("soniox error: {err}");
            return Vec::new();
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
        events
    }
}
