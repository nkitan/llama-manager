//! Tauri command layer. Thin wrappers that validate input, call into the native
//! services, and (for streaming work) emit events. No business logic lives in the
//! frontend (see [[migration-conventions]]).

pub mod agent;
pub mod chat;
pub mod config;
pub mod server;
pub mod store;
pub mod window;
pub mod remaining;
