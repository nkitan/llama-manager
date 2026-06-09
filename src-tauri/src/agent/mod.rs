//! Revamped agent engine. Layered and foolproof (see [[migration-conventions]]):
//!
//!   L1  engine.rs      — structured ReAct tool-calling loop (OpenAI tool schemas)
//!   L2  planner.rs     — up-front task decomposition into a plan
//!   L3  supervisor.rs  — `spawn_subagent`, tracked recursive sub-agents
//!   L4  approval.rs    — human approval gates + timeouts/retries/cancellation
//!
//! All streaming flows over the single `agent://event` channel as tagged
//! [`shared::ipc::AgentEvent`]s, keyed by `agent_id` so the UI can draw a
//! sub-agent tree.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use tauri::AppHandle;

use crate::state::AgentHandle;

pub mod approval;
pub mod engine;
pub mod llm;
pub mod memory;
pub mod planner;
pub mod supervisor;
pub mod tools;

/// Maximum sub-agent nesting depth (L3 guardrail against runaway spawning).
pub const MAX_DEPTH: u8 = 2;
/// Maximum reasoning/tool steps per agent (L1/L4 budget).
pub const MAX_STEPS: u32 = 8;

/// Everything an agent task needs to run. Cheaply cloneable so sub-agents inherit
/// it. `AppHandle` is the gateway both to event emission and to managed state
/// (`app.state::<AppState>()`) from inside the spawned task.
#[derive(Clone)]
pub struct AgentContext {
    pub app: AppHandle,
    pub model: String,
    pub host: String,
    pub port: u16,
    pub searxng_url: Option<String>,
    pub config_dir: PathBuf,
}

/// Run an agent to completion, returning its final answer text.
///
/// Boxed because the engine recurses here for sub-agents (L3); a plain `async fn`
/// calling itself would have infinite size.
pub fn run_agent(
    ctx: AgentContext,
    handle: Arc<AgentHandle>,
    parent: Option<String>,
    role: String,
    task: String,
    depth: u8,
) -> Pin<Box<dyn Future<Output = String> + Send>> {
    Box::pin(async move { engine::run(ctx, handle, parent, role, task, depth).await })
}
