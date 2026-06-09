//! Filesystem persistence for [`ServerConfig`].
//!
//! These helpers were intentionally kept out of the wasm-safe `shared` crate
//! (they use `std::fs`). The backend is the *only* place that touches the config
//! file on disk — the frontend always goes through the `get_config` /
//! `update_config` commands (see [[migration-conventions]]).

use std::path::PathBuf;

use shared::ServerConfig;

/// Directory holding `config.json` and the app's sidecar JSON files
/// (`agent_activities.json`, `model_index.json`, …).
///
/// Preserves the legacy behaviour of living in the repo root so the migrated app
/// reads the *same* files as the old one. Overridable via `LLAMA_MANAGER_DIR`.
pub fn config_dir() -> PathBuf {
    std::env::var("LLAMA_MANAGER_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/notroot/Work/llama-manager"))
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Load the config from disk, falling back to defaults (and writing them out) if
/// the file is missing or invalid.
pub fn load() -> ServerConfig {
    let dir = config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(json) => match serde_json::from_str::<ServerConfig>(&json) {
            Ok(cfg) => {
                tracing::info!(?path, "loaded config");
                cfg
            }
            Err(e) => {
                tracing::warn!(%e, "config.json invalid; using defaults");
                ServerConfig::default()
            }
        },
        Err(_) => {
            tracing::info!("config.json missing; writing defaults");
            let cfg = ServerConfig::default();
            let _ = save(&cfg);
            cfg
        }
    }
}

/// Persist the config to disk (pretty-printed JSON).
pub fn save(cfg: &ServerConfig) -> Result<(), String> {
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), json).map_err(|e| e.to_string())?;
    tracing::debug!("config saved");
    Ok(())
}
