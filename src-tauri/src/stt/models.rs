//! Live list of streaming speech models, straight from the provider.
//!
//! A vendor ships a new model (Soniox `stt-rt-v6`, say) and expects clients to switch
//! by changing one string. A hard-coded list makes every release wait for an app
//! update, so the picker asks the provider what it currently offers instead and the
//! built-in list on the frontend is only the offline fallback.

use crate::settings::{SttProviderKind, SttSettings};
use anyhow::{anyhow, Context, Result};
use std::time::Duration;

const SONIOX_MODELS: &str = "https://api.soniox.com/v1/models";
const DEEPGRAM_MODELS: &str = "https://api.deepgram.com/v1/models";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Deepgram architectures that are superseded or are not speech-to-text for dictation.
/// Everything else is offered, so a brand-new architecture shows up without a release.
const DEEPGRAM_LEGACY_ARCHITECTURES: &[&str] = &["base", "polaris", "unknown", "whisper"];

/// Model ids the provider offers for realtime dictation, newest first.
/// AssemblyAI has no model setting, so it yields an error the UI treats as "no list".
pub async fn catalog(settings: &SttSettings, api_key: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    let (request, parse): (_, fn(&serde_json::Value) -> Vec<String>) = match settings.provider {
        SttProviderKind::Soniox => (
            client.get(SONIOX_MODELS).bearer_auth(api_key),
            parse_soniox,
        ),
        SttProviderKind::Deepgram => (
            client
                .get(DEEPGRAM_MODELS)
                .header("authorization", format!("Token {api_key}")),
            parse_deepgram,
        ),
        SttProviderKind::AssemblyAi => {
            return Err(anyhow!("AssemblyAI has no model to choose"));
        }
    };

    let response = request.send().await.context("listing speech models")?;
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(anyhow!("speech model list failed ({status}): {text}"));
    }
    let payload: serde_json::Value = response.json().await.context("reading speech model list")?;
    let models = parse(&payload);
    if models.is_empty() {
        return Err(anyhow!("the provider returned no usable speech models"));
    }
    Ok(models)
}

/// Soniox lists realtime and async models together, plus aliases (`stt-rt-v4` is just
/// a pointer to `stt-rt-v5`). Dictation streams, so only real-time, non-alias entries
/// are useful.
fn parse_soniox(payload: &serde_json::Value) -> Vec<String> {
    let mut ids: Vec<String> = payload
        .get("models")
        .and_then(|m| m.as_array())
        .map(|models| {
            models
                .iter()
                .filter(|m| m.get("transcription_mode").and_then(|v| v.as_str()) == Some("real_time"))
                .filter(|m| m.get("aliased_model_id").map_or(true, |v| v.is_null()))
                .filter_map(|m| m.get("id").and_then(|v| v.as_str()))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    sort_newest_first(&mut ids);
    ids
}

/// Deepgram returns one row per model *and language*, with the architecture
/// (`nova-3`) being the thing a streaming request actually names. Distinct
/// architectures are the short, stable list a person wants to choose from.
fn parse_deepgram(payload: &serde_json::Value) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for model in payload
        .get("stt")
        .and_then(|m| m.as_array())
        .map(|v| v.as_slice())
        .unwrap_or_default()
    {
        if !model.get("streaming").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }
        let Some(arch) = model.get("architecture").and_then(|v| v.as_str()) else {
            continue;
        };
        if DEEPGRAM_LEGACY_ARCHITECTURES.contains(&arch) || ids.iter().any(|i| i == arch) {
            continue;
        }
        ids.push(arch.to_string());
    }
    sort_newest_first(&mut ids);
    ids
}

/// Highest version number first (`…-v6` before `…-v5`, `nova-3` before `nova-2`);
/// ids without a number sort last.
fn sort_newest_first(ids: &mut [String]) {
    ids.sort_by(|a, b| version_of(b).cmp(&version_of(a)).then_with(|| a.cmp(b)));
}

fn version_of(id: &str) -> Option<u32> {
    let tail: String = id
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    tail.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn soniox_keeps_only_real_time_models_that_are_not_aliases_newest_first() {
        let payload = json!({"models": [
            {"id": "stt-async-v6", "transcription_mode": "async", "aliased_model_id": null},
            {"id": "stt-rt-v5", "transcription_mode": "real_time", "aliased_model_id": null},
            {"id": "stt-rt-v4", "transcription_mode": "real_time", "aliased_model_id": "stt-rt-v5"},
            {"id": "stt-rt-v6", "transcription_mode": "real_time", "aliased_model_id": null},
        ]});
        assert_eq!(parse_soniox(&payload), vec!["stt-rt-v6", "stt-rt-v5"]);
    }

    #[test]
    fn deepgram_offers_each_current_architecture_once_and_skips_legacy_ones() {
        let payload = json!({"stt": [
            {"architecture": "nova-2", "streaming": true},
            {"architecture": "nova-3", "streaming": true},
            {"architecture": "nova-3", "streaming": true},
            {"architecture": "base", "streaming": true},
            {"architecture": "whisper", "streaming": true},
            {"architecture": "nova-4", "streaming": false},
        ]});
        assert_eq!(parse_deepgram(&payload), vec!["nova-3", "nova-2"]);
    }

    #[test]
    fn an_id_without_a_version_number_sorts_last() {
        let mut ids = vec!["stt-rt-preview".to_string(), "stt-rt-v6".to_string()];
        sort_newest_first(&mut ids);
        assert_eq!(ids, vec!["stt-rt-v6", "stt-rt-preview"]);
    }
}
