# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build                  # dev build
cargo build --release        # release build
cargo run                    # run dev
cargo run --release          # run release
```

No test suite. No linter config beyond rustfmt defaults.

## Architecture

Single-binary Dioxus desktop app. Entry point is `fn main()` in `src/main.rs` (~10k lines). All UI lives in that file; all non-UI logic is split into modules.

### Module layout

| File | Purpose |
|------|---------|
| `src/main.rs` | UI components, CSS, state, llama-server process management |
| `src/config.rs` | `ServerConfig` struct + `CustomTheme` + all enums (`SplitMode`, `CacheType`, etc.) |
| `src/agent.rs` | Agent task runner, `MemoryManager` (episodic memory files), `McpRegistry` |
| `src/library.rs` | Model file scanning, index persistence, HuggingFace metadata enrichment |
| `src/logger.rs` | `log_info!` / `log_debug!` / `log_warn!` / `log_error!` macros + global log level |
| `src/planner.rs` | Kanban task persistence |
| `src/todo.rs` | Todo persistence |
| `src/notes.rs` | Quick notes persistence |
| `src/calendar.rs` | Calendar events + firing-trigger logic |

### State model

`ServerConfig` (defined in `config.rs`) is the single global state, held in a Dioxus `Signal<ServerConfig>` in `App`. It is passed by signal reference to every tab component. A `use_effect` auto-saves it to `config.json` on every change. A polling `use_resource` (1 s interval) watches `config.json` on disk and reloads if it drifts.

Config file location: `get_default_config_path()` → currently returns the repo root (`/home/notroot/Work/llama-manager`). Adjust this function to change the path.

### Tab routing

`Tab` enum drives sidebar navigation. Each variant maps to a `Tab*` component rendered by a `match active_tab()` in `App`. To add a tab: add the variant to `Tab`, implement `label()` / `icon()` / `as_str()`, create a `TabFoo(config: Signal<ServerConfig>, ...)` component, and add it to the render match.

### Theme system

All rendering uses `cfg.ui_*` fields directly (working-copy model). `theme_name` is just a display label. When a preset is selected, `apply_preset_to_config` copies its values into `ui_*`. On config load, `migrate_preset_to_ui_fields` does the same for any preset-named theme. Never look up preset static values at render time — always read `cfg.ui_*`.

Built-in presets are in the `THEME_PRESETS` const. Custom themes are stored in `cfg.custom_themes: Vec<CustomTheme>` and saved in `config.json`.

### llama-server process

`App` spawns `llama-server` as a child process via `std::process::Command`. Stdout/stderr are streamed via threads into a `Arc<Mutex<Vec<String>>>` log buffer. The process handle is stored in a signal. Start/stop buttons in the UI write to this signal.

### External integrations

- **SearXNG**: HTTP requests to `cfg.searxng_url` for search in Chat and Deep Research tabs.
- **Deep Research**: spawns `python deep_research.py` in the repo root.
- **Chroma memory**: `chroma_memory.py` called via subprocess from `agent.rs`.
- **MCP tools**: `McpRegistry` loads JSON config from the config dir, calls tool endpoints over HTTP.

### CSS

All styles are in `const CSS: &str = r#"..."#` in `main.rs` (starts at line ~587). The stylesheet uses CSS custom properties (`--color-*`, `--font-*`) set dynamically from `ui_*` config fields via inline `style` on the root element.
