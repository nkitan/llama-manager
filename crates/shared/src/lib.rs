//! Types shared between the native Tauri backend (`src-tauri`) and the Leptos
//! WASM frontend (`src-ui`).
//!
//! This crate is compiled for both targets, so it must remain free of any
//! platform-specific code (no `std::fs`, `std::process`, native `reqwest`, …).
//! Filesystem persistence of [`config::ServerConfig`] lives in the backend
//! (`src-tauri/src/config_io.rs`); the pure argument-building / serialization
//! logic lives here so both sides agree on the wire format.

pub mod config;
pub mod ipc;

pub use config::ServerConfig;
pub use config::ModelOverride;
pub use config::CustomTheme;
