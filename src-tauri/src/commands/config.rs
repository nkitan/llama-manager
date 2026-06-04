//! Config commands. The frontend reads the canonical config via `get_config` and
//! writes through `update_config`, which persists to disk and broadcasts
//! `config://changed` so every window stays in sync (replacing the legacy 1 s
//! disk-polling loop and its read/write race).

use shared::{ServerConfig, ipc::CONFIG_CHANGED_EVENT};
use tauri::{AppHandle, Emitter, State};

use crate::{config_io, state::AppState};

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> ServerConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: ServerConfig,
) -> Result<(), String> {
    tracing::debug!(theme = %config.theme_name, "update_config");
    *state.config.lock().unwrap() = config.clone();
    config_io::save(&config)?;
    // Notify the UI (and any other window) that the canonical config moved.
    if let Err(e) = app.emit(CONFIG_CHANGED_EVENT, &config) {
        tracing::warn!(%e, "failed to emit config changed event");
    }
    Ok(())
}
