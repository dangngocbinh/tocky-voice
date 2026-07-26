//! Optional AI pass over the raw transcript.
//!
//! Two wire formats cover every vendor we support: Anthropic's Messages API, and the
//! OpenAI chat-completions shape that OpenAI, Gemini, DeepSeek, Qwen, Kimi, GLM,
//! MiniMax, Groq, xAI, OpenRouter and Ollama all speak.

pub mod anthropic;
pub mod openai_compatible;

use crate::settings::{defaults, LlmSettings, LlmWire};
use anyhow::{anyhow, Context, Result};
use std::time::Duration;

/// Cleanup sits between speech and paste, so a hung request is worse than a missing
/// one — the caller falls back to the raw transcript when this elapses.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

pub struct RefineRequest {
    pub system_prompt: String,
    pub transcript: String,
    pub llm: LlmSettings,
    pub api_key: Option<String>,
}

/// Where a request goes and what shape it takes.
pub struct Endpoint {
    pub wire: LlmWire,
    pub base_url: String,
    /// Vendor-specific fields the preset wants merged into the request body.
    pub extra_body: Option<serde_json::Value>,
}

/// Resolves the endpoint for a settings block, honouring a custom base URL.
pub fn resolve_endpoint(llm: &LlmSettings) -> Result<Endpoint> {
    let preset = defaults::preset(&llm.preset)
        .ok_or_else(|| anyhow!("unknown LLM preset: {}", llm.preset))?;
    let base = llm
        .base_url
        .as_deref()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or(preset.base_url)
        .trim_end_matches('/')
        .to_string();
    if base.is_empty() {
        return Err(anyhow!(
            "no base URL configured for LLM preset {}",
            llm.preset
        ));
    }
    require_safe_transport(&base)?;

    // A custom base URL means the user pointed the preset somewhere else; the vendor
    // quirk that came with the preset no longer necessarily applies.
    let extra_body = match (preset.extra_body, llm.base_url.as_deref()) {
        (Some(raw), None | Some("")) => Some(
            serde_json::from_str(raw)
                .with_context(|| format!("preset {} has malformed extra_body", preset.id))?,
        ),
        _ => None,
    };

    Ok(Endpoint {
        wire: preset.wire,
        base_url: base,
        extra_body,
    })
}

/// Refusing plaintext keeps the API key and the transcript off the wire in the clear.
/// Loopback is exempt so local runtimes (Ollama, LM Studio) still work.
fn require_safe_transport(base: &str) -> Result<()> {
    if base.starts_with("https://") {
        return Ok(());
    }
    if let Some(rest) = base.strip_prefix("http://") {
        let host = rest
            .split('/')
            .next()
            .unwrap_or_default()
            .rsplit_once(':')
            .map(|(host, _port)| host)
            .unwrap_or_else(|| rest.split('/').next().unwrap_or_default());
        if matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]") {
            return Ok(());
        }
    }
    Err(anyhow!(
        "refusing to send your API key and transcript to {base} over an unencrypted \
         connection — use https:// (http:// is only allowed for localhost)"
    ))
}

#[cfg(test)]
mod endpoint_tests {
    use super::resolve_endpoint;
    use crate::settings::{defaults, LlmSettings};

    fn settings(preset: &str, base_url: Option<&str>) -> LlmSettings {
        LlmSettings {
            preset: preset.into(),
            model: "m".into(),
            base_url: base_url.map(Into::into),
            max_tokens: 2048,
        }
    }

    #[test]
    fn deepseek_turns_thinking_off_so_cleanup_does_not_wait_on_hidden_reasoning() {
        let endpoint = resolve_endpoint(&settings("deepseek", None)).unwrap();
        assert_eq!(
            endpoint.extra_body.unwrap().pointer("/thinking/type"),
            Some(&serde_json::json!("disabled"))
        );
    }

    #[test]
    fn custom_base_url_drops_the_preset_quirk() {
        let endpoint =
            resolve_endpoint(&settings("deepseek", Some("https://proxy.example/v1"))).unwrap();
        assert!(endpoint.extra_body.is_none());
    }

    /// A typo here would fail at dictation time, on the user's machine, as a confusing
    /// "malformed extra_body" instead of at build time.
    #[test]
    fn every_preset_extra_body_is_a_json_object() {
        for preset in defaults::llm_presets() {
            let Some(raw) = preset.extra_body else { continue };
            let parsed: serde_json::Value = serde_json::from_str(raw)
                .unwrap_or_else(|e| panic!("preset {}: {e}", preset.id));
            assert!(parsed.is_object(), "preset {} is not an object", preset.id);
        }
    }
}

#[cfg(test)]
mod transport_tests {
    use super::require_safe_transport;

    #[test]
    fn accepts_https() {
        assert!(require_safe_transport("https://api.openai.com/v1").is_ok());
    }

    #[test]
    fn accepts_loopback_over_http_for_local_runtimes() {
        assert!(require_safe_transport("http://localhost:11434/v1").is_ok());
        assert!(require_safe_transport("http://127.0.0.1:1234/v1").is_ok());
    }

    #[test]
    fn rejects_plaintext_to_a_remote_host() {
        let err = require_safe_transport("http://evil.example.com/v1").unwrap_err();
        assert!(err.to_string().contains("unencrypted"));
    }

    /// A host that merely starts with "localhost" is a different machine.
    #[test]
    fn rejects_lookalike_loopback_hosts() {
        assert!(require_safe_transport("http://localhost.evil.com/v1").is_err());
    }
}

pub async fn refine(req: RefineRequest) -> Result<String> {
    if req.transcript.trim().is_empty() {
        return Ok(String::new());
    }
    let endpoint = resolve_endpoint(&req.llm)?;
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("building http client")?;

    let raw = match endpoint.wire {
        LlmWire::Anthropic => anthropic::complete(&client, &endpoint.base_url, &req).await?,
        LlmWire::OpenAiCompatible => {
            openai_compatible::complete(&client, &endpoint, &req).await?
        }
    };
    Ok(cleanup_output(&raw))
}

/// Families that are never useful for text cleanup. Filtering by name is crude, but the
/// alternative is asking the user to find one chat model among eighty embedding, audio
/// and image entries — and an unknown-but-real model still shows up, because only these
/// known-irrelevant prefixes are dropped.
const NON_CHAT_PREFIXES: [&str; 9] = [
    "whisper",
    "tts-",
    "dall-e",
    "text-embedding",
    "omni-moderation",
    "text-moderation",
    "babbage",
    "davinci",
    "imagen",
];

/// Live model list from the provider itself, so a newly released model is selectable
/// the day it ships instead of waiting for this app to be updated.
pub async fn list_models(llm: &LlmSettings, api_key: Option<String>) -> Result<Vec<String>> {
    let Endpoint { wire, base_url, .. } = resolve_endpoint(llm)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .context("building http client")?;

    let mut request = client.get(format!("{base_url}/models"));
    request = match wire {
        LlmWire::Anthropic => {
            let key = api_key.ok_or_else(|| anyhow!("no API key configured"))?;
            request
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
        }
        LlmWire::OpenAiCompatible => match api_key.filter(|k| !k.trim().is_empty()) {
            Some(key) => request.bearer_auth(key),
            None => request,
        },
    };

    let response = request.send().await.context("listing models")?;
    let status = response.status();
    let payload: serde_json::Value = response.json().await.context("reading model list")?;
    if !status.is_success() {
        let message = payload
            .pointer("/error/message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        return Err(anyhow!("model list failed ({status}): {message}"));
    }

    let mut models: Vec<String> = payload
        .get("data")
        .and_then(|d| d.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
                // Gemini's OpenAI-compatible endpoint prefixes ids with `models/`.
                .map(|id| id.trim_start_matches("models/").to_string())
                .filter(|id| {
                    let lower = id.to_lowercase();
                    !NON_CHAT_PREFIXES.iter().any(|p| lower.starts_with(p))
                })
                .collect()
        })
        .unwrap_or_default();

    models.sort();
    models.dedup();
    if models.is_empty() {
        return Err(anyhow!("the provider returned no usable models"));
    }
    Ok(models)
}

/// Models occasionally wrap the answer in a fence or quotes despite being told not to.
/// Stripping that here is cheaper and more reliable than another round of prompting.
fn cleanup_output(raw: &str) -> String {
    let mut text = raw.trim();

    if text.starts_with("```") {
        // Drop the opening fence (with optional language tag) and the closing one.
        if let Some(rest) = text.split_once('\n').map(|(_, rest)| rest) {
            text = rest.trim_end().strip_suffix("```").unwrap_or(rest).trim();
        }
    }

    // Only unwrap quotes that enclose the entire response — an inner quote is content.
    let unwrapped = text
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .filter(|inner| !inner.contains('"'));

    unwrapped.unwrap_or(text).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::cleanup_output;

    #[test]
    fn strips_code_fences() {
        assert_eq!(cleanup_output("```\nxin chào\n```"), "xin chào");
        assert_eq!(cleanup_output("```text\nxin chào\n```"), "xin chào");
    }

    #[test]
    fn strips_enclosing_quotes() {
        assert_eq!(cleanup_output("\"xin chào\""), "xin chào");
    }

    #[test]
    fn keeps_interior_quotes_intact() {
        let input = "anh ấy nói \"được\" rồi";
        assert_eq!(cleanup_output(input), input);
    }

    #[test]
    fn leaves_plain_text_alone() {
        assert_eq!(cleanup_output("  xin chào  "), "xin chào");
    }
}
