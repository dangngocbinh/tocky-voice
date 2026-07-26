//! One client for every vendor exposing an OpenAI `/chat/completions` endpoint:
//! OpenAI, Gemini, DeepSeek, Qwen, Moonshot Kimi, Zhipu GLM, MiniMax, Groq, xAI,
//! OpenRouter, and local Ollama. Only the base URL, model, and key differ.

use super::{Endpoint, RefineRequest};
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

pub async fn complete(
    client: &reqwest::Client,
    endpoint: &Endpoint,
    req: &RefineRequest,
) -> Result<String> {
    let mut body = json!({
        "model": req.llm.model,
        "max_tokens": req.llm.max_tokens,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": req.system_prompt },
            { "role": "user", "content": req.transcript },
        ],
    });

    // Merged at the top level only, and never over an existing field: a preset tweaks
    // how the vendor behaves, it does not get to rewrite the model or the transcript.
    if let (Some(Value::Object(extra)), Some(target)) =
        (endpoint.extra_body.as_ref(), body.as_object_mut())
    {
        for (key, value) in extra {
            target.entry(key.clone()).or_insert_with(|| value.clone());
        }
    }

    let base_url = &endpoint.base_url;
    let mut request = client
        .post(format!("{base_url}/chat/completions"))
        .header("content-type", "application/json")
        .json(&body);

    // Local endpoints (Ollama) accept requests without a credential.
    if let Some(key) = req.api_key.as_deref().filter(|k| !k.trim().is_empty()) {
        request = request.bearer_auth(key);
    }

    let response = request
        .send()
        .await
        .context("calling chat completions endpoint")?;

    let status = response.status();
    let payload: Value = response.json().await.context("reading response body")?;

    if !status.is_success() {
        let message = payload
            .pointer("/error/message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        return Err(anyhow!("LLM API {status}: {message}"));
    }

    let content = payload
        .pointer("/choices/0/message/content")
        .and_then(|c| c.as_str())
        .unwrap_or_default();

    if !content.trim().is_empty() {
        return Ok(content.to_string());
    }

    // Reasoning models (DeepSeek V4, o-series, …) spend the token budget on hidden
    // reasoning first. If it runs out, the request "succeeds" with empty content —
    // an easy failure to misread as a broken key, so name the real cause.
    let spent_reasoning = payload
        .pointer("/usage/completion_tokens_details/reasoning_tokens")
        .and_then(|t| t.as_u64())
        .unwrap_or(0);
    if spent_reasoning > 0 || payload.pointer("/choices/0/message/reasoning_content").is_some() {
        return Err(anyhow!(
            "{} is a reasoning model and used the whole {} token budget thinking, \
             leaving no answer. Raise Max tokens, or pick a non-reasoning model for cleanup.",
            req.llm.model,
            req.llm.max_tokens,
        ));
    }

    Err(anyhow!("chat completions response contained no message content"))
}
