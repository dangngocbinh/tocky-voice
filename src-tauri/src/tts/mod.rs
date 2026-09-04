//! Text-to-speech: turning selected text into audio, provider-agnostic.
//!
//! Dispatch is a plain enum rather than `dyn Trait`/`async-trait`, the same shape
//! `stt::build_protocol` uses for the realtime speech-to-text side — one match arm per
//! vendor, no extra crate for object-safe async traits.

pub mod chunker;
pub mod player;

mod elevenlabs;
mod gemini;
mod openai;
mod soniox;
mod vbee;

use crate::settings::{TtsProviderKind, TtsSettings};
use anyhow::Result;
use serde::Serialize;

/// One chunk of text to synthesize, and how to say it.
pub struct SpeechRequest {
    pub text: String,
    /// Provider-specific voice id/name; empty lets the provider pick its default.
    pub voice: String,
    /// 1.0 is normal pace. Not every provider can honour this — see `gemini.rs`.
    pub speed: f32,
}

/// Raw PCM audio, provider-normalized to 16-bit mono so it goes straight into
/// `rodio::buffer::SamplesBuffer` with no decoder.
pub struct Audio {
    pub pcm: Vec<i16>,
    pub sample_rate: u32,
}

#[derive(Serialize, Clone)]
pub struct Voice {
    pub id: String,
    pub label: String,
    /// `None` when the provider doesn't scope voices by language.
    pub language: Option<String>,
}

/// One selectable model and everything the UI needs to render its controls: which
/// voices it has, and what speed range it will actually accept.
///
/// Speed lives here rather than being a per-provider constant because it is genuinely
/// per-model — Soniox's own `/v1/tts-models` reports `speed_min`/`speed_max`, and a
/// slider hardcoded to a wider range just produces vendor rejections at read time.
#[derive(Serialize, Clone)]
pub struct TtsModel {
    pub id: String,
    pub label: String,
    pub voices: Vec<Voice>,
    pub speed_min: f32,
    pub speed_max: f32,
    pub supports_speed: bool,
}

impl TtsModel {
    /// A model whose voices and speed range are known statically rather than fetched —
    /// every provider except Soniox publishes these as documentation, not as an API.
    fn fixed(id: &str, label: &str, voices: &[&str], speed: Option<(f32, f32)>) -> Self {
        let (speed_min, speed_max) = speed.unwrap_or((1.0, 1.0));
        Self {
            id: id.to_string(),
            label: label.to_string(),
            voices: voices
                .iter()
                .map(|v| Voice { id: (*v).to_string(), label: (*v).to_string(), language: None })
                .collect(),
            speed_min,
            speed_max,
            supports_speed: speed.is_some(),
        }
    }
}

pub enum Engine {
    Soniox { api_key: String, model: String },
    Gemini { api_key: String, model: String },
    OpenAi { api_key: String, model: String },
    ElevenLabs { api_key: String, model: String },
    /// Vbee's auth is two parts — see `vbee.rs` — so it carries `app_id` alongside the
    /// vault-stored `token` instead of a single `api_key`.
    Vbee { app_id: String, token: String },
}

pub fn build_engine(tts: &TtsSettings, api_key: String) -> Engine {
    match tts.provider {
        TtsProviderKind::Soniox => Engine::Soniox { api_key, model: tts.model.clone() },
        TtsProviderKind::Gemini => Engine::Gemini { api_key, model: tts.model.clone() },
        TtsProviderKind::OpenAi => Engine::OpenAi { api_key, model: tts.model.clone() },
        TtsProviderKind::ElevenLabs => Engine::ElevenLabs { api_key, model: tts.model.clone() },
        TtsProviderKind::Vbee => Engine::Vbee { app_id: tts.vbee_app_id.clone(), token: api_key },
    }
}

pub async fn synthesize(engine: &Engine, req: &SpeechRequest) -> Result<Audio> {
    match engine {
        Engine::Soniox { api_key, model } => soniox::synthesize(api_key, model, req).await,
        Engine::Gemini { api_key, model } => gemini::synthesize(api_key, model, req).await,
        Engine::OpenAi { api_key, model } => openai::synthesize(api_key, model, req).await,
        Engine::ElevenLabs { api_key, model } => elevenlabs::synthesize(api_key, model, req).await,
        Engine::Vbee { app_id, token } => vbee::synthesize(app_id, token, req).await,
    }
}

/// Every model this provider offers, each with its own voices and speed range, so the
/// settings UI can populate both dropdowns and bound the slider from one call.
///
/// Takes the whole `TtsSettings` rather than just `provider` — Vbee's second auth
/// field (`vbee_app_id`) lives there, and threading individual fields through would
/// only grow this signature further as more providers are added.
pub async fn catalog(tts: &TtsSettings, api_key: &str) -> Result<Vec<TtsModel>> {
    match tts.provider {
        TtsProviderKind::Soniox => soniox::catalog(api_key).await,
        TtsProviderKind::Gemini => Ok(gemini::catalog()),
        TtsProviderKind::OpenAi => Ok(openai::catalog()),
        TtsProviderKind::ElevenLabs => elevenlabs::catalog(api_key).await,
        TtsProviderKind::Vbee => vbee::catalog(&tts.vbee_app_id, api_key).await,
    }
}

/// Keychain account holding this provider's key. Soniox/Gemini/OpenAI reuse the
/// account already saved for speech recognition or AI cleanup — the whole point of
/// defaulting to Soniox is that an existing user has nothing new to enter. ElevenLabs
/// and Vbee each need a dedicated credential (`settings::all_secret_accounts` lists
/// both) — for Vbee this account holds the bearer *token* half of its two-part auth.
pub fn secret_account(provider: TtsProviderKind) -> &'static str {
    match provider {
        TtsProviderKind::Soniox => "soniox",
        TtsProviderKind::Gemini => "gemini",
        TtsProviderKind::OpenAi => "openai",
        TtsProviderKind::ElevenLabs => "elevenlabs",
        TtsProviderKind::Vbee => "vbee",
    }
}

/// Whether this provider's synthesis endpoint accepts a speed parameter at all —
/// Gemini does not (see `gemini.rs`), so the UI dims the speed slider for it.
pub fn supports_speed(provider: TtsProviderKind) -> bool {
    !matches!(provider, TtsProviderKind::Gemini)
}

/// Every provider the settings UI can offer. Exists so the tests below cover new
/// variants automatically instead of only the ones someone remembered to list.
pub const ALL_PROVIDERS: [TtsProviderKind; 5] = [
    TtsProviderKind::Soniox,
    TtsProviderKind::Gemini,
    TtsProviderKind::OpenAi,
    TtsProviderKind::ElevenLabs,
    TtsProviderKind::Vbee,
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: `key_status` used to keep its own hand-written account list, which
    /// did not include Vbee. The credential saved fine and the badge still read "no
    /// key" every time, which is indistinguishable from the save being broken.
    #[test]
    fn every_provider_credential_is_tracked_by_the_credential_store() {
        let known = crate::settings::all_secret_accounts();
        for provider in ALL_PROVIDERS {
            let account = secret_account(provider);
            assert!(
                known.contains(&account),
                "{provider:?} stores its key under {account:?}, which \
                 all_secret_accounts() does not list — its badge will read \
                 \"no key\" forever and switching credential store will drop it"
            );
        }
    }
}
