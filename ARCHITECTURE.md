# Architecture

llama-manager is a **Tauri v2 + Leptos (CSR)** desktop app. This document is the
binding standard for the migration and all future work — apply it to every tab.

## Workspace

```
crates/shared   wasm-safe types shared by backend + frontend (no fs/process/native net)
src-tauri       native backend: all OS access + canonical state
src-ui          Leptos CSR frontend (wasm32), bundled by Trunk
legacy          original Dioxus app, preserved for reference (built on demand)
```

`cargo build` / `cargo check` build only `shared` + `src-tauri` (native). Build the
others explicitly: `trunk build` (frontend), `cargo build -p llama-manager-legacy`.

## IPC model (one model, no exceptions)

- **Request/response** → `#[tauri::command]` in `src-tauri/src/commands/`. The
  frontend calls them only through the typed wrappers in `src-ui/src/api.rs`
  (which go through `src-ui/src/ipc.rs` — the *only* file that touches
  `window.__TAURI__`).
- **Backend → frontend streaming** → the Tauri **event system**. Each domain emits
  a single tagged enum on one channel (`chat://event`, `agent://event`,
  `config://changed`), defined in `crates/shared/src/ipc.rs`. The frontend
  registers one `ipc::listen` per channel and matches on the tag.
- **The frontend never touches the OS.** No `std::fs`, `std::process`, dialogs, or
  native HTTP in `src-ui` — those live in `src-tauri`.

## State

The backend owns the truth: `AppState` (`src-tauri/src/state.rs`) holds the config,
the `llama-server` child handle, and the live-agent registry behind `tauri::State`.
The frontend keeps a *working copy* (`AppCtx` in `src-ui/src/state.rs`) hydrated via
`get_config` and re-synced on `config://changed`. All config writes go through
`update_config`, which persists to disk and broadcasts the change.

## Agent engine (`src-tauri/src/agent/`)

Layered and foolproof:

- **L1 `engine.rs`** — ReAct loop using structured OpenAI tool calls validated
  against each tool's JSON schema (never XML-tag parsing).
- **L2 `planner.rs`** — decomposes the task into a plan before acting.
- **L3 `supervisor.rs`** — `spawn_subagent` runs tracked, depth-capped sub-agents,
  each streaming under its own `agent_id`.
- **L4 `approval.rs`** — sensitive tools block on a human approval gate; every tool
  call is bounded by a timeout + retries; runs are cancellable; errors are
  structured and fed back to the model.

Tools implement one `Tool` trait (`src-tauri/src/agent/tools/`): `name`,
`description`, `parameters` (JSON schema), `is_sensitive`, `async run`. Adding a
tool is one file + one line in `ToolRegistry::builtin`.

## Performance

Fine-grained Leptos signals (no whole-tree re-renders); stream long outputs
incrementally (never refetch transcripts); virtualize long lists; heavy work on
tokio tasks off the UI thread; lean release WASM (`opt-level="z"` + lto, set in the
root `Cargo.toml`); bounded buffers.

## UI / theming

Native window transparency (Tauri) + real CSS `backdrop-filter` blur on `.glass`
panels, with a frosted-surface fallback if a platform ghosts (drive via
`ui_blur_intensity`). All styling reads CSS custom properties set on `.app-root`
from `cfg.ui_*` (`src-ui/src/theme.rs`) — never read preset statics at render time.
Shared UI primitives live in `src-ui/src/components.rs`.

## Logging

Backend: `tracing` (`RUST_LOG`), logging at command entry/exit, every spawned
process, every tool call, and every IPC error. Frontend: `tracing-wasm` +
`console_error_panic_hook`. Comments explain *why* at IPC boundaries and agent
decision points.

## Adding a tab (the pattern)

1. Add backend command(s) in `src-tauri/src/commands/` (+ events if streaming).
2. Add DTOs to `crates/shared/src/ipc.rs`.
3. Add typed wrappers in `src-ui/src/api.rs`.
4. Add a `Tab` variant + a component in `src-ui/src/tabs/`, reading `AppCtx`.
