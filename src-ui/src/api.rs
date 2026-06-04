//! Typed wrappers over [`crate::ipc::invoke`] — one function per backend command.
//! Argument object keys match the command parameter names.

#![allow(dead_code)]


use serde_json::json;
use shared::ServerConfig;
use shared::ipc::{
    AgentRequest, ApprovalDecision, ChatRequest, ModelList, TodoItem, NotesStore,
    ScannedModel, KanbanTask, PlannerState, MonitorState, CalendarEvent, CalendarState,
    LlamaInstance, DownloadStatus, BenchmarkOutput, ResearchStatus, ResearchReportInfo,
    OptimizationSuggestion, Memory, ChatMessage, SkillOrAgentFile,
};

use crate::ipc;

// ── Config ──────────────────────────────────────────────────────────────────
pub async fn get_config() -> Result<ServerConfig, String> {
    ipc::invoke("get_config", &ipc::no_args()).await
}

pub async fn update_config(config: ServerConfig) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("update_config", &json!({ "config": config })).await?;
    Ok(())
}

// ── Chat ──────────────────────────────────────────────────────────────────
pub async fn chat_list_models(host: String, port: u16) -> Result<ModelList, String> {
    ipc::invoke("chat_list_models", &json!({ "host": host, "port": port })).await
}

pub async fn chat_send(req: ChatRequest) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("chat_send", &json!({ "req": req })).await?;
    Ok(())
}

// ── Agent ──────────────────────────────────────────────────────────────────
pub async fn agent_start(req: AgentRequest) -> Result<String, String> {
    ipc::invoke("agent_start", &json!({ "req": req })).await
}

pub async fn agent_approve(decision: ApprovalDecision) -> Result<(), String> {
    let _: serde_json::Value =
        ipc::invoke("agent_approve", &json!({ "decision": decision })).await?;
    Ok(())
}

pub async fn agent_cancel(agent_id: String) -> Result<(), String> {
    let _: serde_json::Value =
        ipc::invoke("agent_cancel", &json!({ "agentId": agent_id })).await?;
    Ok(())
}

// ── Server lifecycle ─────────────────────────────────────────────────────────
pub async fn server_start() -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("server_start", &ipc::no_args()).await?;
    Ok(())
}

pub async fn server_stop() -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("server_stop", &ipc::no_args()).await?;
    Ok(())
}

pub async fn server_status() -> Result<bool, String> {
    ipc::invoke("server_status", &ipc::no_args()).await
}

pub async fn pick_path(directory: bool) -> Result<Option<String>, String> {
    ipc::invoke("pick_path", &json!({ "directory": directory })).await
}

// ── Window controls ──────────────────────────────────────────────────────────
pub async fn win_minimize() {
    let _: Result<serde_json::Value, _> = ipc::invoke("win_minimize", &ipc::no_args()).await;
}
pub async fn win_toggle_maximize() {
    let _: Result<serde_json::Value, _> =
        ipc::invoke("win_toggle_maximize", &ipc::no_args()).await;
}
pub async fn win_close() {
    let _: Result<serde_json::Value, _> = ipc::invoke("win_close", &ipc::no_args()).await;
}

// ── Notes / Todos ────────────────────────────────────────────────────────────
pub async fn notes_get() -> Result<String, String> {
    ipc::invoke("notes_get", &ipc::no_args()).await
}
pub async fn notes_set(content: String) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("notes_set", &json!({ "content": content })).await?;
    Ok(())
}
pub async fn notes_store_get(scope: String) -> Result<NotesStore, String> {
    ipc::invoke("notes_store_get", &json!({ "scope": scope })).await
}
pub async fn notes_store_set(scope: String, store: NotesStore) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("notes_store_set", &json!({ "scope": scope, "store": store })).await?;
    Ok(())
}
pub async fn todos_get() -> Result<Vec<TodoItem>, String> {
    ipc::invoke("todos_get", &ipc::no_args()).await
}
pub async fn todos_set(items: Vec<TodoItem>) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("todos_set", &json!({ "items": items })).await?;
    Ok(())
}

// ── Model Library ───────────────────────────────────────────────────────────
pub async fn library_get_index() -> Result<Vec<ScannedModel>, String> {
    ipc::invoke("library_get_index", &ipc::no_args()).await
}
pub async fn library_scan() -> Result<Vec<ScannedModel>, String> {
    ipc::invoke("library_scan", &ipc::no_args()).await
}
pub async fn library_enrich_single(path: String) -> Result<Vec<ScannedModel>, String> {
    ipc::invoke("library_enrich_single", &json!({ "path": path })).await
}
pub async fn library_enrich_all() -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("library_enrich_all", &ipc::no_args()).await?;
    Ok(())
}

// ── Planner / Kanban ─────────────────────────────────────────────────────────
pub async fn planner_load() -> Result<PlannerState, String> {
    ipc::invoke("planner_load", &ipc::no_args()).await
}
pub async fn planner_save(tasks: Vec<KanbanTask>) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("planner_save", &json!({ "tasks": tasks })).await?;
    Ok(())
}

// ── Agent Monitor ────────────────────────────────────────────────────────────
pub async fn monitor_load() -> Result<MonitorState, String> {
    ipc::invoke("monitor_load", &ipc::no_args()).await
}
pub async fn monitor_clear_events() -> Result<MonitorState, String> {
    ipc::invoke("monitor_clear_events", &ipc::no_args()).await
}

// ── Calendar ─────────────────────────────────────────────────────────────────
pub async fn calendar_load() -> Result<CalendarState, String> {
    ipc::invoke("calendar_load", &ipc::no_args()).await
}
pub async fn calendar_save(events: Vec<CalendarEvent>) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("calendar_save", &json!({ "events": events })).await?;
    Ok(())
}
pub async fn calendar_add_event(event: CalendarEvent) -> Result<CalendarState, String> {
    ipc::invoke("calendar_add_event", &json!({ "event": event })).await
}
pub async fn calendar_delete_event(id: String) -> Result<CalendarState, String> {
    ipc::invoke("calendar_delete_event", &json!({ "id": id })).await
}

// ── Downloader ───────────────────────────────────────────────────────────────
pub async fn download_get_targets() -> Result<String, String> {
    ipc::invoke("download_get_targets", &ipc::no_args()).await
}
pub async fn download_save_targets(targets: String) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("download_save_targets", &json!({ "targets": targets })).await?;
    Ok(())
}
pub async fn download_start(targets: String) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("download_start", &json!({ "targets": targets })).await?;
    Ok(())
}
pub async fn download_cancel() -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("download_cancel", &ipc::no_args()).await?;
    Ok(())
}
pub async fn download_status() -> Result<DownloadStatus, String> {
    ipc::invoke("download_status", &ipc::no_args()).await
}

// ── Instance Monitor ─────────────────────────────────────────────────────────
pub async fn instances_detect() -> Result<Vec<LlamaInstance>, String> {
    ipc::invoke("instances_detect", &ipc::no_args()).await
}

// ── MCP Tools ────────────────────────────────────────────────────────────────
pub async fn mcp_get_registry() -> Result<serde_json::Value, String> {
    ipc::invoke("mcp_get_registry", &ipc::no_args()).await
}
pub async fn mcp_save_registry(registry: serde_json::Value) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("mcp_save_registry", &json!({ "registry": registry })).await?;
    Ok(())
}

// ── Deep Research ────────────────────────────────────────────────────────────
pub async fn deep_research_start(query: String, host: String, port: u16, model: String) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("deep_research_start", &json!({
        "query": query,
        "host": host,
        "port": port,
        "model": model
    })).await?;
    Ok(())
}
pub async fn deep_research_status() -> Result<ResearchStatus, String> {
    ipc::invoke("deep_research_status", &ipc::no_args()).await
}
pub async fn deep_research_cancel() -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("deep_research_cancel", &ipc::no_args()).await?;
    Ok(())
}
pub async fn deep_research_list_reports() -> Result<Vec<ResearchReportInfo>, String> {
    ipc::invoke("deep_research_list_reports", &ipc::no_args()).await
}
pub async fn deep_research_read_report(filename: String) -> Result<String, String> {
    ipc::invoke("deep_research_read_report", &json!({ "filename": filename })).await
}

// ── Model Comparison / llama-benchy ──────────────────────────────────────────
pub async fn compare_run_bench(
    url: String,
    model: String,
    pp: String,
    tg: String,
    runs: String,
) -> Result<BenchmarkOutput, String> {
    ipc::invoke("compare_run_bench", &json!({
        "url": url,
        "model": model,
        "pp": pp,
        "tg": tg,
        "runs": runs
    })).await
}
pub async fn compare_run_eval(url: String, model: String, prompt: String) -> Result<String, String> {
    ipc::invoke("compare_run_eval", &json!({
        "url": url,
        "model": model,
        "prompt": prompt
    })).await
}

// ── Agent Memory ─────────────────────────────────────────────────────────────
pub async fn memory_get() -> Result<Vec<String>, String> {
    ipc::invoke("memory_get", &ipc::no_args()).await
}
pub async fn memory_clear() -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("memory_clear", &ipc::no_args()).await?;
    Ok(())
}

pub async fn server_suggest_optimizations(config: ServerConfig) -> Result<Vec<OptimizationSuggestion>, String> {
    ipc::invoke("server_suggest_optimizations", &json!({ "config": config })).await
}

pub async fn obsidian_memories_get() -> Result<Vec<Memory>, String> {
    ipc::invoke("obsidian_memories_get", &ipc::no_args()).await
}

pub async fn obsidian_memory_save(mem: Memory) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("obsidian_memory_save", &json!({ "mem": mem })).await?;
    Ok(())
}

pub async fn obsidian_memory_delete(id: String, scope: String) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("obsidian_memory_delete", &json!({ "id": id, "scope": scope })).await?;
    Ok(())
}

pub async fn chat_save_history(history: Vec<ChatMessage>) -> Result<(), String> {
    let _: serde_json::Value = ipc::invoke("chat_save_history", &json!({ "history": history })).await?;
    Ok(())
}

pub async fn chat_load_history() -> Result<Vec<ChatMessage>, String> {
    ipc::invoke("chat_load_history", &ipc::no_args()).await
}

pub async fn list_skills_and_agents() -> Result<Vec<SkillOrAgentFile>, String> {
    ipc::invoke("list_skills_and_agents", &ipc::no_args()).await
}


