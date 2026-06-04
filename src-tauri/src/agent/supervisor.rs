//! Sub-agent supervision (L3). `spawn_subagent` is intercepted by the engine and
//! routed here. A child agent is a full agent run with its own id, handle, and
//! event stream; we cap nesting via [`MAX_DEPTH`] and run children to completion,
//! returning their final text back to the parent as the tool result.

use std::sync::Arc;

use shared::ipc::{AGENT_EVENT, AgentEvent, ToolCall};
use tauri::{AppHandle, Emitter, Manager};

use super::{AgentContext, MAX_DEPTH, run_agent};
use crate::state::{AgentHandle, AppState};
use crate::util::new_id;

/// Run a sub-agent for the given `spawn_subagent` call and return its result text.
pub async fn spawn(
    ctx: &AgentContext,
    parent: &Arc<AgentHandle>,
    call: &ToolCall,
    depth: u8,
) -> String {
    if depth >= MAX_DEPTH {
        return "Sub-agent depth limit reached; handle this step directly.".to_string();
    }

    let role = call.args["role"].as_str().unwrap_or("subagent").to_string();
    let prompt = call.args["prompt"].as_str().unwrap_or("").trim().to_string();
    if prompt.is_empty() {
        return "spawn_subagent requires a non-empty `prompt`.".to_string();
    }

    let child_id = new_id("subagent");
    let child_handle = AgentHandle::new(child_id.clone());
    register(&ctx.app, child_handle.clone());

    // Tie the child's cancellation to the parent's, so cancelling the root stops
    // the whole tree.
    let parent_cancel = parent.cancel.clone();
    let child_cancel = child_handle.cancel.clone();
    tokio::spawn(async move {
        parent_cancel.cancelled().await;
        child_cancel.cancel();
    });

    let _ = ctx.app.emit(
        AGENT_EVENT,
        AgentEvent::SubAgentSpawned {
            agent_id: parent.id.clone(),
            child_id: child_id.clone(),
            role: role.clone(),
            task: prompt.clone(),
        },
    );

    let result = run_agent(
        ctx.clone(),
        child_handle,
        Some(parent.id.clone()),
        role,
        prompt,
        depth + 1,
    )
    .await;

    unregister(&ctx.app, &child_id);
    result
}

fn register(app: &AppHandle, handle: Arc<AgentHandle>) {
    app.state::<AppState>().register_agent(handle);
}

fn unregister(app: &AppHandle, id: &str) {
    app.state::<AppState>().remove_agent(id);
}
