//! Backend tracing setup. Replaces the legacy `log_*!` macros with structured
//! `tracing`, configurable via the `RUST_LOG` env var (defaults to `info`).

use tracing_subscriber::{EnvFilter, fmt};

/// Initialise the global tracing subscriber. Safe to call once at startup.
pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,llama_manager_app=debug"));
    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
    tracing::info!("tracing initialised");
}
