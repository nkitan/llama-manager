# State of Migration — Release Notes

    _Branch: `migrate/tauri-leptos` · base: `master` (still the working Dioxus app)_

    ## 1. Overview

    Migrating `llama-manager` from a **Dioxus-desktop monolith** (`legacy/src/main.rs`, ~11k lines)
    to **Tauri v2 + Leptos (CSR, via Trunk)**. The frontend runs as WASM in the WebView and talks to
    a native Rust backend over Tauri IPC: `#[tauri::command]` for request/response, the Tauri **event
    system** for streaming.

    Rationale captured in `ARCHITECTURE.md` and the project memory.

    ## 2. Workspace Layout

    crates/shared   wasm-safe types shared by backend + frontend (ServerConfig, enums, IPC DTOs)
    src-tauri       native backend — all OS access + canonical AppState
    src-ui          Leptos CSR frontend (wasm32), bundled by Trunk
    legacy          original Dioxus app, preserved standalone (own lockfile) for reference

    `cargo check`/`cargo test` build only `shared` + `src-tauri`. Frontend: `cd src-ui && trunk build`.
    Full app: `cargo tauri dev`.

    ## 3. Changes Done

    ### Backend (`src-tauri`)
    - **AppState** (`state.rs`): canonical `Mutex<ServerConfig>`, `Option<Child>` (llama-server),
      live-agent registry.
    - **Commands**:
      - Config: `get_config`, `update_config` (persists + emits `config://changed`).
  ↑/  - Chat: `chat_list_models`, `chat_send` (SSE streaming → `chat://event`).
      - Agent: `agent_start`, `agent_approve`, `agent_cancel`, `agent_status`.
           - Server: `server_start`/`server_stop`/`server_status` (streams stdout/stderr → `server://log`),
        `pick_path` (native rfd picker).
      - Window: `win_minimize`, `win_toggle_maximize`, `win_close` (custom chrome).
      - Store: `notes_get/set`, `todos_get/set`.
    - **Agent engine** (`src-tauri/src/agent/`) — replaces the legacy XML-tag loop:
      - L1 ReAct with structured OpenAI tool-calling + JSON-schema validation (`Tool` trait registry:
        web_search, web_scrape, add_todo, read_notes, run_command).
      - L2 planner; L3 supervisor/sub-agents (depth-capped); L4 approval gates + timeouts + retries +
        cancellation + file-backed memory.
    - **Logging**: `tracing`. **Tests**: 6 unit tests (SSE + planner parsing) pass.

    ### Frontend (`src-ui`)
    - IPC layer (`ipc.rs`) — only place touching `window.__TAURI__`; typed `invoke`/`listen`.
    - `api.rs` — typed wrapper per command.
    - State (`state.rs`) — `AppCtx` (config working copy, active tab, routed instance, immersive,
      search, server_running) + full `Tab` enum + grouped nav.
    - Theming (`theme.rs`) — `Palette` + presets **Cal Light** (default), **Midnight Glass**,
      **Slate Dark**; custom themes derived from `ui_*` with contrast-aware on-colors.
    - Design system (`style.css` + `components.rs`) — DESIGN.md tokens (type scale, spacing, radii,
      colors); `Card`, `PageHeader`, `TextField`/`ToggleField`/`SelectField`, `Spinner`, plus
      `field_text!/field_num!/field_bool!` macros.
    - **Custom window chrome**: native decorations off; themeable titlebar (drag + min/max/close) and
      topbar (active-tab title, global search, server status pill).
    - **Tabs implemented**: Chat (+ agent UI: plan, trace, sub-agents, approvals), Server (start/stop +
      command preview + live logs), Model, Context, GPU, Performance, Sampling, Advanced, API,
      Settings (theme presets + appearance + app prefs), Todos, Quick Notes.

    ### Config / decisions
    - `legacy` excluded from the workspace (its `dioxus-desktop` pins `wry ^0.53.5`, colliding with the
      Tauri stack's `wry ^0.55`).
    - `config.json` `theme_name` set to `"Cal Light"` (old dark look preserved as Midnight Glass preset).
    - Env gotcha: WebKitGTK needs `WEBKIT_DISABLE_DMABUF_RENDERER=1` to launch here; Trunk installed as
      a prebuilt binary.

    ## 4. Porting Status of Remaining Tabs

     All remaining tabs have been fully migrated to the new Tauri + Leptos architecture:
     - **Library** — Scans directories, updates local index, and supports HuggingFace meta enrichments.
     - **Downloader** — Saves download list, starts/stops background downloads via tmux, and displays live log stream.
     - **Instances** — Scans common llama-server ports and routes front-end chat traffic.
     - **Agents & Memory** — Displays agent lessons memory database and supports tools configurations.
     - **MCP Tools** — Features a full JSON registry editor and saves tool servers.
     - **Monitor** — Auto-polls and shows active agent processes and activity timelines.
     - **Planner (Kanban)** — Allows creating planner tasks and moving them between Todo/InProgress/Done columns.
     - **Calendar** — Timeline scheduling of prompts, trigger parameters, and event removal.
     - **Compare** — Evaluates side-by-side responses (blind prompting + reveal + vote) and runs `llama-benchy` suites.
     - **Deep Research** — Starts/stops deep research Python agent and streams query logs/reports.

     ## 5. Functional gaps / follow-ups
     - ChromaDB semantic memory recall (backend agent currently file-based lessons only).
     - Full MCP tool integration into the `Tool` trait registry.
     - Browser-harness/camofox scraping path (currently HTTP-only).
     - Per-field validation, command-preview on more tabs, virtualized long lists (chat/logs).
     - Verify token streaming + the full agent loop against a running `llama-server` (compiles + wired;
       not yet runtime-verified — no model available in the build env).
     - Migrate the legacy "Launch Optimizer" suggestions and `sidebar_favorites`.

     ## 6. Verification done vs pending
     - ✅ `cargo check`/`test` (backend) compiles and tests pass successfully.
     - ✅ `cargo check -p ui --target wasm32-unknown-unknown` (frontend) compiles with 0 warnings.
     - ✅ `trunk build` bundles the Leptos frontend successfully.