//! Chat commands.
//!
//! `chat_list_models` ports the legacy `/v1/models` poll. `chat_send` is the
//! streaming upgrade over the legacy single-shot request: it requests
//! `"stream": true` and forwards each token to the UI as a `chat://event`
//! [`ChatEvent::Token`], finishing with `Done` (or `Error`). Streaming is the
//! decisive pattern the whole app reuses (see [[migration-conventions]]).

use std::time::Duration;

use futures_util::StreamExt;
use shared::ipc::{CHAT_EVENT, ChatEvent, ChatRequest, ModelList};
use tauri::{AppHandle, Emitter};

/// Query the OpenAI-compatible `/v1/models` endpoint. Returns `online: false`
/// (rather than erroring) when the server is unreachable, so the UI can render a
/// calm "offline" state.
#[tauri::command]
pub async fn chat_list_models(host: String, port: u16) -> ModelList {
    let url = format!("http://{host}:{port}/v1/models");
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .timeout(Duration::from_secs(1))
        .send()
        .await
    {
        Ok(res) => {
            if res.status().is_success() {
                #[derive(serde::Deserialize)]
                struct Model {
                    id: String,
                }
                #[derive(serde::Deserialize)]
                struct Models {
                    data: Vec<Model>,
                }
                match res.json::<Models>().await {
                    Ok(list) => ModelList {
                        online: true,
                        models: list.data.into_iter().map(|m| m.id).collect(),
                    },
                    Err(e) => {
                        tracing::warn!(%e, "failed to parse /v1/models");
                        ModelList {
                            online: true,
                            models: vec![],
                        }
                    }
                }
            } else {
                tracing::info!(status = ?res.status(), "server responded with error status; treating as online with no models");
                ModelList {
                    online: true,
                    models: vec![],
                }
            }
        }
        Err(e) => {
            tracing::debug!(%e, "chat_list_models connection failed");
            ModelList {
                online: false,
                models: vec![],
            }
        }
    }
}

/// Start a streaming chat completion. Returns immediately; tokens arrive on
/// `chat://event`. We spawn the streaming task so the `invoke` promise resolves
/// right away and the UI is driven entirely by events.
#[tauri::command]
pub async fn chat_send(app: AppHandle, req: ChatRequest) -> Result<(), String> {
    tracing::debug!(stream_id = %req.stream_id, model = %req.model, "chat_send");
    tokio::spawn(async move {
        if let Err(e) = stream_completion(&app, &req).await {
            tracing::warn!(%e, "chat stream failed");
            let _ = app.emit(
                CHAT_EVENT,
                ChatEvent::Error {
                    stream_id: req.stream_id.clone(),
                    message: e,
                },
            );
        }
    });
    Ok(())
}

async fn reflect_chat(host: String, port: u16, model: String, user_msg: String, assistant_msg: String) {
    let url = format!("http://{}:{}/v1/chat/completions", host, port);
    let system_prompt = "You are an agent memory consolidation system. Review the user's message, and the assistant's response. Extract a single concise lesson, fact, rule, or user preference that should be stored in memory to help future runs of the agent. Respond with ONLY that single key takeaway (max 100 characters, no prefix like 'Lesson:', no quotes, no conversational filler).";
    let user_prompt = format!("User Message: {}\n\nAssistant Response: {}", user_msg, assistant_msg);
    let payload = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ],
        "temperature": 0.1,
    });
    
    let client = reqwest::Client::new();
    match client.post(&url).json(&payload).send().await {
        Ok(res) => {
            if res.status().is_success() {
                #[derive(serde::Deserialize)]
                struct Choice {
                    message: serde_json::Value,
                }
                #[derive(serde::Deserialize)]
                struct Response {
                    choices: Vec<Choice>,
                }
                if let Ok(v) = res.json::<Response>().await {
                    if let Some(choice) = v.choices.first() {
                        if let Some(content) = choice.message["content"].as_str() {
                            let lesson = content.trim().trim_matches('"').trim().to_string();
                            if !lesson.is_empty() {
                                let memory = crate::agent::memory::MemoryManager::new(&config_dir());
                                memory.add_lesson(&lesson);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!(%e, "failed to send chat reflection request");
        }
    }
}

async fn stream_completion(app: &AppHandle, req: &ChatRequest) -> Result<(), String> {
    let url = format!("http://{}:{}/v1/chat/completions", req.host, req.port);
    let payload = serde_json::json!({
        "model": req.model,
        "messages": req.messages,
        "temperature": req.temperature,
        "stream": true,
    });

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Error contacting model server: {e}"))?;

    if !res.status().is_success() {
        return Err(format!("Model server returned status {}", res.status()));
    }

    // The body is an SSE stream; bytes may split mid-line, so we buffer and
    // process complete lines only.
    let mut stream = res.bytes_stream();
    let mut buf = String::new();
    let mut assistant_response = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream error: {e}"))?;
        buf.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(nl) = buf.find('\n') {
            let line: String = buf.drain(..=nl).collect();
            match parse_sse_line(line.trim()) {
                SseLine::Token(delta) if !delta.is_empty() => {
                    assistant_response.push_str(&delta);
                    let _ = app.emit(
                        CHAT_EVENT,
                        ChatEvent::Token {
                            stream_id: req.stream_id.clone(),
                            delta,
                        },
                    );
                }
                SseLine::Done => {
                    let _ = app.emit(
                        CHAT_EVENT,
                        ChatEvent::Done {
                            stream_id: req.stream_id.clone(),
                        },
                    );
                    
                    if !assistant_response.trim().is_empty() {
                        if let Some(last_user) = req.messages.iter().rev().find(|m| m.role == "user") {
                            let user_content = last_user.content.clone();
                            let assistant_content = assistant_response.clone();
                            let host = req.host.clone();
                            let port = req.port;
                            let model = req.model.clone();
                            tokio::spawn(async move {
                                reflect_chat(host, port, model, user_content, assistant_content).await;
                            });
                        }
                    }
                    
                    return Ok(());
                }
                _ => {}
            }
        }
    }
    // Stream ended without an explicit [DONE].
    let _ = app.emit(
        CHAT_EVENT,
        ChatEvent::Done {
            stream_id: req.stream_id.clone(),
        },
    );
    
    if !assistant_response.trim().is_empty() {
        if let Some(last_user) = req.messages.iter().rev().find(|m| m.role == "user") {
            let user_content = last_user.content.clone();
            let assistant_content = assistant_response.clone();
            let host = req.host.clone();
            let port = req.port;
            let model = req.model.clone();
            tokio::spawn(async move {
                reflect_chat(host, port, model, user_content, assistant_content).await;
            });
        }
    }
    
    Ok(())
}

/// Outcome of parsing a single SSE line from an OpenAI-compatible stream.
#[derive(Debug, PartialEq)]
enum SseLine {
    Token(String),
    Done,
    Ignore,
}

/// Parse one already-trimmed SSE line. Lines look like `data: {json}` or
/// `data: [DONE]`; anything else (comments, blanks) is ignored. Pure + testable.
fn parse_sse_line(line: &str) -> SseLine {
    let Some(data) = line.strip_prefix("data:") else {
        return SseLine::Ignore;
    };
    let data = data.trim();
    if data == "[DONE]" {
        return SseLine::Done;
    }
    match serde_json::from_str::<serde_json::Value>(data) {
        Ok(v) => {
            let delta = v["choices"][0]["delta"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();
            SseLine::Token(delta)
        }
        Err(_) => SseLine::Ignore,
    }
}

use crate::config_io::config_dir;
use shared::ipc::ChatMessage;

#[tauri::command]
pub fn chat_save_history(history: Vec<ChatMessage>) -> Result<(), String> {
    let path = config_dir().join("chat_history.json");
    let json = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn chat_load_history() -> Result<Vec<ChatMessage>, String> {
    let path = config_dir().join("chat_history.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let history: Vec<ChatMessage> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_token_lines() {
        let line = r#"data: {"choices":[{"delta":{"content":"Hello"}}]}"#;
        assert_eq!(parse_sse_line(line), SseLine::Token("Hello".into()));
    }

    #[test]
    fn parses_done() {
        assert_eq!(parse_sse_line("data: [DONE]"), SseLine::Done);
    }

    #[test]
    fn ignores_non_data_and_empty_delta() {
        assert_eq!(parse_sse_line(": keep-alive"), SseLine::Ignore);
        let role = r#"data: {"choices":[{"delta":{"role":"assistant"}}]}"#;
        assert_eq!(parse_sse_line(role), SseLine::Token(String::new()));
    }
}
