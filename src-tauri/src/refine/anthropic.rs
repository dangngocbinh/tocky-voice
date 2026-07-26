//! Anthropic Messages API client.
//!
//! Cleanup is latency-critical, so thinking is explicitly disabled and effort pinned
//! low. That matters on the Claude 4.6+ / 5 family, where thinking is on by default
//! and would add seconds before a single character reaches the clipboard.

use super::RefineRequest;
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

const API_VERSION: &str = "2023-06-01";

/// Models that removed `temperature` and gained adaptive thinking. Sending sampling
/// parameters to these returns a 400, and omitting the thinking config leaves it on.
fn uses_adaptive_thinking(model: &str) -> bool {
    const PREFIXES: [&str; 7] = [
        "claude-opus-5",
        "claude-opus-4-6",
        "claude-opus-4-7",
        "claude-opus-4-8",
        "claude-sonnet-5",
        "claude-sonnet-4-6",
        "claude-fable-5",
    ];
    PREFIXES.iter().any(|p| model.starts_with(p))
}

pub async fn complete(
    client: &reqwest::Client,
    base_url: &str,
    req: &RefineRequest,
) -> Result<String> {
    let api_key = req
        .api_key
        .as_deref()
        .ok_or_else(|| anyhow!("no Anthropic API key configured"))?;

    let mut body = json!({
        "model": req.llm.model,
        "max_tokens": req.llm.max_tokens,
        "system": req.system_prompt,
        "messages": [{ "role": "user", "content": req.transcript }],
    });

    if uses_adaptive_thinking(&req.llm.model) {
        // `disabled` is only accepted at effort `high` or lower, and `low` is what we want anyway.
        body["thinking"] = json!({ "type": "disabled" });
        body["output_config"] = json!({ "effort": "low" });
    } else {
        // Older models still accept sampling parameters; keep the edit conservative.
        body["temperature"] = json!(0.2);
    }

    let response = client
        .post(format!("{base_url}/messages"))
        .header("x-api-key", api_key)
        .header("anthropic-version", API_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .context("calling Anthropic Messages API")?;

    let status = response.status();
    let payload: Value = response
        .json()
        .await
        .context("reading Anthropic response body")?;

    if !status.is_success() {
        let message = payload
            .pointer("/error/message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        return Err(anyhow!("Anthropic API {status}: {message}"));
    }

    // A safety decline arrives as HTTP 200 with an empty content array, so check
    // stop_reason before indexing into the blocks.
    if payload.get("stop_reason").and_then(|s| s.as_str()) == Some("refusal") {
        return Err(anyhow!("Anthropic declined to process this transcript"));
    }

    payload
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|blocks| {
            blocks
                .iter()
                .find(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
                .and_then(|b| b.get("text"))
                .and_then(|t| t.as_str())
        })
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Anthropic response contained no text block"))
}

#[cfg(test)]
mod tests {
    use super::uses_adaptive_thinking;

    #[test]
    fn detects_models_that_need_thinking_disabled() {
        assert!(uses_adaptive_thinking("claude-opus-5"));
        assert!(uses_adaptive_thinking("claude-sonnet-5"));
    }

    #[test]
    fn leaves_haiku_on_the_sampling_path() {
        assert!(!uses_adaptive_thinking("claude-haiku-4-5"));
    }
}
