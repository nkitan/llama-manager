//! Native Tauri v2 backend for llama-manager.
//!
//! Owns all OS access (process spawning, filesystem, networking) and the
//! canonical application state, exposing them to the Leptos frontend through
//! commands (request/response) and events (streaming). See [[migration-conventions]].

mod agent;
mod commands;
mod config_io;
mod logging;
mod state;
mod util;
pub mod library;

use state::AppState;

/// Build and run the desktop application.
pub fn run() {
    logging::init();

    // Load the canonical config once at startup; the frontend reads it via
    // `get_config` and mutates it via `update_config` (which re-persists + emits).
    let config = config_io::load();

    tauri::Builder::default()
        .manage(AppState::new(config))
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::update_config,
            commands::chat::chat_list_models,
            commands::chat::chat_send,
            commands::chat::chat_save_history,
            commands::chat::chat_load_history,
            commands::agent::agent_start,
            commands::agent::agent_approve,
            commands::agent::agent_cancel,
            commands::agent::agent_status,
            commands::server::server_start,
            commands::server::server_stop,
            commands::server::server_status,
            commands::server::pick_path,
            commands::window::win_minimize,
            commands::window::win_toggle_maximize,
            commands::window::win_close,
            commands::store::notes_get,
            commands::store::notes_set,
            commands::store::notes_store_get,
            commands::store::notes_store_set,
            commands::store::todos_get,
            commands::store::todos_set,
            commands::remaining::library_get_index,
            commands::remaining::library_scan,
            commands::remaining::library_enrich_single,
            commands::remaining::library_enrich_all,
            commands::remaining::planner_load,
            commands::remaining::planner_save,
            commands::remaining::monitor_load,
            commands::remaining::monitor_clear_events,
            commands::remaining::calendar_load,
            commands::remaining::calendar_save,
            commands::remaining::calendar_add_event,
            commands::remaining::calendar_delete_event,
            commands::remaining::download_get_targets,
            commands::remaining::download_save_targets,
            commands::remaining::download_start,
            commands::remaining::download_cancel,
            commands::remaining::download_status,
            commands::remaining::instances_detect,
            commands::remaining::mcp_get_registry,
            commands::remaining::mcp_save_registry,
            commands::remaining::deep_research_start,
            commands::remaining::deep_research_status,
            commands::remaining::deep_research_cancel,
            commands::remaining::deep_research_list_reports,
            commands::remaining::deep_research_read_report,
            commands::remaining::compare_run_bench,
            commands::remaining::compare_run_eval,
            commands::remaining::memory_get,
            commands::remaining::memory_clear,
            commands::remaining::server_suggest_optimizations,
            commands::remaining::obsidian_memories_get,
            commands::remaining::obsidian_memory_save,
            commands::remaining::obsidian_memory_delete,
            commands::remaining::list_skills_and_agents,
        ])
        .setup(|app| {
            tracing::info!("llama-manager backend ready");
            commands::remaining::spawn_calendar_scheduler(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
