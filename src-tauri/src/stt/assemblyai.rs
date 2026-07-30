//! AssemblyAI Universal-Streaming (v3). Very accurate on English; its streaming tier
//! does not currently cover Vietnamese, so it is offered as a choice rather than a default.
//!
//! Unlike the other two, each `Turn` message carries the *whole* turn so far rather than
//! a delta — so interim messages are previews and only the formatted end-of-turn is committed.

use super::{request_with_header, SttEvent, WsProtocol};
use crate::audio::capture::TARGET_SAMPLE_RATE;
use anyhow::{anyhow, Result};
use serde_json::json;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::Message;

pub struct AssemblyAi {
    api_key: String,
    /// Guards against committing the same turn twice when both the raw and the
    /// formatted end-of-turn messages arrive.
    last_committed_turn: i64,
}

impl AssemblyAi {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            last_committed_turn: -1,
        }
    }
}

impl WsProtocol for AssemblyAi {
    fn request(&self) -> Result<Request<()>> {
        let url = format!(
            "wss://streaming.assemblyai.com/v3/ws?sample_rate={TARGET_SAMPLE_RATE}\
             &encoding=pcm_s16le&format_turns=true"
        );
        request_with_header(&url, "authorization", &self.api_key)
    }

    fn init_message(&self) -> Option<Message> {
        None
    }

    fn finish_message(&self) -> Message {
        Message::Text(json!({ "type": "Terminate" }).to_string())
    }

    fn parse(&mut self, text: &str) -> Result<Vec<SttEvent>> {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
            return Ok(Vec::new());
        };
        // v3 reports a rejected session — bad key, unsupported audio, quota — as an
        // `error` field, then closes. Reported rather than ignored so the user sees why.
        if let Some(err) = value.get("error").and_then(|e| e.as_str()) {
            return Err(anyhow!("AssemblyAI: {err}"));
        }
        if value.get("type").and_then(|t| t.as_str()) != Some("Turn") {
            return Ok(Vec::new());
        }
        let transcript = value
            .get("transcript")
            .and_then(|t| t.as_str())
            .unwrap_or_default();
        if transcript.trim().is_empty() {
            return Ok(Vec::new());
        }

        let end_of_turn = value
            .get("end_of_turn")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let formatted = value
            .get("turn_is_formatted")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let turn_order = value.get("turn_order").and_then(|v| v.as_i64()).unwrap_or(0);

        Ok(if end_of_turn && formatted && turn_order > self.last_committed_turn {
            self.last_committed_turn = turn_order;
            vec![SttEvent::Final(transcript.to_string())]
        } else {
            vec![SttEvent::Partial(transcript.to_string())]
        })
    }
}
