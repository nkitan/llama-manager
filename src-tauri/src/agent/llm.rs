//! Thin wrapper around the OpenAI-compatible `/v1/chat/completions` endpoint used
//! by the planner and engine. Non-streaming: agent reasoning needs the *complete*
//! message (and any tool calls) before it can act.

use serde_json::{Value, json};

use super::AgentContext;

#[derive(Debug, Default)]
pub struct LlmReply {
    pub content: String,
    pub tool_calls: Vec<ParsedToolCall>,
}

#[derive(Debug, Clone)]
pub struct ParsedToolCall {
    pub id: String,
    pub name: String,
    /// Raw JSON string of arguments, as returned by the model.
    pub arguments: String,
}

/// Call the model. When `tools` is provided the request advertises them with
/// `tool_choice: "auto"`, enabling structured tool calls (L1).
pub async fn call(
    ctx: &AgentContext,
    messages: &[Value],
    tools: Option<&[Value]>,
    temperature: f32,
) -> Result<LlmReply, String> {
    let url = format!("http://{}:{}/v1/chat/completions", ctx.host, ctx.port);
    let mut payload = json!({
        "model": ctx.model,
        "messages": messages,
        "temperature": temperature,
    });
    if let Some(t) = tools {
        payload["tools"] = json!(t);
        payload["tool_choice"] = json!("auto");
    }

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("model request failed: {e}"))?;
    if !res.status().is_success() {
        return Err(format!("model returned status {}", res.status()));
    }
    let v: Value = res
        .json()
        .await
        .map_err(|e| format!("model returned non-JSON: {e}"))?;

    let msg = &v["choices"][0]["message"];
    let content = msg["content"].as_str().unwrap_or("").to_string();

    let mut tool_calls = Vec::new();
    if let Some(arr) = msg["tool_calls"].as_array() {
        for tc in arr {
            let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
            if name.is_empty() {
                continue;
            }
            tool_calls.push(ParsedToolCall {
                id: tc["id"].as_str().unwrap_or("").to_string(),
                name,
                arguments: tc["function"]["arguments"]
                    .as_str()
                    .unwrap_or("{}")
                    .to_string(),
            });
        }
    }

    Ok(LlmReply {
        content,
        tool_calls,
    })
}
