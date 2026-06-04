//! Cross-tab reactive state + the `Tab` enum that drives navigation.

use leptos::prelude::*;
use shared::ipc::{ApprovalRequest, ChatMessage, PlanStep};
use shared::ServerConfig;
use wasm_bindgen_futures::spawn_local;

use crate::api;

/// A single observed I/O event, stored in AppCtx for the Observability tab.
#[derive(Clone, PartialEq)]
pub struct ObsEvent {
    pub ts: f64,
    pub kind: String,
    pub id: String,
    pub content: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    // AI
    Chat,
    Agents,
    Mcp,
    // Workspace
    Planner,
    Todos,
    QuickNotes,
    Calendar,
    Monitor,
    // Models
    Model,
    Library,
    Download,
    Compare,
    DeepResearch,
    // Server
    Server,
    Instances,
    // Launch config
    Context,
    Gpu,
    Performance,
    Sampling,
    Advanced,
    Api,
    // App
    Settings,
    // System overview + observability
    Overview,
    Observability,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        use Tab::*;
        match self {
            Chat => "Chat",
            Agents => "Agents & Memory",
            Mcp => "MCP Tools",
            Planner => "Task Planner",
            Todos => "Todos",
            QuickNotes => "Quick Notes",
            Calendar => "Calendar",
            Monitor => "Agent Monitor",
            Model => "Model",
            Library => "Model Index",
            Download => "Downloader",
            Compare => "Compare",
            DeepResearch => "Deep Research",
            Server => "Server",
            Instances => "Instances",
            Context => "Context",
            Gpu => "GPU & Memory",
            Performance => "Performance",
            Sampling => "Sampling",
            Advanced => "Advanced",
            Api => "API & Output",
            Settings => "Settings",
            Overview => "System Overview",
            Observability => "Observability",
        }
    }

    pub fn icon(&self) -> &'static str {
        use Tab::*;
        match self {
            Chat => "💬",
            Agents => "🧠",
            Mcp => "🔌",
            Planner => "🗂",
            Todos => "✓",
            QuickNotes => "📝",
            Calendar => "📅",
            Monitor => "📊",
            Model => "🦙",
            Library => "📚",
            Download => "⬇",
            Compare => "⚖",
            DeepResearch => "🔬",
            Server => "🖥",
            Instances => "🧩",
            Context => "📐",
            Gpu => "🎛",
            Performance => "⚡",
            Sampling => "🎲",
            Advanced => "⚙",
            Api => "🔑",
            Settings => "🎨",
            Overview => "🗺",
            Observability => "🔭",
        }
    }

    pub fn mono_icon(&self) -> &'static str {
        use Tab::*;
        match self {
            Chat => "◻",
            Agents => "◈",
            Mcp => "⊕",
            Planner => "▤",
            Todos => "☐",
            QuickNotes => "≡",
            Calendar => "▦",
            Monitor => "▣",
            Model => "◉",
            Library => "▥",
            Download => "↓",
            Compare => "⇌",
            DeepResearch => "⊗",
            Server => "▪",
            Instances => "◫",
            Context => "◳",
            Gpu => "▨",
            Performance => "▸",
            Sampling => "◇",
            Advanced => "◎",
            Api => "◑",
            Settings => "◈",
            Overview => "◉",
            Observability => "◍",
        }
    }

    pub fn as_str(&self) -> &'static str {
        use Tab::*;
        match self {
            Chat => "chat",
            Agents => "agents",
            Mcp => "mcp",
            Planner => "planner",
            Todos => "todos",
            QuickNotes => "quick_notes",
            Calendar => "calendar",
            Monitor => "monitor",
            Model => "model",
            Library => "library",
            Download => "download",
            Compare => "compare",
            DeepResearch => "deep_research",
            Server => "server",
            Instances => "instances",
            Context => "context",
            Gpu => "gpu",
            Performance => "performance",
            Sampling => "sampling",
            Advanced => "advanced",
            Api => "api",
            Settings => "settings",
            Overview => "overview",
            Observability => "observability",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        use Tab::*;
        match s {
            "chat" => Some(Chat),
            "agents" => Some(Agents),
            "mcp" => Some(Mcp),
            "planner" => Some(Planner),
            "todos" => Some(Todos),
            "quick_notes" => Some(QuickNotes),
            "calendar" => Some(Calendar),
            "monitor" => Some(Monitor),
            "model" => Some(Model),
            "library" => Some(Library),
            "download" => Some(Download),
            "compare" => Some(Compare),
            "deep_research" => Some(DeepResearch),
            "server" => Some(Server),
            "instances" => Some(Instances),
            "context" => Some(Context),
            "gpu" => Some(Gpu),
            "performance" => Some(Performance),
            "sampling" => Some(Sampling),
            "advanced" => Some(Advanced),
            "api" => Some(Api),
            "settings" => Some(Settings),
            "overview" => Some(Overview),
            "observability" => Some(Observability),
            _ => None,
        }
    }
}


#[derive(Clone, Copy)]
pub struct AppCtx {
    /// Working copy of the canonical config (hydrated from `get_config`).
    pub config: RwSignal<ServerConfig>,
    pub active_tab: RwSignal<Tab>,
    /// Chat target override (host, port) when routed to a non-local instance.
    pub routed_instance: RwSignal<Option<(String, u16)>>,
    /// Selected model for the active instance.
    pub routed_model: RwSignal<Option<String>>,
    /// Chat history that persists across tab changes and app restarts.
    pub chat_history: RwSignal<Vec<shared::ipc::ChatMessage>>,
    pub immersive: RwSignal<bool>,
    /// Global search box text (sidebar filter / quick nav).
    pub search: RwSignal<String>,
    /// Whether llama-server is currently running (polled).
    pub server_running: RwSignal<bool>,
    /// Persistent collapse state for Built-in Tools pane.
    pub tools_collapsed: RwSignal<bool>,
    /// Persists the chat input draft across tab switches.
    pub chat_draft: RwSignal<String>,

    // ── Persistent chat streaming state (survives tab switches) ──────────────
    /// Live message list — updated by the global ipc::listen handler in App.
    pub chat_messages: RwSignal<Vec<ChatMessage>>,
    pub chat_generating: RwSignal<bool>,
    pub chat_error: RwSignal<Option<String>>,
    pub chat_current_stream: RwSignal<String>,
    pub chat_current_agent: RwSignal<Option<String>>,
    pub chat_plan: RwSignal<Vec<PlanStep>>,
    pub chat_trace: RwSignal<Vec<String>>,
    pub chat_approvals: RwSignal<Vec<(String, ApprovalRequest)>>,
    /// Toggle for the context-overview panel inside ChatTab.
    pub chat_show_context: RwSignal<bool>,

    // ── Observability event log ──────────────────────────────────────────────
    pub obs_events: RwSignal<Vec<ObsEvent>>,
}

impl AppCtx {
    /// Resolve the host, port, and model for a given use case, taking overrides and global routing into account.
    pub fn resolve_target(&self, usecase: &str) -> (String, u16, String) {
        let cfg = self.config.get_untracked();

        // 1. Check for overrides
        let ovr = match usecase {
            "planner" => Some(&cfg.override_planner),
            "calendar" => Some(&cfg.override_calendar),
            "memory" => Some(&cfg.override_memory),
            "research" => Some(&cfg.override_research),
            "compare" => Some(&cfg.override_compare),
            _ => None,
        };

        if let Some(o) = ovr {
            if o.enabled {
                let model = if o.model.is_empty() {
                    if cfg.model_alias.is_empty() { cfg.model_path.clone() } else { cfg.model_alias.clone() }
                } else {
                    o.model.clone()
                };
                return (o.host.clone(), o.port, model);
            }
        }

        // 2. Fall back to global routed target
        let (host, port) = match self.routed_instance.get() {
            Some((h, p)) => (h, p),
            None => (cfg.host.clone(), cfg.port),
        };

        let model = match self.routed_model.get() {
            Some(m) => m,
            None => {
                if cfg.model_alias.is_empty() { cfg.model_path.clone() } else { cfg.model_alias.clone() }
            }
        };

        (host, port, model)
    }
}


impl AppCtx {
    /// Persist the current working-copy config to the backend asynchronously.
    pub async fn save_async(self) -> Result<(), String> {
        let cfg = self.config.get_untracked();
        api::update_config(cfg).await
    }

    /// Persist the current working-copy config to the backend (debounce-free; we
    /// call this on commit events, not on every keystroke).
    pub fn save(self) {
        spawn_local(async move {
            if let Err(e) = self.save_async().await {
                tracing::error!("update_config failed: {e}");
            }
        });
    }

    /// Mutate the working-copy config and persist.
    pub fn update_cfg(self, f: impl FnOnce(&mut ServerConfig)) {
        self.config.update(f);
        self.save();
    }
}
