//! Human-in-the-loop approval gate (L4). A sensitive tool call emits an
//! `ApprovalRequest` and blocks until the `agent_approve` command resolves it —
//! or until the timeout fires, in which case we **deny** (fail safe).

use std::sync::Arc;
use std::time::Duration;

use shared::ipc::{AGENT_EVENT, AgentEvent, ApprovalRequest, ToolCall};
use tauri::{AppHandle, Emitter};

use crate::state::AgentHandle;

/// How long to wait for a human decision before denying.
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(120);

/// Emit an approval request and await the user's decision. Returns `true` only on
/// an explicit approval; timeouts, cancellation, and dropped channels deny.
pub async fn gate(app: &AppHandle, handle: &Arc<AgentHandle>, call: &ToolCall, reason: &str) -> bool {
    let rx = handle.register_approval(&call.call_id);

    let _ = app.emit(
        AGENT_EVENT,
        AgentEvent::ApprovalRequest {
            agent_id: handle.id.clone(),
            request: ApprovalRequest {
                call_id: call.call_id.clone(),
                tool: call.tool.clone(),
                args: call.args.clone(),
                reason: reason.to_string(),
            },
        },
    );

    tokio::select! {
        _ = handle.cancel.cancelled() => {
            tracing::info!(agent = %handle.id, "approval cancelled");
            false
        }
        res = tokio::time::timeout(APPROVAL_TIMEOUT, rx) => match res {
            Ok(Ok(decision)) => decision,
            _ => {
                tracing::info!(call = %call.call_id, "approval timed out → deny");
                false
            }
        }
    }
}
