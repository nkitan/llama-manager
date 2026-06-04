//! Data-transfer objects exchanged over the Tauri IPC bridge.
//!
//! Two kinds of traffic (see [[migration-conventions]]):
//!   * request/response — command argument + return types (`*Request` / `ModelList` / …)
//!   * backend → frontend streaming — the [`ChatEvent`] and [`AgentEvent`] enums.
//!
//! We stream a single *tagged* enum per domain on one event channel
//! (`chat://event`, `agent://event`) rather than many event names. The frontend
//! registers one listener and `match`es on the tag — fewer moving parts, and new
//! event variants don't require new listeners.

use serde::{Deserialize, Serialize};

/// Event channel names (the single source of truth for both sides).
pub const CHAT_EVENT: &str = "chat://event";
pub const AGENT_EVENT: &str = "agent://event";
pub const CONFIG_CHANGED_EVENT: &str = "config://changed";
/// Each llama-server stdout/stderr line is emitted as a plain `String` here.
pub const SERVER_LOG_EVENT: &str = "server://log";

// ── Chat ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system" | "user" | "assistant"
    pub content: String,
}

// ── Productivity (todos / notes) ─────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub done: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Note {
    pub name: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct NotesStore {
    pub notes: Vec<Note>,
}

/// Arguments for the `chat_send` command. `host`/`port` target either the local
/// server or a routed instance; `stream_id` correlates the streamed events back
/// to the in-flight assistant message in the UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub stream_id: String,
    pub host: String,
    pub port: u16,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
}

/// Return type of `chat_list_models`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModelList {
    pub online: bool,
    pub models: Vec<String>,
}

/// Streamed on [`CHAT_EVENT`]. `stream_id` lets the UI route deltas to the right
/// message when (rarely) more than one request is in flight.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatEvent {
    Token { stream_id: String, delta: String },
    Done { stream_id: String },
    Error { stream_id: String, message: String },
}

// ── Agent ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub status: PlanStatus,
}

/// A structured tool invocation proposed by the model (L1, validated against the
/// tool's JSON schema before execution).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub call_id: String,
    pub tool: String,
    pub args: serde_json::Value,
    /// True for side-effecting tools that require human approval (L4).
    pub sensitive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub ok: bool,
    pub output: String,
}

/// Emitted when a sensitive tool wants to run; the backend awaits an
/// `agent_approve` command carrying the matching `call_id`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub call_id: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub reason: String,
}

/// Arguments for `agent_start`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentRequest {
    pub host: String,
    pub port: u16,
    pub model: String,
    pub task: String,
}

/// Arguments for `agent_approve`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub agent_id: String,
    pub call_id: String,
    pub approved: bool,
}

/// Streamed on [`AGENT_EVENT`]. `agent_id` identifies the originating agent or
/// sub-agent so the UI can render a sub-agent tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentEvent {
    Started {
        agent_id: String,
        parent: Option<String>,
        role: String,
        task: String,
    },
    Plan {
        agent_id: String,
        steps: Vec<PlanStep>,
    },
    PlanUpdate {
        agent_id: String,
        step_id: String,
        status: PlanStatus,
    },
    Step {
        agent_id: String,
        index: u32,
        thought: String,
    },
    Token {
        agent_id: String,
        delta: String,
    },
    ToolCall {
        agent_id: String,
        call: ToolCall,
    },
    ToolResult {
        agent_id: String,
        result: ToolResult,
    },
    ApprovalRequest {
        agent_id: String,
        request: ApprovalRequest,
    },
    SubAgentSpawned {
        agent_id: String,
        child_id: String,
        role: String,
        task: String,
    },
    Log {
        agent_id: String,
        message: String,
    },
    Done {
        agent_id: String,
        final_text: String,
    },
    Error {
        agent_id: String,
        message: String,
    },
}

// ── Shared structs for remaining tabs ────────────────────────────────────────

// ── Model Library ──
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScannedModel {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub clean_name: String,
    pub use_case: String,
    pub hf_link: String,
    pub github_link: String,
    pub size_info: String,
    pub status: String, // "pending_enrichment", "enriching", "enriched", "failed"
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

// ── Planner / Kanban ──
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    Done,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Backlog => "Backlog",
            Self::Todo => "Todo",
            Self::InProgress => "In Progress",
            Self::Done => "Done",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "Backlog" => Self::Backlog,
            "In Progress" => Self::InProgress,
            "Done" => Self::Done,
            _ => Self::Todo,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KanbanTask {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub assigned_agent: String,
    pub status: TaskStatus,
    pub priority: String, // "Low", "Medium", "High"
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PlannerState {
    pub tasks: Vec<KanbanTask>,
}

// ── Agent Monitor ──
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentStatus {
    pub name: String,
    pub current_action: String,
    pub status: String, // "Idle", "Active", "Offline"
    pub last_active: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentActivityEvent {
    pub timestamp: String,
    pub agent_name: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MonitorState {
    pub agents: Vec<AgentStatus>,
    pub events: Vec<AgentActivityEvent>,
}

// ── Calendar ──
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Pending,
    Firing,
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub time: String,   // Format: "YYYY-MM-DDTHH:MM:SS"
    pub prompt: String, // Prompt to fire when the event time is reached
    pub note: Option<String>,
    pub color: Option<String>,
    pub status: EventStatus,
    pub result: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CalendarState {
    pub events: Vec<CalendarEvent>,
}

// ── Instance Monitor ──
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlamaInstance {
    pub hostname: String,
    pub port: u16,
    pub status: String, // "online" | "offline" | "checking"
    pub models: Vec<String>,
}

// ── Downloader / Benchmark / Deep Research ──
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadStatus {
    pub is_downloading: bool,
    pub log: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BenchmarkResult {
    pub pp: String,
    pub tg: String,
    pub pp_speed: String,
    pub tg_speed: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BenchmarkOutput {
    pub raw: String,
    pub results: Vec<BenchmarkResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResearchStatus {
    pub is_running: bool,
    pub log: String,
    pub report: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResearchReportInfo {
    pub filename: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptimizationSuggestion {
    pub key: String,
    pub label: String,
    pub current: String,
    pub recommended: String,
    pub reason: String,
    pub selected: bool,
}
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Memory {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub scope: String, // "global" or "project"
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SkillOrAgentFile {
    pub name: String,
    pub path: String,
    pub is_skill: bool,
    pub content: String,
    pub source: String,
}
