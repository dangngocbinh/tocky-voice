//! Deepgram realtime. Lowest latency of the three; `nova-2` is the model tier with
//! Vietnamese support (`nova-3` multilingual does not currently include it), so the
//! default sends a single explicit language rather than a hint list.

use super::{request_with_header, SttEvent, WsProtocol};
use crate::audio::capture::TARGET_SAMPLE_RATE;
use crate::settings::SttSettings;
use anyhow::{anyhow, Result};
use serde_json::json;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::Message;

pub struct Deepgram {
    api_key: String,
    model: String,
    language: String,
}

impl Deepgram {
    pub fn new(settings: &SttSettings, api_key: String) -> Self {
        Self {
            api_key,
            model: settings.deepgram_model.clone(),
            language: settings.language.clone(),
        }
    }

    fn url(&self) -> String {
        format!(
            "wss://api.deepgram.com/v1/listen\
             ?model={model}&language={language}\
             &encoding=linear16&sample_rate={rate}&channels=1\
             &interim_results=true&punctuate=true&smart_format=true&endpointing=300",
            model = urlencoding::encode(&self.model),
            language = urlencoding::encode(&self.language),
            rate = TARGET_SAMPLE_RATE,
        )
    }
}

impl WsProtocol for Deepgram {
    fn request(&self) -> Result<Request<()>> {
        request_with_header(&self.url(), "authorization", &format!("Token {}", self.api_key))
    }

    fn init_message(&self) -> Option<Message> {
        // Everything is configured through query parameters.
        None
    }

    fn finish_message(&self) -> Message {
        Message::Text(json!({ "type": "CloseStream" }).to_string())
    }

    fn parse(&mut self, text: &str) -> Result<Vec<SttEvent>> {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
            return Ok(Vec::new());
        };
        // Deepgram announces a rejected request as an `Error` frame and then closes.
        if value.get("type").and_then(|t| t.as_str()) == Some("Error") {
            let description = value
                .get("description")
                .or_else(|| value.get("message"))
                .and_then(|d| d.as_str())
                .unwrap_or("stream rejected");
            return Err(anyhow!("Deepgram: {description}"));
        }
        if value.get("type").and_then(|t| t.as_str()) != Some("Results") {
            return Ok(Vec::new());
        }
        let transcript = value
            .pointer("/channel/alternatives/0/transcript")
            .and_then(|t| t.as_str())
            .unwrap_or_default();
        if transcript.trim().is_empty() {
            return Ok(Vec::new());
        }

        let is_final = value
            .get("is_final")
            .and_then(|f| f.as_bool())
            .unwrap_or(false);
        Ok(vec![if is_final {
            SttEvent::Final(transcript.to_string())
        } else {
            SttEvent::Partial(transcript.to_string())
        }])
    }
}
