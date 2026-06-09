//! Tab components. Each reads cross-tab state from `AppCtx` and talks to the
//! backend via `crate::api`, following the Chat reference pattern.

pub mod chat;
pub mod config_tabs;
pub mod observability;
pub mod overview;
pub mod productivity;
pub mod server;
pub mod settings;
pub mod remaining;
