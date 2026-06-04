//! Agent commands. `agent_start` kicks off a run on a background task and returns
//! its id immediately; all progress streams over `agent://event`. `agent_approve`
//! / `agent_cancel` drive the L4 control surface from the UI.

use shared::ipc::{AgentRequest, ApprovalDecision};
use tauri::{AppHandle, Manager, State};

use crate::agent::{self, AgentContext};
use crate::state::{AgentHandle, AppState};
use crate::{config_io, util};

#[tauri::command]
pub async fn agent_start(
    app: AppHandle,
    state: State<'_, AppState>,
    req: AgentRequest,
) -> Result<String, String> {
    let agent_id = util::new_id("agent");
    let handle = AgentHandle::new(agent_id.clone());
    state.register_agent(handle.clone());

    let cfg = state.config.lock().unwrap().clone();
    let ctx = AgentContext {
        app: app.clone(),
        model: req.model.clone(),
        host: req.host.clone(),
        port: req.port,
        searxng_url: (!cfg.searxng_url.is_empty()).then_some(cfg.searxng_url),
        config_dir: config_io::config_dir(),
    };

    tracing::info!(agent = %agent_id, model = %req.model, "agent_start");
    let id = agent_id.clone();
    // Detach the run; the UI is driven by events, not this command's return.
    tokio::spawn(async move {
        agent::run_agent(ctx, handle, None, "task".into(), req.task, 0).await;
        app.state::<AppState>().remove_agent(&id);
    });

    Ok(agent_id)
}

#[tauri::command]
pub fn agent_approve(state: State<'_, AppState>, decision: ApprovalDecision) -> Result<(), String> {
    let handle = state
        .get_agent(&decision.agent_id)
        .ok_or("unknown agent")?;
    if handle.resolve_approval(&decision.call_id, decision.approved) {
        Ok(())
    } else {
        Err("no pending approval for that call".into())
    }
}

#[tauri::command]
pub fn agent_cancel(state: State<'_, AppState>, agent_id: String) -> Result<(), String> {
    let handle = state.get_agent(&agent_id).ok_or("unknown agent")?;
    handle.cancel.cancel();
    tracing::info!(agent = %agent_id, "agent_cancel");
    Ok(())
}

#[tauri::command]
pub fn agent_status(state: State<'_, AppState>) -> Vec<String> {
    state.agents.lock().unwrap().keys().cloned().collect()
}
