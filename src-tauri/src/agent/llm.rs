//! Thin wrapper around the OpenAI-compatible `/v1/chat/completions` endpoint used
//! by the planner and engine. Non-streaming: agent reasoning needs the *complete*
//! message (and any tool calls) before it can act.
//!
//! Hardened for "bulletproof":
//!  - Per-call timeout (the model server can stall for minutes under load).
//!  - Cancellation via the agent's [`CancellationToken`] (the in-flight HTTP
//!    request aborts immediately instead of waiting for the timeout).
//!  - Bounded exponential backoff with jitter on transient failures (network
//!    errors, 5xx, 429). 4xx other than 429 fail fast (the model won't change
//!    its mind about a malformed request).
//!  - Structured error type ([`LlmError`]) so the engine can distinguish a
//!    "rate limited / model busy" situation (retry) from "schema mismatch"
//!    (propagate immediately).

use std::time::Duration;

use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

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

/// Hard call budget. Most model servers respond in <2s; even a 70B on CPU tops
/// out around 30-60s. Anything past 90s is almost certainly wedged.
const LLM_TIMEOUT: Duration = Duration::from_secs(90);
const LLM_RETRIES: u32 = 2;
const LLM_BACKOFF_BASE_MS: u64 = 300;
const LLM_BACKOFF_CAP_MS: u64 = 3_000;

#[derive(Debug)]
pub enum LlmError {
    /// Caller cancelled (user pressed stop, or parent was cancelled).
    Cancelled,
    /// Hard timeout — the model server never responded in time.
    Timeout(Duration),
    /// Network / DNS / connection reset. Worth retrying.
    Transport(String),
    /// Server replied 5xx or 429. Retryable.
    Server(u16),
    /// Server replied 4xx (other than 429). Non-retryable; the request is bad.
    Client(u16, String),
    /// Server returned non-JSON. Retryable (could be a truncated stream).
    Invalid(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Cancelled => write!(f, "cancelled"),
            LlmError::Timeout(d) => write!(f, "model timed out after {:?}", d),
            LlmError::Transport(s) => write!(f, "transport: {}", s),
            LlmError::Server(s) => write!(f, "server error: status {}", s),
            LlmError::Client(s, body) => write!(f, "client error: status {}, body: {}", s, body),
            LlmError::Invalid(s) => write!(f, "invalid response: {}", s),
        }
    }
}

impl LlmError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlmError::Transport(_) | LlmError::Server(_) | LlmError::Invalid(_) | LlmError::Timeout(_)
        )
    }
}

/// Call the model. When `tools` is provided the request advertises them with
/// `tool_choice: "auto"`, enabling structured tool calls (L1).
pub async fn call(
    ctx: &AgentContext,
    messages: &[Value],
    tools: Option<&[Value]>,
    temperature: f32,
    cancel: &CancellationToken,
) -> Result<LlmReply, LlmError> {
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

    // Shared client across retries (keeps the connection pool warm).
    let client = reqwest::Client::builder()
        .pool_idle_timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| LlmError::Transport(e.to_string()))?;

    let mut last_err: Option<LlmError> = None;
    for attempt in 0..=LLM_RETRIES {
        // Cancellation check between attempts.
        if cancel.is_cancelled() {
            return Err(LlmError::Cancelled);
        }

        let req = client
            .post(&url)
            .json(&payload)
            .timeout(LLM_TIMEOUT);

        let send_result = tokio::select! {
            _ = cancel.cancelled() => {
                return Err(LlmError::Cancelled);
            }
            r = req.send() => r,
        };

        let res = match send_result {
            Ok(r) => r,
            Err(e) if e.is_timeout() => {
                last_err = Some(LlmError::Timeout(LLM_TIMEOUT));
                if attempt < LLM_RETRIES {
                    backoff(attempt).await;
                    continue;
                }
                return Err(last_err.unwrap());
            }
            Err(e) if e.is_connect() || e.is_request() => {
                last_err = Some(LlmError::Transport(e.to_string()));
                if attempt < LLM_RETRIES {
                    backoff(attempt).await;
                    continue;
                }
                return Err(last_err.unwrap());
            }
            Err(e) => {
                return Err(LlmError::Transport(e.to_string()));
            }
        };

        let status = res.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
            last_err = Some(LlmError::Server(status.as_u16()));
            if attempt < LLM_RETRIES {
                backoff(attempt).await;
                continue;
            }
            return Err(last_err.unwrap());
        }
        if status.is_client_error() {
            let body = res.text().await.unwrap_or_default();
            return Err(LlmError::Client(status.as_u16(), truncate(&body, 300)));
        }
        if !status.is_success() {
            return Err(LlmError::Server(status.as_u16()));
        }

        // Body read is also cancel-aware.
        let body_result = tokio::select! {
            _ = cancel.cancelled() => return Err(LlmError::Cancelled),
            r = res.json::<Value>() => r,
        };
        let v: Value = match body_result {
            Ok(v) => v,
            Err(e) => {
                last_err = Some(LlmError::Invalid(e.to_string()));
                if attempt < LLM_RETRIES {
                    backoff(attempt).await;
                    continue;
                }
                return Err(last_err.unwrap());
            }
        };

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

        return Ok(LlmReply { content, tool_calls });
    }

    Err(last_err.unwrap_or(LlmError::Transport("exhausted retries".into())))
}

/// Exponential backoff with full jitter, capped at LLM_BACKOFF_CAP_MS.
async fn backoff(attempt: u32) {
    let exp = LLM_BACKOFF_BASE_MS.saturating_mul(1u64 << attempt.min(4));
    let capped = exp.min(LLM_BACKOFF_CAP_MS);
    // 0..=capped ms of jitter; on average capped/2.
    let jitter = fastrand_u64(capped + 1);
    tokio::time::sleep(Duration::from_millis(jitter)).await;
}

/// Tiny dependency-free jitter (rand_core isn't pulled in by this crate).
fn fastrand_u64(bound: u64) -> u64 {
    use std::cell::Cell;
    use std::time::SystemTime;
    thread_local! {
        static STATE: Cell<u64> = const { Cell::new(0) };
    }
    STATE.with(|s| {
        let mut x = s.get();
        if x == 0 {
            x = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0xdead_beef_cafe_babe);
        }
        // xorshift64*
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        s.set(x);
        if bound == 0 { 0 } else { (x.wrapping_mul(0x2545_F491_4F6C_DD1D)) % bound }
    })
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_not_retryable() {
        assert!(!LlmError::Cancelled.is_retryable());
    }

    #[test]
    fn transport_and_server_are_retryable() {
        assert!(LlmError::Transport("boom".into()).is_retryable());
        assert!(LlmError::Server(500).is_retryable());
        assert!(LlmError::Server(429).is_retryable());
        assert!(LlmError::Invalid("not json".into()).is_retryable());
    }

    #[test]
    fn client_4xx_is_not_retryable() {
        assert!(!LlmError::Client(400, "bad".into()).is_retryable());
        assert!(!LlmError::Client(401, "no".into()).is_retryable());
        assert!(!LlmError::Client(404, "?".into()).is_retryable());
    }

    #[test]
    fn timeout_is_retryable() {
        assert!(LlmError::Timeout(Duration::from_secs(1)).is_retryable());
    }

    #[test]
    fn jitter_helper_returns_in_range() {
        for _ in 0..100 {
            let v = fastrand_u64(10);
            assert!(v < 10);
        }
    }
}
