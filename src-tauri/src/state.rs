//! Canonical application state, owned by the backend and shared with commands via
//! `tauri::State` (see [[migration-conventions]]: the frontend never owns truth).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use shared::ServerConfig;
use tokio::process::Child;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

/// Per-agent control block: cancellation + the set of approval gates currently
/// awaiting a human decision (keyed by tool-call id).
pub struct AgentHandle {
    pub id: String,
    pub cancel: CancellationToken,
    pending_approvals: Mutex<HashMap<String, oneshot::Sender<bool>>>,
}

impl AgentHandle {
    pub fn new(id: String) -> Arc<Self> {
        Arc::new(Self {
            id,
            cancel: CancellationToken::new(),
            pending_approvals: Mutex::new(HashMap::new()),
        })
    }

    /// Register an approval gate; returns the receiver the agent awaits.
    pub fn register_approval(&self, call_id: &str) -> oneshot::Receiver<bool> {
        let (tx, rx) = oneshot::channel();
        self.pending_approvals
            .lock()
            .unwrap()
            .insert(call_id.to_string(), tx);
        rx
    }

    /// Resolve a pending approval (from the `agent_approve` command). Returns
    /// false if no such gate exists (e.g. it already timed out).
    pub fn resolve_approval(&self, call_id: &str, approved: bool) -> bool {
        if let Some(tx) = self.pending_approvals.lock().unwrap().remove(call_id) {
            let _ = tx.send(approved);
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct AppState {
    /// Canonical config; the only in-memory source of truth. Persisted to disk by
    /// the `update_config` command, which also emits `config://changed`.
    pub config: Mutex<ServerConfig>,
    /// Handle to the running `llama-server` child, if any. Reserved for the
    /// Server tab migration (next slice); kept here so the lifecycle pattern lives
    /// in one place.
    #[allow(dead_code)]
    pub server: Mutex<Option<Child>>,
    /// Handle to the running Deep Research child process, if any.
    pub deep_research_child: Mutex<Option<Child>>,
    /// Handle to the running Benchmark child process, if any.
    pub benchmark_child: Mutex<Option<Child>>,
    /// Live agents (top-level and sub-agents) by id, for approval routing + cancel.
    pub agents: Mutex<HashMap<String, Arc<AgentHandle>>>,
}

impl AppState {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config: Mutex::new(config),
            server: Mutex::new(None),
            deep_research_child: Mutex::new(None),
            benchmark_child: Mutex::new(None),
            agents: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_agent(&self, handle: Arc<AgentHandle>) {
        self.agents
            .lock()
            .unwrap()
            .insert(handle.id.clone(), handle);
    }

    pub fn get_agent(&self, id: &str) -> Option<Arc<AgentHandle>> {
        self.agents.lock().unwrap().get(id).cloned()
    }

    pub fn remove_agent(&self, id: &str) {
        self.agents.lock().unwrap().remove(id);
    }
}
