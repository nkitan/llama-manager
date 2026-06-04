#![allow(non_snake_case)]

mod agent;
mod calendar;
mod config;
mod library;
mod logger;
mod notes;
mod planner;
mod todo;

use config::*;
use dioxus::prelude::*;
use chrono::Datelike;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

// ─────────────────────────────────────────────────────────────────────────────
// Tabs
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Chat,
    Planner,
    Monitor,
    Mcp,
    Agents,
    Model,
    Library,
    Download,
    Server,
    Instances,
    Context,
    Gpu,
    Performance,
    Sampling,
    Advanced,
    Api,
    Calendar,
    Todos,
    QuickNotes,
    Compare,
    DeepResearch,
    AppSettings,
}

impl Tab {
    fn label(&self) -> &str {
        match self {
            Self::Chat => "Chat",
            Self::Planner => "Task Planner (Kanban)",
            Self::Monitor => "Agent Monitor",
            Self::Mcp => "MCP Tools",
            Self::Agents => "Agents & Memory",
            Self::Model => "Model Configuration",
            Self::Library => "Model Index",
            Self::Download => "Downloader",
            Self::Server => "Server",
            Self::Instances => "Instances",
            Self::Context => "Context",
            Self::Gpu => "GPU & Memory",
            Self::Performance => "Performance",
            Self::Sampling => "Sampling",
            Self::Advanced => "Advanced",
            Self::Api => "API & Security",
            Self::Calendar => "Calendar & Triggers",
            Self::Todos => "Todos (Project/Global)",
            Self::QuickNotes => "Quick Notes",
            Self::Compare => "Model Comparison (Blind)",
            Self::DeepResearch => "Deep Research Agent",
            Self::AppSettings => "App Settings",
        }
    }
    fn icon(&self) -> &str {
        match self {
            Self::Chat => "\u{1F4AC}",         // 💬
            Self::Planner => "\u{1F4CB}",      // 📋
            Self::Monitor => "\u{1F4CA}",      // 📊
            Self::Mcp => "\u{1F6E0}",          // 🛠️
            Self::Agents => "\u{1F9E0}",       // 🧠
            Self::Model => "\u{1F4E6}",        // 📦
            Self::Library => "\u{1F4DA}",      // 📚
            Self::Download => "\u{1F4E5}",     // 📥
            Self::Server => "\u{1F310}",       // 🌐
            Self::Instances => "\u{1F5A5}",    // 🖥️
            Self::Context => "\u{1F4CF}",      // 📏
            Self::Gpu => "\u{1F3AE}",          // 🎮
            Self::Performance => "\u{26A1}",   // ⚡
            Self::Sampling => "\u{1F3B2}",     // 🎲
            Self::Advanced => "\u{1F527}",     // 🔧
            Self::Api => "\u{1F512}",          // 🔒
            Self::Calendar => "\u{1F4C5}",     // 📅
            Self::Todos => "\u{2705}",         // ✅
            Self::QuickNotes => "\u{1F4DD}",   // 📝
            Self::Compare => "\u{1F441}",      // 👁️
            Self::DeepResearch => "\u{1F50E}", // 🔍
            Self::AppSettings => "\u{2699}",   // ⚙️
        }
    }
    #[allow(dead_code)]
    fn all() -> &'static [Tab] {
        &[
            Tab::Chat,
            Tab::Planner,
            Tab::Monitor,
            Tab::Mcp,
            Tab::Agents,
            Tab::Model,
            Tab::Library,
            Tab::Download,
            Tab::Server,
            Tab::Instances,
            Tab::Context,
            Tab::Gpu,
            Tab::Performance,
            Tab::Sampling,
            Tab::Advanced,
            Tab::Api,
            Tab::Calendar,
            Tab::Todos,
            Tab::QuickNotes,
            Tab::Compare,
            Tab::DeepResearch,
            Tab::AppSettings,
        ]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Planner => "planner",
            Self::Monitor => "monitor",
            Self::Mcp => "mcp",
            Self::Agents => "agents",
            Self::Model => "model",
            Self::Library => "library",
            Self::Download => "download",
            Self::Server => "server",
            Self::Instances => "instances",
            Self::Context => "context",
            Self::Gpu => "gpu",
            Self::Performance => "performance",
            Self::Sampling => "sampling",
            Self::Advanced => "advanced",
            Self::Api => "api",
            Self::Calendar => "calendar",
            Self::Todos => "todos",
            Self::QuickNotes => "quick_notes",
            Self::Compare => "compare",
            Self::DeepResearch => "deep_research",
            Self::AppSettings => "app_settings",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "chat" => Some(Self::Chat),
            "planner" => Some(Self::Planner),
            "monitor" => Some(Self::Monitor),
            "mcp" => Some(Self::Mcp),
            "agents" => Some(Self::Agents),
            "model" => Some(Self::Model),
            "library" => Some(Self::Library),
            "download" => Some(Self::Download),
            "server" => Some(Self::Server),
            "instances" => Some(Self::Instances),
            "context" => Some(Self::Context),
            "gpu" => Some(Self::Gpu),
            "performance" => Some(Self::Performance),
            "sampling" => Some(Self::Sampling),
            "advanced" => Some(Self::Advanced),
            "api" => Some(Self::Api),
            "calendar" => Some(Self::Calendar),
            "todos" => Some(Self::Todos),
            "quick_notes" => Some(Self::QuickNotes),
            "compare" => Some(Self::Compare),
            "deep_research" => Some(Self::DeepResearch),
            "app_settings" => Some(Self::AppSettings),
            _ => None,
        }
    }
}

#[derive(Clone)]
struct ConfigSearchEntry {
    label: String,
    section: String,
    tab: Tab,
    target_id: String,
}

fn config_search_entries() -> Vec<ConfigSearchEntry> {
    vec![
        ConfigSearchEntry {
            label: "llama-server path".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-llama-server-path-1".into(),
        },
        ConfigSearchEntry {
            label: "Model File (.gguf)".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-model-file-gguf-1".into(),
        },
        ConfigSearchEntry {
            label: "Model Alias".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-model-alias-1".into(),
        },
        ConfigSearchEntry {
            label: "Model Directory".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-model-directory-models-dir-1".into(),
        },
        ConfigSearchEntry {
            label: "Model URL".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-model-url-1".into(),
        },
        ConfigSearchEntry {
            label: "HuggingFace Repo".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-huggingface-repo-1".into(),
        },
        ConfigSearchEntry {
            label: "HuggingFace File".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-huggingface-file-1".into(),
        },
        ConfigSearchEntry {
            label: "HuggingFace Token".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-huggingface-token-1".into(),
        },
        ConfigSearchEntry {
            label: "Chat Template".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-chat-template-1".into(),
        },
        ConfigSearchEntry {
            label: "System Prompt".into(),
            section: "Llama-Server".into(),
            tab: Tab::Model,
            target_id: "form-system-prompt-1".into(),
        },
        ConfigSearchEntry {
            label: "Host".into(),
            section: "Server".into(),
            tab: Tab::Server,
            target_id: "form-host-1".into(),
        },
        ConfigSearchEntry {
            label: "Port".into(),
            section: "Server".into(),
            tab: Tab::Server,
            target_id: "form-port-1".into(),
        },
        ConfigSearchEntry {
            label: "Timeout".into(),
            section: "Server".into(),
            tab: Tab::Server,
            target_id: "form-server-timeout-s-1".into(),
        },
        ConfigSearchEntry {
            label: "HTTP Threads".into(),
            section: "Server".into(),
            tab: Tab::Server,
            target_id: "form-http-threads-1".into(),
        },
        ConfigSearchEntry {
            label: "Context Size".into(),
            section: "Context".into(),
            tab: Tab::Context,
            target_id: "form-context-size-c-1".into(),
        },
        ConfigSearchEntry {
            label: "Predict".into(),
            section: "Context".into(),
            tab: Tab::Context,
            target_id: "form-max-predict-n-1".into(),
        },
        ConfigSearchEntry {
            label: "Batch Size".into(),
            section: "Context".into(),
            tab: Tab::Context,
            target_id: "form-batch-size-b-1".into(),
        },
        ConfigSearchEntry {
            label: "Micro Batch Size".into(),
            section: "Context".into(),
            tab: Tab::Context,
            target_id: "form-micro-batch-size-ub-1".into(),
        },
        ConfigSearchEntry {
            label: "Parallel Sequences".into(),
            section: "Context".into(),
            tab: Tab::Context,
            target_id: "form-parallel-sequences-np-1".into(),
        },
        ConfigSearchEntry {
            label: "GPU Layers".into(),
            section: "GPU & Memory".into(),
            tab: Tab::Gpu,
            target_id: "form-gpu-layers-ngl-1".into(),
        },
        ConfigSearchEntry {
            label: "Main GPU".into(),
            section: "GPU & Memory".into(),
            tab: Tab::Gpu,
            target_id: "form-main-gpu-mg-1".into(),
        },
        ConfigSearchEntry {
            label: "Split Mode".into(),
            section: "GPU & Memory".into(),
            tab: Tab::Gpu,
            target_id: "form-split-mode-sm-1".into(),
        },
        ConfigSearchEntry {
            label: "Tensor Split".into(),
            section: "GPU & Memory".into(),
            tab: Tab::Gpu,
            target_id: "form-tensor-split-ts-1".into(),
        },
        ConfigSearchEntry {
            label: "Cache Type K".into(),
            section: "GPU & Memory".into(),
            tab: Tab::Gpu,
            target_id: "form-k-cache-type-ctk-1".into(),
        },
        ConfigSearchEntry {
            label: "Cache Type V".into(),
            section: "GPU & Memory".into(),
            tab: Tab::Gpu,
            target_id: "form-v-cache-type-ctv-1".into(),
        },
        ConfigSearchEntry {
            label: "Threads".into(),
            section: "Performance".into(),
            tab: Tab::Performance,
            target_id: "form-threads-t-1".into(),
        },
        ConfigSearchEntry {
            label: "Threads Batch".into(),
            section: "Performance".into(),
            tab: Tab::Performance,
            target_id: "form-batch-threads-tb-1".into(),
        },
        ConfigSearchEntry {
            label: "Flash Attention".into(),
            section: "Performance".into(),
            tab: Tab::Performance,
            target_id: "".into(),
        },
        ConfigSearchEntry {
            label: "No Warmup".into(),
            section: "Performance".into(),
            tab: Tab::Performance,
            target_id: "".into(),
        },
        ConfigSearchEntry {
            label: "Check Tensors".into(),
            section: "Performance".into(),
            tab: Tab::Performance,
            target_id: "".into(),
        },
        ConfigSearchEntry {
            label: "Temperature".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-temperature-temp-1".into(),
        },
        ConfigSearchEntry {
            label: "Seed".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-seed-s-1".into(),
        },
        ConfigSearchEntry {
            label: "Top K".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-top-k-top-k-1".into(),
        },
        ConfigSearchEntry {
            label: "Top P".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-top-p-top-p-1".into(),
        },
        ConfigSearchEntry {
            label: "Min P".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-min-p-min-p-1".into(),
        },
        ConfigSearchEntry {
            label: "Repeat Penalty".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-repeat-penalty-1".into(),
        },
        ConfigSearchEntry {
            label: "Presence Penalty".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-presence-penalty-1".into(),
        },
        ConfigSearchEntry {
            label: "Frequency Penalty".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-frequency-penalty-1".into(),
        },
        ConfigSearchEntry {
            label: "Grammar".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-grammar-bnf-1".into(),
        },
        ConfigSearchEntry {
            label: "Grammar File".into(),
            section: "Sampling".into(),
            tab: Tab::Sampling,
            target_id: "form-grammar-file-1".into(),
        },
        ConfigSearchEntry {
            label: "Rope Scaling".into(),
            section: "Advanced".into(),
            tab: Tab::Advanced,
            target_id: "form-scaling-type-1".into(),
        },
        ConfigSearchEntry {
            label: "RoPE Freq Base".into(),
            section: "Advanced".into(),
            tab: Tab::Advanced,
            target_id: "form-freq-base-1".into(),
        },
        ConfigSearchEntry {
            label: "RoPE Freq Scale".into(),
            section: "Advanced".into(),
            tab: Tab::Advanced,
            target_id: "form-freq-scale-1".into(),
        },
        ConfigSearchEntry {
            label: "Yarn Orig Ctx".into(),
            section: "Advanced".into(),
            tab: Tab::Advanced,
            target_id: "form-yarn-orig-ctx-1".into(),
        },
        ConfigSearchEntry {
            label: "Yarn Ext Factor".into(),
            section: "Advanced".into(),
            tab: Tab::Advanced,
            target_id: "form-extension-factor-1".into(),
        },
        ConfigSearchEntry {
            label: "Yarn Attn Factor".into(),
            section: "Advanced".into(),
            tab: Tab::Advanced,
            target_id: "form-attention-factor-1".into(),
        },
        ConfigSearchEntry {
            label: "Yarn Beta Fast".into(),
            section: "Advanced".into(),
            tab: Tab::Advanced,
            target_id: "form-beta-fast-1".into(),
        },
        ConfigSearchEntry {
            label: "Yarn Beta Slow".into(),
            section: "Advanced".into(),
            tab: Tab::Advanced,
            target_id: "form-beta-slow-1".into(),
        },
        ConfigSearchEntry {
            label: "API Key".into(),
            section: "API & Security".into(),
            tab: Tab::Api,
            target_id: "form-api-key-api-key-1".into(),
        },
        ConfigSearchEntry {
            label: "API Key File".into(),
            section: "API & Security".into(),
            tab: Tab::Api,
            target_id: "form-api-key-file-api-key-file-1".into(),
        },
        ConfigSearchEntry {
            label: "Metrics".into(),
            section: "API & Security".into(),
            tab: Tab::Api,
            target_id: "".into(),
        },
        ConfigSearchEntry {
            label: "Slots".into(),
            section: "API & Security".into(),
            tab: Tab::Api,
            target_id: "".into(),
        },
        ConfigSearchEntry {
            label: "Slot Save Path".into(),
            section: "API & Security".into(),
            tab: Tab::Api,
            target_id: "form-slot-save-path-1".into(),
        },
        ConfigSearchEntry {
            label: "Log Format".into(),
            section: "API & Security".into(),
            tab: Tab::Api,
            target_id: "form-log-format-1".into(),
        },
        ConfigSearchEntry {
            label: "Verbose Logging".into(),
            section: "API & Security".into(),
            tab: Tab::Api,
            target_id: "".into(),
        },
        ConfigSearchEntry {
            label: "Model Scan Directories".into(),
            section: "App Settings".into(),
            tab: Tab::AppSettings,
            target_id: "form-scan-directories-1".into(),
        },
        ConfigSearchEntry {
            label: "SearxNG URL".into(),
            section: "App Settings".into(),
            tab: Tab::AppSettings,
            target_id: "form-searxng-service-url-1".into(),
        },
        ConfigSearchEntry {
            label: "Launch Optimizer".into(),
            section: "Server".into(),
            tab: Tab::Server,
            target_id: "launch-optimizer-1".into(),
        },
        ConfigSearchEntry {
            label: "Optimize Configs".into(),
            section: "Server".into(),
            tab: Tab::Server,
            target_id: "launch-optimizer-1".into(),
        },
        ConfigSearchEntry {
            label: "Chat".into(),
            section: "Navigation".into(),
            tab: Tab::Chat,
            target_id: "".into(),
        },
        ConfigSearchEntry {
            label: "Downloader".into(),
            section: "Navigation".into(),
            tab: Tab::Download,
            target_id: "".into(),
        },
        ConfigSearchEntry {
            label: "Window Transparency".into(),
            section: "App Settings".into(),
            tab: Tab::AppSettings,
            target_id: "form-ui-transparency".into(),
        },
        ConfigSearchEntry {
            label: "Background Tint / Color".into(),
            section: "App Settings".into(),
            tab: Tab::AppSettings,
            target_id: "form-ui-bg-color".into(),
        },
        ConfigSearchEntry {
            label: "Frosted Glass Panels".into(),
            section: "App Settings".into(),
            tab: Tab::AppSettings,
            target_id: "form-ui-blur".into(),
        },
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// CSS
// ─────────────────────────────────────────────────────────────────────────────

const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap');

* { margin: 0; padding: 0; box-sizing: border-box; }

:root {
    color-scheme: dark;
    --color-primary: var(--ui-accent-color);
    --color-primary-active: color-mix(in srgb, var(--ui-accent-color) 85%, #000000 15%);
    --color-primary-disabled: color-mix(in srgb, var(--ui-text-color) 12%, transparent);
    --color-ink: var(--ui-text-color);
    --color-body: color-mix(in srgb, var(--ui-text-color) 82%, transparent);
    --color-muted: color-mix(in srgb, var(--ui-text-color) 58%, transparent);
    --color-muted-soft: color-mix(in srgb, var(--ui-text-color) 42%, transparent);
    --color-hairline: var(--ui-border-color);
    --color-hairline-soft: color-mix(in srgb, var(--ui-border-color) 50%, transparent);
    --color-on-primary: #ffffff;
    --color-on-dark: var(--ui-text-color);
    --color-on-dark-soft: color-mix(in srgb, var(--ui-text-color) 70%, transparent);
    --color-brand-accent: var(--ui-accent-color);
    --color-success: #10b981;
    --color-warning: #f59e0b;
    --color-error: #ef4444;

    /* ── Uniform frosted-glass system ────────────────────────────────
       ONE transparency for the whole window. --app-opacity (injected from the
       Window Transparency slider) tints the entire .app uniformly, so the
       desktop shows through sidebar, top-bar, content and log panel equally.
       Panels are NOT separate opaque blocks — they're the same glass, gently
       lifted by a faint frost (--panel-frost, from the Glass Frost control),
       a soft specular sheen, a border and a shadow. No backdrop-filter: it
       needs the GPU compositor that webkit2gtk renders unreliably on
       transparent windows (the ghosting bug). */
    --app-opacity: 86%;
    --panel-frost: 7%;
    --glass-frost: color-mix(in srgb, var(--ui-text-color) var(--panel-frost), transparent);
    --glass-sheen: linear-gradient(135deg, rgba(255,255,255,0.06) 0%, rgba(255,255,255,0.012) 50%, rgba(255,255,255,0.04) 100%);

    /* readable "wells" (inputs, logs, nested surfaces) — translucent but solid
       enough for text; they usually nest over a panel so opacity stacks. */
    --color-canvas: color-mix(in srgb, var(--ui-bg-color) 62%, transparent);
    --color-input-bg: color-mix(in srgb, var(--ui-text-color) 11%, transparent);
    --color-surface-soft: color-mix(in srgb, var(--ui-text-color) 5%, transparent);
    --color-surface-card: var(--ui-card-bg);
    --color-surface-strong: color-mix(in srgb, var(--ui-text-color) 12%, transparent);
    --color-surface-dark: color-mix(in srgb, var(--ui-bg-color) 92%, var(--ui-text-color) 8%);
    --color-surface-dark-elevated: color-mix(in srgb, var(--ui-bg-color) 86%, var(--ui-text-color) 14%);

    /* theme-aware overlays (replace hardcoded rgba(0,0,0,X) which disappears
       on dark themes). Tinted from the text color so they read on both light
       and dark surfaces. */
    --color-overlay-1: color-mix(in srgb, var(--ui-text-color) 4%, transparent);
    --color-overlay-2: color-mix(in srgb, var(--ui-text-color) 8%, transparent);
    --color-overlay-3: color-mix(in srgb, var(--ui-text-color) 14%, transparent);
    --color-overlay-4: color-mix(in srgb, var(--ui-text-color) 22%, transparent);
    --color-overlay-white-1: rgba(255, 255, 255, 0.04);
    --color-overlay-white-2: rgba(255, 255, 255, 0.08);
    --color-overlay-white-3: rgba(255, 255, 255, 0.16);

    /* status badge palette — text + matching translucent background.
       Works for both light and dark themes. */
    --color-badge-success-text: #10b981;
    --color-badge-success-bg: color-mix(in srgb, #10b981 16%, transparent);
    --color-badge-success-border: color-mix(in srgb, #10b981 32%, transparent);
    --color-badge-warning-text: #d97706;
    --color-badge-warning-bg: color-mix(in srgb, #f59e0b 18%, transparent);
    --color-badge-warning-border: color-mix(in srgb, #f59e0b 35%, transparent);
    --color-badge-error-text: #ef4444;
    --color-badge-error-bg: color-mix(in srgb, #ef4444 16%, transparent);
    --color-badge-error-border: color-mix(in srgb, #ef4444 32%, transparent);
    --color-badge-info-text: var(--ui-accent-color);
    --color-badge-info-bg: color-mix(in srgb, var(--ui-accent-color) 16%, transparent);
    --color-badge-info-border: color-mix(in srgb, var(--ui-accent-color) 32%, transparent);
    --color-badge-neutral-text: var(--color-muted);
    --color-badge-neutral-bg: var(--color-overlay-2);
    --color-badge-neutral-border: var(--color-hairline-soft);

    /* category palette for tags/chips that need a hint of color without
       being tied to a single brand accent. */
    --color-tag-purple: #a78bfa;
    --color-tag-blue: #60a5fa;
    --color-tag-green: #4ade80;
    --color-tag-yellow: #facc15;
    --color-tag-pink: #f472b6;
    --color-tag-orange: #fb923c;
    --color-tag-cyan: #22d3ee;
    --color-tag-red: #f87171;
    --color-tag-slate: #cbd5e1;
}

/* html and body stay fully transparent so the window's own alpha shows the
   desktop; the tint lives on .app. */
html {
    background: transparent !important;
    height: 100%;
}
body {
    font-family: var(--ui-font-family), 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: transparent !important;
    color: var(--color-body);
    overflow: hidden;
    height: 100%;
    margin: 0;
    padding: 0;
}

.app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    /* the single uniform window-transparency layer for the whole app */
    background: color-mix(in srgb, var(--ui-bg-color) var(--app-opacity), transparent);
}

/* ── Focus States ────────────────────────────────────────────────── */
.btn:focus-visible,
.form-input:focus-visible,
.form-select:focus-visible,
.form-textarea:focus-visible,
.sidebar-item:focus-visible,
[role="button"]:focus-visible {
    outline: 2px solid var(--color-brand-accent);
    outline-offset: 2px;
}

/* ── Top Bar ─────────────────────────────────────────────────────── */
.top-bar {
    display: flex; align-items: center; justify-content: space-between;
    flex-wrap: nowrap; gap: 8px;
    padding: 8px 12px;
    background: var(--ui-sidebar-bg), var(--glass-sheen), var(--glass-frost);
    border-bottom: 1px solid var(--color-hairline);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.18);
    min-height: 52px;
    z-index: 20;
    flex-shrink: 0;
}
.top-title {
    display: flex; align-items: center; gap: 8px;
    font-size: 15px; font-weight: 700; letter-spacing: -0.02em;
    color: var(--color-ink);
    flex-shrink: 0;
}
.top-title span { font-size: 18px; }
.top-right { display: flex; align-items: center; gap: 6px; flex-wrap: nowrap; flex-shrink: 0; }

.search-group {
    position: relative;
    display: inline-flex;
    align-items: center;
    flex: 1 1 0;
    min-width: 80px;
    max-width: 320px;
}
.search-input {
    width: 100%; padding: 6px 10px;
    border: 1px solid var(--color-hairline);
    border-radius: 8px;
    background: var(--color-input-bg);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.12);
    color: var(--color-ink);
    font-size: 12.5px;
    transition: all 0.2s;
    font-family: inherit;
    appearance: none;
    -webkit-appearance: none;
    outline: none;
}
.search-input:focus {
    border-color: var(--color-brand-accent);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.12), 0 0 0 2px color-mix(in srgb, var(--color-brand-accent) 22%, transparent);
    background: color-mix(in srgb, var(--color-input-bg) 92%, var(--color-brand-accent) 8%);
}
.search-results {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    width: 100%;
    max-height: 300px;
    background: color-mix(in srgb, var(--ui-bg-color) 97%, transparent);
    border: 1px solid var(--color-hairline);
    border-radius: 8px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.35);
    overflow: hidden;
    z-index: 50;
}
.search-results-list {
    max-height: 260px;
    overflow-y: auto;
}
.search-results-header {
    padding: 10px 12px;
    border-bottom: 1px solid var(--color-hairline);
    font-size: 11px; font-weight: 600; color: var(--color-muted);
    text-transform: uppercase; letter-spacing: 0.05em;
}
.search-result-item {
    display: flex; align-items: center; justify-content: space-between;
    gap: 10px;
    padding: 8px 12px;
    text-decoration: none;
    color: inherit;
    transition: background 0.15s;
    border-bottom: 1px solid var(--color-hairline-soft);
    background: transparent;
    border-left: none;
    border-right: none;
    border-top: none;
    cursor: pointer;
    text-align: left;
    width: 100%;
}
.search-result-item:hover {
    background: var(--color-overlay-white-1);
}
.search-result-label {
    display: flex; flex-direction: column;
    gap: 2px;
    min-width: 0;
}
.search-result-title {
    font-size: 13px; font-weight: 600; color: var(--color-ink);
}
.search-result-meta {
    font-size: 11px; color: var(--color-muted);
}
.search-chip {
    display: inline-flex; align-items: center; justify-content: center;
    padding: 2px 8px; border-radius: 4px;
    background: var(--color-overlay-white-1); color: var(--color-muted);
    font-size: 10px; font-weight: 500; white-space: nowrap;
    border: 1px solid var(--color-hairline);
}

.status-badge {
    display: flex; align-items: center; gap: 6px;
    padding: 4px 10px; border-radius: 6px;
    font-size: 12px; font-weight: 600;
    border: 1px solid transparent;
}
.status-badge.running {
    background: var(--color-badge-success-bg);
    color: var(--color-success);
    border-color: var(--color-badge-success-border);
}
.status-badge.stopped {
    background: var(--color-overlay-white-1);
    color: var(--color-muted);
    border-color: var(--color-overlay-white-2);
}
.status-dot {
    width: 8px; height: 8px; border-radius: 50%;
}
.status-dot.running {
    background: var(--color-success);
    box-shadow: 0 0 10px rgba(16,185,129,0.6);
    animation: pulse 2s infinite;
}
.status-dot.stopped { background: var(--color-muted); }
@keyframes pulse { 0%,100%{opacity:1;} 50%{opacity:0.4;} }

/* ── Buttons ─────────────────────────────────────────────────────── */
.btn {
    padding: 8px 14px; border: none; border-radius: 8px;
    font-size: 13px; font-weight: 600; cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    display: inline-flex; align-items: center; gap: 6px;
    outline: none;
    font-family: 'Inter', sans-serif;
}
.btn:active { transform: scale(0.98); }

.btn-start {
    background: linear-gradient(135deg, #4f46e5, #6366f1);
    color: #ffffff;
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}
.btn-start:hover {
    background: linear-gradient(135deg, #4338ca, #4f46e5);
    box-shadow: 0 6px 16px rgba(99, 102, 241, 0.5);
    transform: translateY(-1px);
}
.btn-start:disabled {
    background: var(--color-primary-disabled);
    color: var(--color-muted);
    cursor: not-allowed;
    box-shadow: none;
}

.btn-stop {
    background: linear-gradient(135deg, #ef4444, #dc2626);
    color: #ffffff;
    box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}
.btn-stop:hover {
    background: linear-gradient(135deg, #dc2626, #b91c1c);
    box-shadow: 0 6px 16px rgba(239, 68, 68, 0.5);
    transform: translateY(-1px);
}

.btn-ghost {
    background: color-mix(in srgb, var(--color-ink) 6%, transparent);
    color: var(--color-body);
    border: 1px solid var(--color-hairline);
}
.btn-ghost:hover {
    background: color-mix(in srgb, var(--color-ink) 10%, transparent);
    color: var(--color-ink);
    border-color: color-mix(in srgb, var(--color-ink) 20%, transparent);
}

.btn-sm { padding: 5px 10px; font-size: 12px; }

.btn-browse {
    padding: 6px 10px;
    background: color-mix(in srgb, var(--color-ink) 6%, transparent);
    color: var(--color-body);
    border: 1px solid var(--color-hairline);
    border-radius: 6px; cursor: pointer;
    font-size: 12px; font-weight: 500; font-family: inherit;
    white-space: nowrap; transition: all 0.2s;
    outline: none;
}
.btn-browse:hover {
    background: color-mix(in srgb, var(--color-ink) 10%, transparent);
    color: var(--color-ink);
    border-color: color-mix(in srgb, var(--color-ink) 20%, transparent);
}

/* ── Main area ───────────────────────────────────────────────────── */
.main { display: flex; flex: 1; min-height: 0; overflow: hidden; background: transparent; }

/* ── Sidebar ─────────────────────────────────────────────────────── */
.sidebar {
    width: 220px; min-width: 220px;
    /* user gradient + specular sheen over the frosted-glass tint */
    background: var(--ui-sidebar-bg), var(--glass-sheen), var(--glass-frost);
    border-right: 1px solid var(--color-hairline);
    box-shadow: inset 1px 0 0 rgba(255, 255, 255, 0.05), 10px 0 30px rgba(0, 0, 0, 0.2);
    display: flex; flex-direction: column;
    padding: 12px 8px;
    gap: 4px;
    transition: width 0.4s cubic-bezier(0.16, 1, 0.3, 1),
                opacity 0.3s cubic-bezier(0.16, 1, 0.3, 1),
                padding 0.4s cubic-bezier(0.16, 1, 0.3, 1),
                border-right-color 0.3s;
    overflow-x: hidden;
    overflow-y: auto;
}
.sidebar-item {
    display: flex; align-items: center; gap: 10px;
    padding: 8px 14px; cursor: pointer;
    border-radius: 10px;
    color: var(--color-body);
    background: transparent;
    border: none;
    transition: all 0.25s cubic-bezier(0.25, 0.8, 0.25, 1);
    text-align: left;
    width: 100%;
    font-weight: 500;
    font-size: 13px;
    position: relative;
    user-select: none;
}
.sidebar-item:hover {
    background: color-mix(in srgb, var(--color-brand-accent) 10%, transparent);
    color: var(--color-ink);
    transform: translateY(-0.5px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
.sidebar-item:active {
    transform: scale(0.97);
}
.sidebar-item.active {
    background: color-mix(in srgb, var(--color-brand-accent) 22%, transparent);
    color: var(--color-brand-accent);
    font-weight: 600;
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-brand-accent) 40%, transparent),
                0 4px 15px color-mix(in srgb, var(--color-brand-accent) 18%, transparent);
}
.sidebar-item.active:hover {
    background: color-mix(in srgb, var(--color-brand-accent) 28%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-brand-accent) 55%, transparent),
                0 6px 20px color-mix(in srgb, var(--color-brand-accent) 25%, transparent);
}
.sidebar-icon {
    font-size: 14px;
    opacity: 0.85;
}
.sidebar-fav-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 13px;
    color: var(--color-muted);
    opacity: 0;
    transition: all 0.2s;
    padding: 2px 4px;
    border-radius: 4px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
}
.sidebar-item:hover .sidebar-fav-btn {
    opacity: 0.7;
}
.sidebar-fav-btn:hover {
    opacity: 1 !important;
    color: var(--color-warning) !important;
    transform: scale(1.15);
}
.sidebar-item.active .sidebar-fav-btn {
    color: var(--color-ink);
}
.sidebar-fav-btn-active {
    opacity: 0.95 !important;
    color: var(--color-warning) !important;
}
.sidebar-sub-item {
    padding-left: 20px;
    font-size: 12.5px;
}
.sidebar-section-header {
    font-size: 11px; font-weight: 700; color: var(--color-muted);
    text-transform: uppercase; letter-spacing: 0.08em;
    padding: 12px 10px 4px 10px;
}
.sidebar-section-header.interactive {
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    user-select: none;
    transition: color 0.2s, background-color 0.2s;
    border-radius: 6px;
    padding: 8px 10px;
    margin-top: 8px;
    margin-bottom: 2px;
}
.sidebar-section-header.interactive:hover {
    color: var(--color-ink);
    background: var(--color-overlay-white-1);
}
.section-arrow {
    display: inline-block;
    width: 12px;
    font-size: 9px;
    color: var(--color-muted);
    transition: transform 0.2s;
}
.sidebar-section-header.interactive:hover .section-arrow {
    color: var(--color-ink);
}
.btn-sidebar-toggle {
    background: var(--color-overlay-white-1);
    border: 1px solid var(--color-overlay-white-2);
    border-radius: 6px;
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: var(--color-body);
    font-size: 14px;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    margin-right: 8px;
    outline: none;
}
.btn-sidebar-toggle:hover {
    background: var(--color-overlay-white-2);
    border-color: var(--color-overlay-white-3);
    color: var(--color-ink);
    transform: scale(1.05);
}
.btn-sidebar-toggle:active {
    transform: scale(0.95);
}

/* ── Resizer ─────────────────────────────────────────────────────── */
.resizer-h {
    width: 4px; cursor: col-resize; background: transparent;
    transition: background-color 0.2s;
    border-left: 1px solid var(--color-hairline);
    z-index: 10;
}
.resizer-h:hover, .resizer-h:active {
    background-color: var(--color-brand-accent);
}

.resizer-v {
    height: 4px; cursor: row-resize; background: transparent;
    transition: background-color 0.2s;
    border-top: 1px solid var(--color-hairline);
    z-index: 10;
}
.resizer-v:hover, .resizer-v:active {
    background-color: var(--color-brand-accent);
}

/* ── Content ─────────────────────────────────────────────────────── */
.content {
    flex: 1 1 0; min-height: 0; overflow-y: auto; overflow-x: hidden;
    /* Use the app background color as base — prevents WebKit2GTK ghosting
       when switching tabs on transparent windows. A truly transparent
       content area causes the compositor to keep the previous tab's
       paint layer visible until the next full repaint. */
    background: color-mix(in srgb, var(--ui-bg-color) var(--app-opacity), transparent);
    display: block;
    padding: 20px;
    /* Stacking context forces WebKit to composite this layer independently,
       which fixes both ghosting and scroll tracking. */
    isolation: isolate;
    position: relative;
}
/* space between top-level children inside any tab */
.content > * + * { margin-top: 16px; }
.content::-webkit-scrollbar { width: 10px; }
.content::-webkit-scrollbar-track { background: transparent; }
.content::-webkit-scrollbar-thumb {
    background: color-mix(in srgb, var(--color-ink) 35%, transparent);
    border-radius: 5px;
    border: 2px solid transparent;
    background-clip: padding-box;
}
.content::-webkit-scrollbar-thumb:hover {
    background: color-mix(in srgb, var(--color-ink) 55%, transparent);
    background-clip: padding-box;
    border: 2px solid transparent;
}

.section-title {
    font-size: 18px; font-weight: 700; letter-spacing: -0.02em; color: var(--color-ink); margin-bottom: 2px;
    font-family: 'Inter', sans-serif;
}
.section-desc {
    font-size: 13px; color: var(--color-muted); margin-bottom: 8px; line-height: 1.4;
}

/* ── Cards ────────────────────────────────────────────────────────── */
.card {
    /* user tint + specular sheen over the frosted-glass tint */
    background: var(--ui-card-bg), var(--glass-sheen), var(--glass-frost);
    border: 1px solid var(--color-hairline);
    border-radius: 14px; padding: 18px;
    display: flex; flex-direction: column; gap: 12px;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.22), inset 0 1px 0 rgba(255,255,255,0.06);
    transition: transform 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease;
}
.card:hover {
    background:
        linear-gradient(135deg, color-mix(in srgb, var(--color-brand-accent) 9%, transparent), transparent 55%),
        var(--ui-card-bg), var(--glass-sheen), var(--glass-frost);
    border-color: color-mix(in srgb, var(--color-brand-accent) 45%, var(--color-hairline));
    box-shadow: 0 16px 44px rgba(0,0,0,0.32), inset 0 1px 0 rgba(255,255,255,0.08);
    transform: translateY(-1px);
}
.card-title {
    font-size: 11px; font-weight: 700; color: var(--color-muted);
    text-transform: uppercase; letter-spacing: 0.08em;
    margin-bottom: 2px;
}

/* ── Form  ────────────────────────────────────────────────────────── */
.form-group { display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px; }
.form-group:last-child { margin-bottom: 0; }
.form-label {
    font-size: 12.5px; font-weight: 600;
    color: var(--color-ink);
}
.form-hint { font-size: 11px; color: var(--color-muted); margin-top: 1px; line-height: 1.3; }

.form-input, .form-select {
    width: 100%; padding: 6px 10px;
    background: var(--color-input-bg);
    border: 1px solid var(--color-hairline);
    border-radius: 6px; color: var(--color-ink); font-size: 12.5px;
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.12);
    transition: border-color 0.2s, box-shadow 0.2s, background 0.2s; outline: none;
    font-family: inherit;
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
}
/* custom chevron arrow for selects */
select.form-input, select.form-select {
    padding-right: 28px;
    background-image:
        linear-gradient(45deg, transparent 50%, var(--color-muted) 50%),
        linear-gradient(135deg, var(--color-muted) 50%, transparent 50%);
    background-position: calc(100% - 14px) 50%, calc(100% - 9px) 50%;
    background-size: 5px 5px, 5px 5px;
    background-repeat: no-repeat;
    cursor: pointer;
}
.form-input:focus, .form-select:focus, .tag-input-container:focus-within {
    border-color: var(--color-brand-accent);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.12), 0 0 0 2px color-mix(in srgb, var(--color-brand-accent) 22%, transparent);
    background: color-mix(in srgb, var(--color-input-bg) 92%, var(--color-brand-accent) 8%);
}
.form-input::placeholder { color: var(--color-muted-soft); }

/* ── Range sliders ────────────────────────────────────────────────────
   The global appearance:none above strips the native slider, so range
   inputs need an explicit track + thumb or they render invisible. */
input[type="range"].form-input {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 26px;
    padding: 0;
    margin: 0;
    background: transparent;
    border: none;
    border-radius: 0;
    box-shadow: none;
    cursor: pointer;
}
input[type="range"].form-input::-webkit-slider-runnable-track {
    height: 6px;
    border-radius: 999px;
    background:
        linear-gradient(90deg,
            color-mix(in srgb, var(--color-brand-accent) 60%, transparent),
            color-mix(in srgb, var(--color-brand-accent) 26%, transparent)),
        color-mix(in srgb, var(--color-ink) 16%, transparent);
}
input[type="range"].form-input::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    box-sizing: border-box;
    width: 16px;
    height: 16px;
    margin-top: -5px;
    border-radius: 50%;
    background: var(--color-brand-accent);
    border: 2px solid color-mix(in srgb, var(--ui-bg-color) 70%, #ffffff 30%);
    box-shadow: 0 2px 6px rgba(0,0,0,0.45);
    cursor: pointer;
    transition: transform 0.15s ease, box-shadow 0.15s ease;
}
input[type="range"].form-input::-webkit-slider-thumb:hover {
    transform: scale(1.18);
}
input[type="range"].form-input:focus { outline: none; box-shadow: none; }
input[type="range"].form-input:focus::-webkit-slider-thumb {
    box-shadow: 0 2px 6px rgba(0,0,0,0.45), 0 0 0 4px color-mix(in srgb, var(--color-brand-accent) 25%, transparent);
}

.form-textarea {
    width: 100%; padding: 6px 10px; min-height: 80px; resize: vertical;
    background: var(--color-input-bg);
    border: 1px solid var(--color-hairline);
    border-radius: 6px; color: var(--color-ink); font-size: 12.5px;
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.12);
    font-family: 'JetBrains Mono', monospace;
    outline: none; transition: border-color 0.2s, box-shadow 0.2s;
    appearance: none;
    -webkit-appearance: none;
}
.form-textarea:focus {
    border-color: var(--color-brand-accent);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.12), 0 0 0 2px color-mix(in srgb, var(--color-brand-accent) 22%, transparent);
}
.form-row { display: flex; gap: 16px; flex-wrap: wrap; }
.form-row > * { flex: 1; min-width: 200px; }
.input-group { display: flex; gap: 8px; }
.input-group .form-input { flex: 1; }

.resizer-v {
    width: 6px;
    cursor: col-resize;
    background: transparent;
    transition: background 0.15s;
    position: relative;
    flex-shrink: 0;
    align-self: stretch;
    z-index: 10;
}
.resizer-v:hover, .resizer-v.dragging {
    background: var(--color-brand-accent);
}
.resizer-h {
    height: 6px;
    cursor: row-resize;
    background: transparent;
    transition: background 0.15s;
    position: relative;
    flex-shrink: 0;
    width: 100%;
    z-index: 10;
}
.resizer-h:hover, .resizer-h.dragging {
    background: var(--color-brand-accent);
}

.tag-badge {
    display: inline-flex;
    align-items: center;
    background: var(--color-hairline-soft);
    color: var(--color-ink);
    border: 1px solid var(--color-hairline);
    border-radius: 6px;
    padding: 4px 10px;
    font-size: 13px;
    font-family: 'JetBrains Mono', monospace;
    gap: 6px;
    max-width: 100%;
    word-break: break-all;
    box-shadow: 0 1px 2px rgba(0,0,0,0.02);
}
.tag-badge button {
    background: transparent;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    padding: 2px;
    font-size: 12px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    transition: background-color 0.15s, color 0.15s;
    margin-left: 4px;
}
.tag-badge button:hover {
    background: var(--color-badge-error-bg);
    color: var(--color-error);
}

/* ── Toggles ──────────────────────────────────────────────────────── */
.toggle-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 0; border-bottom: 1px solid var(--color-hairline);
    gap: 16px;
}
.toggle-row:last-child { border-bottom: none; }
.toggle-info { display: flex; flex-direction: column; gap: 2px; }
.toggle-name { font-size: 14px; color: var(--color-ink); font-weight: 500; }
.toggle-desc { font-size: 12px; color: var(--color-muted); }

.toggle-switch {
    width: 44px; height: 24px; border-radius: 9999px;
    background: var(--color-surface-strong); cursor: pointer;
    position: relative; transition: background-color 0.2s;
    flex-shrink: 0;
}
.toggle-switch.on { background: var(--color-primary); }
.toggle-thumb {
    position: absolute; width: 18px; height: 18px;
    background: white; border-radius: 50%;
    top: 3px; left: 3px;
    transition: transform 0.2s;
    box-shadow: 0 1px 3px rgba(0,0,0,0.15);
}
.toggle-switch.on .toggle-thumb { transform: translateX(20px); }

/* ── Log Panel ────────────────────────────────────────────────────── */
.log-panel {
    min-height: 160px; display: flex; flex-direction: column;
    flex: 0 0 auto;
    /* The actual height is set inline (driven by the resizable splitter)
       so the panel cannot grow past the user-chosen size — long logs scroll
       inside .log-content instead of stretching the whole window. */
    background: var(--glass-sheen), var(--glass-frost);
    border: 1px solid var(--color-hairline);
    border-radius: 14px;
    overflow: hidden;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.22), inset 0 1px 0 rgba(255,255,255,0.06);
}
.log-panel .log-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 16px; background: var(--color-canvas);
    border-bottom: 1px solid var(--color-hairline);
    flex: 0 0 auto;
}
.log-panel .log-title {
    font-size: 12px; font-weight: 700; color: var(--color-ink);
    text-transform: uppercase; letter-spacing: 0.12em;
}
.log-panel .log-content {
    flex: 1 1 0; min-height: 0; overflow-y: auto; padding: 14px 16px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px; line-height: 1.7;
    background: var(--color-canvas);
}
.log-panel .log-content::-webkit-scrollbar { width: 10px; }
.log-panel .log-content::-webkit-scrollbar-track { background: transparent; }
.log-panel .log-content::-webkit-scrollbar-thumb {
    background: color-mix(in srgb, var(--color-ink) 40%, transparent);
    border-radius: 5px;
    border: 2px solid transparent;
    background-clip: padding-box;
}
.log-panel .log-content::-webkit-scrollbar-thumb:hover {
    background: color-mix(in srgb, var(--color-ink) 60%, transparent);
    background-clip: padding-box;
    border: 2px solid transparent;
}
.log-line { white-space: pre-wrap; word-break: break-word; margin-bottom: 6px; }
.log-out { color: var(--color-body); }
.log-err { color: var(--color-error); }
.log-info { color: var(--color-brand-accent); }
.log-empty { color: var(--color-muted); font-style: italic; padding: 10px 0; }

/* ── Command preview ──────────────────────────────────────────────── */
.cmd-preview {
    background: var(--color-surface-dark); border: 1px solid var(--color-hairline);
    border-radius: 8px; padding: 14px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px; line-height: 1.6;
    color: var(--color-on-dark-soft); word-break: break-all;
    max-height: 100px; overflow-y: auto;
}
.cmd-preview::-webkit-scrollbar { width: 6px; }
.cmd-preview::-webkit-scrollbar-thumb { background: color-mix(in srgb, var(--color-ink) 40%, transparent); border-radius: 3px; }

/* ── nav-pill-group for tabs ──────────────────────────────────────── */
.nav-pill-group {
    background: var(--color-surface-soft);
    border-radius: 9999px;
    padding: 4px;
    display: inline-flex;
    gap: 4px;
    border: 1px solid var(--color-hairline);
}
.category-tab {
    background: transparent;
    border: none;
    color: var(--color-muted);
    padding: 6px 16px;
    font-size: 13px;
    font-weight: 600;
    border-radius: 9999px;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
    outline: none;
}
.category-tab:hover {
    color: var(--color-ink);
}
.category-tab.active {
    background: var(--color-canvas);
    color: var(--color-ink);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

/* ── Accordion elements ───────────────────────────────────────────── */
.family-header {
    padding: 12px 16px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-hairline);
    border-radius: 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-weight: 600;
    color: var(--color-ink);
    user-select: none;
    transition: background-color 0.2s, border-color 0.2s;
    outline: none;
}
.family-header:hover {
    background: var(--color-surface-strong);
}

.version-header {
    padding: 10px 14px;
    background: var(--color-surface-soft);
    border: 1px solid var(--color-hairline);
    border-radius: 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-weight: 500;
    color: var(--color-ink);
    user-select: none;
    transition: background-color 0.2s, border-color 0.2s;
    outline: none;
}
.version-header:hover {
    background: var(--color-surface-strong);
}

.version-content {
    margin-left: 20px;
    overflow-x: auto;
    background: var(--color-canvas);
    padding: 8px 16px;
    border-radius: 8px;
    border: 1px solid var(--color-hairline);
}

.scan-dir-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--color-canvas);
    padding: 8px 12px;
    border-radius: 8px;
    border: 1px solid var(--color-hairline);
}

.stat-card {
    background: var(--color-canvas); border: 1px solid var(--color-hairline); padding: 12px; border-radius: 8px; text-align: center;
}
.stat-card-title {
    font-size: 11px; color: var(--color-muted); text-transform: uppercase; letter-spacing: 0.05em;
}
.stat-card-value {
    font-size: 20px; font-weight: 600; color: var(--color-ink); margin-top: 4px;
}

/* ── model-table ──────────────────────────────────────────────────── */
.model-table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
    font-size: 13px;
}
.model-table thead tr {
    border-bottom: 1px solid var(--color-hairline);
    color: var(--color-muted);
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}
.model-table th {
    padding: 10px 8px;
}
.model-table td {
    padding: 12px 8px;
}
.model-table tbody tr {
    border-bottom: 1px solid var(--color-hairline-soft);
}
.model-table tbody tr:last-child {
    border-bottom: none;
}

/* ── Misc ─────────────────────────────────────────────────────────── */
.error-toast {
    background: var(--color-badge-error-bg); border: 1px solid var(--color-badge-error-border);
    color: var(--color-error); border-radius: 8px; padding: 12px 16px;
    font-size: 14px; display: flex; align-items: center; gap: 8px;
    font-weight: 500;
}
.lora-row:last-child { border-bottom: none; }
.table-row-hover:hover { background: var(--color-surface-soft); }

/* ── Title / Top Bar Custom Styling ─────────────────────────────── */
.top-bar {
    user-select: none;
}
.top-bar.draggable {
    cursor: move;
}
.top-menu-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: 8px;
    flex-shrink: 0;
}
.menu-item-container {
    position: relative;
}
.top-menu-btn {
    background: transparent;
    border: none;
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--color-body);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
    outline: none;
}
.top-menu-btn:hover, .top-menu-btn.active {
    background: var(--color-overlay-white-2);
    color: var(--color-ink);
}
.menu-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    background: color-mix(in srgb, var(--ui-bg-color) 97%, var(--ui-text-color) 3%);
    border: 1px solid var(--color-hairline);
    border-radius: 8px;
    min-width: 180px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
    z-index: 1000;
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.dropdown-item {
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    color: var(--color-body);
    font-size: 12.5px;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s;
    width: 100%;
    outline: none;
    display: flex;
    align-items: center;
    gap: 8px;
}
.dropdown-item:hover {
    background: var(--color-overlay-white-2);
    color: var(--color-ink);
}
.dropdown-item.item-danger:hover {
    background: var(--color-badge-error-bg);
    color: var(--color-badge-error-text);
}
.dropdown-divider {
    height: 1px;
    background: var(--color-overlay-white-2);
    margin: 4px 0;
}
.window-controls {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: 8px;
    flex-shrink: 0;
}
.window-control-btn {
    background: transparent;
    border: none;
    border-radius: 4px;
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: var(--color-body);
    font-size: 11px;
    transition: all 0.15s;
    outline: none;
}
.window-control-btn:hover {
    background: var(--color-overlay-white-2);
    color: var(--color-ink);
}
.window-control-btn.control-close:hover {
    background: var(--color-error);
    color: var(--color-on-primary);
}
"#;

// ─────────────────────────────────────────────────────────────────────────────
// Entry Point
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    // webkit2gtk's GPU compositor mishandles dirty-region clearing on transparent
    // windows: when a panel changes, stale pixels from the previous frame are not
    // cleared and "ghost" through (old tab visible behind the new one, glitches that
    // only clear on hover). Forcing WebKit's CPU paint path repaints the full surface
    // cleanly and fixes the ghosting. Window-level alpha transparency is unaffected —
    // it is applied at the window/RGBA layer, independent of the WebKit compositor.
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_background_color((0, 0, 0, 0))
                .with_disable_dma_buf_on_wayland(true)
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Llama-Manager")
                        .with_transparent(true)
                        .with_decorations(false)
                        .with_resizable(true)
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1400.0_f64, 900.0_f64)),
                ),
        )
        .launch(App);
}

// ─────────────────────────────────────────────────────────────────────────────
// App
// ─────────────────────────────────────────────────────────────────────────────

pub fn get_default_config_path() -> std::path::PathBuf {
    std::path::PathBuf::from("/home/notroot/Work/llama-manager")
}

fn load_default_config() -> ServerConfig {
    let dir = get_default_config_path();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("config.json");
    if path.exists() {
        if let Ok(mut cfg) = ServerConfig::load_from_file(&path.to_string_lossy()) {
            migrate_preset_to_ui_fields(&mut cfg);
            return cfg;
        }
    }
    let default_cfg = ServerConfig::default();
    let _ = default_cfg.save_to_file(&path.to_string_lossy());
    default_cfg
}

#[derive(Clone, Copy, PartialEq)]
enum DragType {
    None,
    Sidebar,
    LogPanel,
}

struct ThemePreset {
    name: &'static str,
    label: &'static str,
    background_color: &'static str,
    text_color: &'static str,
    accent_color: &'static str,
    card_bg: &'static str,
    sidebar_bg: &'static str,
    border_color: &'static str,
    font_family: &'static str,
    transparency: f32,
    blur: bool,
    blur_intensity: u32,
}

const THEME_PRESETS: &[ThemePreset] = &[
    ThemePreset {
        name: "midnight",
        label: "Midnight Indigo",
        background_color: "#0f172a",
        text_color: "#f1f5f9",
        accent_color: "#6366f1",
        card_bg: "linear-gradient(135deg, rgba(99,102,241,0.06) 0%, rgba(99,102,241,0) 60%)",
        sidebar_bg: "linear-gradient(180deg, rgba(99,102,241,0.08) 0%, rgba(15,23,42,0) 100%)",
        border_color: "rgba(148, 163, 184, 0.18)",
        font_family: "Inter",
        transparency: 0.14,
        blur: true,
        blur_intensity: 34,
    },
    ThemePreset {
        name: "tokyo_night",
        label: "Tokyo Night",
        background_color: "#1a1b26",
        text_color: "#c0caf5",
        accent_color: "#7aa2f7",
        card_bg: "linear-gradient(135deg, rgba(122,162,247,0.06) 0%, rgba(122,162,247,0) 60%)",
        sidebar_bg: "linear-gradient(180deg, rgba(122,162,247,0.08) 0%, rgba(26,27,38,0) 100%)",
        border_color: "rgba(122, 162, 247, 0.2)",
        font_family: "JetBrains Mono",
        transparency: 0.18,
        blur: true,
        blur_intensity: 40,
    },
    ThemePreset {
        name: "nord",
        label: "Nord Frost",
        background_color: "#2e3440",
        text_color: "#eceff4",
        accent_color: "#88c0d0",
        card_bg: "linear-gradient(135deg, rgba(136,192,208,0.07) 0%, rgba(136,192,208,0) 60%)",
        sidebar_bg: "linear-gradient(180deg, rgba(136,192,208,0.08) 0%, rgba(46,52,64,0) 100%)",
        border_color: "rgba(136, 192, 208, 0.2)",
        font_family: "Inter",
        transparency: 0.16,
        blur: true,
        blur_intensity: 38,
    },
    ThemePreset {
        name: "emerald",
        label: "Emerald Forest",
        background_color: "#052e23",
        text_color: "#d1fae5",
        accent_color: "#34d399",
        card_bg: "linear-gradient(135deg, rgba(52,211,153,0.07) 0%, rgba(52,211,153,0) 60%)",
        sidebar_bg: "linear-gradient(180deg, rgba(52,211,153,0.09) 0%, rgba(5,46,35,0) 100%)",
        border_color: "rgba(52, 211, 153, 0.2)",
        font_family: "Inter",
        transparency: 0.16,
        blur: true,
        blur_intensity: 40,
    },
    ThemePreset {
        name: "dracula",
        label: "Dracula",
        background_color: "#282a36",
        text_color: "#f8f8f2",
        accent_color: "#bd93f9",
        card_bg: "linear-gradient(135deg, rgba(189,147,249,0.07) 0%, rgba(189,147,249,0) 60%)",
        sidebar_bg: "linear-gradient(180deg, rgba(189,147,249,0.08) 0%, rgba(40,42,54,0) 100%)",
        border_color: "rgba(189, 147, 249, 0.22)",
        font_family: "JetBrains Mono",
        transparency: 0.16,
        blur: true,
        blur_intensity: 38,
    },
    ThemePreset {
        name: "cyberpunk",
        label: "Cyberpunk Neon",
        background_color: "#0a0118",
        text_color: "#f5d0fe",
        accent_color: "#ff2d95",
        card_bg: "linear-gradient(135deg, rgba(255,45,149,0.06) 0%, rgba(0,229,255,0.04) 100%)",
        sidebar_bg: "linear-gradient(180deg, rgba(255,45,149,0.1) 0%, rgba(10,1,24,0) 100%)",
        border_color: "rgba(255, 45, 149, 0.3)",
        font_family: "JetBrains Mono",
        transparency: 0.2,
        blur: true,
        blur_intensity: 44,
    },
    ThemePreset {
        name: "glass_light",
        label: "Glass Light",
        background_color: "#eef2f7",
        text_color: "#0f172a",
        accent_color: "#2563eb",
        card_bg: "linear-gradient(135deg, rgba(255,255,255,0.5) 0%, rgba(255,255,255,0.15) 100%)",
        sidebar_bg: "linear-gradient(180deg, rgba(255,255,255,0.45) 0%, rgba(238,242,247,0) 100%)",
        border_color: "rgba(15, 23, 42, 0.12)",
        font_family: "Inter",
        transparency: 0.12,
        blur: true,
        blur_intensity: 48,
    },
];

fn apply_preset_to_config(cfg: &mut ServerConfig, preset: &ThemePreset) {
    cfg.ui_background_color = preset.background_color.to_string();
    cfg.ui_text_color = preset.text_color.to_string();
    cfg.ui_accent_color = preset.accent_color.to_string();
    cfg.ui_card_bg = preset.card_bg.to_string();
    cfg.ui_sidebar_bg = preset.sidebar_bg.to_string();
    cfg.ui_border_color = preset.border_color.to_string();
    cfg.ui_font_family = preset.font_family.to_string();
    cfg.ui_transparency = preset.transparency;
    cfg.ui_blur = preset.blur;
    cfg.ui_blur_intensity = preset.blur_intensity;
}

fn apply_custom_to_config(cfg: &mut ServerConfig, custom: &crate::config::CustomTheme) {
    cfg.ui_background_color = custom.background_color.clone();
    cfg.ui_text_color = custom.text_color.clone();
    cfg.ui_accent_color = custom.accent_color.clone();
    cfg.ui_card_bg = custom.card_bg.clone();
    cfg.ui_sidebar_bg = custom.sidebar_bg.clone();
    cfg.ui_border_color = custom.border_color.clone();
    cfg.ui_font_family = custom.font_family.clone();
    cfg.ui_transparency = custom.transparency;
    cfg.ui_blur = custom.blur;
    cfg.ui_blur_intensity = custom.blur_intensity;
}

fn migrate_preset_to_ui_fields(cfg: &mut ServerConfig) {
    let name = cfg.theme_name.clone();
    if name == "default" || name == "custom" {
        return;
    }
    if let Some(preset) = THEME_PRESETS.iter().find(|p| p.name == name) {
        apply_preset_to_config(cfg, preset);
    } else {
        let custom = cfg.custom_themes.iter().find(|t| t.name == name).cloned();
        if let Some(custom) = custom {
            apply_custom_to_config(cfg, &custom);
        }
    }
}

#[component]
fn App() -> Element {
    let window = dioxus::desktop::use_window();
    // ── State ───────────────────────────────────────────────────────
    let mut config = use_signal(load_default_config);
    let mut show_config_modal = use_signal(|| false);
    let mut immersive_mode = use_signal(|| false);

    // Resizer States
    let mut sidebar_width = use_signal(|| 220.0);
    let mut log_panel_height = use_signal(|| 180.0);
    let mut drag_state = use_signal(|| DragType::None);
    let mut drag_start_y = use_signal(|| 0.0);
    let mut drag_start_height = use_signal(|| 180.0);

    // Watch config.json on disk for changes and load it in real time
    let _config_watcher = use_resource(move || {
        let mut config = config;
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                let dir = get_default_config_path();
                let path = dir.join("config.json");
                if path.exists() {
                    if let Ok(disk_cfg) = ServerConfig::load_from_file(&path.to_string_lossy()) {
                        let current_cfg = config.read().clone();
                        let current_str = serde_json::to_string(&current_cfg).unwrap_or_default();
                        let disk_str = serde_json::to_string(&disk_cfg).unwrap_or_default();
                        if current_str != disk_str {
                            config.set(disk_cfg);
                        }
                    }
                }
            }
        }
    });

    // Background Calendar Prompt Trigger Watcher
    let _calendar_watcher = use_resource(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            let to_fire = calendar::CalendarState::checkout_firing_events(&now);

            if !to_fire.is_empty() {
                for ev in to_fire {
                    let host = "127.0.0.1".to_string();
                    let port = load_default_config().port;
                    let prompt = ev.prompt.clone();
                    let title = ev.title.clone();
                    let ev_id = ev.id.clone();

                    tokio::spawn(async move {
                        log_info!(
                            "[CALENDAR TRIGGER] Event '{}' hit! Firing prompt: '{}'",
                            title,
                            prompt
                        );

                        let mut memory_manager =
                            agent::MemoryManager::new(get_default_config_path());
                        let mcp_registry = agent::McpRegistry::load(get_default_config_path());
                        let target_model =
                            library::get_target_model_with_host(&host, port, "").await;

                        let result = match agent::run_agent_task(
                            &host,
                            port,
                            &target_model,
                            &prompt,
                            &mcp_registry,
                            &mut memory_manager,
                            None,
                        )
                        .await
                        {
                            Ok(resp) => {
                                log_info!(
                                    "[CALENDAR TRIGGER] Prompt ran successfully. Output: {}",
                                    resp.text
                                );
                                format!("Success:\n{}", resp.text)
                            }
                            Err(e) => {
                                log_error!("[CALENDAR TRIGGER] Prompt failed: {}", e);
                                format!("Error: {}", e)
                            }
                        };

                        let _ = calendar::CalendarState::update_event(&ev_id, |inner_ev| {
                            inner_ev.status = calendar::EventStatus::Completed;
                            inner_ev.result = Some(result);
                        });
                    });
                }
            }
        }
    });

    // Synchronize application log level with config.json
    use_effect(move || {
        let level_str = config.read().app_log_level.clone();
        logger::set_log_level(logger::LogLevel::from_str(&level_str));
    });

    // Auto-save config to ~/.local/share/llama-manager/config.json whenever it changes
    use_effect(move || {
        let current_config = config.read().clone();
        let dir = get_default_config_path();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.json");
        match current_config.save_to_file(&path.to_string_lossy()) {
            Ok(_) => {
                log_debug!("Auto-saved config to: {:?}", path);
            }
            Err(e) => {
                log_error!("Failed to auto-save config: {}", e);
            }
        }
    });
    let mut active_tab = use_signal(|| Tab::Chat);
    let mut active_menu = use_signal(|| None::<&'static str>);
    let mut sidebar_collapsed = use_signal(|| false);
    let mut favorites_expanded = use_signal(|| true);
    let mut general_expanded = use_signal(|| true);
    let mut models_expanded = use_signal(|| true);
    let mut server_expanded = use_signal(|| true);
    let mut productivity_expanded = use_signal(|| true);

    // Auto-expand sidebar sections when active_tab changes
    use_effect(move || {
        let act = active_tab();
        let mut general_sig = general_expanded;
        let mut models_sig = models_expanded;
        let mut server_sig = server_expanded;
        let mut productivity_sig = productivity_expanded;
        match act {
            Tab::Chat | Tab::Planner | Tab::Monitor | Tab::Mcp | Tab::Agents | Tab::AppSettings => {
                general_sig.set(true);
            }
            Tab::Model | Tab::Library | Tab::Download | Tab::Instances => {
                models_sig.set(true);
            }
            Tab::Server
            | Tab::Context
            | Tab::Gpu
            | Tab::Performance
            | Tab::Sampling
            | Tab::Advanced
            | Tab::Api => {
                server_sig.set(true);
            }
            Tab::Calendar | Tab::Todos | Tab::QuickNotes | Tab::Compare | Tab::DeepResearch => {
                productivity_sig.set(true);
            }
        }
    });

    let routed_instance = use_signal(|| None::<(String, u16)>);
    let mut logs = use_signal(Vec::<String>::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut available_models = use_signal(Vec::<String>::new);
    let mut selected_model = use_signal(String::new);
    let mut loading_models = use_signal(|| false);

    let mut search_query = use_signal(|| String::new());
    let mut search_results = use_signal(|| Vec::<ConfigSearchEntry>::new());
    let mut search_target = use_signal(|| String::new());
    let search_entries = config_search_entries();

    let mut scanned_models = use_signal(|| {
        let index_path = get_default_config_path().join("model_index.json");
        library::load_index(&index_path.to_string_lossy())
    });
    let mut index_status = use_signal(|| "Idle".to_string());
    let mut scan_trigger = use_signal(|| 0u64);

    // Server process + shared log buffer
    let mut server_child: Signal<Option<Arc<Mutex<Child>>>> = use_signal(|| None);
    let log_buffer: Signal<Arc<Mutex<Vec<String>>>> =
        use_signal(|| Arc::new(Mutex::new(Vec::new())));

    let is_running = server_child.read().is_some();

    let on_search_input = move |e: Event<FormData>| {
        let query = e.value();
        let normalized = query.trim().to_lowercase();
        search_query.set(query.clone());

        if normalized.is_empty() {
            search_results.set(Vec::new());
            return;
        }

        let mut filtered: Vec<ConfigSearchEntry> = search_entries
            .iter()
            .filter(|entry| {
                let label = entry.label.to_lowercase();
                let section = entry.section.to_lowercase();
                label.contains(&normalized) || section.contains(&normalized)
            })
            .cloned()
            .collect();

        filtered.sort_by_key(|entry| {
            entry
                .label
                .to_lowercase()
                .find(&normalized)
                .unwrap_or(usize::MAX)
        });

        if filtered.len() > 12 {
            filtered.truncate(12);
        }

        search_results.set(filtered);
    };

    // ── Indexer & LLM Metadata Enricher ──────────────────────────────
    let _indexer = use_resource(move || {
        async move {
            let mut last_scan_time =
                std::time::Instant::now() - std::time::Duration::from_secs(600);

            loop {
                let trigger_val = scan_trigger();
                let now = std::time::Instant::now();
                let time_since_last = now.duration_since(last_scan_time);

                if time_since_last.as_secs() >= 300 || trigger_val > 0 {
                    index_status.set("Scanning directories\u{2026}".to_string());

                    let dirs = {
                        let cfg = config.read();
                        if cfg.model_scan_dirs.is_empty() {
                            let mut auto_dirs = Vec::new();
                            let base = "/mnt/modelsext/models";
                            let subdirs = [
                                "3D",
                                "all",
                                "asr",
                                "audio",
                                "coder",
                                "dflash",
                                "diffusion",
                                "embedding",
                                "embodied",
                                "image",
                                "infographic",
                                "mtp",
                                "multilingual",
                                "multimodal",
                                "ocr",
                                "osworld",
                                "privacy",
                                "research",
                                "reshoot",
                                "text",
                                "timeseries",
                                "tts",
                                "ui",
                                "uncensored",
                                "video",
                                "vision",
                                "web",
                                "world",
                            ];
                            for sub in &subdirs {
                                let p = format!("{}/{}", base, sub);
                                if std::path::Path::new(&p).exists() {
                                    auto_dirs.push(p);
                                }
                            }
                            auto_dirs
                        } else {
                            cfg.model_scan_dirs.clone()
                        }
                    };

                    if !dirs.is_empty() {
                        let existing = scanned_models.read().clone();
                        let updated = tokio::task::spawn_blocking(move || {
                            let scanned = library::scan_directories(&dirs);
                            library::merge_indexes(scanned, existing)
                        })
                        .await
                        .unwrap_or_default();

                        let index_path = get_default_config_path().join("model_index.json");
                        let _ = library::save_index(&index_path.to_string_lossy(), &updated);
                        scanned_models.set(updated);
                    }

                    if trigger_val > 0 {
                        *scan_trigger.write() = 0;
                    }

                    last_scan_time = std::time::Instant::now();
                    index_status.set("Scan completed. Idle.".to_string());
                }

                // Enrichment check
                let mut pending_indices = Vec::new();
                for (idx, m) in scanned_models.read().iter().enumerate() {
                    if m.status == "pending_enrichment" {
                        pending_indices.push(idx);
                    }
                }

                if !pending_indices.is_empty() {
                    let port = config.read().port;
                    let client = reqwest::Client::new();
                    let ping_url = format!("http://127.0.0.1:{}/v1/models", port);
                    let server_active = client
                        .get(&ping_url)
                        .timeout(std::time::Duration::from_millis(500))
                        .send()
                        .await
                        .is_ok();

                    if server_active {
                        if let Some(&first_idx) = pending_indices.first() {
                            // Set status to enriching first
                            {
                                let mut current = scanned_models.read().clone();
                                current[first_idx].status = "enriching".to_string();
                                let index_path = get_default_config_path().join("model_index.json");
                                let _ =
                                    library::save_index(&index_path.to_string_lossy(), &current);
                                scanned_models.set(current);
                            }

                            let mut model_to_enrich = scanned_models.read()[first_idx].clone();
                            index_status.set(format!(
                                "Enriching metadata for {}\u{2026}",
                                model_to_enrich.filename
                            ));

                            let searx_url = config.read().searxng_url.clone();
                            let result =
                                library::enrich_model(&mut model_to_enrich, &searx_url, port).await;

                            let mut current = scanned_models.read().clone();
                            match result {
                                Ok(_) => {
                                    index_status
                                        .set(format!("Enriched: {}", model_to_enrich.filename));
                                    current[first_idx] = model_to_enrich;
                                }
                                Err(e) => {
                                    index_status.set(format!(
                                        "Enrichment failed for {}: {}",
                                        model_to_enrich.filename, e
                                    ));
                                    model_to_enrich.status = "failed".to_string();
                                    current[first_idx] = model_to_enrich;
                                }
                            }
                            let index_path = get_default_config_path().join("model_index.json");
                            let _ = library::save_index(&index_path.to_string_lossy(), &current);
                            scanned_models.set(current);
                        }
                    } else {
                        index_status.set("Model server offline. Enrichment paused.".to_string());
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    });

    // ── Log poller ──────────────────────────────────────────────────
    // Drain the shared buffer into the displayed logs every 200ms.
    let _poller = use_resource(move || {
        let buf = (log_buffer)();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                let new: Vec<String> = {
                    let mut lock = buf.lock().unwrap();
                    lock.drain(..).collect()
                };
                if !new.is_empty() {
                    logs.write().extend(new);
                }
            }
        }
    });

    // ── Models fetcher ──────────────────────────────────────────────
    // Parse available models from server logs
    let _models_fetcher = use_resource(move || {
        let log_data = logs.read().clone();
        async move {
            if !is_running {
                available_models.set(Vec::new());
                selected_model.set(String::new());
                return;
            }

            // Parse models from log output
            // Look for lines like: "srv   load_models:     model-name"
            let mut models = Vec::new();
            let mut in_models_section = false;

            for line in &log_data {
                // Check if we're entering the models section
                if line.contains("Available models") {
                    in_models_section = true;
                    continue;
                }

                // Stop at the router server line
                if in_models_section && line.contains("starting router server") {
                    break;
                }

                // Extract model names from the models list
                if in_models_section
                    && line.contains("srv   load_models:")
                    && !line.contains("Available models")
                {
                    if let Some(model_name) =
                        line.split("srv   load_models:").nth(1).map(|s| s.trim())
                    {
                        if !model_name.is_empty() && !model_name.contains("Loaded") {
                            models.push(model_name.to_string());
                        }
                    }
                }
            }

            available_models.set(models);
            loading_models.set(false);
        }
    });

    // ── Start / Stop ────────────────────────────────────────────────
    let toggle_server = move |_| {
        if is_running {
            // Stop
            if let Some(arc_child) = server_child.write().take() {
                if let Ok(mut child) = arc_child.lock() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
            logs.write().push("[MANAGER] Server stopped.".into());
        } else {
            // Validate
            let cfg = config.read();
            if cfg.model_path.is_empty()
                && cfg.hf_repo.is_empty()
                && cfg.model_url.is_empty()
                && cfg.model_dir.is_empty()
            {
                error_msg.set(Some(
                    "Please provide a model path, HuggingFace repo, model directory, or model URL."
                        .into(),
                ));
                return;
            }
            error_msg.set(None);

            let exe = cfg.exe_path.clone();
            let args = cfg.to_args();

            logs.write().clear();
            logs.write()
                .push(format!("[MANAGER] Starting: {} {}", exe, args.join(" ")));

            let mut cmd = Command::new(&exe);
            cmd.args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }

            match cmd.spawn() {
                Ok(mut child) => {
                    // Capture stdout
                    if let Some(out) = child.stdout.take() {
                        let buf = (*log_buffer.read()).clone();
                        std::thread::spawn(move || {
                            let reader = BufReader::new(out);
                            for line in reader.lines().map_while(Result::ok) {
                                buf.lock().unwrap().push(line);
                            }
                        });
                    }
                    // Capture stderr
                    if let Some(err) = child.stderr.take() {
                        let buf = (*log_buffer.read()).clone();
                        std::thread::spawn(move || {
                            let reader = BufReader::new(err);
                            for line in reader.lines().map_while(Result::ok) {
                                buf.lock().unwrap().push(line);
                            }
                        });
                    }
                    server_child.set(Some(Arc::new(Mutex::new(child))));
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to start: {e}")));
                    logs.write().push(format!("[ERROR] {e}"));
                }
            }
        }
    };

    // ── Save / Load config ──────────────────────────────────────────
    let save_config = move |_| {
        spawn(async move {
            let config_data = config.read().clone();
            if let Ok(Some(path)) = tokio::task::spawn_blocking(move || {
                rfd::FileDialog::new()
                    .set_file_name("llama-config.json")
                    .add_filter("JSON", &["json"])
                    .save_file()
            })
            .await
            {
                if let Err(e) = config_data.save_to_file(&path.to_string_lossy()) {
                    error_msg.set(Some(format!("Save failed: {e}")));
                }
            }
        });
    };

    let load_config = move |_| {
        spawn(async move {
            if let Ok(Some(path)) = tokio::task::spawn_blocking(move || {
                rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file()
            })
            .await
            {
                match ServerConfig::load_from_file(&path.to_string_lossy()) {
                    Ok(loaded_cfg) => config.set(loaded_cfg),
                    Err(e) => error_msg.set(Some(format!("Load failed: {e}"))),
                }
            }
        });
    };

    let on_sidebar_resizer_mousedown = move |_| {
        drag_state.set(DragType::Sidebar);
    };

    let on_log_resizer_mousedown = move |evt: MouseEvent| {
        drag_state.set(DragType::LogPanel);
        drag_start_y.set(evt.client_coordinates().y);
        drag_start_height.set(*log_panel_height.read());
    };

    let search_matches = search_results.read().clone();

    // Resolve theme-specific variables — always use ui_* working copy
    let (
        bg_color,
        text_color,
        accent_color,
        card_bg,
        sidebar_bg,
        border_color,
        font_family,
        transparency,
        blur,
    ) = {
        let cfg = config.read();
        (
            cfg.ui_background_color.clone(),
            cfg.ui_text_color.clone(),
            cfg.ui_accent_color.clone(),
            cfg.ui_card_bg.clone(),
            cfg.ui_sidebar_bg.clone(),
            cfg.ui_border_color.clone(),
            cfg.ui_font_family.clone(),
            cfg.ui_transparency,
            cfg.ui_blur,
        )
    };

    let render_sidebar_item = move |tab: Tab, is_sub: bool| {
        let is_fav = config
            .read()
            .sidebar_favorites
            .contains(&tab.as_str().to_string());
        let active = active_tab() == tab;
        let class_name = if active {
            if is_sub {
                "sidebar-item sidebar-sub-item active"
            } else {
                "sidebar-item active"
            }
        } else {
            if is_sub {
                "sidebar-item sidebar-sub-item"
            } else {
                "sidebar-item"
            }
        };
        let fav_class = if is_fav {
            "sidebar-fav-btn sidebar-fav-btn-active"
        } else {
            "sidebar-fav-btn"
        };

        rsx! {
            button {
                key: "{tab.as_str()}",
                class: "{class_name}",
                style: "display: flex; justify-content: space-between; align-items: center; width: 100%;",
                onclick: move |_| active_tab.set(tab),
                div { style: "display: flex; align-items: center; gap: 10px;",
                    span { class: "sidebar-icon", aria_hidden: "true", {tab.icon()} }
                    span { {tab.label()} }
                }
                button {
                    class: "{fav_class}",
                    onclick: move |e| {
                        e.stop_propagation();
                        let mut current = config.read().sidebar_favorites.clone();
                        let t_str = tab.as_str().to_string();
                        if let Some(pos) = current.iter().position(|x| x == &t_str) {
                            current.remove(pos);
                        } else {
                            current.push(t_str);
                        }
                        config.write().sidebar_favorites = current;
                    },
                    if is_fav { "★" } else { "☆" }
                }
            }
        }
    };

    // ── Render ──────────────────────────────────────────────────────
    rsx! {
        style { {CSS} }
        style {
            {format!(
                r#"
                :root {{
                    --ui-bg-color: {};
                    --ui-text-color: {};
                    --ui-accent-color: {};
                    --ui-card-bg: {};
                    --ui-sidebar-bg: {};
                    --ui-border-color: {};
                    --ui-font-family: '{}';
                    --app-opacity: {};
                    --panel-frost: {};
                }}
                "#,
                bg_color,
                text_color,
                accent_color,
                card_bg,
                sidebar_bg,
                border_color,
                font_family,
                // Window Transparency → uniform opacity of the whole window.
                format!("{}%", ((1.0 - transparency) * 100.0).round() as i32),
                {
                    // Glass Frost → how much panels are lifted above the transparent base.
                    let intensity = config.read().ui_blur_intensity;
                    if blur {
                        format!("{}%", (4.0 + (intensity as f32 / 64.0) * 9.0).round() as i32)
                    } else {
                        "2%".to_string()
                    }
                },
            )}
        }
        if show_config_modal() {
            div {
                style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(5, 5, 8, 0.78); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                onclick: move |_| show_config_modal.set(false),
                div {
                    style: "background: var(--color-canvas); border: 1px solid var(--color-hairline); border-radius: 12px; width: 80%; max-width: 650px; max-height: 80vh; display: flex; flex-direction: column; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.15);",
                    onclick: move |e| e.stop_propagation(),
                    div {
                        style: "padding: 16px 20px; border-bottom: 1px solid var(--color-hairline); display: flex; justify-content: space-between; align-items: center;",
                        span { style: "font-size: 14px; font-weight: 600; color: var(--color-ink); display: flex; align-items: center; gap: 8px;",
                            span { style: "font-size: 16px;", "\u{1F4DD}" }
                            span { "Current Configuration JSON" }
                        }
                        button {
                            style: "background: none; border: none; color: var(--color-muted); font-size: 20px; cursor: pointer; transition: color 0.2s;",
                            aria_label: "Close configuration modal",
                            onclick: move |_| show_config_modal.set(false),
                            "\u{2715}"
                        }
                    }
                    div {
                        style: "padding: 20px; overflow-y: auto; flex-grow: 1; font-family: monospace; font-size: 12px; line-height: 1.5; color: var(--color-on-dark-soft); background: var(--color-surface-dark); border-radius: 0 0 12px 12px; border-top: 1px solid var(--color-hairline);",
                        pre {
                            style: "margin: 0; white-space: pre-wrap; word-break: break-all; user-select: text;",
                            {
                                match serde_json::to_string_pretty(&config.read().clone()) {
                                    Ok(json) => json,
                                    Err(e) => format!("Error serializing config: {}", e)
                                }
                            }
                        }
                    }
                }
            }
        }
        div {
            class: "app",
            onmousemove: move |evt: MouseEvent| {
                match drag_state() {
                    DragType::Sidebar => {
                        let new_w = evt.client_coordinates().x;
                        if new_w >= 150.0 && new_w <= 450.0 {
                            sidebar_width.set(new_w);
                        }
                    }
                    DragType::LogPanel => {
                        let delta_y = evt.client_coordinates().y - *drag_start_y.read();
                        let new_h = *drag_start_height.read() - delta_y;
                        if new_h >= 80.0 && new_h <= 500.0 {
                            log_panel_height.set(new_h);
                        }
                    }
                    DragType::None => {}
                }
            },
            onmouseup: move |_| {
                drag_state.set(DragType::None);
            },
            onmouseleave: move |_| {
                drag_state.set(DragType::None);
            },

            // ── Top bar ─────────────────────────────────────────────
            div {
                class: "top-bar draggable",
                style: if immersive_mode() { "display: none;" } else { "" },
                onmousedown: {
                    let w = window.clone();
                    move |_| {
                        let _ = w.drag_window();
                    }
                },
                div {
                    class: "top-title",
                    onmousedown: move |e| e.stop_propagation(),
                    button {
                        class: "btn-sidebar-toggle",
                        title: if sidebar_collapsed() { "Expand Sidebar" } else { "Collapse Sidebar" },
                        onclick: move |_| sidebar_collapsed.set(!sidebar_collapsed()),
                        if sidebar_collapsed() { "☰" } else { "◂" }
                    }
                    span { aria_hidden: "true", "\u{1F999}" }
                    "Llama-Manager"
                }

                // Window, Edit, Help menus
                div {
                    class: "top-menu-bar",
                    onmousedown: move |e| e.stop_propagation(),
                    div { class: "menu-item-container",
                        button {
                            class: if active_menu() == Some("window") { "top-menu-btn active" } else { "top-menu-btn" },
                            onclick: move |_| {
                                if active_menu() == Some("window") {
                                    active_menu.set(None);
                                } else {
                                    active_menu.set(Some("window"));
                                }
                            },
                            "Window"
                        }
                        if active_menu() == Some("window") {
                            div { class: "menu-dropdown",
                                button {
                                    class: "dropdown-item",
                                    onclick: {
                                        let w = window.clone();
                                        move |_| {
                                            active_menu.set(None);
                                            w.set_minimized(true);
                                        }
                                    },
                                    "\u{1F5D5}  Minimize"
                                }
                                button {
                                    class: "dropdown-item",
                                    onclick: {
                                        let w = window.clone();
                                        move |_| {
                                            active_menu.set(None);
                                            let max = w.is_maximized();
                                            w.set_maximized(!max);
                                        }
                                    },
                                    "\u{1F5D6}  Maximize / Restore"
                                }
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        active_menu.set(None);
                                        let curr = immersive_mode();
                                        immersive_mode.set(!curr);
                                    },
                                    if immersive_mode() { "\u{26F6}  Exit Immersive Mode" } else { "\u{26F6}  Immersive Mode" }
                                }
                                div { class: "dropdown-divider" }
                                button {
                                    class: "dropdown-item item-danger",
                                    onclick: {
                                        let w = window.clone();
                                        move |_| {
                                            active_menu.set(None);
                                            w.close();
                                        }
                                    },
                                    "\u{2715}  Close Application"
                                }
                            }
                        }
                    }
                    div { class: "menu-item-container",
                        button {
                            class: if active_menu() == Some("edit") { "top-menu-btn active" } else { "top-menu-btn" },
                            onclick: move |_| {
                                if active_menu() == Some("edit") {
                                    active_menu.set(None);
                                } else {
                                    active_menu.set(Some("edit"));
                                }
                            },
                            "Edit"
                        }
                        if active_menu() == Some("edit") {
                            div { class: "menu-dropdown",
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        active_menu.set(None);
                                        let _ = dioxus::document::eval("document.execCommand('undo')");
                                    },
                                    "\u{21A9}  Undo"
                                }
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        active_menu.set(None);
                                        let _ = dioxus::document::eval("document.execCommand('redo')");
                                    },
                                    "\u{21AA}  Redo"
                                }
                                div { class: "dropdown-divider" }
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        active_menu.set(None);
                                        let _ = dioxus::document::eval("document.execCommand('cut')");
                                    },
                                    "\u{2702}  Cut"
                                }
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        active_menu.set(None);
                                        let _ = dioxus::document::eval("document.execCommand('copy')");
                                    },
                                    "\u{1F4CB}  Copy"
                                }
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        active_menu.set(None);
                                        let _ = dioxus::document::eval("document.execCommand('paste')");
                                    },
                                    "\u{1F4CB}  Paste"
                                }
                            }
                        }
                    }
                    div { class: "menu-item-container",
                        button {
                            class: if active_menu() == Some("help") { "top-menu-btn active" } else { "top-menu-btn" },
                            onclick: move |_| {
                                if active_menu() == Some("help") {
                                    active_menu.set(None);
                                } else {
                                    active_menu.set(Some("help"));
                                }
                            },
                            "Help"
                        }
                        if active_menu() == Some("help") {
                            div { class: "menu-dropdown",
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        active_menu.set(None);
                                        let _ = dioxus::document::eval("window.open('https://github.com/nkitan/llama-manager', '_blank')");
                                    },
                                    "\u{1F5C3}  Documentation"
                                }
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        active_menu.set(None);
                                        let js_focus = "setTimeout(() => { const el = document.querySelector('.search-input'); if (el) el.focus(); }, 100);";
                                        let _ = dioxus::document::eval(js_focus);
                                    },
                                    "\u{1F50D}  Search Settings"
                                }
                                div { class: "dropdown-divider" }
                                button {
                                    class: "dropdown-item",
                                    onclick: move |_| {
                                        active_menu.set(None);
                                        let _ = dioxus::document::eval("window.open('https://github.com/nkitan/llama-manager/issues', '_blank')");
                                    },
                                    "\u{1F41E}  Report an Issue"
                                }
                            }
                        }
                    }
                }

                div {
                    class: "search-group",
                    onmousedown: move |e| e.stop_propagation(),
                    input {
                        class: "search-input",
                        placeholder: "Search config entries…",
                        value: search_query(),
                        oninput: on_search_input,
                    }
                    if !search_matches.is_empty() {
                        div { class: "search-results",
                            div { class: "search-results-header", "Search results" }
                            div { class: "search-results-list",
                                for result in search_matches.into_iter() {
                                    button {
                                        class: "search-result-item",
                                        r#type: "button",
                                        onclick: move |e: MouseEvent| {
                                            e.stop_propagation();
                                            e.prevent_default();

                                            let tgt = result.target_id.clone();
                                            active_tab.set(result.tab);
                                            // clear UI first so autofocus will trigger when we re-set
                                            search_results.set(Vec::new());
                                            search_query.set(String::new());
                                            search_target.set(String::new());

                                            let mut st = search_target.clone();
                                            spawn(async move {
                                                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                                                st.set(tgt.clone());
                                                let js_scroll = format!(
                                                    r#"
                                                    setTimeout(() => {{
                                                        const el = document.getElementById("{}");
                                                        if (el) {{
                                                            el.scrollIntoView({{ behavior: "smooth", block: "center" }});
                                                            el.style.outline = "2px solid var(--color-brand-accent)";
                                                            setTimeout(() => el.style.outline = "", 1500);
                                                        }}
                                                    }}, 100);
                                                    "#,
                                                    tgt
                                                );
                                                let _ = dioxus::document::eval(&js_scroll);
                                            });
                                        },
                                        div { class: "search-result-label",
                                            div { class: "search-result-title", "{result.label}" }
                                            div { class: "search-result-meta", "{result.section}" }
                                        }
                                        div { class: "search-chip", "{result.tab.label()}" }
                                    }
                                }
                            }
                        }
                    }
                }
                div {
                    class: "top-right",
                    onmousedown: move |e| e.stop_propagation(),
                    // Save / Load
                    button { class: "btn btn-ghost btn-sm", onclick: save_config, "Save Config" }
                    button { class: "btn btn-ghost btn-sm", onclick: load_config, "Load Config" }
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: move |_| {
                            let current = show_config_modal();
                            show_config_modal.set(!current);
                        },
                        "Show Config"
                    }
                    // Status badge
                    div {
                        class: if is_running { "status-badge running" } else { "status-badge stopped" },
                        div {
                            class: if is_running { "status-dot running" } else { "status-dot stopped" },
                        }
                        if is_running { "Running" } else { "Stopped" }
                    }
                    // Start / Stop
                    button {
                        class: if is_running { "btn btn-stop" } else { "btn btn-start" },
                        onclick: toggle_server,
                        if is_running { "\u{23F9}  Stop" } else { "\u{25B6}  Start" }
                    }
                }

                // Window Control Buttons (Minimize, Maximize, Close)
                div {
                    class: "window-controls",
                    onmousedown: move |e| e.stop_propagation(),
                    button {
                        class: "window-control-btn control-minimize",
                        title: "Minimize",
                        onclick: {
                            let w = window.clone();
                            move |_| {
                                w.set_minimized(true);
                            }
                        },
                        "\u{2014}"
                    }
                    button {
                        class: "window-control-btn control-maximize",
                        title: "Maximize / Restore",
                        onclick: {
                            let w = window.clone();
                            move |_| {
                                let max = w.is_maximized();
                                w.set_maximized(!max);
                            }
                        },
                        "\u{25A2}"
                    }
                    button {
                        class: "window-control-btn control-close",
                        title: "Close",
                        onclick: {
                            let w = window.clone();
                            move |_| {
                                w.close();
                            }
                        },
                        "\u{2715}"
                    }
                }
            }

            // ── Body (sidebar + content) ────────────────────────────
            div { class: "main",

                // ── Sidebar ─────────────────────────────────────────
                div {
                    class: "sidebar",
                    style: if immersive_mode() {
                        "display: none;"
                    } else if sidebar_collapsed() {
                        "width: 0px; min-width: 0px; padding: 0px; margin: 0px; border-right-color: transparent; opacity: 0; pointer-events: none;"
                    } else {
                        "width: {sidebar_width}px; min-width: {sidebar_width}px; padding: 12px 8px; margin: 0px; opacity: 1; pointer-events: auto;"
                    },

                    // Favorites section
                    if !config.read().sidebar_favorites.is_empty() {
                        div {
                            class: "sidebar-section-header interactive",
                            onclick: move |_| favorites_expanded.set(!favorites_expanded()),
                            span { class: "section-arrow", if favorites_expanded() { "▼" } else { "▶" } }
                            span { "★ Favorites" }
                        }
                        if favorites_expanded() {
                            for tab_str in config.read().sidebar_favorites.clone().iter() {
                                if let Some(tab) = Tab::from_str(tab_str) {
                                    {render_sidebar_item(tab, false)}
                                }
                            }
                        }
                    }

                    // General/Top-level section
                    div {
                        class: "sidebar-section-header interactive",
                        onclick: move |_| general_expanded.set(!general_expanded()),
                        span { class: "section-arrow", if general_expanded() { "▼" } else { "▶" } }
                        span { "General" }
                    }
                    if general_expanded() {
                        for tab in &[Tab::Chat, Tab::Planner, Tab::Monitor, Tab::Mcp, Tab::Agents] {
                            {render_sidebar_item(*tab, false)}
                        }
                    }

                    // Models & Instances section
                    div {
                        class: "sidebar-section-header interactive",
                        onclick: move |_| models_expanded.set(!models_expanded()),
                        span { class: "section-arrow", if models_expanded() { "▼" } else { "▶" } }
                        span { "Models & Instances" }
                    }
                    if models_expanded() {
                        for tab in &[Tab::Model, Tab::Library, Tab::Download, Tab::Instances] {
                            {render_sidebar_item(*tab, true)}
                        }
                    }

                    // Llama-Server sub-section
                    div {
                        class: "sidebar-section-header interactive",
                        onclick: move |_| server_expanded.set(!server_expanded()),
                        span { class: "section-arrow", if server_expanded() { "▼" } else { "▶" } }
                        span { "Llama-Server" }
                    }
                    if server_expanded() {
                        for tab in &[Tab::Server, Tab::Context, Tab::Gpu, Tab::Performance, Tab::Sampling, Tab::Advanced, Tab::Api] {
                            {render_sidebar_item(*tab, true)}
                        }
                    }

                    // Productivity & Research sub-section
                    div {
                        class: "sidebar-section-header interactive",
                        onclick: move |_| productivity_expanded.set(!productivity_expanded()),
                        span { class: "section-arrow", if productivity_expanded() { "▼" } else { "▶" } }
                        span { "Productivity & Research" }
                    }
                    if productivity_expanded() {
                        for tab in &[Tab::Calendar, Tab::Todos, Tab::QuickNotes, Tab::Compare, Tab::DeepResearch] {
                            {render_sidebar_item(*tab, true)}
                        }
                    }

                    // Space pusher to push App Settings to the bottom
                    div { style: "flex: 1;" }

                    // App Settings standalone button at the bottom of the sidebar
                    button {
                        class: if active_tab() == Tab::AppSettings { "sidebar-item active" } else { "sidebar-item" },
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; margin-top: auto; border-top: 1px solid var(--color-hairline); border-radius: 10px; padding: 10px 14px;",
                        onclick: move |_| active_tab.set(Tab::AppSettings),
                        div { style: "display: flex; align-items: center; gap: 10px;",
                            span { class: "sidebar-icon", aria_hidden: "true", {Tab::AppSettings.icon()} }
                            span { {Tab::AppSettings.label()} }
                        }
                    }
                }

                // Vertical Resizer Handle
                if !immersive_mode() && !sidebar_collapsed() {
                    div {
                        class: if drag_state() == DragType::Sidebar { "resizer-v dragging" } else { "resizer-v" },
                        onmousedown: on_sidebar_resizer_mousedown,
                    }
                }

                // ── Content area ────────────────────────────────────
                div {
                    class: "content",
                    key: "{active_tab().as_str()}",
                    style: if active_tab() == Tab::Chat {
                        if immersive_mode() {
                            "height: 100%; overflow: scroll; display: flex; flex-direction: column; padding: 0;"
                        } else {
                            "height: 100%; overflow: scroll; display: flex; flex-direction: column;"
                        }
                    } else { "" },
                    // Error toast
                    if let Some(ref msg) = *error_msg.read() {
                        div { class: "error-toast",
                            "\u{26A0}\u{FE0F} "
                            {msg.clone()}
                        }
                    }

                    match active_tab() {
                        Tab::Chat        => rsx! { TabChat        { config, search_target, routed_instance, immersive_mode } },
                        Tab::Planner     => rsx! { TabPlanner     { config, search_target } },
                        Tab::Monitor     => rsx! { TabMonitor     { config, search_target } },
                        Tab::Mcp         => rsx! { TabMcp         { config, search_target } },
                        Tab::Agents      => rsx! { TabAgents      { config, search_target } },
                        Tab::Model       => rsx! { TabModel       { config, search_target } },
                        Tab::Library     => rsx! { TabLibrary     { config, scanned_models, index_status, scan_trigger, search_target } },
                        Tab::Download    => rsx! { TabDownload    { search_target } },
                        Tab::Server      => rsx! { TabServer      { config, search_target } },
                        Tab::Instances   => rsx! { TabInstances   { config, search_target, routed_instance } },
                        Tab::Context     => rsx! { TabContext      { config, search_target } },
                        Tab::Gpu         => rsx! { TabGpu         { config, search_target } },
                        Tab::Performance => rsx! { TabPerformance { config, search_target } },
                        Tab::Sampling    => rsx! { TabSampling    { config, search_target } },
                        Tab::Advanced    => rsx! { TabAdvanced    { config, search_target } },
                        Tab::Api         => rsx! { TabApi         { config, search_target } },
                        Tab::Calendar    => rsx! { TabCalendar    { config } },
                        Tab::Todos       => rsx! { TabTodos       {} },
                        Tab::QuickNotes  => rsx! { TabQuickNotes  {} },
                        Tab::Compare     => rsx! { TabCompare     { config } },
                        Tab::DeepResearch => rsx! { TabDeepResearch { config } },
                        Tab::AppSettings => rsx! { TabAppSettings { config, search_target, scan_trigger } },
                    }

                    // Available models
                    if active_tab() != Tab::Chat && active_tab() != Tab::Planner && active_tab() != Tab::Monitor && active_tab() != Tab::Calendar && active_tab() != Tab::Todos && active_tab() != Tab::QuickNotes && active_tab() != Tab::Compare && active_tab() != Tab::DeepResearch && active_tab() != Tab::AppSettings && is_running && !available_models.read().is_empty() {
                        div { class: "card",
                            div { class: "card-title", "Available Models" }
                            if loading_models() {
                                div { class: "form-hint", "Loading models\u{2026}" }
                            } else {
                                div { class: "form-group",
                                    label { r#for: "form-select-a-model-to-load-1", class: "form-label", "Select a model to load" }
                                    select {
                    id: "form-select-a-model-to-load-1",
                    autofocus: search_target.read().as_str() == "form-select-a-model-to-load-1",
                                        class: "form-input",
                                        value: selected_model(),
                                        onchange: move |e: Event<FormData>| {
                                            selected_model.set(e.value());
                                        },
                                        option { value: "", "Choose a model\u{2026}" }
                                        for model in available_models.read().iter() {
                                            option { value: model.clone(), {model.clone()} }
                                        }
                                    }
                                }
                                if !selected_model().is_empty() {
                                    button {
                                        class: "btn btn-start",
                                        onclick: {
                                            let model = selected_model();
                                            let port = config.read().port;
                                            move |_| {
                                                let model = model.clone();
                                                let port = port;
                                                spawn(async move {
                                                    let url = format!("http://127.0.0.1:{}/api/models/load", port);
                                                    let payload = serde_json::json!({ "model": model });
                                                    let _ = reqwest::Client::new()
                                                        .post(&url)
                                                        .json(&payload)
                                                        .send()
                                                        .await;
                                                });
                                            }
                                        },
                                        "Load Model"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Horizontal Resizer Handle
            if !immersive_mode() {
                div {
                    class: if drag_state() == DragType::LogPanel { "resizer-h dragging" } else { "resizer-h" },
                    onmousedown: on_log_resizer_mousedown,
                }
            }

            // ── Output panel integrated into main UI ───────────────
            if !immersive_mode() {
                div {
                    class: "log-panel",
                    style: "height: {log_panel_height}px;",
                div { class: "log-header",
                    div { class: "log-title", "Output" }
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: move |_| logs.write().clear(),
                        "Clear"
                    }
                }
                div { class: "log-content",
                    if logs.read().is_empty() {
                        div { class: "log-empty", "Server output will appear here\u{2026}" }
                    }
                    for (i, line) in logs.read().iter().enumerate() {
                        div {
                            key: "{i}",
                            class: {
                                let cls = if line.starts_with("[ERROR]") || line.starts_with("[ERR]") {
                                    "log-line log-err"
                                } else if line.starts_with("[MANAGER]") {
                                    "log-line log-info"
                                } else {
                                    "log-line log-out"
                                };
                                cls
                            },
                            {line.clone()}
                        }
                    }
                }
            }
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  Tab Components
// ═════════════════════════════════════════════════════════════════════════════

// ── Model ────────────────────────────────────────────────────────────────────

#[component]
fn TabModel(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    let pick_model = move |_| {
        spawn(async move {
            if let Ok(Some(path)) = tokio::task::spawn_blocking(move || {
                rfd::FileDialog::new()
                    .add_filter("GGUF Model", &["gguf"])
                    .add_filter("All Files", &["*"])
                    .pick_file()
            })
            .await
            {
                config.write().model_path = path.to_string_lossy().to_string();
            }
        });
    };

    let pick_exe = move |_| {
        spawn(async move {
            if let Ok(Some(path)) = tokio::task::spawn_blocking(move || {
                rfd::FileDialog::new()
                    .add_filter("Executable", &["exe", ""])
                    .add_filter("All Files", &["*"])
                    .pick_file()
            })
            .await
            {
                config.write().exe_path = path.to_string_lossy().to_string();
            }
        });
    };

    let pick_model_dir = move |_| {
        spawn(async move {
            if let Ok(Some(path)) =
                tokio::task::spawn_blocking(move || rfd::FileDialog::new().pick_folder()).await
            {
                config.write().model_dir = path.to_string_lossy().to_string();
            }
        });
    };

    rsx! {
        div { class: "section-title", "Model Configuration" }
        div { class: "section-desc", "Set the model source. Provide a local GGUF file, a URL, or a HuggingFace repository." }

        // Executable
        div { class: "card",
            div { class: "card-title", "Executable" }
            div { class: "form-group",
                label { r#for: "form-llama-server-path-1", class: "form-label", "llama-server path" }
                div { class: "input-group",
                    input {
                    id: "form-llama-server-path-1",
                    autofocus: search_target.read().as_str() == "form-llama-server-path-1",
                        class: "form-input",
                        value: config.read().exe_path.clone(),
                        placeholder: "llama-server",
                        oninput: move |e: Event<FormData>| config.write().exe_path = e.value(),
                    }
                    button { class: "btn-browse", onclick: pick_exe, "Browse" }
                }
                div { class: "form-hint", "Path to the llama-server binary. Use \u{201C}llama-server\u{201D} if it is on your PATH." }
            }
        }

        // Local model
        div { class: "card",
            div { class: "card-title", "Local Model" }
            div { class: "form-group",
                label { r#for: "form-model-file-gguf-1", class: "form-label", "Model File (.gguf)" }
                div { class: "input-group",
                    input {
                    id: "form-model-file-gguf-1",
                    autofocus: search_target.read().as_str() == "form-model-file-gguf-1",
                        class: "form-input",
                        value: config.read().model_path.clone(),
                        placeholder: "C:\\models\\my-model.gguf",
                        oninput: move |e: Event<FormData>| config.write().model_path = e.value(),
                    }
                    button { class: "btn-browse", onclick: pick_model, "Browse" }
                }
            }
            div { class: "form-group",
                label { r#for: "form-model-alias-1", class: "form-label", "Model Alias" }
                input {
                    id: "form-model-alias-1",
                    autofocus: search_target.read().as_str() == "form-model-alias-1",
                    class: "form-input",
                    value: config.read().model_alias.clone(),
                    placeholder: "Optional friendly name",
                    oninput: move |e: Event<FormData>| config.write().model_alias = e.value(),
                }
            }
        }

        // Model directory
        div { class: "card",
            div { class: "card-title", "Model Directory" }
            div { class: "form-group",
                label { r#for: "form-model-directory-models-dir-1", class: "form-label", "Model Directory (--models-dir)" }
                div { class: "input-group",
                    input {
                    id: "form-model-directory-models-dir-1",
                    autofocus: search_target.read().as_str() == "form-model-directory-models-dir-1",
                        class: "form-input",
                        value: config.read().model_dir.clone(),
                        placeholder: "C:\\models",
                        oninput: move |e: Event<FormData>| config.write().model_dir = e.value(),
                    }
                    button { class: "btn-browse", onclick: pick_model_dir, "Browse" }
                }
                div { class: "form-hint", "Point to a directory containing .gguf models. The server will expose an endpoint to list and select which model to load." }
            }
        }

        // Remote model
        div { class: "card",
            div { class: "card-title", "Remote Model (Alternative)" }
            div { class: "form-group",
                label { r#for: "form-model-url-1", class: "form-label", "Model URL" }
                input {
                    id: "form-model-url-1",
                    autofocus: search_target.read().as_str() == "form-model-url-1",
                    class: "form-input",
                    value: config.read().model_url.clone(),
                    placeholder: "https://example.com/model.gguf",
                    oninput: move |e: Event<FormData>| config.write().model_url = e.value(),
                }
            }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-huggingface-repo-1", class: "form-label", "HuggingFace Repo" }
                    input {
                    id: "form-huggingface-repo-1",
                    autofocus: search_target.read().as_str() == "form-huggingface-repo-1",
                        class: "form-input",
                        value: config.read().hf_repo.clone(),
                        placeholder: "org/model-name-GGUF",
                        oninput: move |e: Event<FormData>| config.write().hf_repo = e.value(),
                    }
                }
                div { class: "form-group",
                    label { r#for: "form-huggingface-file-1", class: "form-label", "HuggingFace File" }
                    input {
                    id: "form-huggingface-file-1",
                    autofocus: search_target.read().as_str() == "form-huggingface-file-1",
                        class: "form-input",
                        value: config.read().hf_file.clone(),
                        placeholder: "model-Q4_K_M.gguf",
                        oninput: move |e: Event<FormData>| config.write().hf_file = e.value(),
                    }
                }
            }
            div { class: "form-group",
                label { r#for: "form-huggingface-token-1", class: "form-label", "HuggingFace Token" }
                input {
                    id: "form-huggingface-token-1",
                    autofocus: search_target.read().as_str() == "form-huggingface-token-1",
                    class: "form-input",
                    r#type: "password",
                    value: config.read().hf_token.clone(),
                    placeholder: "hf_\u{2026}",
                    oninput: move |e: Event<FormData>| config.write().hf_token = e.value(),
                }
                div { class: "form-hint", "Required for gated models. Stored locally only." }
            }
        }

        // Chat / System Prompt
        div { class: "card",
            div { class: "card-title", "Chat Template & System Prompt" }
            div { class: "form-group",
                label { r#for: "form-chat-template-1", class: "form-label", "Chat Template" }
                input {
                    id: "form-chat-template-1",
                    autofocus: search_target.read().as_str() == "form-chat-template-1",
                    class: "form-input",
                    value: config.read().chat_template.clone(),
                    placeholder: "e.g. chatml, llama3, gemma\u{2026}",
                    oninput: move |e: Event<FormData>| config.write().chat_template = e.value(),
                }
                div { class: "form-hint", "Override the model's built-in chat template." }
            }
            div { class: "form-group",
                label { r#for: "form-system-prompt-1", class: "form-label", "System Prompt" }
                textarea {
                    id: "form-system-prompt-1",
                    autofocus: search_target.read().as_str() == "form-system-prompt-1",
                    class: "form-textarea",
                    value: config.read().system_prompt.clone(),
                    placeholder: "You are a helpful assistant.",
                    oninput: move |e: Event<FormData>| config.write().system_prompt = e.value(),
                }
            }
        }
    }
}

// ── Server ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct OptimizationSuggestion {
    key: String,
    label: String,
    current: String,
    recommended: String,
    reason: String,
    selected: bool,
}

fn suggest_server_optimizations(cfg: &ServerConfig) -> Vec<OptimizationSuggestion> {
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4);
    let mut suggestions = Vec::new();

    if cfg.threads == 0 {
        suggestions.push(OptimizationSuggestion {
            key: "threads".into(),
            label: "Worker threads (-t)".into(),
            current: "auto".into(),
            recommended: cpu_count.to_string(),
            reason: format!(
                "Use all available CPU cores ({}) for better throughput.",
                cpu_count
            ),
            selected: false,
        });
    } else if cfg.threads > cpu_count && cpu_count > 0 {
        suggestions.push(OptimizationSuggestion {
            key: "threads".into(),
            label: "Worker threads (-t)".into(),
            current: cfg.threads.to_string(),
            recommended: cpu_count.to_string(),
            reason: "Limit thread count to available CPUs to prevent oversubscription.".into(),
            selected: false,
        });
    }

    let recommended_batch = if cfg.threads > 0 {
        std::cmp::max(1, cfg.threads / 2)
    } else {
        std::cmp::max(1, cpu_count / 2)
    };
    if cfg.threads_batch == 0 {
        suggestions.push(OptimizationSuggestion {
            key: "threads_batch".into(),
            label: "Batch threads (-tb)".into(),
            current: "auto".into(),
            recommended: recommended_batch.to_string(),
            reason: "Use fewer batch threads than worker threads for balanced throughput and lower latency.".into(),
            selected: false,
        });
    }

    if cfg.threads_http == 0 {
        suggestions.push(OptimizationSuggestion {
            key: "threads_http".into(),
            label: "HTTP threads".into(),
            current: "auto".into(),
            recommended: std::cmp::max(2, cpu_count / 2).to_string(),
            reason: "Allocate API server workers without overwhelming the host.".into(),
            selected: false,
        });
    }

    if !cfg.cont_batching {
        suggestions.push(OptimizationSuggestion {
            key: "cont_batching".into(),
            label: "Continuous batching (-cb)".into(),
            current: "off".into(),
            recommended: "on".into(),
            reason: "Continuous batching improves request throughput for many clients.".into(),
            selected: false,
        });
    }

    if !cfg.fit {
        suggestions.push(OptimizationSuggestion {
            key: "fit".into(),
            label: "Fit memory (--fit)".into(),
            current: "off".into(),
            recommended: "on".into(),
            reason: "Enable memory fitting to reduce peak RAM usage for large models.".into(),
            selected: false,
        });
    }

    if !cfg.flash_attn {
        suggestions.push(OptimizationSuggestion {
            key: "flash_attn".into(),
            label: "Flash attention (-fa)".into(),
            current: "off".into(),
            recommended: "on".into(),
            reason: "Use flash attention when supported for faster execution.".into(),
            selected: false,
        });
    }

    if cfg.parallel <= 1 && cpu_count > 1 {
        suggestions.push(OptimizationSuggestion {
            key: "parallel".into(),
            label: "Parallel requests (-np)".into(),
            current: cfg.parallel.to_string(),
            recommended: "2".into(),
            reason: "Enable a small amount of request parallelism to improve throughput.".into(),
            selected: false,
        });
    }

    if cfg.ubatch_size > cfg.batch_size && cfg.batch_size > 0 {
        suggestions.push(OptimizationSuggestion {
            key: "ubatch_size".into(),
            label: "Micro-batch size (-ub)".into(),
            current: cfg.ubatch_size.to_string(),
            recommended: cfg.batch_size.to_string(),
            reason: "Micro-batch size (-ub) should not exceed physical batch size (-b). Aligning them prevents execution issues.".into(),
            selected: false,
        });
    }

    if cfg.ctx_size >= 16384
        && (cfg.cache_type_k == CacheType::F16 || cfg.cache_type_v == CacheType::F16)
    {
        suggestions.push(OptimizationSuggestion {
            key: "kv_cache_quant".into(),
            label: "K/V Cache Quantization".into(),
            current: "f16".into(),
            recommended: "q8_0".into(),
            reason: "Large context sizes use significant memory. Quantizing K/V cache to q8_0 saves memory with virtually no loss in quality.".into(),
            selected: false,
        });
    }

    if !cfg.draft_model.is_empty() && cfg.draft_gpu_layers <= 0 && cfg.gpu_layers > 0 {
        suggestions.push(OptimizationSuggestion {
            key: "draft_gpu_layers".into(),
            label: "Draft GPU Layers".into(),
            current: "0".into(),
            recommended: "99".into(),
            reason: "Draft model should be offloaded to GPU to match main model GPU usage and speed up speculative decoding.".into(),
            selected: false,
        });
    }

    if cfg.no_warmup {
        suggestions.push(OptimizationSuggestion {
            key: "no_warmup".into(),
            label: "Disable Warmup Skip".into(),
            current: "skipped".into(),
            recommended: "false".into(),
            reason: "Warmup runs check tensors and load cache, reducing latency for the user's first query.".into(),
            selected: false,
        });
    }

    if cfg.no_mmap {
        suggestions.push(OptimizationSuggestion {
            key: "no_mmap".into(),
            label: "Enable Memory Mapping (mmap)".into(),
            current: "off".into(),
            recommended: "false".into(),
            reason: "Enable memory mapping to load models instantly and reduce memory overhead."
                .into(),
            selected: false,
        });
    }

    suggestions
}

fn apply_server_optimization(cfg: &mut ServerConfig, key: &str, value: &str) {
    match key {
        "threads" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.threads = v;
            }
        }
        "threads_batch" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.threads_batch = v;
            }
        }
        "threads_http" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.threads_http = v;
            }
        }
        "cont_batching" => {
            cfg.cont_batching = value.eq_ignore_ascii_case("on")
                || value.eq_ignore_ascii_case("enabled")
                || value.eq_ignore_ascii_case("true");
        }
        "fit" => {
            cfg.fit = value.eq_ignore_ascii_case("on")
                || value.eq_ignore_ascii_case("enabled")
                || value.eq_ignore_ascii_case("true");
        }
        "flash_attn" => {
            cfg.flash_attn = value.eq_ignore_ascii_case("on")
                || value.eq_ignore_ascii_case("enabled")
                || value.eq_ignore_ascii_case("true");
        }
        "parallel" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.parallel = v;
            }
        }
        "ubatch_size" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.ubatch_size = v;
            }
        }
        "kv_cache_quant" => {
            cfg.cache_type_k = CacheType::from_str(value);
            cfg.cache_type_v = CacheType::from_str(value);
        }
        "draft_gpu_layers" => {
            if let Ok(v) = value.parse::<i32>() {
                cfg.draft_gpu_layers = v;
            }
        }
        "no_warmup" => {
            cfg.no_warmup = value.eq_ignore_ascii_case("true");
        }
        "no_mmap" => {
            cfg.no_mmap = value.eq_ignore_ascii_case("true");
        }
        _ => {}
    }
}

#[component]
fn TabServer(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    let mut suggestions = use_signal(Vec::<OptimizationSuggestion>::new);
    let analyze = move |_| {
        suggestions.set(suggest_server_optimizations(&config.read().clone()));
    };

    let apply_selected = move |_| {
        let selected = suggestions
            .read()
            .iter()
            .filter(|s| s.selected)
            .cloned()
            .collect::<Vec<_>>();
        if !selected.is_empty() {
            let mut cfg = config.write();
            for suggestion in selected {
                apply_server_optimization(&mut cfg, &suggestion.key, &suggestion.recommended);
            }
        }
    };

    let apply_all = move |_| {
        let mut cfg = config.write();
        for suggestion in suggestions.read().iter() {
            apply_server_optimization(&mut cfg, &suggestion.key, &suggestion.recommended);
        }
    };

    rsx! {
        div { class: "section-title", "Server Settings" }
        div { class: "section-desc", "Network binding and connection management." }

        div { class: "card",
            div { class: "card-title", "Network" }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-host-1", class: "form-label", "Host" }
                    input {
                        id: "form-host-1",
                        autofocus: search_target.read().as_str() == "form-host-1",
                        class: "form-input",
                        value: config.read().host.clone(),
                        placeholder: "127.0.0.1",
                        oninput: move |e: Event<FormData>| config.write().host = e.value(),
                    }
                    div { class: "form-hint", "Use 0.0.0.0 to expose to the network." }
                }
                div { class: "form-group",
                    label { r#for: "form-port-1", class: "form-label", "Port" }
                    input {
                        id: "form-port-1",
                        autofocus: search_target.read().as_str() == "form-port-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().port.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u16>() { config.write().port = v; }
                        },
                    }
                }
            }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-server-timeout-s-1", class: "form-label", "Server Timeout (s)" }
                    input {
                        id: "form-server-timeout-s-1",
                        autofocus: search_target.read().as_str() == "form-server-timeout-s-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().timeout.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().timeout = v; }
                        },
                    }
                    div { class: "form-hint", "0 = no timeout" }
                }
                div { class: "form-group",
                    label { r#for: "form-http-threads-1", class: "form-label", "HTTP Threads" }
                    input {
                        id: "form-http-threads-1",
                        autofocus: search_target.read().as_str() == "form-http-threads-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().threads_http.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().threads_http = v; }
                        },
                    }
                    div { class: "form-hint", "0 = auto" }
                }
            }
        }

        div { class: "card", id: "launch-optimizer-1",
            div { class: "card-title", "Launch Optimizer" }
            div { class: "form-hint", "Analyze your current llama-server launch settings and get reviewable optimization suggestions." }
            div { style: "display: flex; gap: 12px; flex-wrap: wrap;",
                button {
                    class: "btn btn-start",
                    onclick: analyze,
                    "Analyze Launch Parameters"
                }
                button {
                    class: "btn btn-outline",
                    onclick: apply_selected,
                    disabled: suggestions.read().iter().all(|s| !s.selected),
                    "Apply Selected"
                }
                button {
                    class: "btn btn-outline",
                    onclick: apply_all,
                    disabled: suggestions.read().is_empty(),
                    "Apply All"
                }
            }
            if suggestions.read().is_empty() {
                div { class: "form-hint", "Click Analyze to see suggested launch parameter optimizations." }
            } else {
                div { style: "display: grid; gap: 12px; margin-top: 16px;",
                    for (idx, suggestion) in suggestions.read().iter().enumerate().map(|(idx, suggestion)| (idx, suggestion.clone())) {
                        div { class: "card", style: "padding: 16px; background: var(--color-surface-soft);",
                            div { style: "display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;",
                                div {
                                    div { style: "font-weight: 600; color: var(--color-ink);", "{suggestion.label}" }
                                    div { class: "form-hint", style: "margin-top: 6px;", "{suggestion.reason}" }
                                }
                                button {
                                    class: if suggestion.selected { "btn btn-start btn-sm" } else { "btn btn-outline btn-sm" },
                                    onclick: move |_| {
                                        let mut items = suggestions.read().clone();
                                        if let Some(item) = items.get_mut(idx) {
                                            item.selected = !item.selected;
                                        }
                                        suggestions.set(items);
                                    },
                                    if suggestion.selected { "Selected" } else { "Select" }
                                }
                            }
                            div { style: "display: flex; gap: 12px; flex-wrap: wrap; margin-top: 12px; font-size: 13px;",
                                span { style: "color: var(--color-muted);", "Current: {suggestion.current}" }
                                span { style: "color: var(--color-muted);", "Recommended: {suggestion.recommended}" }
                            }
                        }
                    }
                }
            }
        }

        // Command preview
        div { class: "card",
            div { class: "card-title", "Command Preview" }
            div { class: "cmd-preview", {config.read().command_preview()} }
        }
    }
}

// ── Context & Batching ───────────────────────────────────────────────────────

#[component]
fn TabContext(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    rsx! {
        div { class: "section-title", "Context & Batching" }
        div { class: "section-desc", "Control context window size, batch processing, and parallel request handling." }

        div { class: "card",
            div { class: "card-title", "Context Window" }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-context-size-c-1", class: "form-label", "Context Size (-c)" }
                    input {
                    id: "form-context-size-c-1",
                    autofocus: search_target.read().as_str() == "form-context-size-c-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().ctx_size.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().ctx_size = v; }
                        },
                    }
                    div { class: "form-hint", "Total context length in tokens. Default 8192." }
                }
                div { class: "form-group",
                    label { r#for: "form-max-predict-n-1", class: "form-label", "Max Predict (-n)" }
                    input {
                    id: "form-max-predict-n-1",
                    autofocus: search_target.read().as_str() == "form-max-predict-n-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().predict.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<i32>() { config.write().predict = v; }
                        },
                    }
                    div { class: "form-hint", "-1 = infinite generation" }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "Batching" }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-batch-size-b-1", class: "form-label", "Batch Size (-b)" }
                    input {
                    id: "form-batch-size-b-1",
                    autofocus: search_target.read().as_str() == "form-batch-size-b-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().batch_size.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().batch_size = v; }
                        },
                    }
                    div { class: "form-hint", "Logical max batch size (prompt processing)." }
                }
                div { class: "form-group",
                    label { r#for: "form-micro-batch-size-ub-1", class: "form-label", "Micro-Batch Size (-ub)" }
                    input {
                    id: "form-micro-batch-size-ub-1",
                    autofocus: search_target.read().as_str() == "form-micro-batch-size-ub-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().ubatch_size.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().ubatch_size = v; }
                        },
                    }
                    div { class: "form-hint", "Physical batch size per compute call." }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "Parallelism" }
            div { class: "form-group",
                label { r#for: "form-parallel-sequences-np-1", class: "form-label", "Parallel Sequences (-np)" }
                input {
                    id: "form-parallel-sequences-np-1",
                    autofocus: search_target.read().as_str() == "form-parallel-sequences-np-1",
                    class: "form-input",
                    r#type: "number",
                    value: config.read().parallel.to_string(),
                    oninput: move |e: Event<FormData>| {
                        if let Ok(v) = e.value().parse::<u32>() { config.write().parallel = v; }
                    },
                }
                div { class: "form-hint", "Number of concurrent request slots. Each uses ctx_size / parallel tokens." }
            }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Continuous Batching" }
                    div { class: "toggle-desc", "Process new tokens from multiple requests together for higher throughput." }
                }
                div {
                    class: if config.read().cont_batching { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| {
                        let cur = config.read().cont_batching;
                        config.write().cont_batching = !cur;
                    },
                    div { class: "toggle-thumb" }
                }
            }
        }
    }
}

// ── GPU & Memory ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub temperature: f32,
    pub utilization: f32,
    pub mem_used: f32,
    pub mem_total: f32,
    pub power: f32,
}

#[component]
fn TabGpu(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    let mut gpu_info = use_signal(|| None::<GpuInfo>);

    let _gpu_poller = use_resource(move || async move {
        loop {
            let info = tokio::task::spawn_blocking(move || {
                    let output = std::process::Command::new("nvidia-smi")
                        .args(&[
                            "--query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total,power.draw",
                            "--format=csv,noheader,nounits"
                        ])
                        .output();

                    if let Ok(out) = output {
                        if out.status.success() {
                            let text = String::from_utf8_lossy(&out.stdout);
                            let parts: Vec<&str> = text.trim().split(',').map(|s| s.trim()).collect();
                            if parts.len() >= 6 {
                                return Some(GpuInfo {
                                    name: parts[0].to_string(),
                                    temperature: parts[1].parse::<f32>().unwrap_or(0.0),
                                    utilization: parts[2].parse::<f32>().unwrap_or(0.0),
                                    mem_used: parts[3].parse::<f32>().unwrap_or(0.0),
                                    mem_total: parts[4].parse::<f32>().unwrap_or(0.0),
                                    power: parts[5].parse::<f32>().unwrap_or(0.0),
                                });
                            }
                        }
                    }
                    None
                }).await.unwrap_or(None);

            gpu_info.set(info);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    let vram_pct = gpu_info()
        .map(|info| {
            if info.mem_total > 0.0 {
                info.mem_used / info.mem_total * 100.0
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);

    rsx! {
        div { class: "section-title", "GPU & Memory" }
        div { class: "section-desc", "Control GPU offloading, memory mapping, and KV cache quantization." }

        // Live GPU Dashboard
        if let Some(info) = gpu_info() {
            div { class: "card",
                div { class: "card-title", "GPU Status ({info.name})" }
                div { style: "display: flex; gap: 24px; flex-wrap: wrap; margin-top: 8px;",
                    div { style: "flex: 1; min-width: 200px; display: flex; flex-direction: column; gap: 8px;",
                        div { style: "display: flex; justify-content: space-between; font-size: 13px;",
                            span { style: "font-weight: 500; color: var(--color-ink);", "VRAM Usage" }
                            span { style: "color: var(--color-muted);", "{info.mem_used as u32} MiB / {info.mem_total as u32} MiB" }
                        }
                        div {
                            style: "width: 100%; height: 8px; background: var(--color-surface-strong); border-radius: 9999px; overflow: hidden;",
                            div {
                                style: "height: 100%; width: {vram_pct}%; background: var(--color-brand-accent); border-radius: 9999px; transition: width 0.3s ease-in-out;"
                            }
                        }
                    }
                    div { style: "flex: 1; min-width: 200px; display: flex; flex-direction: column; gap: 8px;",
                        div { style: "display: flex; justify-content: space-between; font-size: 13px;",
                            span { style: "font-weight: 500; color: var(--color-ink);", "GPU Core Load" }
                            span { style: "color: var(--color-muted);", "{info.utilization as u32}%" }
                        }
                        div {
                            style: "width: 100%; height: 8px; background: var(--color-surface-strong); border-radius: 9999px; overflow: hidden;",
                            div {
                                style: "height: 100%; width: {info.utilization}%; background: var(--color-success); border-radius: 9999px; transition: width 0.3s ease-in-out;"
                            }
                        }
                    }
                }
                div { style: "display: flex; gap: 32px; margin-top: 12px; font-size: 13px; border-top: 1px solid var(--color-hairline); padding-top: 12px;",
                    div {
                        "Temperature: "
                        span {
                            style: if info.temperature > 75.0 { "color: var(--color-error); font-weight: 600;" } else { "color: var(--color-ink); font-weight: 600;" },
                            "{info.temperature}\u{00B0}C"
                        }
                    }
                    div {
                        "Power Draw: "
                        span { style: "color: var(--color-ink); font-weight: 600;", "{info.power} W" }
                    }
                }
            }
        } else {
            div { class: "card",
                div { class: "card-title", "GPU Status" }
                div { style: "font-size: 13px; color: var(--color-muted); font-style: italic;", "Querying GPU info..." }
            }
        }

        div { class: "card",
            div { class: "card-title", "GPU Offload" }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-gpu-layers-ngl-1", class: "form-label", "GPU Layers (-ngl)" }
                    input {
                    id: "form-gpu-layers-ngl-1",
                    autofocus: search_target.read().as_str() == "form-gpu-layers-ngl-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().gpu_layers.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<i32>() { config.write().gpu_layers = v; }
                        },
                    }
                    div { class: "form-hint", "-1 = offload all layers to GPU. 0 = CPU only." }
                }
                div { class: "form-group",
                    label { r#for: "form-main-gpu-mg-1", class: "form-label", "Main GPU (-mg)" }
                    input {
                    id: "form-main-gpu-mg-1",
                    autofocus: search_target.read().as_str() == "form-main-gpu-mg-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().main_gpu.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().main_gpu = v; }
                        },
                    }
                    div { class: "form-hint", "GPU index when using multiple GPUs." }
                }
            }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-split-mode-sm-1", class: "form-label", "Split Mode (-sm)" }
                    select {
                    id: "form-split-mode-sm-1",
                    autofocus: search_target.read().as_str() == "form-split-mode-sm-1",
                        class: "form-select",
                        value: config.read().split_mode.as_str().to_string(),
                        onchange: move |e: Event<FormData>| config.write().split_mode = SplitMode::from_str(&e.value()),
                        option { value: "layer", "Layer (default)" }
                        option { value: "row", "Row" }
                        option { value: "none", "None" }
                    }
                }
                div { class: "form-group",
                    label { r#for: "form-tensor-split-ts-1", class: "form-label", "Tensor Split (-ts)" }
                    input {
                    id: "form-tensor-split-ts-1",
                    autofocus: search_target.read().as_str() == "form-tensor-split-ts-1",
                        class: "form-input",
                        value: config.read().tensor_split.clone(),
                        placeholder: "e.g. 3,2 for 60/40 split",
                        oninput: move |e: Event<FormData>| config.write().tensor_split = e.value(),
                    }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "Memory Management" }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Auto-Fit (--fit on)" }
                    div { class: "toggle-desc", "Automatically determine how many layers to offload based on available VRAM. Recommended." }
                }
                div {
                    class: if config.read().fit { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().fit; config.write().fit = !c; },
                    div { class: "toggle-thumb" }
                }
            }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Lock in RAM (--mlock)" }
                    div { class: "toggle-desc", "Prevent the OS from swapping model weights to disk." }
                }
                div {
                    class: if config.read().mlock { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().mlock; config.write().mlock = !c; },
                    div { class: "toggle-thumb" }
                }
            }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Disable Memory Mapping (--no-mmap)" }
                    div { class: "toggle-desc", "Load the entire model into RAM instead of memory mapping." }
                }
                div {
                    class: if config.read().no_mmap { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().no_mmap; config.write().no_mmap = !c; },
                    div { class: "toggle-thumb" }
                }
            }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "No KV Offload (-nkvo)" }
                    div { class: "toggle-desc", "Keep the KV cache on CPU even when layers are on GPU." }
                }
                div {
                    class: if config.read().no_kv_offload { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().no_kv_offload; config.write().no_kv_offload = !c; },
                    div { class: "toggle-thumb" }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "KV Cache Quantization" }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-k-cache-type-ctk-1", class: "form-label", "K Cache Type (-ctk)" }
                    select {
                    id: "form-k-cache-type-ctk-1",
                    autofocus: search_target.read().as_str() == "form-k-cache-type-ctk-1",
                        class: "form-select",
                        value: config.read().cache_type_k.as_str().to_string(),
                        onchange: move |e: Event<FormData>| config.write().cache_type_k = CacheType::from_str(&e.value()),
                        option { value: "f16", "f16 (default)" }
                        option { value: "q8_0", "q8_0 (saves ~50% KV VRAM)" }
                        option { value: "q4_0", "q4_0 (saves ~75% KV VRAM)" }
                    }
                }
                div { class: "form-group",
                    label { r#for: "form-v-cache-type-ctv-1", class: "form-label", "V Cache Type (-ctv)" }
                    select {
                    id: "form-v-cache-type-ctv-1",
                    autofocus: search_target.read().as_str() == "form-v-cache-type-ctv-1",
                        class: "form-select",
                        value: config.read().cache_type_v.as_str().to_string(),
                        onchange: move |e: Event<FormData>| config.write().cache_type_v = CacheType::from_str(&e.value()),
                        option { value: "f16", "f16 (default)" }
                        option { value: "q8_0", "q8_0 (saves ~50% KV VRAM)" }
                        option { value: "q4_0", "q4_0 (saves ~75% KV VRAM)" }
                    }
                }
            }
            div { class: "form-hint", "Quantizing the KV cache significantly reduces VRAM usage with minimal quality impact." }
        }
    }
}

// ── Performance ──────────────────────────────────────────────────────────────

#[component]
fn TabPerformance(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    rsx! {
        div { class: "section-title", "Performance" }
        div { class: "section-desc", "Threading, Flash Attention, and other performance knobs." }

        div { class: "card",
            div { class: "card-title", "Threading" }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-threads-t-1", class: "form-label", "Threads (-t)" }
                    input {
                    id: "form-threads-t-1",
                    autofocus: search_target.read().as_str() == "form-threads-t-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().threads.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().threads = v; }
                        },
                    }
                    div { class: "form-hint", "CPU threads for generation. 0 = auto-detect." }
                }
                div { class: "form-group",
                    label { r#for: "form-batch-threads-tb-1", class: "form-label", "Batch Threads (-tb)" }
                    input {
                    id: "form-batch-threads-tb-1",
                    autofocus: search_target.read().as_str() == "form-batch-threads-tb-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().threads_batch.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().threads_batch = v; }
                        },
                    }
                    div { class: "form-hint", "CPU threads for prompt processing. 0 = auto-detect." }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "Optimizations" }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Flash Attention (-fa)" }
                    div { class: "toggle-desc", "Faster and more memory-efficient attention. Strongly recommended for newer GPUs." }
                }
                div {
                    class: if config.read().flash_attn { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().flash_attn; config.write().flash_attn = !c; },
                    div { class: "toggle-thumb" }
                }
            }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Skip Warmup (--no-warmup)" }
                    div { class: "toggle-desc", "Skip the initial warmup run. Faster startup, slower first request." }
                }
                div {
                    class: if config.read().no_warmup { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().no_warmup; config.write().no_warmup = !c; },
                    div { class: "toggle-thumb" }
                }
            }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Check Tensors (--check-tensors)" }
                    div { class: "toggle-desc", "Validate model tensor data on load. Adds startup time." }
                }
                div {
                    class: if config.read().check_tensors { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().check_tensors; config.write().check_tensors = !c; },
                    div { class: "toggle-thumb" }
                }
            }
        }
    }
}

// ── Sampling ─────────────────────────────────────────────────────────────────

#[component]
fn TabSampling(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    rsx! {
        div { class: "section-title", "Sampling Parameters" }
        div { class: "section-desc", "Default sampling configuration. Clients can override these per-request via the API." }

        div { class: "card",
            div { class: "card-title", "Core Sampling" }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-temperature-temp-1", class: "form-label", "Temperature (--temp)" }
                    input {
                    id: "form-temperature-temp-1",
                    autofocus: search_target.read().as_str() == "form-temperature-temp-1",
                        class: "form-input",
                        r#type: "number",
                        step: "0.05",
                        value: format!("{:.2}", config.read().temp),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f32>() { config.write().temp = v; }
                        },
                    }
                    div { class: "form-hint", "0 = greedy, higher = more random. Default 0.8." }
                }
                div { class: "form-group",
                    label { r#for: "form-seed-s-1", class: "form-label", "Seed (-s)" }
                    input {
                    id: "form-seed-s-1",
                    autofocus: search_target.read().as_str() == "form-seed-s-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().seed.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<i64>() { config.write().seed = v; }
                        },
                    }
                    div { class: "form-hint", "-1 = random seed each request." }
                }
            }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-top-k-top-k-1", class: "form-label", "Top-K (--top-k)" }
                    input {
                    id: "form-top-k-top-k-1",
                    autofocus: search_target.read().as_str() == "form-top-k-top-k-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().top_k.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().top_k = v; }
                        },
                    }
                    div { class: "form-hint", "0 = disabled. Default 40." }
                }
                div { class: "form-group",
                    label { r#for: "form-top-p-top-p-1", class: "form-label", "Top-P (--top-p)" }
                    input {
                    id: "form-top-p-top-p-1",
                    autofocus: search_target.read().as_str() == "form-top-p-top-p-1",
                        class: "form-input",
                        r#type: "number",
                        step: "0.05",
                        value: format!("{:.2}", config.read().top_p),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f32>() { config.write().top_p = v; }
                        },
                    }
                    div { class: "form-hint", "1.0 = disabled. Default 0.95." }
                }
            }
            div { class: "form-group",
                label { r#for: "form-min-p-min-p-1", class: "form-label", "Min-P (--min-p)" }
                input {
                    id: "form-min-p-min-p-1",
                    autofocus: search_target.read().as_str() == "form-min-p-min-p-1",
                    class: "form-input",
                    r#type: "number",
                    step: "0.01",
                    value: format!("{:.2}", config.read().min_p),
                    oninput: move |e: Event<FormData>| {
                        if let Ok(v) = e.value().parse::<f32>() { config.write().min_p = v; }
                    },
                }
                div { class: "form-hint", "Minimum token probability relative to the top token. Default 0.05." }
            }
        }

        div { class: "card",
            div { class: "card-title", "Penalties" }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-repeat-penalty-1", class: "form-label", "Repeat Penalty" }
                    input {
                    id: "form-repeat-penalty-1",
                    autofocus: search_target.read().as_str() == "form-repeat-penalty-1",
                        class: "form-input",
                        r#type: "number",
                        step: "0.05",
                        value: format!("{:.2}", config.read().repeat_penalty),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f32>() { config.write().repeat_penalty = v; }
                        },
                    }
                    div { class: "form-hint", "1.0 = no penalty." }
                }
                div { class: "form-group",
                    label { r#for: "form-presence-penalty-1", class: "form-label", "Presence Penalty" }
                    input {
                    id: "form-presence-penalty-1",
                    autofocus: search_target.read().as_str() == "form-presence-penalty-1",
                        class: "form-input",
                        r#type: "number",
                        step: "0.05",
                        value: format!("{:.2}", config.read().presence_penalty),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f32>() { config.write().presence_penalty = v; }
                        },
                    }
                }
            }
            div { class: "form-group",
                label { r#for: "form-frequency-penalty-1", class: "form-label", "Frequency Penalty" }
                input {
                    id: "form-frequency-penalty-1",
                    autofocus: search_target.read().as_str() == "form-frequency-penalty-1",
                    class: "form-input",
                    r#type: "number",
                    step: "0.05",
                    value: format!("{:.2}", config.read().frequency_penalty),
                    oninput: move |e: Event<FormData>| {
                        if let Ok(v) = e.value().parse::<f32>() { config.write().frequency_penalty = v; }
                    },
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "Grammar Constraint" }
            div { class: "form-group",
                label { r#for: "form-grammar-bnf-1", class: "form-label", "Grammar (BNF)" }
                textarea {
                    id: "form-grammar-bnf-1",
                    autofocus: search_target.read().as_str() == "form-grammar-bnf-1",
                    class: "form-textarea",
                    value: config.read().grammar.clone(),
                    placeholder: "root ::=\u{2026}",
                    oninput: move |e: Event<FormData>| config.write().grammar = e.value(),
                }
            }
            div { class: "form-group",
                label { r#for: "form-grammar-file-1", class: "form-label", "Grammar File" }
                div { class: "input-group",
                    input {
                    id: "form-grammar-file-1",
                    autofocus: search_target.read().as_str() == "form-grammar-file-1",
                        class: "form-input",
                        value: config.read().grammar_file.clone(),
                        placeholder: "Path to .gbnf file",
                        oninput: move |e: Event<FormData>| config.write().grammar_file = e.value(),
                    }
                    button {
                        class: "btn-browse",
                        onclick: move |_| {
                            let mut cfg = config;
                            spawn(async move {
                                if let Ok(Some(p)) = tokio::task::spawn_blocking(move || {
                                    rfd::FileDialog::new()
                                        .add_filter("Grammar", &["gbnf", "bnf"])
                                        .add_filter("All", &["*"])
                                        .pick_file()
                                })
                                .await
                                {
                                    cfg.write().grammar_file = p.to_string_lossy().to_string();
                                }
                            });
                        },
                        "Browse"
                    }
                }
            }
        }
    }
}

// ── Advanced ─────────────────────────────────────────────────────────────────

#[component]
fn TabAdvanced(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    rsx! {
        div { class: "section-title", "Advanced" }
        div { class: "section-desc", "RoPE scaling, LoRA adapters, speculative decoding, and embeddings." }

        // RoPE
        div { class: "card",
            div { class: "card-title", "RoPE Scaling" }
            div { class: "form-group",
                label { r#for: "form-scaling-type-1", class: "form-label", "Scaling Type" }
                select {
                    id: "form-scaling-type-1",
                    autofocus: search_target.read().as_str() == "form-scaling-type-1",
                    class: "form-select",
                    value: config.read().rope_scaling.as_str().to_string(),
                    onchange: move |e: Event<FormData>| config.write().rope_scaling = RopeScaling::from_str(&e.value()),
                    option { value: "none", "None (default)" }
                    option { value: "linear", "Linear" }
                    option { value: "yarn", "YaRN" }
                }
            }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-freq-base-1", class: "form-label", "Freq Base" }
                    input {
                    id: "form-freq-base-1",
                    autofocus: search_target.read().as_str() == "form-freq-base-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().rope_freq_base.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f32>() { config.write().rope_freq_base = v; }
                        },
                    }
                    div { class: "form-hint", "0 = model default" }
                }
                div { class: "form-group",
                    label { r#for: "form-freq-scale-1", class: "form-label", "Freq Scale" }
                    input {
                    id: "form-freq-scale-1",
                    autofocus: search_target.read().as_str() == "form-freq-scale-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().rope_freq_scale.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f32>() { config.write().rope_freq_scale = v; }
                        },
                    }
                }
            }
            if config.read().rope_scaling == RopeScaling::Yarn {
                div { class: "form-row",
                    div { class: "form-group",
                        label { r#for: "form-yarn-orig-ctx-1", class: "form-label", "YaRN Orig Ctx" }
                        input {
                    id: "form-yarn-orig-ctx-1",
                    autofocus: search_target.read().as_str() == "form-yarn-orig-ctx-1",
                            class: "form-input",
                            r#type: "number",
                            value: config.read().yarn_orig_ctx.to_string(),
                            oninput: move |e: Event<FormData>| {
                                if let Ok(v) = e.value().parse::<u32>() { config.write().yarn_orig_ctx = v; }
                            },
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "form-extension-factor-1", class: "form-label", "Extension Factor" }
                        input {
                    id: "form-extension-factor-1",
                    autofocus: search_target.read().as_str() == "form-extension-factor-1",
                            class: "form-input",
                            r#type: "number",
                            step: "0.1",
                            value: config.read().yarn_ext_factor.to_string(),
                            oninput: move |e: Event<FormData>| {
                                if let Ok(v) = e.value().parse::<f32>() { config.write().yarn_ext_factor = v; }
                            },
                        }
                    }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { r#for: "form-attention-factor-1", class: "form-label", "Attention Factor" }
                        input {
                    id: "form-attention-factor-1",
                    autofocus: search_target.read().as_str() == "form-attention-factor-1",
                            class: "form-input",
                            r#type: "number",
                            step: "0.1",
                            value: config.read().yarn_attn_factor.to_string(),
                            oninput: move |e: Event<FormData>| {
                                if let Ok(v) = e.value().parse::<f32>() { config.write().yarn_attn_factor = v; }
                            },
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "form-beta-fast-1", class: "form-label", "Beta Fast" }
                        input {
                    id: "form-beta-fast-1",
                    autofocus: search_target.read().as_str() == "form-beta-fast-1",
                            class: "form-input",
                            r#type: "number",
                            step: "0.5",
                            value: config.read().yarn_beta_fast.to_string(),
                            oninput: move |e: Event<FormData>| {
                                if let Ok(v) = e.value().parse::<f32>() { config.write().yarn_beta_fast = v; }
                            },
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "form-beta-slow-1", class: "form-label", "Beta Slow" }
                        input {
                    id: "form-beta-slow-1",
                    autofocus: search_target.read().as_str() == "form-beta-slow-1",
                            class: "form-input",
                            r#type: "number",
                            step: "0.1",
                            value: config.read().yarn_beta_slow.to_string(),
                            oninput: move |e: Event<FormData>| {
                                if let Ok(v) = e.value().parse::<f32>() { config.write().yarn_beta_slow = v; }
                            },
                        }
                    }
                }
            }
        }

        // LoRA
        div { class: "card",
            div { class: "card-title", "LoRA Adapters" }
            for (i, _lora) in config.read().lora_adapters.iter().enumerate() {
                div { class: "lora-row",
                    div { style: "flex:3;",
                        div { class: "form-group",
                            label { r#for: "form-adapter-path-1", class: "form-label", "Adapter Path" }
                            div { class: "input-group",
                                input {
                    id: "form-adapter-path-1",
                    autofocus: search_target.read().as_str() == "form-adapter-path-1",
                                    class: "form-input",
                                    value: config.read().lora_adapters[i].path.clone(),
                                    placeholder: "path/to/lora.gguf",
                                    oninput: {
                                        let idx = i;
                                        move |e: Event<FormData>| config.write().lora_adapters[idx].path = e.value()
                                    },
                                }
                                button {
                                    class: "btn-browse",
                                    onclick: {
                                        let idx = i;
                                        move |_| {
                                            let mut cfg = config;
                                            spawn(async move {
                                                if let Ok(Some(p)) = tokio::task::spawn_blocking(move || {
                                                    rfd::FileDialog::new()
                                                        .add_filter("GGUF", &["gguf"])
                                                        .add_filter("All", &["*"])
                                                        .pick_file()
                                                })
                                                .await
                                                {
                                                    cfg.write().lora_adapters[idx].path = p.to_string_lossy().to_string();
                                                }
                                            });
                                        }
                                    },
                                    "Browse"
                                }
                            }
                        }
                    }
                    div { style: "flex:1;",
                        div { class: "form-group",
                            label { r#for: "form-scale-1", class: "form-label", "Scale" }
                            input {
                    id: "form-scale-1",
                    autofocus: search_target.read().as_str() == "form-scale-1",
                                class: "form-input",
                                r#type: "number",
                                step: "0.1",
                                value: format!("{:.1}", config.read().lora_adapters[i].scale),
                                oninput: {
                                    let idx = i;
                                    move |e: Event<FormData>| {
                                        if let Ok(v) = e.value().parse::<f32>() {
                                            config.write().lora_adapters[idx].scale = v;
                                        }
                                    }
                                },
                            }
                        }
                    }
                    button {
                        class: "btn-browse",
                        style: "align-self: center; color: var(--color-error);",
                        aria_label: "Remove LoRA adapter",
                        onclick: {
                            let idx = i;
                            move |_| { config.write().lora_adapters.remove(idx); }
                        },
                        "\u{2715}"
                    }
                }
            }
            button {
                class: "btn btn-ghost btn-sm",
                style: "margin-top: 8px;",
                onclick: move |_| {
                    config.write().lora_adapters.push(LoraAdapter {
                        path: String::new(),
                        scale: 1.0,
                    });
                },
                "+ Add LoRA Adapter"
            }
        }

        // Speculative Decoding
        div { class: "card",
            div { class: "card-title", "Speculative Decoding" }
            div { class: "form-group",
                label { r#for: "form-draft-model-md-1", class: "form-label", "Draft Model (-md)" }
                div { class: "input-group",
                    input {
                    id: "form-draft-model-md-1",
                    autofocus: search_target.read().as_str() == "form-draft-model-md-1",
                        class: "form-input",
                        value: config.read().draft_model.clone(),
                        placeholder: "Path to smaller draft model.gguf",
                        oninput: move |e: Event<FormData>| config.write().draft_model = e.value(),
                    }
                    button {
                        class: "btn-browse",
                        onclick: move |_| {
                            let mut cfg = config;
                            spawn(async move {
                                if let Ok(Some(p)) = tokio::task::spawn_blocking(move || {
                                    rfd::FileDialog::new()
                                        .add_filter("GGUF", &["gguf"])
                                        .add_filter("All", &["*"])
                                        .pick_file()
                                })
                                .await
                                {
                                    cfg.write().draft_model = p.to_string_lossy().to_string();
                                }
                            });
                        },
                        "Browse"
                    }
                }
                div { class: "form-hint", "A smaller, faster model used to draft tokens that the main model verifies." }
            }
            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-draft-gpu-layers-ngld-1", class: "form-label", "Draft GPU Layers (-ngld)" }
                    input {
                    id: "form-draft-gpu-layers-ngld-1",
                    autofocus: search_target.read().as_str() == "form-draft-gpu-layers-ngld-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().draft_gpu_layers.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<i32>() { config.write().draft_gpu_layers = v; }
                        },
                    }
                }
                div { class: "form-group",
                    label { r#for: "form-draft-tokens-draft-1", class: "form-label", "Draft Tokens (--draft)" }
                    input {
                    id: "form-draft-tokens-draft-1",
                    autofocus: search_target.read().as_str() == "form-draft-tokens-draft-1",
                        class: "form-input",
                        r#type: "number",
                        value: config.read().draft_tokens.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() { config.write().draft_tokens = v; }
                        },
                    }
                }
            }
        }

        // Embedding
        div { class: "card",
            div { class: "card-title", "Embedding Mode" }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Enable Embeddings (--embedding)" }
                    div { class: "toggle-desc", "Serve embedding requests instead of text generation." }
                }
                div {
                    class: if config.read().embedding { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().embedding; config.write().embedding = !c; },
                    div { class: "toggle-thumb" }
                }
            }
            if config.read().embedding {
                div { class: "form-group", style: "margin-top: 12px;",
                    label { r#for: "form-pooling-type-1", class: "form-label", "Pooling Type" }
                    select {
                    id: "form-pooling-type-1",
                    autofocus: search_target.read().as_str() == "form-pooling-type-1",
                        class: "form-select",
                        value: config.read().pooling.as_str().to_string(),
                        onchange: move |e: Event<FormData>| config.write().pooling = PoolingType::from_str(&e.value()),
                        option { value: "none", "None" }
                        option { value: "mean", "Mean" }
                        option { value: "cls", "CLS" }
                        option { value: "last", "Last" }
                    }
                }
            }
        }
    }
}

// ── API & Security ───────────────────────────────────────────────────────────

#[component]
fn TabApi(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    rsx! {
        div { class: "section-title", "API & Security" }
        div { class: "section-desc", "Authentication, monitoring endpoints, and logging configuration." }

        div { class: "card",
            div { class: "card-title", "Authentication" }
            div { class: "form-group",
                label { r#for: "form-api-key-api-key-1", class: "form-label", "API Key (--api-key)" }
                input {
                    id: "form-api-key-api-key-1",
                    autofocus: search_target.read().as_str() == "form-api-key-api-key-1",
                    class: "form-input",
                    r#type: "password",
                    value: config.read().api_key.clone(),
                    placeholder: "Optional bearer token",
                    oninput: move |e: Event<FormData>| config.write().api_key = e.value(),
                }
                div { class: "form-hint", "If set, clients must provide this key as a Bearer token." }
            }
            div { class: "form-group",
                label { r#for: "form-api-key-file-api-key-file-1", class: "form-label", "API Key File (--api-key-file)" }
                div { class: "input-group",
                    input {
                    id: "form-api-key-file-api-key-file-1",
                    autofocus: search_target.read().as_str() == "form-api-key-file-api-key-file-1",
                        class: "form-input",
                        value: config.read().api_key_file.clone(),
                        placeholder: "Path to file containing API key",
                        oninput: move |e: Event<FormData>| config.write().api_key_file = e.value(),
                    }
                    button {
                        class: "btn-browse",
                        onclick: move |_| {
                            let mut cfg = config;
                            spawn(async move {
                                if let Ok(Some(p)) = tokio::task::spawn_blocking(move || {
                                    rfd::FileDialog::new().pick_file()
                                })
                                .await
                                {
                                    cfg.write().api_key_file = p.to_string_lossy().to_string();
                                }
                            });
                        },
                        "Browse"
                    }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "Endpoints" }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Prometheus Metrics (--metrics)" }
                    div { class: "toggle-desc", "Expose /metrics endpoint for monitoring." }
                }
                div {
                    class: if config.read().metrics { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().metrics; config.write().metrics = !c; },
                    div { class: "toggle-thumb" }
                }
            }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Slots Endpoint (--slots)" }
                    div { class: "toggle-desc", "Expose /slots endpoint to inspect active request slots." }
                }
                div {
                    class: if config.read().slots { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().slots; config.write().slots = !c; },
                    div { class: "toggle-thumb" }
                }
            }
            div { class: "form-group", style: "margin-top: 12px;",
                label { r#for: "form-slot-save-path-1", class: "form-label", "Slot Save Path" }
                div { class: "input-group",
                    input {
                    id: "form-slot-save-path-1",
                    autofocus: search_target.read().as_str() == "form-slot-save-path-1",
                        class: "form-input",
                        value: config.read().slot_save_path.clone(),
                        placeholder: "Directory for slot snapshots",
                        oninput: move |e: Event<FormData>| config.write().slot_save_path = e.value(),
                    }
                    button {
                        class: "btn-browse",
                        onclick: move |_| {
                            let mut cfg = config;
                            spawn(async move {
                                if let Ok(Some(p)) = tokio::task::spawn_blocking(move || {
                                    rfd::FileDialog::new().pick_folder()
                                })
                                .await
                                {
                                    cfg.write().slot_save_path = p.to_string_lossy().to_string();
                                }
                            });
                        },
                        "Browse"
                    }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "Logging" }
            div { class: "form-group",
                label { r#for: "form-log-format-1", class: "form-label", "Log Format" }
                select {
                    id: "form-log-format-1",
                    autofocus: search_target.read().as_str() == "form-log-format-1",
                    class: "form-select",
                    value: config.read().log_format.as_str().to_string(),
                    onchange: move |e: Event<FormData>| config.write().log_format = LogFormat::from_str(&e.value()),
                    option { value: "text", "Text (human-readable)" }
                    option { value: "json", "JSON (machine-readable)" }
                }
            }
            div { class: "toggle-row",
                div { class: "toggle-info",
                    div { class: "toggle-name", "Verbose (-v)" }
                    div { class: "toggle-desc", "Print detailed debug information to stderr." }
                }
                div {
                    class: if config.read().verbose { "toggle-switch on" } else { "toggle-switch" },
                    onclick: move |_| { let c = config.read().verbose; config.write().verbose = !c; },
                    div { class: "toggle-thumb" }
                }
            }
        }
    }
}

// ── Model Library / Index Tab ────────────────────────────────────────────────

#[component]
fn TabLibrary(
    config: Signal<ServerConfig>,
    scanned_models: Signal<Vec<library::ScannedModel>>,
    index_status: Signal<String>,
    scan_trigger: Signal<u64>,
    search_target: Signal<String>,
) -> Element {
    let mut error_local = use_signal(|| None::<String>);
    let mut library_view_mode = use_signal(|| "index".to_string());

    let enrich_all_models = move |_| {
        let mut current = scanned_models.read().clone();
        for m in &mut current {
            m.status = "pending_enrichment".to_string();
        }
        let index_path = get_default_config_path().join("model_index.json");
        let _ = library::save_index(&index_path.to_string_lossy(), &current);
        scanned_models.set(current);
        *scan_trigger.write() += 1;
    };

    // Track collapsed state (defaults to expanded)
    let collapsed_families = use_signal(Vec::<String>::new);
    let collapsed_versions = use_signal(Vec::<String>::new);

    let force_scan = move |_| {
        *scan_trigger.write() += 1;
    };

    let is_family_collapsed = move |fam: &str| collapsed_families.read().contains(&fam.to_string());

    let toggle_family = move |fam: String| {
        let mut collapsed_families = collapsed_families;
        let mut current = collapsed_families.read().clone();
        if let Some(pos) = current.iter().position(|x| x == &fam) {
            current.remove(pos);
        } else {
            current.push(fam);
        }
        collapsed_families.set(current);
    };

    let is_version_collapsed =
        move |ver: &str| collapsed_versions.read().contains(&ver.to_string());

    let toggle_version = move |ver: String| {
        let mut collapsed_versions = collapsed_versions;
        let mut current = collapsed_versions.read().clone();
        if let Some(pos) = current.iter().position(|x| x == &ver) {
            current.remove(pos);
        } else {
            current.push(ver);
        }
        collapsed_versions.set(current);
    };

    // Grouping calculations
    let mut unique_families = Vec::new();
    for m in scanned_models.read().iter() {
        if !unique_families.contains(&m.family) {
            unique_families.push(m.family.clone());
        }
    }
    unique_families.sort_by_key(|f| f.to_lowercase());

    rsx! {
        div { class: "section-title", "Model Library & Indexer" }
        div { class: "section-desc", "Scan directories containing GGUF/safetensors/ckpt models. Enriches metadata (use-case, size, HF/GitHub links) automatically using SearXNG and a running local model." }

        if let Some(ref err) = *error_local.read() {
            div { class: "error-toast", style: "margin-bottom: 12px;",
                "\u{26A0}\u{FE0F} "
                {err.clone()}
            }
        }

        // Sub-navigation Tabs
        div {
            style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; flex-wrap: wrap; gap: 12px;",
            div {
                class: "nav-pill-group",
                button {
                    class: if library_view_mode() == "index" { "category-tab active" } else { "category-tab" },
                    onclick: move |_| library_view_mode.set("index".to_string()),
                    "\u{1F4C1} Directory & Index"
                }
                button {
                    class: if library_view_mode() == "progress" { "category-tab active" } else { "category-tab" },
                    onclick: move |_| library_view_mode.set("progress".to_string()),
                    "\u{1F4CA} Enrichment Progress"
                }
            }
            div { style: "display: flex; gap: 8px;",
                button {
                    class: "btn btn-ghost btn-sm",
                    style: "color: var(--color-warning); border-color: var(--color-hairline);",
                    onclick: enrich_all_models,
                    "\u{26A1} Enrich All Models"
                }
                button {
                    class: "btn btn-ghost btn-sm",
                    style: "color: var(--color-brand-accent); border-color: var(--color-hairline);",
                    onclick: force_scan,
                    "\u{1F50E} Scan & Index Now"
                }
            }
        }

        if library_view_mode() == "index" {
            // Indexed Models List
            div { class: "card",
                div { style: "display: flex; align-items: center; justify-content: space-between;",
                    div { class: "card-title", "Indexed Models ({scanned_models.read().len()})" }
                    div { class: "form-hint", style: "margin: 0;", "Status: " span { style: "color: var(--color-brand-accent); font-weight: 600;", "{index_status()}" } }
                }

            if scanned_models.read().is_empty() {
                div { class: "log-empty", "No models indexed yet. Add scan directories and run a scan." }
            } else {
                div { style: "display: flex; flex-direction: column; gap: 10px;",
                    for family in unique_families.iter() {
                        {
                            let family_models: Vec<_> = scanned_models.read().iter().enumerate()
                                .filter(|(_, m)| m.family == *family)
                                .map(|(idx, m)| (idx, m.clone()))
                                .collect();
                            let is_collapsed = is_family_collapsed(family);

                            rsx! {
                                div {
                                    key: "{family}",
                                    style: "display: flex; flex-direction: column; gap: 4px;",

                                    // Family Header
                                    div {
                                        class: "family-header table-row-hover",
                                        role: "button",
                                        tabindex: "0",
                                        onclick: {
                                            let fam = family.clone();
                                            move |_| toggle_family(fam.clone())
                                        },
                                        onkeydown: {
                                            let fam = family.clone();
                                            let toggle_family = toggle_family.clone();
                                            move |evt| {
                                                if evt.key() == Key::Enter || evt.key() == Key::Character(" ".into()) {
                                                    toggle_family(fam.clone());
                                                }
                                            }
                                        },
                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                            span { style: "font-size: 14px; color: var(--color-primary);", if is_collapsed { "\u{25B6}" } else { "\u{25BC}" } }
                                            span { "\u{1F4C1} " }
                                            span { "{family} Family" }
                                        }
                                        span { style: "font-size: 11px; color: var(--color-muted); background: var(--color-surface-soft); padding: 2px 8px; border-radius: 12px; font-weight: 600; border: 1px solid var(--color-hairline);", "{family_models.len()} models" }
                                    }

                                    // Family Content
                                    if !is_collapsed {
                                        {
                                            let mut unique_versions = Vec::new();
                                            for (_, m) in &family_models {
                                                if !unique_versions.contains(&m.version) {
                                                    unique_versions.push(m.version.clone());
                                                }
                                            }
                                            unique_versions.sort_by_key(|v| v.to_lowercase());

                                            rsx! {
                                                for version in unique_versions.iter() {
                                                    {
                                                        let version_models: Vec<_> = family_models.iter()
                                                            .filter(|(_, m)| m.version == *version)
                                                            .collect();
                                                        let ver_key = format!("{}:{}", family, version);
                                                        let is_ver_collapsed = is_version_collapsed(&ver_key);

                                                        rsx! {
                                                            div {
                                                                key: "{version}",
                                                                style: "display: flex; flex-direction: column; gap: 4px; margin-left: 20px;",

                                                                // Version Header
                                                                div {
                                                                    class: "version-header table-row-hover",
                                                                    role: "button",
                                                                    tabindex: "0",
                                                                    onclick: {
                                                                        let vk = ver_key.clone();
                                                                        move |_| toggle_version(vk.clone())
                                                                    },
                                                                    onkeydown: {
                                                                        let vk = ver_key.clone();
                                                                        let toggle_version = toggle_version.clone();
                                                                        move |evt| {
                                                                            if evt.key() == Key::Enter || evt.key() == Key::Character(" ".into()) {
                                                                                toggle_version(vk.clone());
                                                                            }
                                                                        }
                                                                    },
                                                                    div { style: "display: flex; align-items: center; gap: 8px;",
                                                                        span { style: "font-size: 11px;", if is_ver_collapsed { "\u{25B6}" } else { "\u{25BC}" } }
                                                                        span { "\u{21B3} \u{1F5C0} " }
                                                                        span { "{version}" }
                                                                    }
                                                                    span { style: "font-size: 10px; color: var(--color-muted);", "{version_models.len()} versions/files" }
                                                                }

                                                                // Version Content
                                                                if !is_ver_collapsed {
                                                                    div { class: "version-content",
                                                                        table {
                                                                            class: "model-table",
                                                                            thead {
                                                                                tr {
                                                                                    th { "Model Name / File" }
                                                                                    th { "Use-case" }
                                                                                    th { "Size" }
                                                                                    th { "Links" }
                                                                                    th { style: "text-align: right;", "Action" }
                                                                                }
                                                                            }
                                                                            tbody {
                                                                                for &(idx, m) in version_models.iter() {
                                                                                    tr {
                                                                                        key: "{m.path}",
                                                                                        class: "table-row-hover",

                                                                                        // Column 1: Clean Name, Filename, and Tags
                                                                                        td { style: "padding: 10px 6px; max-width: 250px; word-break: break-all;",
                                                                                            div { style: "font-weight: 600; color: var(--color-ink); display: flex; align-items: center; gap: 6px; flex-wrap: wrap;",
                                                                                                {
                                                                                                    if m.clean_name == "Unknown" || m.clean_name.is_empty() {
                                                                                                        m.filename.clone()
                                                                                                    } else {
                                                                                                        m.clean_name.clone()
                                                                                                    }
                                                                                                }
                                                                                                span {
                                                                                                    style: "font-size: 9px; background: var(--color-surface-soft); color: var(--color-primary); padding: 1px 5px; border-radius: 4px; font-weight: 600; text-transform: uppercase; border: 1px solid var(--color-hairline);",
                                                                                                    "{m.version}"
                                                                                                }
                                                                                            }
                                                                                            div { style: "font-size: 10px; color: var(--color-muted); font-family: monospace; margin-top: 2px;", {m.filename.clone()} }

                                                                                                     // Hierarchy Tags BADGES!
                                                                                                     if !m.tags.is_empty() {
                                                                                                         div { style: "display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px;",
                                                                                                             for tag in m.tags.iter() {
                                                                                                                 span {
                                                                                                                     style: {
                                                                                                                         let color = if tag.starts_with("Q") || tag.starts_with("IQ") || tag == "BF16" || tag == "F16" || tag == "FP16" || tag == "F32" || tag == "FP32" {
                                                                                                                             "background: color-mix(in srgb, var(--color-tag-blue) 12%, transparent); color: var(--color-tag-blue); border: 1px solid color-mix(in srgb, var(--color-tag-blue) 28%, transparent);"
                                                                                                                         } else {
                                                                                                                             match tag.as_str() {
                                                                                                                                 "GGUF" => "background: color-mix(in srgb, var(--color-tag-purple) 14%, transparent); color: var(--color-tag-purple); border: 1px solid color-mix(in srgb, var(--color-tag-purple) 30%, transparent);",
                                                                                                                                 "Safetensors" => "background: color-mix(in srgb, var(--color-tag-blue) 12%, transparent); color: var(--color-tag-blue); border: 1px solid color-mix(in srgb, var(--color-tag-blue) 28%, transparent);",
                                                                                                                                 "Uncensored" => "background: var(--color-badge-error-bg); color: var(--color-badge-error-text); border: 1px solid var(--color-badge-error-border);",
                                                                                                                                 "Thinking" => "background: var(--color-badge-warning-bg); color: var(--color-badge-warning-text); border: 1px solid var(--color-badge-warning-border);",
                                                                                                                                 "Abliterated" => "background: color-mix(in srgb, var(--color-tag-orange) 14%, transparent); color: var(--color-tag-orange); border: 1px solid color-mix(in srgb, var(--color-tag-orange) 30%, transparent);",
                                                                                                                                 _ => "background: var(--color-badge-neutral-bg); color: var(--color-badge-neutral-text); border: 1px solid var(--color-badge-neutral-border);"
                                                                                                                             }
                                                                                                                         };
                                                                                                                         format!("padding: 1px 5px; border-radius: 4px; font-size: 9.5px; font-weight: 500; {}", color)
                                                                                                                     },
                                                                                                                     "{tag}"
                                                                                                                 }
                                                                                                             }
                                                                                                         }
                                                                                                     }
                                                                                        }

                                                                                         // Column 2: Use-case
                                                                                         td { style: "padding: 10px 6px;",
                                                                                             span {
                                                                                                 style: {
                                                                                                     let bg = match m.use_case.as_str() {
                                                                                                         "Text Generation" => "color-mix(in srgb, var(--color-tag-purple) 16%, transparent); color: var(--color-tag-purple);",
                                                                                                         "Text Embedding" => "color-mix(in srgb, var(--color-tag-blue) 16%, transparent); color: var(--color-tag-blue);",
                                                                                                         "OCR" => "color-mix(in srgb, var(--color-tag-yellow) 18%, transparent); color: var(--color-tag-yellow);",
                                                                                                         "ASR" => "color-mix(in srgb, var(--color-tag-green) 16%, transparent); color: var(--color-tag-green);",
                                                                                                         "TTS" => "color-mix(in srgb, var(--color-tag-pink) 16%, transparent); color: var(--color-tag-pink);",
                                                                                                         "Image Generation" => "color-mix(in srgb, var(--color-tag-orange) 18%, transparent); color: var(--color-tag-orange);",
                                                                                                         "Video Generation" => "color-mix(in srgb, var(--color-tag-cyan) 18%, transparent); color: var(--color-tag-cyan);",
                                                                                                         "Coding" => "color-mix(in srgb, var(--color-tag-red) 18%, transparent); color: var(--color-tag-red);",
                                                                                                         _ => "var(--color-overlay-2); color: var(--color-muted);"
                                                                                                     };
                                                                                                     format!("padding: 2px 6px; border-radius: 10px; font-size: 10.5px; font-weight: 500; background: {}", bg)
                                                                                                 },
                                                                                                 {m.use_case.clone()}
                                                                                             }
                                                                                         }

                                                                                         // Column 3: Size
                                                                                         td { style: "padding: 10px 6px;",
                                                                                             div { style: "font-weight: 500; color: var(--color-muted);", {m.size_info.clone()} }
                                                                                         }

                                                                                         // Column 4: Links
                                                                                         td { style: "padding: 10px 6px;",
                                                                                             div { style: "display: flex; gap: 6px;",
                                                                                                 if !m.hf_link.is_empty() {
                                                                                                     a {
                                                                                                         href: "{m.hf_link}",
                                                                                                         target: "_blank",
                                                                                                         style: "color: var(--color-brand-accent); text-decoration: none; display: inline-flex; align-items: center; gap: 2px;",
                                                                                                         "\u{1F917} HF"
                                                                                                     }
                                                                                                 }
                                                                                                 if !m.github_link.is_empty() {
                                                                                                     a {
                                                                                                         href: "{m.github_link}",
                                                                                                         target: "_blank",
                                                                                                         style: "color: var(--color-success); text-decoration: none; display: inline-flex; align-items: center; gap: 2px;",
                                                                                                         "\u{1F431} Git"
                                                                                                     }
                                                                                                 }
                                                                                                 if m.hf_link.is_empty() && m.github_link.is_empty() {
                                                                                                     span { style: "color: var(--color-muted); font-style: italic; font-size: 11px;", "None" }
                                                                                                 }
                                                                                             }
                                                                                         }

                                                                                        // Column 5: Action
                                                                                        td { style: "padding: 10px 6px; text-align: right;",
                                                                                            div { style: "display: flex; gap: 5px; justify-content: flex-end;",
                                                                                                if m.status == "failed" || m.status == "pending_enrichment" {
                                                                                                    button {
                                                                                                        class: "btn-browse",
                                                                                                        style: "font-size: 10.5px; padding: 3px 6px;",
                                                                                                        onclick: {
                                                                                                            let id = *idx;
                                                                                                            move |_| {
                                                                                                                let mut current = scanned_models.read().clone();
                                                                                                                current[id].status = "pending_enrichment".to_string();
                                                                                                                let index_path = get_default_config_path().join("model_index.json");
                            let _ = library::save_index(&index_path.to_string_lossy(), &current);
                                                                                                                scanned_models.set(current);
                                                                                                            }
                                                                                                        },
                                                                                                        "\u{21BB} Enrich"
                                                                                                    }
                                                                                             } else if m.status == "enriching" {
                                                                                                     span { style: "font-size: 10.5px; color: var(--color-warning); font-style: italic;", "Enriching\u{2026}" }
                                                                                             }

                                                                                                button {
                                                                                                    class: "btn btn-ghost btn-sm",
                                                                                                    style: "font-size: 10.5px; padding: 3px 6px;",
                                                                                                    onclick: {
                                                                                                        let path = m.path.clone();
                                                                                                        move |_| {
                                                                                                            config.write().model_path = path.clone();
                                                                                                            error_local.set(Some("Active model updated!".to_string()));
                                                                                                        }
                                                                                                    },
                                                                                                    "\u{2714} Use Model"
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        } else {
            {
                let total = scanned_models.read().len();
                let enriched = scanned_models.read().iter().filter(|m| m.status == "enriched").count();
                let pending = scanned_models.read().iter().filter(|m| m.status == "pending_enrichment").count();
                let failed = scanned_models.read().iter().filter(|m| m.status == "failed").count();
                let enriching = scanned_models.read().iter().filter(|m| m.status == "enriching").count();
                let completed = enriched + failed + enriching;
                let percent = if total > 0 { (completed as f32 / total as f32 * 100.0) as u32 } else { 0 };

                rsx! {
                    div { class: "card",
                        div { class: "card-title", "Enrichment Progress & Stats" }

                        // Progress bar
                        div { style: "margin: 16px 0;",
                            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; font-size: 13px;",
                                span { style: "color: var(--color-muted); font-weight: 500;", "Overall Completion" }
                                span { style: "color: var(--color-primary); font-weight: 700;", "{percent}% ({completed} / {total})" }
                            }
                            div { class: "progress-bar-bg",
                                div { class: "progress-bar-fill", style: "width: {percent}%;" }
                            }
                        }

                        // Counts Grid
                        div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(110px, 1fr)); gap: 12px; margin-bottom: 20px;",
                            div { class: "stat-card",
                                div { class: "stat-card-title", "Total" }
                                div { class: "stat-card-value", "{total}" }
                            }
                            div { class: "stat-card",
                                div { class: "stat-card-title", "Enriched" }
                                div { class: "stat-card-value", style: "color: var(--color-success);", "{enriched}" }
                            }
                            div { class: "stat-card",
                                div { class: "stat-card-title", "Pending" }
                                div { class: "stat-card-value", style: "color: var(--color-warning);", "{pending}" }
                            }
                            div { class: "stat-card",
                                div { class: "stat-card-title", "Failed" }
                                div { class: "stat-card-value", style: "color: var(--color-error);", "{failed}" }
                            }
                            div { class: "stat-card",
                                div { class: "stat-card-title", "Active" }
                                div { class: "stat-card-value", style: "color: var(--color-brand-accent);", "{enriching}" }
                            }
                        }

                        div { style: "border-top: 1px solid var(--color-hairline); padding-top: 16px; display: flex; flex-direction: column; gap: 12px;",
                            div { style: "font-size: 13.5px; font-weight: 600; color: var(--color-ink);", "Background Status" }
                            div { style: "background: var(--color-surface-dark); border: 1px solid var(--color-hairline); padding: 12px; border-radius: 8px; font-family: monospace; font-size: 12px; color: var(--color-on-dark-soft);",
                                "{index_status()}"
                            }
                        }
                    }

                    div { class: "card",
                        div { class: "card-title", "Pending or Failed Models Detail" }
                        {
                            let models_guard = scanned_models.read();
                            let items: Vec<_> = models_guard.iter().enumerate()
                                .filter(|(_, m)| m.status == "pending_enrichment" || m.status == "failed" || m.status == "enriching")
                                .collect();

                            if items.is_empty() {
                                rsx! {
                                    div { class: "log-empty", "No pending or failed models. All models successfully enriched!" }
                                }
                            } else {
                                rsx! {
                                    div { style: "overflow-x: auto;",
                                        table { class: "model-table",
                                            thead {
                                                tr {
                                                    th { "Filename" }
                                                    th { "Family / Version" }
                                                    th { "Status" }
                                                    th { style: "text-align: right;", "Action" }
                                                }
                                            }
                                            tbody {
                                                for &(idx, m) in items.iter() {
                                                    tr { key: "{m.path}",
                                                        td { style: "font-family: monospace; color: var(--color-ink);", "{m.filename}" }
                                                        td { style: "color: var(--color-muted);", "{m.family} - {m.version}" }
                                                        td {
                                                            span {
                                                                style: {
                                                                    let color = match m.status.as_str() {
                                                                        "enriching" => "background: color-mix(in srgb, var(--color-tag-purple) 14%, transparent); color: var(--color-tag-purple); border: 1px solid color-mix(in srgb, var(--color-tag-purple) 30%, transparent);",
                                                                        "failed" => "background: var(--color-badge-error-bg); color: var(--color-badge-error-text); border: 1px solid var(--color-badge-error-border);",
                                                                        _ => "background: var(--color-badge-warning-bg); color: var(--color-badge-warning-text); border: 1px solid var(--color-badge-warning-border);"
                                                                    };
                                                                    format!("padding: 2px 6px; border-radius: 4px; font-size: 10px; font-weight: 600; {}", color)
                                                                },
                                                                "{m.status}"
                                                            }
                                                        }
                                                        td { style: "padding: 10px 6px; text-align: right;",
                                                            if m.status != "enriching" {
                                                                button {
                                                                    class: "btn-browse",
                                                                    style: "font-size: 10.5px; padding: 2px 6px;",
                                                                    onclick: {
                                                                        let id = idx;
                                                                        move |_| {
                                                                            let mut current = scanned_models.read().clone();
                                                                            current[id].status = "pending_enrichment".to_string();
                                                                            let index_path = get_default_config_path().join("model_index.json");
                                                                            let _ = library::save_index(&index_path.to_string_lossy(), &current);
                                                                            scanned_models.set(current);
                                                                        }
                                                                    },
                                                                    "\u{21BB} Retry"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct TargetItem {
    name: String,
    line: String,
}

fn parse_input_targets(input: &str) -> Vec<TargetItem> {
    let mut targets = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.ends_with(':') {
            continue;
        }

        if trimmed.starts_with('-') {
            continue;
        }

        // Parse target type
        let name = if trimmed.ends_with(".git") || trimmed.starts_with("https://github") {
            let mut repo_name = trimmed.split('/').last().unwrap_or("").to_string();
            if repo_name.ends_with(".git") {
                repo_name = repo_name[..repo_name.len() - 4].to_string();
            }
            format!("Git clone: {}", repo_name)
        } else if trimmed.starts_with("http") {
            let parts: Vec<&str> = trimmed.split('/').collect();
            let repo = if parts.len() > 4 { parts[4] } else { "unknown" };
            let filename = parts.last().unwrap_or(&"");

            let target_name = if filename.starts_with("mmproj") {
                let prefix = repo.to_lowercase().replace("-gguf", "");
                format!("{}-{}", prefix, filename)
            } else {
                if let Some(last_dash) = filename.rfind('-') {
                    format!(
                        "{}{}",
                        filename[..last_dash].to_lowercase(),
                        &filename[last_dash..]
                    )
                } else {
                    filename.to_lowercase()
                }
            };
            format!("Aria2c download: {}", target_name)
        } else {
            let repo_name = trimmed.split('/').last().unwrap_or("");
            format!("HF download: {}", repo_name)
        };

        targets.push(TargetItem {
            name,
            line: trimmed.to_string(),
        });
    }
    targets
}

fn parse_tmux_progress(
    lines: &[String],
    targets: &[TargetItem],
) -> (Option<usize>, f32, String, String) {
    fn extract_percentage(line: &str) -> Option<f32> {
        if let Some(pct_pos) = line.find('%') {
            let mut start = pct_pos;
            while start > 0 {
                let prev_char = line[..start].chars().rev().next().unwrap();
                if prev_char.is_ascii_digit() || prev_char == '.' {
                    start -= prev_char.len_utf8();
                } else {
                    break;
                }
            }
            let pct_str = line[start..pct_pos].trim();
            return pct_str.parse::<f32>().ok();
        }
        None
    }

    fn extract_speed(line: &str) -> Option<String> {
        let lower = line.to_lowercase();
        if let Some(dl_idx) = lower.find("dl:") {
            let sub = line[dl_idx + 3..].trim();
            let end = sub.find(']').or_else(|| sub.find(' ')).unwrap_or(sub.len());
            let speed = sub[..end].trim();
            if !speed.is_empty() {
                return Some(speed.to_string());
            }
        }

        for token in line.split_whitespace().rev() {
            let token = token.trim_end_matches(',');
            if token.ends_with("/s") || token.ends_with("b/s") || token.ends_with("B/s") {
                return Some(token.to_string());
            }
        }

        None
    }

    fn extract_eta(line: &str) -> Option<String> {
        let lower = line.to_lowercase();
        if let Some(idx) = lower.find("eta") {
            let after = line[idx + 3..].trim_start_matches(&[':', ' '][..]).trim();
            if !after.is_empty() {
                return Some(
                    after
                        .split_whitespace()
                        .next()
                        .unwrap_or("Unknown")
                        .to_string(),
                );
            }
        }

        if let Some(start) = lower.find("remaining") {
            let after = line[start..].split(':').nth(1).unwrap_or("").trim();
            if !after.is_empty() {
                return Some(
                    after
                        .split_whitespace()
                        .next()
                        .unwrap_or("Unknown")
                        .to_string(),
                );
            }
        }

        None
    }

    let mut active_idx = None;
    let mut percent = 0.0f32;
    let mut speed = "Pending...".to_string();
    let mut eta = "Unknown".to_string();

    'outer: for line in lines.iter().rev() {
        let lower_line = line.to_lowercase();
        for (idx, target) in targets.iter().enumerate().rev() {
            let keyword = target.line.to_lowercase();
            let search_terms = vec![
                keyword.clone(),
                target.name.to_lowercase(),
                target.line.rsplit('/').next().unwrap_or("").to_lowercase(),
            ];

            if search_terms
                .iter()
                .any(|term| !term.is_empty() && lower_line.contains(term))
            {
                active_idx = Some(idx);
                break 'outer;
            }
        }
    }

    for line in lines.iter().rev() {
        if let Some(p) = extract_percentage(line) {
            percent = p;
            if let Some(found_speed) = extract_speed(line) {
                speed = found_speed;
            }
            if let Some(found_eta) = extract_eta(line) {
                eta = found_eta;
            }
            break;
        }

        if line.contains("Receiving objects:") || line.contains("Resolving deltas:") {
            if let Some(p) = extract_percentage(line) {
                percent = p;
            }
            if let Some(found_speed) = extract_speed(line) {
                speed = found_speed;
            }
            eta = "Cloning...".to_string();
            break;
        }
    }

    (active_idx, percent, speed, eta)
}

fn generate_dw_script(input: &str) -> String {
    let mut script = String::new();
    script.push_str("#!/bin/bash\n");
    script.push_str("export PATH=\"$HOME/.local/bin:$PATH\"\n\n");
    script.push_str("# ==========================================\n");
    script.push_str("# TMUX ENFORCEMENT\n");
    script.push_str("# ==========================================\n");
    script.push_str("if [ -z \"$TMUX\" ]; then\n");
    script.push_str("    echo \"Not running in tmux. Starting a background tmux session named 'hf_downloads'...\"\n");
    script.push_str("    SCRIPT_PATH=$(realpath \"$0\")\n");
    script.push_str("    tmux new-session -d -s hf_downloads \"bash \\\"$SCRIPT_PATH\\\"\"\n");
    script.push_str("    echo \"Downloads have started in the background!\"\n");
    script.push_str("    echo \"To view progress, run: tmux attach -t hf_downloads\"\n");
    script.push_str("    exit 0\n");
    script.push_str("fi\n\n");
    script.push_str("# ==========================================\n");
    script.push_str("# DOWNLOAD COMMANDS\n");
    script.push_str("# ==========================================\n");
    script.push_str("MODELS_ROOT=/mnt/modelsext/models\n");
    script.push_str("cd $MODELS_ROOT\n\n");

    let mut current_category = "mtp".to_string();
    let mut current_subfolder: Option<String> = None;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.ends_with(':') {
            current_category = trimmed[..trimmed.len() - 1].trim().to_string();
            current_subfolder = None;
            continue;
        }

        if trimmed.starts_with('-') {
            current_subfolder = Some(trimmed[1..].trim().to_string());
            continue;
        }

        let base_dir = match &current_subfolder {
            Some(sub) => format!("{}/{}", current_category, sub),
            None => current_category.clone(),
        };

        if trimmed.ends_with(".git") || trimmed.starts_with("https://github") {
            let mut repo_name = trimmed.split('/').last().unwrap_or("").to_string();
            if repo_name.ends_with(".git") {
                repo_name = repo_name[..repo_name.len() - 4].to_string();
            }
            script.push_str(&format!(
                "git clone {} ./{}/{}\n",
                trimmed, base_dir, repo_name
            ));
        } else if trimmed.starts_with("http") {
            let parts: Vec<&str> = trimmed.split('/').collect();
            let repo = if parts.len() > 4 { parts[4] } else { "unknown" };
            let filename = parts.last().unwrap_or(&"");

            let target_name = if filename.starts_with("mmproj") {
                let prefix = repo.to_lowercase().replace("-gguf", "");
                format!("{}-{}", prefix, filename)
            } else {
                if let Some(last_dash) = filename.rfind('-') {
                    format!(
                        "{}{}",
                        filename[..last_dash].to_lowercase(),
                        &filename[last_dash..]
                    )
                } else {
                    filename.to_lowercase()
                }
            };

            script.push_str(&format!(
                "aria2c -c -x 16 -s 16 -o ./{}/{} {}\n",
                base_dir, target_name, trimmed
            ));
        } else {
            let repo_name = trimmed.split('/').last().unwrap_or("");
            let target_dir = repo_name.to_lowercase();
            script.push_str(&format!(
                "hf download {} --local-dir ./{}/{}\n",
                trimmed, base_dir, target_dir
            ));
        }
    }

    script.push_str(
        "\n# Keep tmux window open for 10 seconds after completion to see the final logs\n",
    );
    script.push_str("sleep 10\n");
    script
}

#[component]
fn TabDownload(search_target: Signal<String>) -> Element {
    let mut input_text = use_signal(|| {
        std::fs::read_to_string(get_default_config_path().join("t"))
            .or_else(|_| std::fs::read_to_string("/home/notroot/Work/llama-manager/t"))
            .unwrap_or_default()
    });
    let mut tmux_output = use_signal(Vec::<String>::new);
    let mut is_downloading = use_signal(|| false);
    let mut error_local = use_signal(|| None::<String>);
    let mut show_raw_logs = use_signal(|| false);

    let _tmux_poller = use_resource(move || {
        async move {
            loop {
                // Check if session hf_downloads exists
                let has_session = tokio::task::spawn_blocking(move || {
                    std::process::Command::new("tmux")
                        .current_dir(get_default_config_path())
                        .args(&["has-session", "-t", "hf_downloads"])
                        .status()
                        .map(|status| status.success())
                        .unwrap_or(false)
                })
                .await
                .unwrap_or(false);

                is_downloading.set(has_session);

                if has_session {
                    // Capture tmux pane output
                    let output = tokio::task::spawn_blocking(move || {
                        std::process::Command::new("tmux")
                            .current_dir(get_default_config_path())
                            .args(&["capture-pane", "-pt", "hf_downloads"])
                            .output()
                    })
                    .await;

                    if let Ok(Ok(out)) = output {
                        if out.status.success() {
                            let text = String::from_utf8_lossy(&out.stdout);
                            let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
                            tmux_output.set(lines);
                        }
                    }
                } else {
                    tmux_output.set(Vec::new());
                }

                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        }
    });

    let run_generate = move |_| {
        let text_val = input_text.read().clone();
        spawn(async move {
            let config_dir = get_default_config_path();
            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                error_local.set(Some(format!("Failed to create config directory: {}", e)));
                return;
            }

            // Write input text to 't' under config directory
            let t_path = config_dir.join("t");
            if let Err(e) = std::fs::write(&t_path, &text_val) {
                error_local.set(Some(format!("Failed to write file 't': {}", e)));
                return;
            }

            // Generate script natively in Rust
            let script_content = generate_dw_script(&text_val);
            let dw_path = config_dir.join("dw.sh");
            if let Err(e) = std::fs::write(&dw_path, script_content) {
                error_local.set(Some(format!("Failed to write dw.sh: {}", e)));
                return;
            }

            // Make dw.sh executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&dw_path, std::fs::Permissions::from_mode(0o755));
            }

            // Execute dw.sh in background (which will auto-launch inside tmux)
            let run_status = tokio::task::spawn_blocking(move || {
                let dw_path = get_default_config_path().join("dw.sh");
                std::process::Command::new("bash")
                    .current_dir(get_default_config_path())
                    .arg(dw_path)
                    .status()
            })
            .await;

            match run_status {
                Ok(Ok(status)) if status.success() => {
                    error_local.set(None);
                }
                Ok(Ok(status)) => {
                    error_local.set(Some(format!("dw.sh exited with status: {}", status)));
                }
                _ => {
                    error_local.set(Some("Failed to execute dw.sh".to_string()));
                }
            }
        });
    };

    let cancel_download = move |_| {
        spawn(async move {
            let kill_status = tokio::task::spawn_blocking(move || {
                std::process::Command::new("tmux")
                    .current_dir(get_default_config_path())
                    .args(&["kill-session", "-t", "hf_downloads"])
                    .status()
            })
            .await;

            match kill_status {
                Ok(Ok(status)) if status.success() => {
                    error_local.set(None);
                }
                _ => {
                    error_local.set(Some("Failed to stop download".to_string()));
                }
            }
        });
    };

    let targets = parse_input_targets(&input_text());
    let (active_idx, percent, speed, eta) = parse_tmux_progress(&tmux_output(), &targets);

    rsx! {
        div { class: "section-title", "Download Manager" }
        div { class: "section-desc", "Integrated model and repository downloader. Generates and runs download script under a persistent tmux session." }

        if let Some(ref err) = *error_local.read() {
            div { class: "error-toast", style: "margin-bottom: 12px;",
                "\u{26A0}\u{FE0F} "
                {err.clone()}
            }
        }

        div { style: "display: flex; gap: 24px; flex-wrap: wrap;",
            // Left Card: Input configuration editor
            div { class: "card", style: "flex: 1 1 500px; display: flex; flex-direction: column;",
                div { class: "card-title", "Download Targets Input (File 't' format)" }
                div { class: "form-group", style: "flex-grow: 1;",
                    label { r#for: "download-targets-input", class: "form-label", "Edit Targets List" }
                    textarea {
                        id: "download-targets-input",
                        class: "form-textarea",
                        style: "font-family: 'JetBrains Mono', monospace; font-size: 13px; min-height: 380px; flex-grow: 1; width: 100%;",
                        disabled: is_downloading(),
                        value: input_text(),
                        placeholder: "# Example download target file format\n# Category headers create folders under /mnt/modelsext/models\n# Use '-' lines to add a subfolder under the current category\n\ntext:\n- stable\n- llama2\nfacebook/opt-125m\nopenai/whisper-large\n\nmultimodal:\n- vision\nhttps://huggingface.co/your-user/your-model/resolve/main/model.gguf\nhttps://example.com/downloads/other-model.safetensors\n\nrepos:\nhttps://github.com/username/repo.git\nhttps://github.com/another/repo.git\n",
                        oninput: move |e: Event<FormData>| input_text.set(e.value()),
                    }
                }
                div { style: "display: flex; gap: 12px; margin-top: 12px;",
                    button {
                        class: "btn btn-start",
                        disabled: is_downloading(),
                        onclick: run_generate,
                        "\u{2913} Generate & Start Download"
                    }
                    if is_downloading() {
                        button {
                            class: "btn btn-stop",
                            onclick: cancel_download,
                            "\u{23F9} Cancel Download"
                        }
                    }
                }
            }

            // Right Card: Dynamic progress bar monitor
            div { class: "card", style: "flex: 1 1 400px; display: flex; flex-direction: column;",
                div { class: "card-title", "Live Download Monitor" }

                div { style: "display: flex; align-items: center; gap: 8px; margin-bottom: 16px; font-size: 13.5px;",
                    div {
                        class: if is_downloading() { "status-dot running" } else { "status-dot stopped" },
                    }
                    span { style: "font-weight: 600;", if is_downloading() { "Downloading in tmux session 'hf_downloads'" } else { "Idle / Completed" } }
                }

                if is_downloading() {
                    div { class: "form-hint", style: "margin-bottom: 16px;",
                        "Attach: "
                        code { style: "background: var(--color-surface-soft); padding: 2px 6px; border-radius: 4px; font-weight: 600; color: var(--color-brand-accent);", "tmux attach -t hf_downloads" }
                    }
                }

                // Parsed progress items list
                div { style: "display: flex; flex-direction: column; gap: 12px; overflow-y: auto; max-height: 400px; padding-right: 4px;",
                    if targets.is_empty() {
                        div { style: "color: var(--color-muted); font-style: italic; padding: 20px 0; text-align: center;",
                            "No targets defined. Please edit the list on the left."
                        }
                    } else {
                        {targets.iter().enumerate().map(|(idx, target)| {
                            let is_active = is_downloading() && Some(idx) == active_idx;
                            let is_completed = if is_downloading() {
                                match active_idx {
                                    Some(active) => idx < active,
                                    None => false,
                                }
                            } else {
                                !tmux_output.read().is_empty()
                            };

                            rsx! {
                                div {
                                    key: "{idx}",
                                    style: "background: var(--color-surface-soft); border: 1px solid var(--color-hairline); padding: 12px; border-radius: 8px; display: flex; flex-direction: column; gap: 8px;",

                                    div { style: "display: flex; justify-content: space-between; align-items: center; font-size: 13px;",
                                        span { style: "font-weight: 600; color: var(--color-ink); word-break: break-all;", "{target.name}" }
                                        if is_active {
                                            span { style: "color: var(--color-brand-accent); font-weight: 600; white-space: nowrap;", "{percent as u32}%" }
                                        } else if is_completed {
                                            span { style: "color: var(--color-success); font-weight: 600; white-space: nowrap;", "\u{2705} Done" }
                                        } else {
                                            span { style: "color: var(--color-muted); white-space: nowrap;", "\u{23F3} Pending" }
                                        }
                                    }

                                    // Progress Bar
                                    div {
                                        style: "width: 100%; height: 6px; background: var(--color-surface-strong); border-radius: 9999px; overflow: hidden;",
                                        div {
                                            style: if is_active {
                                                "height: 100%; width: {percent}%; background: var(--color-brand-accent); border-radius: 9999px; transition: width 0.3s ease-in-out;"
                                            } else if is_completed {
                                                "height: 100%; width: 100%; background: var(--color-success); border-radius: 9999px;"
                                            } else {
                                                "height: 100%; width: 0%; background: var(--color-surface-strong);"
                                            }
                                        }
                                    }

                                    if is_active {
                                        div { style: "display: flex; justify-content: space-between; font-size: 11px; color: var(--color-muted);",
                                            span { "Speed: {speed}" }
                                            span { "ETA: {eta}" }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }

                // Collapsible Raw Log option
                if is_downloading() {
                    div { style: "margin-top: 16px;",
                        button {
                            class: "btn btn-outline",
                            style: "width: 100%; padding: 6px; font-size: 12px; display: flex; align-items: center; justify-content: center; gap: 6px;",
                            onclick: move |_| { let cur = show_raw_logs(); show_raw_logs.set(!cur); },
                            if show_raw_logs() { "Hide Raw Terminal Logs" } else { "Show Raw Terminal Logs" }
                        }
                    }
                    if show_raw_logs() {
                        div {
                            style: "background: var(--color-surface-dark); color: var(--color-on-dark-soft); font-family: 'JetBrains Mono', monospace; font-size: 11px; padding: 12px; border-radius: 8px; overflow-y: auto; height: 180px; white-space: pre-wrap; word-break: break-all; border: 1px solid var(--color-hairline); line-height: 1.4; text-align: left; margin-top: 8px;",
                            for (i, line) in tmux_output.read().iter().enumerate() {
                                div { key: "{i}", {line.clone()} }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn perform_send(
    config: Signal<ServerConfig>,
    mut messages: Signal<Vec<(String, String, Option<Vec<String>>)>>,
    mut current_input: Signal<String>,
    mut is_generating: Signal<bool>,
    mut chat_error: Signal<Option<String>>,
    routed_instance: Signal<Option<(String, u16)>>,
    selected_chat_model: String,
    use_agent: bool,
) {
    let text = current_input.read().trim().to_string();
    if text.is_empty() || is_generating() {
        return;
    }

    messages
        .write()
        .push(("user".to_string(), text.clone(), None));
    current_input.set(String::new());
    is_generating.set(true);
    chat_error.set(None);

    let (host, port) = if let Some((h, p)) = routed_instance.read().clone() {
        (h, p)
    } else {
        ("127.0.0.1".to_string(), config.read().port)
    };
    let history = messages.read().clone();

    if use_agent {
        spawn(async move {
            let target_model = if !selected_chat_model.is_empty() {
                selected_chat_model
            } else {
                library::get_target_model_with_host(&host, port, "").await
            };
            let mut memory_manager = agent::MemoryManager::new(get_default_config_path());
            let mcp_registry = agent::McpRegistry::load(get_default_config_path());
            let searxng_url = if config.read().searxng_url.is_empty() {
                None
            } else {
                Some(config.read().searxng_url.clone())
            };

            match agent::run_agent_task(
                &host,
                port,
                &target_model,
                &text,
                &mcp_registry,
                &mut memory_manager,
                searxng_url,
            )
            .await
            {
                Ok(resp) => {
                    messages
                        .write()
                        .push(("assistant".to_string(), resp.text, Some(resp.logs)));
                }
                Err(e) => {
                    chat_error.set(Some(format!("Agent execution failed: {}", e)));
                }
            }
            is_generating.set(false);
        });
    } else {
        spawn(async move {
            let target_model = if !selected_chat_model.is_empty() {
                selected_chat_model
            } else {
                library::get_target_model_with_host(&host, port, "").await
            };
            let llm_url = format!("http://{}:{}/v1/chat/completions", host, port);

            let mut req_messages = Vec::new();
            for (role, content, _) in &history {
                req_messages.push(serde_json::json!({
                    "role": role,
                    "content": content
                }));
            }

            let payload = serde_json::json!({
                "model": target_model,
                "messages": req_messages,
                "temperature": 0.7,
            });

            let client = reqwest::Client::new();
            match client.post(&llm_url).json(&payload).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        #[derive(serde::Deserialize, Clone)]
                        struct ChatMessage {
                            content: Option<String>,
                        }
                        #[derive(serde::Deserialize, Clone)]
                        struct ChatChoice {
                            message: Option<ChatMessage>,
                        }
                        #[derive(serde::Deserialize)]
                        struct ChatResponse {
                            choices: Option<Vec<ChatChoice>>,
                        }

                        if let Ok(chat_res) = res.json::<ChatResponse>().await {
                            if let Some(content) = chat_res
                                .choices
                                .and_then(|c| c.first().cloned())
                                .and_then(|c| c.message)
                                .and_then(|m| m.content)
                            {
                                messages
                                    .write()
                                    .push(("assistant".to_string(), content, None));
                            } else {
                                chat_error.set(Some(
                                    "Invalid response format received from model server."
                                        .to_string(),
                                ));
                            }
                        } else {
                            chat_error.set(Some(
                                "Failed to parse JSON response from model server.".to_string(),
                            ));
                        }
                    } else {
                        chat_error.set(Some(format!(
                            "Model server returned error status: {}",
                            res.status()
                        )));
                    }
                }
                Err(e) => {
                    chat_error.set(Some(format!("Failed to connect to model server: {}", e)));
                }
            }
            is_generating.set(false);
        });
    }
}

#[component]
fn TabChat(
    config: Signal<ServerConfig>,
    search_target: Signal<String>,
    mut routed_instance: Signal<Option<(String, u16)>>,
    mut immersive_mode: Signal<bool>,
) -> Element {
    let mut messages = use_signal(Vec::<(String, String, Option<Vec<String>>)>::new);
    let mut use_agent = use_signal(|| false);
    let mut current_input = use_signal(String::new);
    let is_generating = use_signal(|| false);
    let mut chat_error = use_signal(|| None::<String>);
    let mut server_status = use_signal(|| false);
    let mut chat_models = use_signal(Vec::<String>::new);
    let mut selected_chat_model = use_signal(String::new);

    let _server_checker = use_resource(move || async move {
        loop {
            let (host, port) = if let Some((h, p)) = routed_instance.read().clone() {
                (h, p)
            } else {
                ("127.0.0.1".to_string(), config.read().port)
            };
            let client = reqwest::Client::new();
            let url = format!("http://{}:{}/v1/models", host, port);
            let online = match client
                .get(&url)
                .timeout(std::time::Duration::from_secs(1))
                .send()
                .await
            {
                Ok(res) => {
                    if res.status().is_success() {
                        #[derive(serde::Deserialize)]
                        struct ModelInfo {
                            id: String,
                        }
                        #[derive(serde::Deserialize)]
                        struct ModelsList {
                            data: Vec<ModelInfo>,
                        }
                        if let Ok(list) = res.json::<ModelsList>().await {
                            let names: Vec<String> = list.data.into_iter().map(|m| m.id).collect();
                            if !names.contains(&selected_chat_model.read()) {
                                if let Some(first) = names.first() {
                                    selected_chat_model.set(first.clone());
                                } else {
                                    selected_chat_model.set(String::new());
                                }
                            }
                            chat_models.set(names);
                        }
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            };
            server_status.set(online);
            if !online {
                chat_models.set(Vec::new());
                selected_chat_model.set(String::new());
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    let clear_chat = move |_| {
        messages.write().clear();
        chat_error.set(None);
    };
    rsx! {
        if !immersive_mode() {
            div { class: "section-title", "Interactive Model Chat" }
            div { class: "section-desc", "Have live conversations with the loaded model. Start the model server from the Server tab to begin." }
        }

        if let Some((ref h, p)) = *routed_instance.read() {
            if !immersive_mode() {
                div {
                    style: "background: var(--color-brand-accent); color: var(--color-on-primary); padding: 12px; border-radius: 8px; margin-bottom: 16px; font-size: 13.5px; display: flex; align-items: center; justify-content: space-between;",
                    span { "💬 Chat routed to instance: {h}:{p}" }
                    button {
                        class: "btn btn-sm",
                        style: "background: var(--color-overlay-white-3); color: var(--color-on-primary); border: none; padding: 2px 8px; font-size: 11px;",
                        onclick: move |_| routed_instance.set(None),
                        "Reset to Local"
                    }
                }
            }
        }

        if !server_status() {
            if !immersive_mode() {
                div {
                    style: "background: var(--color-warning); color: var(--color-on-primary); padding: 12px; border-radius: 8px; margin-bottom: 16px; font-size: 13.5px; display: flex; align-items: center; justify-content: space-between;",
                    if let Some((ref h, p)) = *routed_instance.read() {
                        span { "⚠️ Routed server {h}:{p} is offline. Please make sure the instance is running or select another instance." }
                    } else {
                        span { "⚠️ Model server is offline. Please make sure a model is configured and click 'Start Server' in the Server tab to chat." }
                    }
                }
            }
        }

        if let Some(ref err) = *chat_error.read() {
            div { class: "error-toast", style: "margin-bottom: 16px;",
                "❌ {err}"
            }
        }

        div {
            style: if immersive_mode() {
                "display: flex; flex-direction: column; flex: 1; min-height: 0; background: var(--color-surface-soft); border: none; border-radius: 0; overflow: hidden; height: 100vh;"
            } else {
                "display: flex; flex-direction: column; flex: 1; min-height: 0; background: var(--color-surface-soft); border: 1px solid var(--color-hairline); border-radius: 12px; overflow: hidden;"
            },

            div {
                style: "display: flex; justify-content: space-between; align-items: center; padding: 8px 16px; background: var(--color-canvas); border-bottom: 1px solid var(--color-hairline);",
                span { style: "font-weight: 600; color: var(--color-ink);", "💬 Chat Session" }
                div { style: "display: flex; align-items: center; gap: 16px;",
                    if !chat_models.read().is_empty() {
                        div { style: "display: flex; align-items: center; gap: 12px;",
                            div { style: "display: flex; align-items: center; gap: 6px;",
                                span { style: "font-size: 12px; color: var(--color-muted); font-weight: 500;", "Model:" }
                                select {
                                    class: "form-select",
                                    style: "width: auto; padding: 2px 24px 2px 8px; font-size: 12px; height: 26px; line-height: 1; border-radius: 4px;",
                                    value: selected_chat_model(),
                                    onchange: move |e: Event<FormData>| {
                                        selected_chat_model.set(e.value());
                                    },
                                    for model in chat_models.read().iter() {
                                        option { value: model.clone(), "{model}" }
                                    }
                                }
                            }
                            label { style: "display: flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 500; color: var(--color-ink); cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: use_agent(),
                                    onchange: move |e: Event<FormData>| {
                                        use_agent.set(e.value() == "true");
                                    }
                                }
                                "🤖 Agent Mode"
                            }
                        }
                    }
                    button {
                        class: "btn btn-ghost btn-sm",
                        style: "padding: 4px 10px; font-size: 12px; display: flex; align-items: center; gap: 4px;",
                        onclick: move |_| immersive_mode.set(!immersive_mode()),
                        if immersive_mode() {
                            span { "✨ Exit Immersive" }
                        } else {
                            span { "✨ Immersive Mode" }
                        }
                    }
                    button {
                        class: "btn btn-ghost btn-sm",
                        style: "padding: 4px 10px; font-size: 12px;",
                        disabled: messages.read().is_empty(),
                        onclick: clear_chat,
                        "Clear Conversation"
                    }
                }
            }

            div {
                style: "flex: 1 1 0; min-height: 0; overflow-y: auto; padding: 16px; display: flex; flex-direction: column; gap: 12px;",
                if messages.read().is_empty() {
                    div {
                        style: "margin: auto; text-align: center; color: var(--color-muted); max-width: 320px;",
                        div { style: "font-size: 32px; margin-bottom: 8px;", "💬" }
                        div { style: "font-weight: 600; margin-bottom: 4px; color: var(--color-ink);", "Start a Conversation" }
                        div { style: "font-size: 12.5px; line-height: 1.4;", "Type a message below to query the running Llama model server in real time." }
                    }
                } else {
                    for (i, (role, text, opt_logs)) in messages.read().iter().enumerate() {
                        div {
                            key: "{i}",
                            style: if role == "user" { "display: flex; justify-content: flex-end;" } else { "display: flex; justify-content: flex-start;" },
                            div {
                                style: if role == "user" {
                                    "background: var(--color-brand-accent); color: var(--color-on-primary); padding: 10px 14px; border-radius: 14px 14px 2px 14px; max-width: 80%; font-size: 13.5px; line-height: 1.4; text-align: left;"
                                } else {
                                    "background: var(--color-canvas); color: var(--color-ink); padding: 10px 14px; border-radius: 14px 14px 14px 2px; max-width: 80%; font-size: 13.5px; line-height: 1.4; text-align: left; border: 1px solid var(--color-hairline);"
                                },
                                div { "{text}" }
                                if let Some(logs) = opt_logs {
                                    div {
                                        style: "margin-top: 8px; border-top: 1px solid var(--color-hairline-soft); padding-top: 6px; font-size: 11px; width: 100%;",
                                        details {
                                            summary { style: "cursor: pointer; font-weight: 600; color: var(--color-brand-accent); outline: none; margin-bottom: 4px;", "🤖 Agent Execution Trace ({logs.len()} steps)" }
                                            div { style: "font-family: monospace; white-space: pre-wrap; padding: 8px; background: var(--color-overlay-1); border: 1px solid var(--color-hairline); border-radius: 4px; max-height: 160px; overflow-y: auto; color: var(--color-muted); font-size: 10.5px; line-height: 1.3;",
                                                for log in logs {
                                                    div { "{log}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if is_generating() {
                    div {
                        style: "display: flex; justify-content: flex-start;",
                        div {
                            style: "background: var(--color-canvas); color: var(--color-muted); padding: 10px 14px; border-radius: 14px; font-size: 13px; font-style: italic; border: 1px solid var(--color-hairline); display: flex; align-items: center; gap: 8px;",
                            span { style: "border: 2px solid var(--color-hairline); border-top: 2px solid var(--color-brand-accent); border-radius: 50%; width: 12px; height: 12px; animation: spin 1s linear infinite;" }
                            "Model is thinking..."
                        }
                    }
                }
            }

            div {
                style: "padding: 12px 16px; background: var(--color-canvas); border-top: 1px solid var(--color-hairline); display: flex; gap: 12px;",
                input {
                    r#type: "text",
                    style: "flex-grow: 1; padding: 8px 12px; border: 1px solid var(--color-hairline); border-radius: 6px; font-size: 13.5px; outline: none; background: var(--color-canvas); color: var(--color-ink);",
                    placeholder: if server_status() { "Type your message and press Enter..." } else { "Please start the server first..." },
                    disabled: !server_status() || is_generating(),
                    value: current_input(),
                    oninput: move |e| current_input.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            perform_send(config, messages, current_input, is_generating, chat_error, routed_instance, selected_chat_model.read().clone(), use_agent());
                        }
                    }
                }
                button {
                    class: "btn btn-primary",
                    style: "padding: 8px 20px;",
                    disabled: !server_status() || is_generating() || current_input.read().trim().is_empty(),
                    onclick: move |_| {
                        perform_send(config, messages, current_input, is_generating, chat_error, routed_instance, selected_chat_model.read().clone(), use_agent());
                    },
                    "Send"
                }
            }
        }
    }
}

// ── Instances Tab ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct LlamaInstance {
    hostname: String,
    port: u16,
    status: String, // "online", "offline", "checking"
    models: Vec<String>,
}

#[component]
fn TabInstances(
    config: Signal<ServerConfig>,
    search_target: Signal<String>,
    mut routed_instance: Signal<Option<(String, u16)>>,
) -> Element {
    let mut detected_instances = use_signal(Vec::<LlamaInstance>::new);
    let mut custom_hostname_input = use_signal(|| "127.0.0.1".to_string());
    let mut custom_port_input = use_signal(String::new);
    let mut last_check_time = use_signal(|| 0i64);
    let mut check_status = use_signal(|| "Idle".to_string());
    let mut refresh_trigger = use_signal(|| 0u64);

    // Common ports to check for llama-server
    let common_ports = vec![8080, 8000, 8001, 8002, 8003, 8004, 5000, 5001, 5002];

    // Async task to periodically detect instances
    let _detector = use_resource(move || {
        let common_ports = common_ports.clone();
        let _ = refresh_trigger();
        async move {
            loop {
                check_status.set("Checking ports...".to_string());
                let mut instances = Vec::new();

                // Check common ports
                for port in &common_ports {
                    let port = *port;
                    let client = reqwest::Client::new();
                    let url = format!("http://127.0.0.1:{}/v1/models", port);

                    match tokio::time::timeout(
                        std::time::Duration::from_millis(500),
                        client.get(&url).send(),
                    )
                    .await
                    {
                        Ok(Ok(response)) => {
                            if response.status().is_success() {
                                if let Ok(body) = response.text().await {
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(&body)
                                    {
                                        let models = json
                                            .get("data")
                                            .and_then(|d| d.as_array())
                                            .map(|arr| {
                                                arr.iter()
                                                    .filter_map(|m| {
                                                        m.get("id")
                                                            .and_then(|id| id.as_str())
                                                            .map(String::from)
                                                    })
                                                    .collect()
                                            })
                                            .unwrap_or_default();

                                        instances.push(LlamaInstance {
                                            hostname: "127.0.0.1".to_string(),
                                            port,
                                            status: "online".to_string(),
                                            models,
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Check custom instances (non-localhost) - use peek() to avoid reactive dependency loops
                for inst in detected_instances.peek().iter() {
                    if !common_ports.contains(&inst.port) {
                        let client = reqwest::Client::new();
                        let url = format!("http://{}:{}/v1/models", inst.hostname, inst.port);

                        match tokio::time::timeout(
                            std::time::Duration::from_millis(500),
                            client.get(&url).send(),
                        )
                        .await
                        {
                            Ok(Ok(response)) => {
                                if response.status().is_success() {
                                    if let Ok(body) = response.text().await {
                                        if let Ok(json) =
                                            serde_json::from_str::<serde_json::Value>(&body)
                                        {
                                            let models = json
                                                .get("data")
                                                .and_then(|d| d.as_array())
                                                .map(|arr| {
                                                    arr.iter()
                                                        .filter_map(|m| {
                                                            m.get("id")
                                                                .and_then(|id| id.as_str())
                                                                .map(String::from)
                                                        })
                                                        .collect()
                                                })
                                                .unwrap_or_default();

                                            instances.push(LlamaInstance {
                                                hostname: inst.hostname.clone(),
                                                port: inst.port,
                                                status: "online".to_string(),
                                                models,
                                            });
                                        }
                                    }
                                }
                            }
                            _ => {
                                instances.push(LlamaInstance {
                                    hostname: inst.hostname.clone(),
                                    port: inst.port,
                                    status: "offline".to_string(),
                                    models: Vec::new(),
                                });
                            }
                        }
                    }
                }

                detected_instances.set(instances);
                *last_check_time.write() = chrono::Local::now().timestamp();
                check_status.set("Ready".to_string());

                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    });

    let add_custom_instance = move |_: Event<MouseData>| {
        if let Ok(port) = custom_port_input().parse::<u16>() {
            let hostname = custom_hostname_input().trim().to_string();
            if !hostname.is_empty() {
                let mut instances = detected_instances.read().clone();
                if !instances
                    .iter()
                    .any(|i| i.hostname == hostname && i.port == port)
                {
                    instances.push(LlamaInstance {
                        hostname,
                        port,
                        status: "checking".to_string(),
                        models: Vec::new(),
                    });
                    detected_instances.set(instances);
                    custom_port_input.set(String::new());
                    refresh_trigger.set(refresh_trigger() + 1);
                }
            }
        }
    };

    let mut remove_instance = move |(hostname, port): (String, u16)| {
        let mut instances = detected_instances.read().clone();
        instances.retain(|i| !(i.hostname == hostname && i.port == port));
        detected_instances.set(instances);
    };

    let manual_refresh = move |_: Event<MouseData>| {
        refresh_trigger.set(refresh_trigger() + 1);
    };

    rsx! {
        div { class: "tab-container",
            // Header
            div { class: "card",
                div { style: "display: flex; justify-content: space-between; align-items: flex-start; gap: 16px; flex-wrap: wrap;",
                    div { style: "flex: 1; min-width: 250px;",
                        div { class: "card-title", "🖥️ Llama-Server Instance Monitor" }
                        div { class: "form-hint", "Discovers running llama-server instances on common ports and displays loaded models per instance." }
                    }
                    div { style: "display: flex; gap: 8px; align-items: center; flex-shrink: 0;",
                        div {
                            style: "font-size: 11.5px; color: var(--color-muted); background: var(--color-surface-soft); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--color-hairline); display: flex; align-items: center; gap: 6px; font-weight: 500;",
                            span { style: "width: 6px; height: 6px; border-radius: 50%; background: var(--color-brand-accent); animation: pulse 2s infinite;" }
                            "{check_status()}"
                        }
                        div {
                            style: "font-size: 11.5px; font-weight: 600; color: var(--color-badge-info-text); background: var(--color-badge-info-bg); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--color-badge-info-border);",
                            "Active: {detected_instances.read().len()}"
                        }
                    }
                }
                if let Some((ref h, p)) = *routed_instance.read() {
                    div {
                        style: "margin-top: 12px; padding: 10px 14px; background: var(--color-brand-accent); color: var(--color-on-primary); border-radius: 4px; font-size: 13px; font-weight: 500; display: flex; align-items: center; justify-content: space-between;",
                        span { "💬 Chat is currently routed to: {h}:{p}" }
                        button {
                            class: "btn btn-sm",
                            style: "background: var(--color-overlay-white-3); color: var(--color-on-primary); border: none; padding: 2px 8px; font-size: 11px;",
                            onclick: move |_| routed_instance.set(None),
                            "Reset to Default Local"
                        }
                    }
                } else {
                    div {
                        style: "margin-top: 12px; padding: 10px 14px; background: var(--color-surface-soft); color: var(--color-muted); border-radius: 4px; font-size: 13px; font-weight: 500; border: 1px solid var(--color-hairline);",
                        "💬 Chat is routed to the default local server configuration (port: {config.read().port})"
                    }
                }
            }

            // Add Custom Instance Card
            div { class: "card",
                div { class: "card-title", "Add Custom Instance" }
                div { class: "form-group",
                    label { r#for: "form-custom-hostname-1", class: "form-label", "Hostname or IP Address" }
                    input {
                        id: "form-custom-hostname-1",
                        autofocus: search_target.read().as_str() == "form-custom-hostname-1",
                        r#type: "text",
                        class: "form-input",
                        placeholder: "e.g., 127.0.0.1, example.com, 192.168.1.100",
                        value: custom_hostname_input(),
                        oninput: move |e: Event<FormData>| {
                            custom_hostname_input.set(e.value());
                        },
                    }
                }
                div { class: "form-group",
                    label { r#for: "form-custom-port-1", class: "form-label", "Port Number" }
                    div { style: "display: flex; gap: 8px;",
                        input {
                            id: "form-custom-port-1",
                            autofocus: search_target.read().as_str() == "form-custom-port-1",
                            r#type: "number",
                            class: "form-input",
                            placeholder: "e.g., 8000",
                            min: "1",
                            max: "65535",
                            value: custom_port_input(),
                            oninput: move |e: Event<FormData>| {
                                custom_port_input.set(e.value());
                            },
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Enter {
                                    if let Ok(port) = custom_port_input().parse::<u16>() {
                                        let hostname = custom_hostname_input().trim().to_string();
                                        if !hostname.is_empty() {
                                            let mut instances = detected_instances.read().clone();
                                            if !instances.iter().any(|i| i.hostname == hostname && i.port == port) {
                                                instances.push(LlamaInstance {
                                                    hostname,
                                                    port,
                                                    status: "checking".to_string(),
                                                    models: Vec::new(),
                                                });
                                                detected_instances.set(instances);
                                                custom_port_input.set(String::new());
                                                refresh_trigger.set(refresh_trigger() + 1);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: add_custom_instance,
                            "Add"
                        }
                        button {
                            class: "btn btn-ghost",
                            onclick: manual_refresh,
                            "🔄 Refresh"
                        }
                    }
                }
                div { class: "form-hint", "Detected instances refresh automatically every 10 seconds. Add remote instances by specifying hostname/IP and port." }
            }

            // Instances List
            if detected_instances.read().is_empty() {
                div { class: "card",
                    div { style: "text-align: center; padding: 24px; color: var(--color-muted); font-size: 13px;",
                        "No instances detected. Make sure llama-server is running on one of the common ports (8080, 8000-8004, 5000-5002) or add a custom instance."
                    }
                }
            } else {
                div { class: "card",
                    div { class: "card-title", "Active Instances" }
                    div { style: "display: flex; flex-direction: column; gap: 12px;",
                        for inst in detected_instances.read().iter() {
                            div { style: "border: 1px solid var(--color-hairline); border-radius: 4px; padding: 12px; background: var(--color-surface-soft); display: flex; flex-direction: column; gap: 10px;",
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    div {
                                        div { style: "font-weight: 600; font-size: 13.5px; color: var(--color-ink);", "{inst.hostname}:{inst.port}" }
                                        div { style: "font-size: 11.5px; margin-top: 2px;",
                                            match inst.status.as_str() {
                                                "online" => rsx! { span { style: "color: var(--color-success); font-weight: 500;", "● Online" } },
                                                "offline" => rsx! { span { style: "color: var(--color-error); font-weight: 500;", "● Offline" } },
                                                _ => rsx! { span { style: "color: var(--color-warning); font-weight: 500;", "● Checking..." } },
                                            }
                                        }
                                    }
                                    {
                                        let is_currently_routed = if let Some((ref h, p)) = *routed_instance.read() {
                                            h == &inst.hostname && p == inst.port
                                        } else {
                                            false
                                        };
                                        rsx! {
                                            div { style: "display: flex; gap: 6px; align-items: center;",
                                                if inst.status == "online" {
                                                    button {
                                                        class: if is_currently_routed { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                                                        onclick: {
                                                            let hostname = inst.hostname.clone();
                                                            let port = inst.port;
                                                            move |_| {
                                                                if is_currently_routed {
                                                                    routed_instance.set(None);
                                                                } else {
                                                                    routed_instance.set(Some((hostname.clone(), port)));
                                                                }
                                                            }
                                                        },
                                                        if is_currently_routed { "💬 Routed" } else { "🔌 Route Chat" }
                                                    }
                                                }
                                                button {
                                                    class: "btn btn-ghost btn-sm",
                                                    onclick: {
                                                        let hostname = inst.hostname.clone();
                                                        let port = inst.port;
                                                        move |_| {
                                                            if is_currently_routed {
                                                                routed_instance.set(None);
                                                            }
                                                            remove_instance((hostname.clone(), port))
                                                        }
                                                    },
                                                    "Remove"
                                                }
                                            }
                                        }
                                    }
                                }
                                if !inst.models.is_empty() {
                                    div { style: "border-top: 1px solid var(--color-hairline); padding-top: 8px;",
                                        div { style: "font-size: 11px; font-weight: 600; color: var(--color-muted); margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.05em;", "Loaded Models" }
                                        div { style: "display: flex; flex-wrap: wrap; gap: 6px;",
                                            for model in inst.models.iter() {
                                                div { style: "font-size: 11.5px; padding: 4px 8px; background: var(--color-canvas); border: 1px solid var(--color-hairline); border-radius: 4px; color: var(--color-body); font-family: monospace;",
                                                    "📦 {model}"
                                                }
                                            }
                                        }
                                    }
                                } else if inst.status == "online" {
                                    div { style: "border-top: 1px solid var(--color-hairline); padding-top: 8px;",
                                        div { style: "font-size: 11.5px; color: var(--color-muted); font-style: italic;", "No models loaded" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TabMcp(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    let base_dir = get_default_config_path();
    let mut mcp_registry = use_signal(move || agent::McpRegistry::load(base_dir.clone()));

    // Form inputs for creating a new tool
    let mut tool_name = use_signal(String::new);
    let mut tool_desc = use_signal(String::new);
    let mut tool_type = use_signal(|| agent::McpType::Local);
    let mut local_cmd = use_signal(String::new);
    let mut local_args = use_signal(String::new);
    let mut remote_url = use_signal(String::new);
    let mut remote_headers = use_signal(String::new);
    let mut ssl_verify_off = use_signal(|| false);
    let mut mcp_error = use_signal(|| None::<String>);

    let add_tool = move |_| {
        let name = tool_name.read().trim().to_string();
        let desc = tool_desc.read().trim().to_string();
        if name.is_empty() || desc.is_empty() {
            mcp_error.set(Some(
                "Tool name and description cannot be empty.".to_string(),
            ));
            return;
        }

        let new_tool = match tool_type() {
            agent::McpType::Local => {
                let cmd = local_cmd.read().trim().to_string();
                if cmd.is_empty() {
                    mcp_error.set(Some("Command is required for local tools.".to_string()));
                    return;
                }
                let args_vec: Vec<String> = local_args
                    .read()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                agent::McpTool {
                    name,
                    description: desc,
                    tool_type: agent::McpType::Local,
                    command: Some(cmd),
                    args: Some(args_vec),
                    url: None,
                    headers: None,
                    ssl_verify_off: false,
                }
            }
            agent::McpType::Remote => {
                let url = remote_url.read().trim().to_string();
                if url.is_empty() {
                    mcp_error.set(Some("URL is required for remote tools.".to_string()));
                    return;
                }
                let mut headers_map = HashMap::new();
                for line in remote_headers.read().lines() {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        headers_map
                            .insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                    }
                }
                agent::McpTool {
                    name,
                    description: desc,
                    tool_type: agent::McpType::Remote,
                    command: None,
                    args: None,
                    url: Some(url),
                    headers: Some(headers_map),
                    ssl_verify_off: ssl_verify_off(),
                }
            }
        };

        let mut reg = mcp_registry.read().clone();
        // Remove tool if already exists with the same name, then add
        reg.tools.retain(|t| t.name != new_tool.name);
        reg.tools.push(new_tool);

        match reg.save(get_default_config_path()) {
            Ok(_) => {
                mcp_registry.set(reg);
                tool_name.set(String::new());
                tool_desc.set(String::new());
                local_cmd.set(String::new());
                local_args.set(String::new());
                remote_url.set(String::new());
                remote_headers.set(String::new());
                ssl_verify_off.set(false);
                mcp_error.set(None);
            }
            Err(e) => {
                mcp_error.set(Some(format!("Failed to save tool: {}", e)));
            }
        }
    };

    let mut remove_tool = move |name: String| {
        let mut reg = mcp_registry.read().clone();
        reg.tools.retain(|t| t.name != name);
        if let Ok(_) = reg.save(get_default_config_path()) {
            mcp_registry.set(reg);
        }
    };

    rsx! {
        div { class: "section-title", "Model Context Protocol (MCP) Tools" }
        div { class: "section-desc", "Connect your LLM to local shell tools and remote web APIs. These tools are available dynamically to the Agent." }

        if let Some(err) = mcp_error() {
            div { class: "error-toast", style: "margin-bottom: 16px;", "❌ {err}" }
        }

        // Add a new Tool Card
        div { class: "card", style: "margin-bottom: 16px;",
            div { class: "card-title", "Add New MCP Tool" }

            div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin-bottom: 12px;",
                div { class: "form-group",
                    label { class: "form-label", "Tool Name" }
                    input {
                        class: "form-input",
                        placeholder: "e.g. read_file",
                        value: tool_name(),
                        oninput: move |e| tool_name.set(e.value())
                    }
                }
                div { class: "form-group",
                    label { class: "form-label", "Description" }
                    input {
                        class: "form-input",
                        placeholder: "e.g. Reads content of a file on disk",
                        value: tool_desc(),
                        oninput: move |e| tool_desc.set(e.value())
                    }
                }
                div { class: "form-group",
                    label { class: "form-label", "Type" }
                    select {
                        class: "form-select",
                        value: if tool_type() == agent::McpType::Local { "local" } else { "remote" },
                        onchange: move |e: Event<FormData>| {
                            if e.value() == "local" {
                                tool_type.set(agent::McpType::Local);
                            } else {
                                tool_type.set(agent::McpType::Remote);
                            }
                        },
                        option { value: "local", "Local Shell Execution" }
                        option { value: "remote", "Remote Web API Endpoint" }
                    }
                }
            }

            {match tool_type() {
                agent::McpType::Local => rsx! {
                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px;",
                        div { class: "form-group",
                            label { class: "form-label", "Command binary" }
                            input {
                                class: "form-input",
                                placeholder: "e.g. cat, grep, python",
                                value: local_cmd(),
                                oninput: move |e| local_cmd.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Arguments (comma separated, use {{param}})" }
                            input {
                                class: "form-input",
                                placeholder: "e.g. -n, {{path}}",
                                value: local_args(),
                                oninput: move |e| local_args.set(e.value())
                            }
                        }
                    }
                },
                agent::McpType::Remote => rsx! {
                    div { style: "display: flex; flex-direction: column; gap: 12px; margin-bottom: 12px;",
                        div { style: "display: grid; grid-template-columns: 1fr auto; gap: 12px; align-items: flex-end;",
                            div { class: "form-group",
                                label { class: "form-label", "URL endpoint (use {{param}})" }
                                input {
                                    class: "form-input",
                                    placeholder: "https://api.github.com/repos/{{owner}}/{{repo}}",
                                    value: remote_url(),
                                    oninput: move |e| remote_url.set(e.value())
                                }
                            }
                            label { style: "display: flex; align-items: center; gap: 6px; font-size: 13px; font-weight: 500; color: var(--color-ink); cursor: pointer; margin-bottom: 12px;",
                                input {
                                    r#type: "checkbox",
                                    checked: ssl_verify_off(),
                                    onchange: move |e: Event<FormData>| {
                                        ssl_verify_off.set(e.value() == "true");
                                    }
                                }
                                "Bypass SSL verification"
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Custom Headers (one Key: Value per line)" }
                            textarea {
                                class: "form-input",
                                style: "font-family: monospace; height: 80px;",
                                placeholder: "Authorization: Bearer <your_token>\nContent-Type: application/json",
                                value: remote_headers(),
                                oninput: move |e| remote_headers.set(e.value())
                            }
                        }
                    }
                }
            }}

            button {
                class: "btn btn-start",
                onclick: add_tool,
                "Register Tool"
            }
        }

        // List Registered Tools
        div { class: "card",
            div { class: "card-title", "Active MCP Tools Registry ({mcp_registry.read().tools.len()})" }

            if mcp_registry.read().tools.is_empty() {
                div { style: "text-align: center; color: var(--color-muted); padding: 24px;", "No custom MCP tools registered yet." }
            } else {
                div { style: "display: flex; flex-direction: column; gap: 12px;",
                    for tool in mcp_registry.read().tools.iter() {
                        div {
                            key: "{tool.name}",
                            style: "display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; padding: 12px; background: var(--color-surface-soft); border: 1px solid var(--color-hairline); border-radius: 6px;",
                            div { style: "display: flex; flex-direction: column; gap: 4px;",
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    span { style: "font-weight: 600; color: var(--color-ink); font-size: 14px;", "{tool.name}" }
                                    span {
                                        style: "font-size: 10px; font-weight: 600; padding: 2px 6px; border-radius: 4px; text-transform: uppercase;",
                                        class: if tool.tool_type == agent::McpType::Local { "status-badge running" } else { "status-badge stopped" },
                                        if tool.tool_type == agent::McpType::Local { "Local Exec" } else { "Remote HTTP" }
                                    }
                                }
                                div { style: "font-size: 12.5px; color: var(--color-body);", "{tool.description}" }
                                {match tool.tool_type {
                                    agent::McpType::Local => rsx! {
                                        div { style: "font-size: 11px; font-family: monospace; color: var(--color-muted); margin-top: 4px;",
                                            "Command: {tool.command.clone().unwrap_or_default()} {tool.args.clone().unwrap_or_default().join(\" \")}"
                                        }
                                    },
                                    agent::McpType::Remote => rsx! {
                                        div { style: "font-size: 11px; font-family: monospace; color: var(--color-muted); margin-top: 4px;",
                                            "Endpoint: {tool.url.clone().unwrap_or_default()}"
                                            if tool.ssl_verify_off {
                                                " [SSL verify: off]"
                                            }
                                        }
                                    }
                                }}
                            }
                            button {
                                class: "btn btn-ghost btn-sm",
                                style: "color: var(--color-error); border-color: var(--color-badge-error-border);",
                                onclick: {
                                    let name = tool.name.clone();
                                    move |_| remove_tool(name.clone())
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct GraphNode {
    id: String,
    title: String,
    scope: String,
    x: f64,
    y: f64,
}

#[derive(Clone, Debug)]
struct GraphEdge {
    source: String,
    target: String,
}

fn compute_node_positions(
    global_mems: &[agent::Memory],
    project_mems: &[agent::Memory],
    width: f64,
    height: f64,
) -> Vec<GraphNode> {
    let mut nodes = Vec::new();
    let mut id_map = std::collections::HashMap::new();

    let total = global_mems.len() + project_mems.len();
    if total == 0 {
        return nodes;
    }

    let mut idx = 0;
    for m in global_mems.iter().chain(project_mems.iter()) {
        id_map.insert(m.id.clone(), idx);

        let angle = idx as f64 * 2.0 * std::f64::consts::PI / total as f64;
        let r = 100.0;
        let x = width / 2.0 + r * angle.cos();
        let y = height / 2.0 + r * angle.sin();

        nodes.push(GraphNode {
            id: m.id.clone(),
            title: m.title.clone(),
            scope: m.scope.clone(),
            x,
            y,
        });
        idx += 1;
    }

    let mut edges = Vec::new();
    for m in global_mems.iter().chain(project_mems.iter()) {
        for link in &m.links {
            if id_map.contains_key(link) {
                edges.push(GraphEdge {
                    source: m.id.clone(),
                    target: link.clone(),
                });
            }
        }
    }

    let iterations = 80;
    let k = 50.0;
    let gravity = 0.04;
    let center_x = width / 2.0;
    let center_y = height / 2.0;

    for _ in 0..iterations {
        let mut dxs = vec![0.0; nodes.len()];
        let mut dys = vec![0.0; nodes.len()];

        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                if i == j {
                    continue;
                }
                let dx = nodes[i].x - nodes[j].x;
                let dy = nodes[i].y - nodes[j].y;
                let dist_sq = dx * dx + dy * dy + 0.1;
                let dist = dist_sq.sqrt();
                if dist < 140.0 {
                    let f = (k * k) / dist;
                    dxs[i] += (dx / dist) * f;
                    dys[i] += (dy / dist) * f;
                }
            }
        }

        for edge in &edges {
            if let (Some(&i), Some(&j)) = (id_map.get(&edge.source), id_map.get(&edge.target)) {
                let dx = nodes[j].x - nodes[i].x;
                let dy = nodes[j].y - nodes[i].y;
                let dist_sq = dx * dx + dy * dy + 0.1;
                let dist = dist_sq.sqrt();

                let f = (dist * dist) / k;
                let force_x = (dx / dist) * f * 0.5;
                let force_y = (dy / dist) * f * 0.5;

                dxs[i] += force_x;
                dys[i] += force_y;
                dxs[j] -= force_x;
                dys[j] -= force_y;
            }
        }

        for i in 0..nodes.len() {
            dxs[i] += (center_x - nodes[i].x) * gravity;
            dys[i] += (center_y - nodes[i].y) * gravity;

            nodes[i].x += dxs[i].clamp(-12.0, 12.0);
            nodes[i].y += dys[i].clamp(-12.0, 12.0);

            nodes[i].x = nodes[i].x.clamp(35.0, width - 35.0);
            nodes[i].y = nodes[i].y.clamp(35.0, height - 35.0);
        }
    }

    nodes
}

#[component]
fn TabAgents(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    let mut memory_manager =
        use_signal(move || agent::MemoryManager::new(get_default_config_path()));
    let mut memory_trigger = use_signal(|| 0);

    // Editor Form States
    let mut selected_mem = use_signal(|| None::<agent::Memory>);
    let mut edit_id = use_signal(String::new);
    let mut edit_title = use_signal(String::new);
    let mut edit_scope = use_signal(|| "project".to_string());
    let mut edit_tags = use_signal(String::new);
    let mut edit_links = use_signal(Vec::<String>::new);
    let mut edit_content = use_signal(String::new);

    // UI filters
    let mut search_query = use_signal(String::new);
    let mut show_new_form = use_signal(|| false);
    let mut hovered_node = use_signal(|| None::<String>);

    let clear_memories = move |_| {
        memory_manager.write().clear_all();
        selected_mem.set(None);
        show_new_form.set(false);
        memory_trigger.write();
    };

    // Load Lists
    let _trigger = memory_trigger.read(); // establish reactive dependency
    let global_mems = memory_manager.read().global_memories.clone();
    let project_mems = memory_manager.read().project_memories.clone();

    let all_memories: Vec<agent::Memory> = global_mems
        .iter()
        .cloned()
        .chain(project_mems.iter().cloned())
        .collect();

    // Filters based on search query
    let query = search_query.read().to_lowercase();
    let filtered_globals: Vec<agent::Memory> = global_mems
        .iter()
        .filter(|m| {
            m.title.to_lowercase().contains(&query)
                || m.tags.iter().any(|t| t.to_lowercase().contains(&query))
        })
        .cloned()
        .collect();
    let filtered_projects: Vec<agent::Memory> = project_mems
        .iter()
        .filter(|m| {
            m.title.to_lowercase().contains(&query)
                || m.tags.iter().any(|t| t.to_lowercase().contains(&query))
        })
        .cloned()
        .collect();

    let selected_id = if show_new_form() {
        edit_id.read().clone()
    } else {
        selected_mem
            .read()
            .as_ref()
            .map_or(String::new(), |m| m.id.clone())
    };

    // Graph variables
    let graph_width = 540.0;
    let graph_height = 420.0;
    let positions = compute_node_positions(&global_mems, &project_mems, graph_width, graph_height);

    let mut edges = Vec::new();
    let mut id_set = std::collections::HashSet::new();
    for n in &positions {
        id_set.insert(n.id.clone());
    }
    for m in &all_memories {
        for l in &m.links {
            if id_set.contains(l) && id_set.contains(&m.id) {
                edges.push(GraphEdge {
                    source: m.id.clone(),
                    target: l.clone(),
                });
            }
        }
    }

    let mut show_gam_simulation = use_signal(|| false);

    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--color-hairline); padding-bottom: 12px; margin-bottom: 20px;",
            div {
                div { class: "section-title", "Agents & Self-Learning Memory" }
                div { class: "section-desc", "Self-learning layers persist project & global memories. Toggle between Obsidian Workspace and GAM memory simulation." }
            }
            div {
                style: "display: flex; border: 1px solid var(--color-hairline); border-radius: 20px; overflow: hidden; background: var(--color-surface-soft); padding: 2px;",
                button {
                    class: if !show_gam_simulation() { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                    style: "border-radius: 18px; margin: 0; padding: 4px 16px; font-weight: 600; display: flex; align-items: center; gap: 6px;",
                    onclick: move |_| show_gam_simulation.set(false),
                    span { "📂" }
                    "Obsidian Workspace"
                }
                button {
                    class: if show_gam_simulation() { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                    style: "border-radius: 18px; margin: 0; padding: 4px 16px; font-weight: 600; display: flex; align-items: center; gap: 6px;",
                    onclick: move |_| show_gam_simulation.set(true),
                    span { "🧠" }
                    "GAM Memory Simulation"
                }
            }
        }

        if show_gam_simulation() {
            GamMemorySimulation { config }
        } else {

        // Clear memory button
        div { style: "display: flex; justify-content: flex-end; margin-bottom: 16px;",
            button {
                class: "btn btn-ghost btn-sm",
                style: "color: var(--color-error); border-color: var(--color-badge-error-border);",
                disabled: global_mems.is_empty() && project_mems.is_empty(),
                onclick: clear_memories,
                "🧹 Clear All Memory Stores"
            }
        }

        // Notion Editor and Node Graph Layout
        div {
            style: "display: flex; gap: 20px; flex-wrap: wrap; margin-bottom: 24px;",

            // Notion-like Document Editor Workspace (60% equivalent)
            div {
                style: "flex: 3; min-width: 500px; display: flex; flex-direction: column;",

                div {
                    style: "display: flex; border: 1px solid var(--color-hairline); border-radius: 8px; overflow: hidden; background: var(--color-canvas); min-height: 520px;",

                    // Notion Sidebar (List of memories)
                    div {
                        style: "width: 200px; border-right: 1px solid var(--color-hairline); background: var(--color-surface-soft); display: flex; flex-direction: column; padding: 12px; gap: 12px;",

                        // Sidebar Header
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center;",
                            span { style: "font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-muted);", "📓 Memories" }
                            button {
                                class: "btn btn-ghost btn-sm",
                                style: "padding: 2px 6px; font-size: 11px; display: flex; align-items: center; gap: 4px;",
                                onclick: move |_| {
                                    let new_id = format!("mem-manual-{}", chrono::Local::now().timestamp_millis());
                                    edit_id.set(new_id);
                                    edit_title.set("Untitled Memory".to_string());
                                    edit_scope.set("project".to_string());
                                    edit_tags.set("".to_string());
                                    edit_links.set(Vec::new());
                                    edit_content.set("".to_string());
                                    show_new_form.set(true);
                                    selected_mem.set(None);
                                },
                                "➕ New"
                            }
                        }

                        // Search Box
                        input {
                            class: "form-input",
                            style: "font-size: 12px; padding: 4px 8px; height: 28px;",
                            placeholder: "🔍 Search memories...",
                            value: search_query(),
                            oninput: move |e| search_query.set(e.value())
                        }

                        // Memories list
                        div {
                            style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 14px;",

                            // Global Memories
                            div {
                                style: "display: flex; flex-direction: column; gap: 4px;",
                                span { style: "font-size: 11px; font-weight: 600; color: var(--color-muted); margin-bottom: 2px;", "🌐 Global Scope" }
                                if filtered_globals.is_empty() {
                                    span { style: "font-size: 11.5px; font-style: italic; color: var(--color-muted); padding-left: 8px;", "No matching global" }
                                } else {
                                    for m in &filtered_globals {
                                        div {
                                            key: "{m.id}",
                                            style: format!("font-size: 12.5px; padding: 6px 8px; border-radius: 4px; cursor: pointer; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: flex; align-items: center; gap: 6px; transition: background 0.15s; {} ",
                                                if selected_id == m.id || (show_new_form() && edit_id() == m.id) { "background: var(--color-accent-soft); color: var(--color-accent); font-weight: 600;" } else { "color: var(--color-body);" }
                                            ),
                                            onclick: {
                                                let m = m.clone();
                                                move |_| {
                                                    edit_id.set(m.id.clone());
                                                    edit_title.set(m.title.clone());
                                                    edit_scope.set(m.scope.clone());
                                                    edit_tags.set(m.tags.join(", "));
                                                    edit_links.set(m.links.clone());
                                                    edit_content.set(m.content.clone());
                                                    selected_mem.set(Some(m.clone()));
                                                    show_new_form.set(false);
                                                }
                                            },
                                            span { "📄" }
                                            "{m.title}"
                                        }
                                    }
                                }
                            }

                            // Project Memories
                            div {
                                style: "display: flex; flex-direction: column; gap: 4px;",
                                span { style: "font-size: 11px; font-weight: 600; color: var(--color-muted); margin-bottom: 2px;", "📂 Project Scope" }
                                if filtered_projects.is_empty() {
                                    span { style: "font-size: 11.5px; font-style: italic; color: var(--color-muted); padding-left: 8px;", "No matching project" }
                                } else {
                                    for m in &filtered_projects {
                                        div {
                                            key: "{m.id}",
                                            style: format!("font-size: 12.5px; padding: 6px 8px; border-radius: 4px; cursor: pointer; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: flex; align-items: center; gap: 6px; transition: background 0.15s; {} ",
                                                if selected_id == m.id || (show_new_form() && edit_id() == m.id) { "background: var(--color-accent-soft); color: var(--color-accent); font-weight: 600;" } else { "color: var(--color-body);" }
                                            ),
                                            onclick: {
                                                let m = m.clone();
                                                move |_| {
                                                    edit_id.set(m.id.clone());
                                                    edit_title.set(m.title.clone());
                                                    edit_scope.set(m.scope.clone());
                                                    edit_tags.set(m.tags.join(", "));
                                                    edit_links.set(m.links.clone());
                                                    edit_content.set(m.content.clone());
                                                    selected_mem.set(Some(m.clone()));
                                                    show_new_form.set(false);
                                                }
                                            },
                                            span { "📄" }
                                            "{m.title}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Notion Editor Workspace Area
                    div {
                        style: "flex: 1; display: flex; flex-direction: column; padding: 24px; background: var(--color-canvas);",

                        if selected_mem.read().is_some() || show_new_form() {
                            // Title Area (Borderless, Large Notion Title)
                            input {
                                style: "width: 100%; border: none; outline: none; font-size: 28px; font-weight: 800; color: var(--color-ink); margin-bottom: 16px; padding: 0; font-family: inherit;",
                                placeholder: "Untitled Memory",
                                value: edit_title(),
                                oninput: move |e| edit_title.set(e.value())
                            }

                            // Meta Properties
                            div {
                                style: "display: grid; grid-template-columns: 80px 1fr; gap: 10px; font-size: 13px; color: var(--color-muted); border-bottom: 1px solid var(--color-hairline); padding-bottom: 16px; margin-bottom: 16px;",

                                // Scope
                                span { style: "display: flex; align-items: center;", "🌐 Scope" }
                                select {
                                    style: "border: 1px solid var(--color-hairline); border-radius: 4px; padding: 3px 6px; max-width: 140px; font-size: 12.5px; outline: none; color: var(--color-body);",
                                    value: edit_scope(),
                                    onchange: move |e| edit_scope.set(e.value()),
                                    option { value: "project", "📂 Project" }
                                    option { value: "global", "🌐 Global" }
                                }

                                // Tags
                                span { style: "display: flex; align-items: center;", "🏷️ Tags" }
                                input {
                                    class: "form-input",
                                    style: "font-size: 12.5px; padding: 4px 8px; max-width: 320px;",
                                    placeholder: "e.g. rust, config (comma separated)",
                                    value: edit_tags(),
                                    oninput: move |e| edit_tags.set(e.value())
                                }

                                // Links
                                span { style: "display: flex; align-items: center;", "🔗 Links" }
                                div {
                                    style: "display: flex; flex-direction: column; gap: 6px;",
                                    if all_memories.len() <= 1 {
                                        span { style: "font-style: italic; font-size: 12px;", "No other memories available to link." }
                                    } else {
                                        div {
                                            style: "border: 1px solid var(--color-hairline); border-radius: 4px; max-height: 100px; overflow-y: auto; padding: 6px; display: flex; flex-direction: column; gap: 4px; max-width: 420px;",
                                            for other in all_memories.iter().filter(|m| m.id != edit_id()) {
                                                label {
                                                    key: "{other.id}",
                                                    style: "display: flex; align-items: center; gap: 6px; font-size: 12px; cursor: pointer; color: var(--color-body);",
                                                    input {
                                                        type: "checkbox",
                                                        checked: edit_links.read().contains(&other.id),
                                                        onchange: {
                                                            let other_id = other.id.clone();
                                                            move |e| {
                                                                let is_checked = e.value() == "true";
                                                                let mut links = edit_links.read().clone();
                                                                if is_checked {
                                                                    if !links.contains(&other_id) {
                                                                        links.push(other_id.clone());
                                                                    }
                                                                } else {
                                                                    links.retain(|l| l != &other_id);
                                                                }
                                                                edit_links.set(links);
                                                            }
                                                        }
                                                    }
                                                    "{other.title}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Markdown editor area
                            textarea {
                                style: "flex: 1; border: none; outline: none; resize: none; font-size: 14px; line-height: 1.6; color: var(--color-ink); background: transparent; font-family: monospace; min-height: 240px; margin-bottom: 16px;",
                                placeholder: "Write details in Markdown here...",
                                value: edit_content(),
                                oninput: move |e| edit_content.set(e.value())
                            }

                            // Save & Cancel Buttons
                            div {
                                style: "display: flex; justify-content: space-between; border-top: 1px solid var(--color-hairline); padding-top: 16px;",

                                if selected_mem.read().is_some() {
                                    button {
                                        class: "btn btn-ghost btn-sm",
                                        style: "color: var(--color-error); border-color: var(--color-badge-error-border);",
                                        onclick: move |_| {
                                            let scope = edit_scope.read().clone();
                                            let id = edit_id.read().clone();
                                            if memory_manager.write().delete_memory(&id, &scope).is_ok() {
                                                selected_mem.set(None);
                                                show_new_form.set(false);
                                                memory_trigger.write();
                                            }
                                        },
                                        "🗑️ Delete"
                                    }
                                } else {
                                    div {}
                                }

                                div { style: "display: flex; gap: 8px;",
                                    button {
                                        class: "btn btn-ghost btn-sm",
                                        onclick: move |_| {
                                            selected_mem.set(None);
                                            show_new_form.set(false);
                                        },
                                        "Cancel"
                                    }
                                    button {
                                        class: "btn btn-start btn-sm",
                                        onclick: move |_| {
                                            let id = edit_id.read().clone();
                                            let title = edit_title.read().trim().to_string();
                                            let scope = edit_scope.read().clone();
                                            let content = edit_content.read().clone();
                                            let tags = edit_tags.read()
                                                .split(',')
                                                .map(|s| s.trim().to_string())
                                                .filter(|s| !s.is_empty())
                                                .collect();
                                            let links = edit_links.read().clone();

                                            let mem = agent::Memory {
                                                id,
                                                title: if title.is_empty() { "Untitled Memory".to_string() } else { title },
                                                created_at: chrono::Local::now().to_rfc3339(),
                                                tags,
                                                links,
                                                scope: scope.clone(),
                                                content: content.clone(),
                                            };

                                            if memory_manager.write().save_memory(mem).is_ok() {
                                                let _ = agent::save_chroma_memory(&content, &scope);
                                                selected_mem.set(None);
                                                show_new_form.set(false);
                                                memory_trigger.write();
                                            }
                                        },
                                        "💾 Save"
                                    }
                                }
                            }
                        } else {
                            div {
                                style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--color-muted); font-size: 13.5px; text-align: center; gap: 12px; padding: 40px;",
                                span { style: "font-size: 48px;", "📓" }
                                p { "Select a document in the sidebar or click a graph node on the right to edit." }
                                button {
                                    class: "btn btn-start btn-sm",
                                    onclick: move |_| {
                                        let new_id = format!("mem-manual-{}", chrono::Local::now().timestamp_millis());
                                        edit_id.set(new_id);
                                        edit_title.set("Untitled Memory".to_string());
                                        edit_scope.set("project".to_string());
                                        edit_tags.set("".to_string());
                                        edit_links.set(Vec::new());
                                        edit_content.set("".to_string());
                                        show_new_form.set(true);
                                        selected_mem.set(None);
                                    },
                                    "➕ Add Memory File"
                                }
                            }
                        }
                    }
                }
            }

            // Obsidian-like Node Graph (40% equivalent)
            div {
                style: "flex: 2; min-width: 320px; display: flex; flex-direction: column;",
                div { class: "card", style: "height: 100%; display: flex; flex-direction: column; padding: 16px;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                        span { class: "card-title", "🕸️ Memory Connections Graph (Obsidian-Style)" }
                        div { style: "display: flex; gap: 8px; font-size: 10px;",
                            span { style: "display: flex; align-items: center; gap: 4px;",
                                span { style: "display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: var(--color-tag-blue);" }
                                "Global"
                            }
                            span { style: "display: flex; align-items: center; gap: 4px;",
                                span { style: "display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: var(--color-tag-purple);" }
                                "Project"
                            }
                        }
                    }

                    if positions.is_empty() {
                        div {
                            style: "flex: 1; display: flex; align-items: center; justify-content: center; border: 1px solid var(--color-hairline); border-radius: 8px; background: var(--color-canvas); font-style: italic; color: var(--color-muted); font-size: 12.5px; height: 420px;",
                            "No memory files to graph. Create some memories to see them visualised!"
                        }
                    } else {
                        div {
                            style: "position: relative; width: 100%; height: 420px;",
                            svg {
                                width: "100%",
                                height: "100%",
                                view_box: "0 0 540 420",
                                style: "background: var(--color-canvas); border-radius: 8px; border: 1px solid var(--color-hairline);",

                                defs {
                                    pattern {
                                        id: "graph-grid",
                                        width: "20",
                                        height: "20",
                                        pattern_units: "userSpaceOnUse",
                                        path {
                                            d: "M 20 0 L 0 0 0 20",
                                            fill: "none",
                                            stroke: "rgba(100, 116, 139, 0.06)",
                                            stroke_width: "1",
                                        }
                                    }
                                }
                                rect {
                                    width: "100%",
                                    height: "100%",
                                    fill: "url(#graph-grid)",
                                }

                                // Draw Edges
                                for edge in &edges {
                                    if let (Some(s_pos), Some(t_pos)) = (positions.iter().find(|n| n.id == edge.source), positions.iter().find(|n| n.id == edge.target)) {
                                        line {
                                            x1: "{s_pos.x}",
                                            y1: "{s_pos.y}",
                                            x2: "{t_pos.x}",
                                            y2: "{t_pos.y}",
                                            stroke: "rgba(148, 163, 184, 0.4)",
                                            stroke_width: "1.5",
                                            stroke_dasharray: if edge.source == selected_id || edge.target == selected_id { "0" } else { "3,3" },
                                            opacity: if edge.source == selected_id || edge.target == selected_id { "0.9" } else { "0.4" },
                                        }
                                    }
                                }

                                // Draw Nodes
                                {positions.iter().map(|node| {
                                    let is_selected = node.id == selected_id;
                                    let is_hovered = hovered_node.read().as_ref() == Some(&node.id);
                                    let fill_color = if node.scope == "global" {
                                        if is_selected { "#4f46e5" } else { "#6366f1" }
                                    } else {
                                        if is_selected { "#7c3aed" } else { "#a78bfa" }
                                    };
                                    let stroke_color = if is_selected {
                                        "#fbbf24"
                                    } else if is_hovered {
                                        "rgba(100, 116, 139, 0.6)"
                                    } else {
                                        "transparent"
                                    };
                                    let radius = if is_selected { 11.0 } else if is_hovered { 9.0 } else { 7.5 };

                                    rsx! {
                                        g {
                                            key: "{node.id}",
                                            onclick: {
                                                let node_id = node.id.clone();
                                                let all_mems = all_memories.clone();
                                                move |_| {
                                                    if let Some(mem) = all_mems.iter().find(|m| m.id == node_id) {
                                                        edit_id.set(mem.id.clone());
                                                        edit_title.set(mem.title.clone());
                                                        edit_scope.set(mem.scope.clone());
                                                        edit_tags.set(mem.tags.join(", "));
                                                        edit_links.set(mem.links.clone());
                                                        edit_content.set(mem.content.clone());
                                                        selected_mem.set(Some(mem.clone()));
                                                        show_new_form.set(false);
                                                    }
                                                }
                                            },
                                            onmouseenter: {
                                                let node_id = node.id.clone();
                                                move |_| hovered_node.set(Some(node_id.clone()))
                                            },
                                            onmouseleave: move |_| hovered_node.set(None),
                                            style: "cursor: pointer;",

                                            circle {
                                                cx: "{node.x}",
                                                cy: "{node.y}",
                                                r: "{radius}",
                                                fill: "{fill_color}",
                                                stroke: "{stroke_color}",
                                                stroke_width: "2.5",
                                                style: "transition: all 0.2s ease;",
                                            }

                                            text {
                                                x: "{node.x}",
                                                y: "{node.y - 14.0}",
                                                text_anchor: "middle",
                                                font_size: "10.5px",
                                                font_weight: if is_selected { "bold" } else { "normal" },
                                                fill: if is_selected { "var(--color-accent)" } else { "var(--color-muted)" },
                                                "{node.title}"
                                            }
                                        }
                                    }
                                })}
                            }

                            // Floating node tooltip
                            if let Some(hover_id) = hovered_node.read().clone() {
                                if let Some(node) = positions.iter().find(|n| n.id == hover_id) {
                                    div {
                                        style: format!("position: absolute; left: {}px; top: {}px; transform: translate(-50%, -100%); background: var(--color-surface-dark-elevated); color: var(--color-on-primary); padding: 4px 8px; border-radius: 4px; font-size: 11px; pointer-events: none; z-index: 10; font-weight: 500; white-space: nowrap;", node.x, node.y - 20.0),
                                        "{node.title} ({node.scope})"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Web Search Integration Settings
        div { class: "card", style: "margin-bottom: 20px;",
            div { class: "card-title", "🌐 Internet Search & Browser Options" }
            div { class: "form-group", style: "max-width: 480px;",
                label { class: "form-label", "SearXNG URL (Optional endpoint, falls back to DuckDuckGo html search)" }
                input {
                    class: "form-input",
                    placeholder: "http://localhost:8888",
                    value: config.read().searxng_url.clone(),
                    oninput: move |e| {
                        let mut new_cfg = config.read().clone();
                        new_cfg.searxng_url = e.value();
                        config.set(new_cfg);
                    }
                }
            }
        }

        // Conformance Card
        div { class: "card",
            div { class: "card-title", "🤖 Conformance & Agent Standards" }
            div { style: "font-size: 13px; line-height: 1.5; color: var(--color-body); display: flex; flex-direction: column; gap: 8px;",
                div { "This agent system strictly conforms to the following specifications:" }
                div {
                    ul { style: "padding-left: 20px; display: flex; flex-direction: column; gap: 4px;",
                        li { "📁 " strong { "AGENTS.md" } " - Dev guidelines, check rules, testing patterns, and PR formatting guidelines." }
                        li { "⚡ " strong { "SKILL.md" } " - Configurable skill structure, tool registry configurations, self-learning loops, and recursive subagents." }
                        li { "🕷️ " strong { "Web-Scraping Layer" } " - Supports DuckDuckGo, SearXNG, Cloakbrowser / Camofox, and browser-harness integrations." }
                    }
                }
            }
        }
        }
    }
}

#[component]
fn TabPlanner(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    let tasks_path = crate::get_default_config_path().join("planner_tasks.json");
    let mut planner_state = use_signal(|| crate::planner::PlannerState::load(tasks_path.clone()));
    let tasks_path_sig = use_signal(|| tasks_path);

    // Form inputs state
    let mut show_create_form = use_signal(|| false);
    let mut new_title = use_signal(String::new);
    let mut new_summary = use_signal(String::new);
    let mut new_agent = use_signal(|| "LlamaManagerAgent".to_string());
    let mut new_priority = use_signal(|| "Medium".to_string());
    let mut new_status = use_signal(|| crate::planner::TaskStatus::Todo);

    // Periodic synchronization
    let _refresh_tasks = use_resource(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let loaded = crate::planner::PlannerState::load(tasks_path_sig.read().clone());
            planner_state.set(loaded);
        }
    });

    let add_task = move |_| {
        let title = new_title.read().trim().to_string();
        let summary = new_summary.read().trim().to_string();
        if title.is_empty() {
            return;
        }

        let new_task = crate::planner::KanbanTask {
            id: format!("task-{}", chrono::Utc::now().timestamp_millis()),
            title,
            summary,
            assigned_agent: new_agent.read().clone(),
            status: new_status.read().clone(),
            priority: new_priority.read().clone(),
            created_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
        };
        let mut state = planner_state.write();
        state.tasks.push(new_task);
        let _ = state.save(tasks_path_sig.read().clone());

        // Reset fields
        new_title.set(String::new());
        new_summary.set(String::new());
        show_create_form.set(false);
    };

    let columns = vec![
        (
            crate::planner::TaskStatus::Backlog,
            "Backlog",
            "📁",
            "border-top: 4px solid var(--color-muted); background: var(--color-overlay-1);",
        ),
        (
            crate::planner::TaskStatus::Todo,
            "To Do",
            "📋",
            "border-top: 4px solid var(--color-tag-blue); background: color-mix(in srgb, var(--color-tag-blue) 8%, transparent);",
        ),
        (
            crate::planner::TaskStatus::InProgress,
            "In Progress",
            "⚡",
            "border-top: 4px solid var(--color-warning); background: var(--color-badge-warning-bg);",
        ),
        (
            crate::planner::TaskStatus::Done,
            "Done",
            "✅",
            "border-top: 4px solid var(--color-success); background: var(--color-badge-success-bg);",
        ),
    ];

    rsx! {
        div { class: "section-title", "Task Planner & Kanban Board" }
        div { class: "section-desc", "Organize autonomous agent execution steps, verify goals, and update work packages." }

        // Expandable task form
        div { class: "card", style: "margin-bottom: 16px;",
            div { style: "display: flex; justify-content: space-between; align-items: center;",
                span { style: "font-size: 13.5px; font-weight: 600; color: var(--color-ink);", "➕ Manage & Create Tasks" }
                button {
                    class: "btn btn-ghost btn-sm",
                    onclick: move |_| show_create_form.set(!show_create_form()),
                    if show_create_form() { "Collapse Form" } else { "Expand Create Task Form" }
                }
            }

            if show_create_form() {
                div { style: "display: flex; flex-direction: column; gap: 12px; margin-top: 12px; border-top: 1px solid var(--color-hairline); padding-top: 12px;",
                    div { class: "form-group",
                        label { class: "form-label", "Task Title *" }
                        input {
                            class: "form-input",
                            placeholder: "Optimize inference performance...",
                            value: new_title(),
                            oninput: move |e| new_title.set(e.value())
                        }
                    }
                    div { class: "form-group",
                        label { class: "form-label", "Summary / Description" }
                        input {
                            class: "form-input",
                            placeholder: "Analyze micro-batch size and layer distribution for better throughput.",
                            value: new_summary(),
                            oninput: move |e| new_summary.set(e.value())
                        }
                    }
                    div { style: "display: flex; gap: 16px; flex-wrap: wrap;",
                        div { class: "form-group", style: "flex: 1; min-width: 150px;",
                            label { class: "form-label", "Assigned Agent" }
                            select {
                                class: "form-select",
                                value: new_agent(),
                                onchange: move |e: Event<FormData>| new_agent.set(e.value()),
                                option { value: "LlamaManagerAgent", "🤖 LlamaManagerAgent" }
                                option { value: "ConfigOptimizer", "⚙️ ConfigOptimizer" }
                                option { value: "WebResearcher", "🌐 WebResearcher" }
                                option { value: "CodeAuditor", "💻 CodeAuditor" }
                            }
                        }
                        div { class: "form-group", style: "flex: 1; min-width: 150px;",
                            label { class: "form-label", "Priority" }
                            select {
                                class: "form-select",
                                value: new_priority(),
                                onchange: move |e: Event<FormData>| new_priority.set(e.value()),
                                option { value: "Low", "Low" }
                                option { value: "Medium", "Medium" }
                                option { value: "High", "High" }
                            }
                        }
                        div { class: "form-group", style: "flex: 1; min-width: 150px;",
                            label { class: "form-label", "Initial Column" }
                            select {
                                class: "form-select",
                                value: new_status().as_str(),
                                onchange: move |e: Event<FormData>| {
                                    let val = e.value();
                                    let status = match val.as_str() {
                                        "Backlog" => crate::planner::TaskStatus::Backlog,
                                        "Todo" => crate::planner::TaskStatus::Todo,
                                        "In Progress" => crate::planner::TaskStatus::InProgress,
                                        "Done" => crate::planner::TaskStatus::Done,
                                        _ => crate::planner::TaskStatus::Todo,
                                    };
                                    new_status.set(status);
                                },
                                option { value: "Backlog", "Backlog" }
                                option { value: "Todo", "Todo" }
                                option { value: "In Progress", "In Progress" }
                                option { value: "Done", "Done" }
                            }
                        }
                    }
                    div { style: "display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px;",
                        button {
                            class: "btn btn-ghost btn-sm",
                            onclick: move |_| {
                                new_title.set(String::new());
                                new_summary.set(String::new());
                                show_create_form.set(false);
                            },
                            "Cancel"
                        }
                        button {
                            class: "btn btn-start btn-sm",
                            disabled: new_title.read().trim().is_empty(),
                            onclick: add_task,
                            "Create Kanban Task"
                        }
                    }
                }
            }
        }

        // Kanban Board Grid
        div {
            style: "display: flex; gap: 16px; overflow-x: auto; flex-grow: 1; min-height: 500px; padding-bottom: 24px; align-items: flex-start;",
            for (col_status, col_name, col_emoji, col_style) in columns {
                div {
                    key: "{col_name}",
                    style: "flex: 1; min-width: 250px; background: var(--color-canvas); border-radius: 8px; border: 1px solid var(--color-hairline); display: flex; flex-direction: column; box-shadow: 0 1px 3px rgba(0,0,0,0.02);",

                    // Column Header
                    div {
                        style: "padding: 10px 14px; font-weight: 600; font-size: 13px; border-bottom: 1px solid var(--color-hairline); border-radius: 8px 8px 0 0; display: flex; justify-content: space-between; align-items: center; {col_style} color: var(--color-ink);",
                        span { "{col_emoji} {col_name}" }
                        span {
                            style: "background: var(--color-overlay-2); padding: 1px 6px; border-radius: 10px; font-size: 11px;",
                            "{planner_state.read().tasks.iter().filter(|t| t.status == col_status).count()}"
                        }
                    }

                    // Column Task Cards
                    div {
                        style: "padding: 12px; display: flex; flex-direction: column; gap: 10px; min-height: 200px; max-height: calc(100vh - 250px); overflow-y: auto;",

                        if !planner_state.read().tasks.iter().any(|t| t.status == col_status) {
                            div {
                                style: "margin: auto; text-align: center; color: var(--color-muted-soft); font-size: 11px; padding: 24px 0;",
                                "No tasks in this column"
                            }
                        } else {
                            for task in planner_state.read().tasks.iter().filter(|t| t.status == col_status).cloned() {
                                div {
                                    key: "{task.id}",
                                    style: "background: var(--color-canvas); border: 1px solid var(--color-hairline); border-radius: 6px; padding: 10px; display: flex; flex-direction: column; gap: 6px; transition: transform 0.15s, box-shadow 0.15s; cursor: default;",

                                    // Card Badges
                                    div {
                                        style: "display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 4px;",
                                        span {
                                            style: match task.priority.as_str() {
                                                "High" => "background: var(--color-badge-error-bg); color: var(--color-badge-error-text); padding: 2px 5px; border-radius: 3px; font-size: 10px; font-weight: 600;",
                                                "Medium" => "background: var(--color-badge-warning-bg); color: var(--color-badge-warning-text); padding: 2px 5px; border-radius: 3px; font-size: 10px; font-weight: 600;",
                                                _ => "background: var(--color-badge-neutral-bg); color: var(--color-badge-neutral-text); padding: 2px 5px; border-radius: 3px; font-size: 10px; font-weight: 600;",
                                            },
                                            "{task.priority}"
                                        }
                                        span {
                                            style: "background: var(--color-badge-info-bg); color: var(--color-badge-info-text); padding: 2px 5px; border-radius: 3px; font-size: 10px; font-weight: 600; display: flex; align-items: center; gap: 2px;",
                                            "🤖 {task.assigned_agent}"
                                        }
                                    }

                                    // Task Title & Summary
                                    div {
                                        style: "font-size: 12.5px; font-weight: 600; color: var(--color-ink); line-height: 1.3;",
                                        "{task.title}"
                                    }
                                    if !task.summary.is_empty() {
                                        div {
                                            style: "font-size: 11px; color: var(--color-body); line-height: 1.35; max-height: 48px; overflow: hidden; text-overflow: ellipsis;",
                                            "{task.summary}"
                                        }
                                    }

                                    // Date
                                    div {
                                        style: "font-size: 9.5px; color: var(--color-muted); display: flex; justify-content: space-between; align-items: center; margin-top: 4px; border-top: 1px solid var(--color-overlay-1); padding-top: 4px;",
                                        span { "Created: {task.created_at}" }
                                    }

                                    // Controls / Movement and deletion
                                    div {
                                        style: "display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-top: 4px;",

                                        select {
                                            class: "form-select",
                                            style: "font-size: 10.5px; padding: 1px 4px; height: 20px; width: auto; background: var(--color-canvas);",
                                            value: task.status.as_str(),
                                            onchange: {
                                                let task_id = task.id.clone();
                                                move |e: Event<FormData>| {
                                                    let new_status_str = e.value();
                                                    let new_status = match new_status_str.as_str() {
                                                        "Backlog" => crate::planner::TaskStatus::Backlog,
                                                        "Todo" => crate::planner::TaskStatus::Todo,
                                                        "In Progress" => crate::planner::TaskStatus::InProgress,
                                                        "Done" => crate::planner::TaskStatus::Done,
                                                        _ => crate::planner::TaskStatus::Todo,
                                                    };
                                                    let mut state = planner_state.write();
                                                    if let Some(t) = state.tasks.iter_mut().find(|t| t.id == task_id) {
                                                        t.status = new_status;
                                                    }
                                                    let _ = state.save(tasks_path_sig.read().clone());
                                                }
                                            },
                                            option { value: "Backlog", "Backlog" }
                                            option { value: "Todo", "Todo" }
                                            option { value: "In Progress", "In Progress" }
                                            option { value: "Done", "Done" }
                                        }

                                        button {
                                            class: "btn btn-ghost btn-sm",
                                            style: "padding: 1px 4px; font-size: 10px; color: var(--color-error); border: 1px solid var(--color-badge-error-border); background: var(--color-badge-error-bg); height: 20px; display: flex; align-items: center;",
                                            onclick: {
                                                let task_id = task.id.clone();
                                                move |_| {
                                                    let mut state = planner_state.write();
                                                    state.tasks.retain(|t| t.id != task_id);
                                                    let _ = state.save(tasks_path_sig.read().clone());
                                                }
                                            },
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TabMonitor(config: Signal<ServerConfig>, search_target: Signal<String>) -> Element {
    let monitor_path = crate::get_default_config_path().join("agent_activities.json");
    let mut monitor_state = use_signal(|| crate::planner::MonitorState::load(monitor_path.clone()));

    // Auto refresh resource
    let refresh_path = monitor_path.clone();
    let _refresh_monitor = use_resource(move || {
        let monitor_path = refresh_path.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let loaded = crate::planner::MonitorState::load(monitor_path.clone());
                monitor_state.set(loaded);
            }
        }
    });

    let clear_activities = {
        let monitor_path = monitor_path.clone();
        move |_| {
            let mut state = monitor_state.write();
            state.events.clear();
            let _ = state.save(monitor_path.clone());
        }
    };

    rsx! {
        div { class: "section-title", "Live Agent Monitor" }
        div { class: "section-desc", "Inspect real-time agent execution traces, current workflows, and subprocess event streams." }

        div {
            style: "display: flex; gap: 16px; flex-wrap: wrap; align-items: flex-start;",

            // Left Column: Agents Registry / Status List
            div {
                style: "flex: 1; min-width: 300px; display: flex; flex-direction: column; gap: 16px;",
                div { class: "card",
                    div { class: "card-title", "Agent Subprocesses" }

                    if monitor_state.read().agents.is_empty() {
                        div { style: "text-align: center; color: var(--color-muted); font-style: italic; font-size: 12.5px; padding: 20px;",
                            "No active or registered agents found."
                        }
                    } else {
                        div {
                            style: "display: flex; flex-direction: column; gap: 10px;",
                            for agent in monitor_state.read().agents.iter() {
                                div {
                                    key: "{agent.name}",
                                    style: "border: 1px solid var(--color-hairline); border-radius: 6px; padding: 10px; background: var(--color-canvas); display: flex; align-items: center; justify-content: space-between; gap: 12px;",
                                    div { style: "display: flex; flex-direction: column; gap: 2px; flex: 1;",
                                        div { style: "font-size: 13px; font-weight: 600; color: var(--color-ink); display: flex; align-items: center; gap: 6px;",
                                            span { "🤖" }
                                            span { "{agent.name}" }
                                        }
                                        div { style: "font-size: 11px; color: var(--color-body); word-break: break-all;",
                                            strong { "Current Action: " }
                                            "{agent.current_action}"
                                        }
                                        div { style: "font-size: 10px; color: var(--color-muted-soft);",
                                            "Last active: {agent.last_active}"
                                        }
                                    }
                                    span {
                                        style: match agent.status.as_str() {
                                            "Active" => "background: var(--color-badge-success-bg); color: var(--color-badge-success-text); padding: 2px 6px; border-radius: 4px; font-size: 10.5px; font-weight: 600; white-space: nowrap;",
                                            "Idle" => "background: var(--color-badge-warning-bg); color: var(--color-badge-warning-text); padding: 2px 6px; border-radius: 4px; font-size: 10.5px; font-weight: 600; white-space: nowrap;",
                                            _ => "background: var(--color-badge-neutral-bg); color: var(--color-badge-neutral-text); padding: 2px 6px; border-radius: 4px; font-size: 10.5px; font-weight: 600; white-space: nowrap;",
                                        },
                                        "{agent.status}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Right Column: Live Logs & Events
            div {
                style: "flex: 2; min-width: 400px; display: flex; flex-direction: column; gap: 16px;",
                div { class: "card",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        div { class: "card-title", "Live Event Feed" }
                        button {
                            class: "btn btn-ghost btn-sm",
                            style: "color: var(--color-error); border: 1px solid var(--color-badge-error-border); background: var(--color-badge-error-bg);",
                            onclick: clear_activities,
                            "Clear Feed"
                        }
                    }

                    if monitor_state.read().events.is_empty() {
                        div { style: "text-align: center; color: var(--color-muted); font-style: italic; font-size: 12.5px; padding: 40px;",
                            "No events logged yet. Trigger agent execution to view real-time traces."
                        }
                    } else {
                        div {
                            style: "display: flex; flex-direction: column; gap: 8px; max-height: 450px; overflow-y: auto; font-family: 'JetBrains Mono', monospace; font-size: 11.5px; line-height: 1.4; border: 1px solid var(--color-hairline); border-radius: 6px; padding: 12px; background: var(--color-surface-dark); color: var(--color-on-dark-soft);",

                            // Render from latest to oldest
                            for event in monitor_state.read().events.iter().rev() {
                                div {
                                    key: "{event.timestamp}-{event.agent_name}",
                                    style: "display: flex; gap: 8px; border-bottom: 1px solid var(--color-overlay-white-1); padding: 4px 0;",
                                    span { style: "color: var(--color-muted); font-weight: 500; min-width: 65px;", "[{event.timestamp}]" }
                                    span { style: "color: var(--color-tag-cyan); font-weight: 600; min-width: 120px;", "🤖 {event.agent_name}:" }
                                    span { style: "color: var(--color-ink); flex: 1; word-break: break-all;", "{event.message}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Calendar View ────────────────────────────────────────────────────────────

#[component]
fn TabCalendar(config: Signal<ServerConfig>) -> Element {
    let title = use_signal(String::new);
    let datetime = use_signal(String::new);
    let prompt = use_signal(String::new);
    let note = use_signal(String::new);
    let color_tag = use_signal(|| "#6366f1".to_string());
    let state = use_signal(|| calendar::CalendarState::load());
    let mut expanded_event = use_signal(|| None::<String>);
    let calendar_view = use_signal(|| "month".to_string()); // "month" | "list"

    // Current calendar month/year
    let today = chrono::Local::now();
    let view_year = use_signal(|| today.year());
    let view_month = use_signal(|| today.month()); // 1-12

    let add_event = move |_| {
        let mut state = state;
        let mut title = title;
        let mut datetime = datetime;
        let mut prompt = prompt;
        let mut note = note;
        let t = title.read().trim().to_string();
        let dt = datetime.read().trim().to_string();
        let p = prompt.read().trim().to_string();
        if t.is_empty() || dt.is_empty() || p.is_empty() {
            return;
        }

        let dt_normalized = match calendar::parse_datetime(&dt) {
            Ok(parsed) => parsed.format("%Y-%m-%dT%H:%M:%S").to_string(),
            Err(e) => {
                log_error!("[CALENDAR] Datetime validation failed: {}", e);
                return;
            }
        };

        let id = format!("cal-{}", chrono::Local::now().timestamp_millis());
        let n_val = note.read().trim().to_string();
        let c_val = color_tag.read().clone();
        let ev = calendar::CalendarEvent {
            id,
            title: t,
            time: dt_normalized,
            prompt: p,
            note: if n_val.is_empty() { None } else { Some(n_val) },
            color: Some(c_val),
            status: calendar::EventStatus::Pending,
            result: None,
        };
        if calendar::CalendarState::add_event(ev).is_ok() {
            state.set(calendar::CalendarState::load());
            title.set(String::new());
            datetime.set(String::new());
            prompt.set(String::new());
            note.set(String::new());
        }
    };

    let delete_event = move |id: String| {
        let mut state = state;
        if calendar::CalendarState::delete_event(&id).is_ok() {
            state.set(calendar::CalendarState::load());
        }
    };

    let _refresh = use_resource(move || {
        let mut state = state;
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                state.set(calendar::CalendarState::load());
            }
        }
    });

    // Helper: days in month
    let days_in_month = |year: i32, month: u32| -> u32 {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
            .pred_opt()
            .unwrap()
            .day()
    };

    let year = view_year();
    let month = view_month();
    let days_count = days_in_month(year, month);
    // weekday of first day (0=Mon … 6=Sun)
    let first_weekday = chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .map(|d| d.weekday().num_days_from_monday() as usize)
        .unwrap_or(0);

    let month_names = ["", "January", "February", "March", "April", "May", "June",
                       "July", "August", "September", "October", "November", "December"];
    let month_label = month_names.get(month as usize).copied().unwrap_or("");

    // Events for this month
    let month_prefix = format!("{}-{:02}", year, month);
    let month_events: Vec<calendar::CalendarEvent> = state.read().events.iter()
        .filter(|e| e.time.starts_with(&month_prefix))
        .cloned()
        .collect();

    // Get events for a specific day
    let events_for_day = |day: u32| -> Vec<calendar::CalendarEvent> {
        let day_str = format!("{}-{:02}-{:02}", year, month, day);
        month_events.iter().filter(|e| e.time.starts_with(&day_str)).cloned().collect()
    };

    rsx! {
        div { class: "section-title", "Calendar & AI Triggers" }
        div { class: "section-desc", "Schedule events with prompt triggers that fire autonomously. The AI can also schedule events." }

        // View toggle
        div {
            style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; flex-wrap: wrap; gap: 12px;",
            div {
                style: "display: flex; border: 1px solid var(--color-hairline); border-radius: 20px; overflow: hidden; background: var(--color-surface-soft); padding: 2px;",
                button {
                    class: if calendar_view() == "month" { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                    style: "border-radius: 18px;",
                    onclick: move |_| { let mut cv = calendar_view; cv.set("month".to_string()); },
                    "\u{1F4C5} Month View"
                }
                button {
                    class: if calendar_view() == "list" { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                    style: "border-radius: 18px;",
                    onclick: move |_| { let mut cv = calendar_view; cv.set("list".to_string()); },
                    "\u{1F4CB} List View"
                }
            }
            div { style: "font-size: 12.5px; color: var(--color-muted);",
                "{state.read().events.len()} event(s) total"
            }
        }

        div {
            style: "display: flex; gap: 20px; flex-wrap: wrap; align-items: flex-start;",

            // LEFT: Calendar Grid or List + Schedule Form
            div {
                style: "flex: 3; min-width: 400px; display: flex; flex-direction: column; gap: 16px;",

                if calendar_view() == "month" {
                    // Month Calendar Grid
                    div { class: "card", style: "padding: 16px;",
                        // Month navigation header
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                            button {
                                class: "btn btn-ghost btn-sm",
                                onclick: move |_| {
                                    let mut view_month = view_month;
                                    let mut view_year = view_year;
                                    let m = view_month();
                                    let y = view_year();
                                    if m == 1 { view_month.set(12); view_year.set(y - 1); }
                                    else { view_month.set(m - 1); }
                                },
                                "\u{2039} Prev"
                            }
                            div {
                                style: "font-size: 16px; font-weight: 700; color: var(--color-ink);",
                                "{month_label} {year}"
                            }
                            button {
                                class: "btn btn-ghost btn-sm",
                                onclick: move |_| {
                                    let mut view_month = view_month;
                                    let mut view_year = view_year;
                                    let m = view_month();
                                    let y = view_year();
                                    if m == 12 { view_month.set(1); view_year.set(y + 1); }
                                    else { view_month.set(m + 1); }
                                },
                                "Next \u{203a}"
                            }
                        }

                        // Weekday headers
                        div {
                            style: "display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; margin-bottom: 8px;",
                            for day_name in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                                div {
                                    style: "text-align: center; font-size: 11px; font-weight: 700; color: var(--color-muted); text-transform: uppercase; letter-spacing: 0.05em; padding: 4px;",
                                    {day_name}
                                }
                            }
                        }

                        // Day cells
                        div {
                            style: "display: grid; grid-template-columns: repeat(7, 1fr); gap: 4px;",

                            // Empty cells before first day
                            for _ in 0..first_weekday {
                                div { style: "min-height: 70px; border-radius: 6px;" }
                            }

                            // Actual day cells
                            for day in 1..=days_count {
                                {
                                    let day_events = events_for_day(day);
                                    let is_today = today.day() == day && today.month() == month && today.year() == year;
                                    rsx! {
                                        div {
                                            key: "{day}",
                                            style: format!("min-height: 70px; border-radius: 6px; padding: 4px; border: 1px solid {}; background: {}; transition: background 0.15s;",
                                                if is_today { "var(--color-brand-accent)" } else { "var(--color-hairline)" },
                                                if is_today { "color-mix(in srgb, var(--color-brand-accent) 12%, transparent)" } else { "var(--color-canvas)" }
                                            ),

                                            div {
                                                style: format!("font-size: 12px; font-weight: {}; color: {}; margin-bottom: 3px;",
                                                    if is_today { "800" } else { "500" },
                                                    if is_today { "var(--color-brand-accent)" } else { "var(--color-ink)" }
                                                ),
                                                "{day}"
                                            }

                                            // Event pills
                                            div {
                                                style: "display: flex; flex-direction: column; gap: 2px; overflow: hidden;",
                                                for ev in day_events.iter().take(3) {
                                                    div {
                                                        key: "{ev.id}",
                                                        title: "{ev.title}",
                                                        style: format!("font-size: 10px; padding: 2px 5px; border-radius: 3px; color: white; background: {}; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; cursor: pointer;",
                                                            ev.color.as_deref().unwrap_or("#6366f1")
                                                        ),
                                                        onclick: {
                                                            let id = ev.id.clone();
                                                            let mut expanded_event = expanded_event;
                                                            move |_| {
                                                                if expanded_event.read().as_ref() == Some(&id) {
                                                                    expanded_event.set(None);
                                                                } else {
                                                                    expanded_event.set(Some(id.clone()));
                                                                }
                                                            }
                                                        },
                                                        "{ev.title}"
                                                    }
                                                }
                                                if day_events.len() > 3 {
                                                    div { style: "font-size: 9px; color: var(--color-muted); padding-left: 2px;",
                                                        { format!("+{} more", day_events.len() - 3) }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Expanded event detail
                    if let Some(ref eid) = *expanded_event.read() {
                        if let Some(ev) = state.read().events.iter().find(|e| &e.id == eid).cloned() {
                            div { class: "card", style: format!("border-left: 4px solid {};", ev.color.as_deref().unwrap_or("#6366f1")),
                                div { style: "display: flex; justify-content: space-between; align-items: flex-start;",
                                    div {
                                        div { style: "font-weight: 700; font-size: 15px; color: var(--color-ink);", "{ev.title}" }
                                        div { style: "font-size: 12px; color: var(--color-muted); margin-top: 2px;", "\u{1F551} {ev.time}" }
                                        if let Some(ref n) = ev.note {
                                            div { style: "margin-top: 8px; font-size: 13px; color: var(--color-body); font-style: italic; background: var(--color-surface-soft); padding: 8px; border-radius: 6px;",
                                                "\u{1F4CC} {n}"
                                            }
                                        }
                                    }
                                    div { style: "display: flex; gap: 8px;",
                                        span {
                                            style: match ev.status {
                                                calendar::EventStatus::Completed => "background: var(--color-badge-success-bg); color: var(--color-badge-success-text); padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600;",
                                                calendar::EventStatus::Firing => "background: var(--color-badge-info-bg); color: var(--color-badge-info-text); padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600;",
                                                calendar::EventStatus::Failed => "background: var(--color-badge-error-bg); color: var(--color-badge-error-text); padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600;",
                                                calendar::EventStatus::Pending => "background: var(--color-badge-warning-bg); color: var(--color-badge-warning-text); padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600;",
                                            },
                                            "{ev.status}"
                                        }
                                        button {
                                            class: "btn btn-ghost btn-sm",
                                            style: "color: var(--color-error); border-color: var(--color-badge-error-border);",
                                            onclick: {
                                                let id = ev.id.clone();
                                                let delete_event = delete_event.clone();
                                                move |_| { expanded_event.set(None); delete_event(id.clone()); }
                                            },
                                            "Delete"
                                        }
                                    }
                                }

                                div { style: "margin-top: 10px; padding: 8px; background: var(--color-surface-card); border-radius: 6px; font-size: 12.5px;",
                                    strong { "Prompt: " }
                                    span { style: "color: var(--color-body);", "{ev.prompt}" }
                                }

                                if let Some(ref res) = ev.result {
                                    pre { style: "margin-top: 8px; background: var(--color-surface-dark); color: var(--color-on-dark-soft); font-family: monospace; font-size: 11.5px; padding: 10px; border-radius: 4px; overflow-x: auto; white-space: pre-wrap; word-break: break-all; max-height: 200px; overflow-y: auto;",
                                        "{res}"
                                    }
                                }
                            }
                        }
                    }

                } else {
                    // List view of all events
                    div { class: "card",
                        div { class: "card-title", "All Scheduled Events" }

                        if state.read().events.is_empty() {
                            div { style: "text-align: center; color: var(--color-muted); font-style: italic; padding: 40px;",
                                "No events scheduled yet."
                            }
                        } else {
                            div { style: "display: flex; flex-direction: column; gap: 12px;",
                                for ev in state.read().events.iter() {
                                    div {
                                        key: "{ev.id}",
                                        style: format!("border-left: 4px solid {}; border: 1px solid var(--color-hairline); border-left: 4px solid {}; border-radius: 6px; padding: 12px; background: var(--color-canvas);",
                                            ev.color.as_deref().unwrap_or("#6366f1"),
                                            ev.color.as_deref().unwrap_or("#6366f1")
                                        ),

                                        div {
                                            style: "display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 8px;",
                                            div {
                                                div { style: "font-weight: 600; color: var(--color-ink); font-size: 14px;", "{ev.title}" }
                                                div { style: "font-size: 11.5px; color: var(--color-muted); margin-top: 2px;",
                                                    "Trigger: {ev.time}"
                                                }
                                                if let Some(ref n) = ev.note {
                                                    div { style: "font-size: 12px; color: var(--color-body); font-style: italic; margin-top: 4px;",
                                                        "\u{1F4CC} {n}"
                                                    }
                                                }
                                            }
                                            div { style: "display: flex; align-items: center; gap: 8px; flex-shrink: 0;",
                                                span {
                                                    style: match ev.status {
                                                        calendar::EventStatus::Completed => "background: var(--color-badge-success-bg); color: var(--color-badge-success-text); padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600;",
                                                        calendar::EventStatus::Firing => "background: var(--color-badge-info-bg); color: var(--color-badge-info-text); padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600;",
                                                        calendar::EventStatus::Failed => "background: var(--color-badge-error-bg); color: var(--color-badge-error-text); padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600;",
                                                        calendar::EventStatus::Pending => "background: var(--color-badge-warning-bg); color: var(--color-badge-warning-text); padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600;",
                                                    },
                                                    "{ev.status}"
                                                }
                                                button {
                                                    class: "btn btn-ghost btn-sm",
                                                    style: "color: var(--color-error); border-color: var(--color-badge-error-border);",
                                                    onclick: {
                                                        let id = ev.id.clone();
                                                        let delete_event = delete_event.clone();
                                                        move |_| delete_event(id.clone())
                                                    },
                                                    "Delete"
                                                }
                                            }
                                        }

                                        div {
                                            style: "font-size: 12.5px; background: var(--color-surface-card); border: 1px solid var(--color-hairline); border-radius: 4px; padding: 8px; margin-bottom: 6px; color: var(--color-body); word-break: break-all;",
                                            strong { "Prompt: " }
                                            "{ev.prompt}"
                                        }

                                        if let Some(ref res) = ev.result {
                                            div {
                                                style: "margin-top: 10px;",
                                                button {
                                                    class: "btn btn-ghost btn-sm",
                                                    onclick: {
                                                        let id = ev.id.clone();
                                                        let expanded_event = expanded_event;
                                                        move |_| {
                                                            let mut expanded_event = expanded_event;
                                                            if expanded_event.read().as_ref() == Some(&id) {
                                                                expanded_event.set(None);
                                                            } else {
                                                                expanded_event.set(Some(id.clone()));
                                                            }
                                                        }
                                                    },
                                                    if expanded_event.read().as_ref() == Some(&ev.id) { "Hide Output" } else { "Show Output" }
                                                }

                                                if expanded_event.read().as_ref() == Some(&ev.id) {
                                                    pre {
                                                        style: "margin-top: 8px; background: var(--color-surface-dark); color: var(--color-on-dark-soft); font-family: monospace; font-size: 11.5px; padding: 10px; border-radius: 4px; overflow-x: auto; white-space: pre-wrap; word-break: break-all; max-height: 250px; overflow-y: auto;",
                                                        "{res}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // RIGHT: Schedule Form
            div {
                style: "flex: 1; min-width: 280px; display: flex; flex-direction: column; gap: 16px;",
                div { class: "card",
                    div { class: "card-title", "Schedule Trigger Event" }

                    div { class: "form-group",
                        label { class: "form-label", "Event Title" }
                        input {
                            class: "form-input",
                            placeholder: "e.g., Daily optimization...",
                            value: title(),
                            oninput: move |e| { let mut title = title; title.set(e.value()); },
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Trigger Time (YYYY-MM-DDTHH:MM:SS)" }
                        input {
                            class: "form-input",
                            placeholder: "e.g., 2026-06-03T23:00:00",
                            value: datetime(),
                            oninput: move |e| { let mut datetime = datetime; datetime.set(e.value()); },
                        }
                        div { class: "form-hint", "Local time. Checked every 5 seconds." }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Event Color" }
                        div { style: "display: flex; gap: 8px; align-items: center;",
                            input {
                                class: "form-input",
                                r#type: "color",
                                style: "width: 48px; height: 36px; padding: 2px;",
                                value: color_tag(),
                                oninput: move |e| { let mut color_tag = color_tag; color_tag.set(e.value()); },
                            }
                            for color in ["#6366f1", "#10b981", "#f59e0b", "#ef4444", "#06b6d4", "#ec4899"] {
                                div {
                                    key: "{color}",
                                    style: format!("width: 24px; height: 24px; border-radius: 50%; background: {}; cursor: pointer; border: 2px solid {}; flex-shrink: 0;",
                                        color,
                                        if color_tag() == color { "white" } else { "transparent" }
                                    ),
                                    onclick: {
                                        let c = color.to_string();
                                        move |_| { let mut color_tag = color_tag; color_tag.set(c.clone()); }
                                    },
                                }
                            }
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Pin / Note (Optional)" }
                        input {
                            class: "form-input",
                            placeholder: "e.g., Remember to check the GPU logs first...",
                            value: note(),
                            oninput: move |e| { let mut note = note; note.set(e.value()); },
                        }
                        div { class: "form-hint", "Shown as a pin on the calendar day cell." }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Agent Prompt to Fire" }
                        textarea {
                            class: "form-input",
                            style: "height: 80px; font-family: inherit;",
                            placeholder: "e.g., Optimize my local server GPU layers...",
                            value: prompt(),
                            oninput: move |e| { let mut prompt = prompt; prompt.set(e.value()); },
                        }
                    }

                    button {
                        class: "btn btn-start",
                        onclick: add_event,
                        "\u{1F4C5} Schedule Event"
                    }
                }
            }
        }
    }
}

// ── Todos View ───────────────────────────────────────────────────────────────

#[component]
fn TabTodos() -> Element {
    let todo_text = use_signal(String::new);
    let todo_scope = use_signal(|| "project".to_string());

    let project_todos = use_signal(|| todo::TodoList::load_project(get_default_config_path()));
    let global_todos = use_signal(|| todo::TodoList::load_global());

    let add_todo = move |_| {
        let mut project_todos = project_todos;
        let mut global_todos = global_todos;
        let mut todo_text = todo_text;
        let text = todo_text.read().trim().to_string();
        if text.is_empty() {
            return;
        }

        let scope_str = todo_scope.read().clone();
        let scope = if scope_str == "global" {
            todo::TodoScope::Global
        } else {
            todo::TodoScope::Project
        };
        let now = chrono::Local::now().to_rfc3339();
        let id = format!("todo-{}", chrono::Local::now().timestamp_millis());

        let entry = todo::TodoEntry {
            id,
            text,
            scope,
            status: todo::TodoStatus::Pending,
            created_at: now,
        };
        if todo::TodoList::add_todo(entry, get_default_config_path()).is_ok() {
            global_todos.set(todo::TodoList::load_global());
            project_todos.set(todo::TodoList::load_project(get_default_config_path()));
        }
        todo_text.set(String::new());
    };

    let complete_todo = move |id: String, scope: todo::TodoScope| {
        let mut project_todos = project_todos;
        let mut global_todos = global_todos;
        if todo::TodoList::complete_todo(&id, scope, get_default_config_path()).is_ok() {
            global_todos.set(todo::TodoList::load_global());
            project_todos.set(todo::TodoList::load_project(get_default_config_path()));
        }
    };

    let delete_todo = move |id: String, scope: todo::TodoScope| {
        let mut project_todos = project_todos;
        let mut global_todos = global_todos;
        if todo::TodoList::delete_todo(&id, scope, get_default_config_path()).is_ok() {
            global_todos.set(todo::TodoList::load_global());
            project_todos.set(todo::TodoList::load_project(get_default_config_path()));
        }
    };

    let _refresh = use_resource(move || {
        let mut project_todos = project_todos;
        let mut global_todos = global_todos;
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                project_todos.set(todo::TodoList::load_project(get_default_config_path()));
                global_todos.set(todo::TodoList::load_global());
            }
        }
    });

    rsx! {
        div { class: "section-title", "Todo List (Project & Global)" }
        div { class: "section-desc", "Manage tasks across scopes. The AI agent retrieves these tasks and can add or complete them automatically." }

        div { class: "card",
            style: "margin-bottom: 20px;",
            div { class: "card-title", "Add Todo Item" }
            div {
                style: "display: flex; gap: 12px; flex-wrap: wrap; align-items: center;",
                input {
                    class: "form-input",
                    style: "flex: 3; min-width: 250px;",
                    placeholder: "e.g., Rewrite models API endpoint in rust...",
                    value: todo_text(),
                    oninput: move |e| {
                        let mut todo_text = todo_text;
                        todo_text.set(e.value());
                    },
                }
                select {
                    class: "form-input",
                    style: "flex: 1; min-width: 120px;",
                    value: todo_scope(),
                    onchange: move |e| {
                        let mut todo_scope = todo_scope;
                        todo_scope.set(e.value());
                    },
                    option { value: "project", "Project Scope" }
                    option { value: "global", "Global Scope" }
                }
                button {
                    class: "btn btn-start",
                    style: "height: 38px;",
                    onclick: add_todo,
                    "Add Todo"
                }
            }
        }

        div {
            style: "display: flex; gap: 20px; flex-wrap: wrap;",

            // Project Scope Card
            div {
                style: "flex: 1; min-width: 300px;",
                div { class: "card",
                    div { class: "card-title", "Project-wide Todos" }

                    if project_todos.read().items.is_empty() {
                        div { style: "text-align: center; color: var(--color-muted); font-style: italic; padding: 25px;",
                            "No project todos."
                        }
                    } else {
                        div {
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            for item in project_todos.read().items.iter() {
                                div {
                                    key: "{item.id}",
                                    style: "display: flex; align-items: center; justify-content: space-between; border: 1px solid var(--color-hairline); padding: 8px 12px; border-radius: 6px; background: var(--color-canvas);",

                                    div {
                                        style: "display: flex; flex-direction: column; gap: 2px;",
                                        span {
                                            style: if item.status == todo::TodoStatus::Completed { "text-decoration: line-through; color: var(--color-muted);" } else { "color: var(--color-ink);" },
                                            "{item.text}"
                                        }
                                        span { style: "font-size: 10px; color: var(--color-muted-soft);", "ID: {item.id}" }
                                    }

                                    div { style: "display: flex; gap: 6px;",
                                        if item.status == todo::TodoStatus::Pending {
                                            button {
                                                class: "btn btn-start btn-sm",
                                                style: "padding: 2px 6px;",
                                                onclick: {
                                                    let id = item.id.clone();
                                                    let scope = item.scope.clone();
                                                    let complete_todo = complete_todo.clone();
                                                    move |_| complete_todo(id.clone(), scope)
                                                },
                                                "\u{2714} Done"
                                            }
                                        }
                                        button {
                                            class: "btn btn-ghost btn-sm",
                                            style: "color: var(--color-error); border-color: var(--color-badge-error-border); padding: 2px 6px;",
                                            onclick: {
                                                let id = item.id.clone();
                                                let scope = item.scope.clone();
                                                let delete_todo = delete_todo.clone();
                                                move |_| delete_todo(id.clone(), scope)
                                            },
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Global Scope Card
            div {
                style: "flex: 1; min-width: 300px;",
                div { class: "card",
                    div { class: "card-title", "Global Todos" }

                    if global_todos.read().items.is_empty() {
                        div { style: "text-align: center; color: var(--color-muted); font-style: italic; padding: 25px;",
                            "No global todos."
                        }
                    } else {
                        div {
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            for item in global_todos.read().items.iter() {
                                div {
                                    key: "{item.id}",
                                    style: "display: flex; align-items: center; justify-content: space-between; border: 1px solid var(--color-hairline); padding: 8px 12px; border-radius: 6px; background: var(--color-canvas);",

                                    div {
                                        style: "display: flex; flex-direction: column; gap: 2px;",
                                        span {
                                            style: if item.status == todo::TodoStatus::Completed { "text-decoration: line-through; color: var(--color-muted);" } else { "color: var(--color-ink);" },
                                            "{item.text}"
                                        }
                                        span { style: "font-size: 10px; color: var(--color-muted-soft);", "ID: {item.id}" }
                                    }

                                    div { style: "display: flex; gap: 6px;",
                                        if item.status == todo::TodoStatus::Pending {
                                            button {
                                                class: "btn btn-start btn-sm",
                                                style: "padding: 2px 6px;",
                                                onclick: {
                                                    let id = item.id.clone();
                                                    let scope = item.scope.clone();
                                                    let complete_todo = complete_todo.clone();
                                                    move |_| complete_todo(id.clone(), scope)
                                                },
                                                "\u{2714} Done"
                                            }
                                        }
                                        button {
                                            class: "btn btn-ghost btn-sm",
                                            style: "color: var(--color-error); border-color: var(--color-badge-error-border); padding: 2px 6px;",
                                            onclick: {
                                                let id = item.id.clone();
                                                let scope = item.scope.clone();
                                                let delete_todo = delete_todo.clone();
                                                move |_| delete_todo(id.clone(), scope)
                                            },
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Quick Notes View ─────────────────────────────────────────────────────────

#[component]
fn TabQuickNotes() -> Element {
    let scope = use_signal(|| "project".to_string());
    let mut store = use_signal(|| notes::NotesStore::load("project", get_default_config_path()));
    let mut active_note_idx = use_signal(|| 0usize);
    let mut last_saved = use_signal(|| "Not saved yet during this session".to_string());
    let mut new_note_name = use_signal(String::new);
    let mut show_rename = use_signal(|| None::<usize>);
    let mut rename_value = use_signal(String::new);

    let load_scope = move |s: String| {
        let mut store = store;
        let mut active_note_idx = active_note_idx;
        store.set(notes::NotesStore::load(&s, get_default_config_path()));
        active_note_idx.set(0);
    };

    let save_current = move |_| {
        let sc = scope.read().clone();
        let st = store.read().clone();
        match st.save(&sc, get_default_config_path()) {
            Ok(_) => {
                let now = chrono::Local::now().format("%H:%M:%S").to_string();
                last_saved.set(format!("Saved at {}", now));
            }
            Err(e) => {
                last_saved.set(format!("Error: {}", e));
            }
        }
    };

    // Auto-refresh notes every 3s to pick up AI edits
    let _refresh = use_resource(move || {
        let scope = scope;
        let mut store = store;
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let fresh = notes::NotesStore::load(&scope.read(), get_default_config_path());
                if *store.read() != fresh {
                    store.set(fresh);
                }
            }
        }
    });

    let notes_list = store.read().notes.clone();
    let note_count = notes_list.len();
    let safe_idx = if active_note_idx() >= note_count && note_count > 0 {
        note_count - 1
    } else {
        active_note_idx()
    };
    let current_content = notes_list.get(safe_idx).map(|n| n.content.clone()).unwrap_or_default();

    rsx! {
        div { class: "section-title", "Quick Notes" }
        div { class: "section-desc", "Multiple named notes per scope. The AI can read and edit these files to store persistent context." }

        // Scope selector
        div {
            style: "display: flex; gap: 8px; margin-bottom: 16px;",
            button {
                class: if scope() == "project" { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                onclick: {
                    let load_scope = load_scope.clone();
                    let mut scope = scope;
                    move |_| { scope.set("project".to_string()); load_scope("project".to_string()); }
                },
                "📂 Project Notes"
            }
            button {
                class: if scope() == "global" { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                onclick: {
                    let load_scope = load_scope.clone();
                    let mut scope = scope;
                    move |_| { scope.set("global".to_string()); load_scope("global".to_string()); }
                },
                "🌐 Global Notes"
            }
        }

        div {
            style: "display: flex; gap: 16px; align-items: flex-start;",

            // Notes sidebar panel
            div {
                class: "card",
                style: "width: 200px; flex-shrink: 0; padding: 12px; display: flex; flex-direction: column; gap: 8px;",

                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;",
                    span { class: "card-title", "Notes" }
                    button {
                        class: "btn btn-ghost btn-sm",
                        style: "padding: 2px 8px; font-size: 11px;",
                        title: "Add a new note",
                        onclick: move |_| {
                            let name = if new_note_name.read().trim().is_empty() {
                                format!("Note {}", store.read().notes.len() + 1)
                            } else {
                                new_note_name.read().trim().to_string()
                            };
                            let new_idx = store.write().add_note(name);
                            active_note_idx.set(new_idx);
                            new_note_name.set(String::new());
                            let sc = scope.read().clone();
                            let st = store.read().clone();
                            let _ = st.save(&sc, get_default_config_path());
                        },
                        "➕"
                    }
                }

                // New note name input
                input {
                    class: "form-input",
                    style: "font-size: 11.5px; padding: 4px 8px; height: 28px;",
                    placeholder: "New note name...",
                    value: new_note_name(),
                    oninput: move |e| new_note_name.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            let name = if new_note_name.read().trim().is_empty() {
                                format!("Note {}", store.read().notes.len() + 1)
                            } else {
                                new_note_name.read().trim().to_string()
                            };
                            let new_idx = store.write().add_note(name);
                            active_note_idx.set(new_idx);
                            new_note_name.set(String::new());
                            let sc = scope.read().clone();
                            let st = store.read().clone();
                            let _ = st.save(&sc, get_default_config_path());
                        }
                    }
                }

                // List of notes
                div {
                    style: "display: flex; flex-direction: column; gap: 4px; overflow-y: auto; max-height: 400px;",
                    for (idx, note) in notes_list.iter().enumerate() {
                        div {
                            key: "{idx}",
                            style: format!("display: flex; align-items: center; justify-content: space-between; padding: 6px 8px; border-radius: 6px; cursor: pointer; transition: background 0.15s; {}",
                                if idx == safe_idx { "background: color-mix(in srgb, var(--color-brand-accent) 18%, transparent); color: var(--color-brand-accent); font-weight: 600;" } else { "color: var(--color-body);" }
                            ),
                            onclick: {
                                let mut active_note_idx = active_note_idx;
                                move |_| active_note_idx.set(idx)
                            },

                            if show_rename.read().as_ref() == Some(&idx) {
                                input {
                                    class: "form-input",
                                    style: "font-size: 11.5px; padding: 2px 4px; height: 22px; flex: 1;",
                                    value: rename_value(),
                                    autofocus: true,
                                    oninput: move |e| rename_value.set(e.value()),
                                    onkeydown: move |e| {
                                        if e.key() == Key::Enter {
                                            let new_name = rename_value.read().trim().to_string();
                                            if !new_name.is_empty() {
                                                store.write().rename_note(idx, new_name);
                                                let sc = scope.read().clone();
                                                let st = store.read().clone();
                                                let _ = st.save(&sc, get_default_config_path());
                                            }
                                            show_rename.set(None);
                                        } else if e.key() == Key::Escape {
                                            show_rename.set(None);
                                        }
                                    },
                                    onblur: move |_| show_rename.set(None),
                                }
                            } else {
                                span {
                                    style: "font-size: 12.5px; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                    ondoubleclick: {
                                        let note_name = note.name.clone();
                                        move |_| {
                                            rename_value.set(note_name.clone());
                                            show_rename.set(Some(idx));
                                        }
                                    },
                                    "📝 {note.name}"
                                }
                            }

                            if note_count > 1 {
                                button {
                                    style: "background: none; border: none; cursor: pointer; color: var(--color-muted); font-size: 12px; padding: 0 2px; opacity: 0.6; flex-shrink: 0;",
                                    title: "Delete this note",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        store.write().delete_note(idx);
                                        let new_idx = if safe_idx >= store.read().notes.len() { store.read().notes.len().saturating_sub(1) } else { safe_idx };
                                        active_note_idx.set(new_idx);
                                        let sc = scope.read().clone();
                                        let st = store.read().clone();
                                        let _ = st.save(&sc, get_default_config_path());
                                    },
                                    "🗑"
                                }
                            }
                        }
                    }
                }
            }

            // Note editor
            div { class: "card", style: "flex: 1; display: flex; flex-direction: column; gap: 12px; min-width: 0;",
                div { style: "display: flex; justify-content: space-between; align-items: center;",
                    div { style: "display: flex; flex-direction: column; gap: 2px;",
                        div { class: "card-title",
                            {notes_list.get(safe_idx).map(|n| n.name.clone()).unwrap_or_else(|| "Note".into())}
                        }
                        div { class: "form-hint",
                            "File: "
                            if scope() == "global" { "~/.local/share/llama-manager/global_notes.json" } else { "./.llama-manager-notes.json" }
                        }
                    }
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        span { style: "font-size: 11px; color: var(--color-muted); font-style: italic;", "{last_saved}" }
                        button {
                            class: "btn btn-start",
                            onclick: save_current,
                            "💾 Save"
                        }
                    }
                }

                textarea {
                    class: "form-textarea",
                    style: "height: 420px; font-family: 'JetBrains Mono', monospace; font-size: 13px; line-height: 1.6; padding: 14px; width: 100%; resize: vertical;",
                    placeholder: "Start writing here... (double-click note name in the list to rename)",
                    value: current_content,
                    oninput: move |e| {
                        let idx = safe_idx;
                        if idx < store.read().notes.len() {
                            store.write().notes[idx].content = e.value();
                        }
                    },
                }
            }
        }
    }
}

// ── Compare View (Blind Evaluations + llama-benchy Benchmarking) ──────────────

#[component]
fn TabCompare(config: Signal<ServerConfig>) -> Element {
    // ── Blind Evaluation ─────────────────────────────────────────────────────
    let prompt = use_signal(String::new);
    let model_a_url = use_signal(|| "http://127.0.0.1:8080/v1".to_string());
    let model_a_name = use_signal(|| "local-model".to_string());
    let model_b_url = use_signal(|| "http://127.0.0.1:11434/v1".to_string());
    let model_b_name = use_signal(|| "ollama-model".to_string());
    let loading = use_signal(|| false);
    let alpha_response = use_signal(String::new);
    let beta_response = use_signal(String::new);
    let error = use_signal(|| None::<String>);
    let alpha_is_a = use_signal(|| true);
    let mut revealed = use_signal(|| false);
    let mut vote_result = use_signal(String::new);
    let mut votes_log = use_signal(Vec::<String>::new);

    // ── llama-benchy Performance Benchmarking ─────────────────────────────────
    let bench_mode = use_signal(|| "blind".to_string());
    let bench_url = use_signal(|| "http://127.0.0.1:8080".to_string());
    let bench_model = use_signal(|| "local-model".to_string());
    let bench_pp = use_signal(|| "512 1024".to_string());
    let bench_tg = use_signal(|| "128 256".to_string());
    let bench_runs = use_signal(|| "3".to_string());
    let bench_running = use_signal(|| false);
    let bench_output = use_signal(String::new);
    let bench_error = use_signal(|| None::<String>);
    let bench_results = use_signal(Vec::<(String, String, String, String)>::new);

    let run_bench = move |_| {
        let url = bench_url.read().trim().to_string();
        let model = bench_model.read().trim().to_string();
        let pp_str = bench_pp.read().trim().to_string();
        let tg_str = bench_tg.read().trim().to_string();
        let runs = bench_runs.read().trim().to_string();

        if url.is_empty() || model.is_empty() {
            let mut bench_error = bench_error;
            bench_error.set(Some("Please specify a base URL and model name.".to_string()));
            return;
        }

        let mut bench_running = bench_running;
        let mut bench_output = bench_output;
        let mut bench_error = bench_error;
        let mut bench_results = bench_results;

        bench_running.set(true);
        bench_output.set("Running llama-benchy benchmark...\n".to_string());
        bench_error.set(None);
        bench_results.set(Vec::new());

        spawn(async move {
            let mut cmd = tokio::process::Command::new("llama-benchy");
            cmd.arg("--base-url").arg(&url)
               .arg("--model").arg(&model)
               .arg("--runs").arg(&runs);
            for pp in pp_str.split_whitespace() { cmd.arg("--pp").arg(pp); }
            for tg in tg_str.split_whitespace() { cmd.arg("--tg").arg(tg); }
            cmd.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());

            match cmd.output().await {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let combined = if stderr.is_empty() { stdout.clone() } else { format!("{}\n\nSTDERR:\n{}", stdout, stderr) };
                    bench_output.set(combined);

                    let mut results = Vec::new();
                    for line in stdout.lines() {
                        let l = line.trim();
                        if l.contains("pp=") && l.contains("tg_speed") {
                            let extract = |prefix: &str| -> String {
                                l.split_whitespace().find(|s| s.starts_with(prefix))
                                    .map(|s| s.trim_start_matches(prefix).to_string())
                                    .unwrap_or_else(|| "?".to_string())
                            };
                            results.push((extract("pp="), extract("tg="), extract("pp_speed="), extract("tg_speed=")));
                        }
                    }
                    bench_results.set(results);
                    if !output.status.success() {
                        bench_error.set(Some("llama-benchy exited with non-zero status.".to_string()));
                    }
                }
                Err(e) => {
                    let msg = if e.kind() == std::io::ErrorKind::NotFound {
                        "llama-benchy not found. Install with: pip install llama-benchy".to_string()
                    } else {
                        format!("Failed to run llama-benchy: {}", e)
                    };
                    bench_error.set(Some(msg.clone()));
                    bench_output.set(msg);
                }
            }
            bench_running.set(false);
        });
    };

    let run_eval = move |_| {
        let mut loading = loading; let mut revealed = revealed; let mut vote_result = vote_result;
        let mut alpha_response = alpha_response; let mut beta_response = beta_response; let mut error = error;
        let mut alpha_is_a = alpha_is_a;
        let pr = prompt.read().clone();
        if pr.trim().is_empty() { return; }
        loading.set(true); revealed.set(false); vote_result.set(String::new());
        alpha_response.set("Generating response...".to_string());
        beta_response.set("Generating response...".to_string());
        error.set(None);
        let is_a = chrono::Local::now().timestamp_nanos_opt().unwrap_or(0) % 2 == 0;
        alpha_is_a.set(is_a);
        let url_a = model_a_url.read().clone(); let name_a = model_a_name.read().clone();
        let url_b = model_b_url.read().clone(); let name_b = model_b_name.read().clone();
        spawn(async move {
            let (url_alpha, name_alpha, url_beta, name_beta) = if is_a { (url_a, name_a, url_b, name_b) } else { (url_b, name_b, url_a, name_a) };
            let client = reqwest::Client::new();
            let alpha_res = async {
                let url = format!("{}/chat/completions", url_alpha.trim_end_matches('/'));
                let payload = serde_json::json!({"model": name_alpha, "messages": [{"role": "user", "content": pr.clone()}], "temperature": 0.7});
                match client.post(&url).json(&payload).send().await {
                    Ok(r) if r.status().is_success() => { if let Ok(json) = r.json::<serde_json::Value>().await { if let Some(txt) = json["choices"][0]["message"]["content"].as_str() { return Ok(txt.to_string()); } } Err("Failed to parse".to_string()) }
                    Ok(r) => Err(format!("Status: {}", r.status())),
                    Err(e) => Err(format!("Request failed: {}", e)),
                }
            };
            let beta_res = async {
                let url = format!("{}/chat/completions", url_beta.trim_end_matches('/'));
                let payload = serde_json::json!({"model": name_beta, "messages": [{"role": "user", "content": pr.clone()}], "temperature": 0.7});
                match client.post(&url).json(&payload).send().await {
                    Ok(r) if r.status().is_success() => { if let Ok(json) = r.json::<serde_json::Value>().await { if let Some(txt) = json["choices"][0]["message"]["content"].as_str() { return Ok(txt.to_string()); } } Err("Failed to parse".to_string()) }
                    Ok(r) => Err(format!("Status: {}", r.status())),
                    Err(e) => Err(format!("Request failed: {}", e)),
                }
            };
            let (a_text, b_text) = tokio::join!(alpha_res, beta_res);
            match a_text { Ok(txt) => { alpha_response.set(txt); } Err(e) => { alpha_response.set(format!("Error: {}", e)); error.set(Some(format!("Alpha Error: {}", e))); } }
            match b_text { Ok(txt) => { beta_response.set(txt); } Err(e) => { beta_response.set(format!("Error: {}", e)); error.set(Some(format!("Beta Error: {}", e))); } }
            loading.set(false);
        });
    };

    let cast_vote = move |voted_alpha: bool| {
        let is_a = *alpha_is_a.read();
        let name_a = model_a_name.read().clone(); let name_b = model_b_name.read().clone();
        let winning_model = if voted_alpha { if is_a { name_a.clone() } else { name_b.clone() } } else { if is_a { name_b.clone() } else { name_a.clone() } };
        let losing_model = if voted_alpha { if is_a { name_b } else { name_a } } else { if is_a { name_a } else { name_b } };
        let choice_text = if voted_alpha { "Alpha" } else { "Beta" };
        vote_result.set(format!("Voted {}! Winner: {} | Loser: {}", choice_text, winning_model, losing_model));
        votes_log.write().push(format!("Winner: {} | Loser: {} | Voted: {}", winning_model, losing_model, choice_text));
        revealed.set(true);
    };

    rsx! {
        div { class: "section-title", "Model Comparison & Benchmarking" }
        div { class: "section-desc", "Blind quality evaluation and quantitative performance benchmarking via llama-benchy." }

        div {
            style: "display: flex; border: 1px solid var(--color-hairline); border-radius: 20px; overflow: hidden; background: var(--color-surface-soft); padding: 2px; width: fit-content; margin-bottom: 20px;",
            button {
                class: if bench_mode() == "blind" { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                style: "border-radius: 18px;",
                onclick: move |_| { let mut bench_mode = bench_mode; bench_mode.set("blind".to_string()); },
                "\u{1F3AF} Blind Evaluation"
            }
            button {
                class: if bench_mode() == "bench" { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                style: "border-radius: 18px;",
                onclick: move |_| { let mut bench_mode = bench_mode; bench_mode.set("bench".to_string()); },
                "\u{26A1} llama-benchy"
            }
        }

        if bench_mode() == "bench" {
            div { class: "card", style: "margin-bottom: 20px;",
                div { class: "card-title", "\u{26A1} llama-benchy Performance Benchmark" }
                div { class: "form-hint", style: "margin-bottom: 16px;",
                    "Measures Prompt Processing (PP) and Token Generation (TG) throughput. Install: "
                    code { style: "background: var(--color-surface-soft); padding: 2px 6px; border-radius: 3px;", "pip install llama-benchy" }
                }

                div { style: "display: flex; gap: 16px; flex-wrap: wrap; margin-bottom: 16px;",
                    div { style: "flex: 2; min-width: 220px; display: flex; flex-direction: column; gap: 10px;",
                        div { class: "form-group", style: "margin: 0;",
                            label { class: "form-label", "Server Base URL" }
                            input { class: "form-input", placeholder: "http://127.0.0.1:8080", value: bench_url(),
                                oninput: move |e| { let mut bench_url = bench_url; bench_url.set(e.value()); }
                            }
                        }
                        div { class: "form-group", style: "margin: 0;",
                            label { class: "form-label", "Model Identifier" }
                            input { class: "form-input", placeholder: "e.g., local-model", value: bench_model(),
                                oninput: move |e| { let mut bench_model = bench_model; bench_model.set(e.value()); }
                            }
                        }
                    }
                    div { style: "flex: 1; min-width: 160px; display: flex; flex-direction: column; gap: 10px;",
                        div { class: "form-group", style: "margin: 0;",
                            label { class: "form-label", "PP sizes (space-sep)" }
                            input { class: "form-input", placeholder: "512 1024 2048", value: bench_pp(),
                                oninput: move |e| { let mut bench_pp = bench_pp; bench_pp.set(e.value()); }
                            }
                        }
                        div { class: "form-group", style: "margin: 0;",
                            label { class: "form-label", "TG sizes (space-sep)" }
                            input { class: "form-input", placeholder: "128 256", value: bench_tg(),
                                oninput: move |e| { let mut bench_tg = bench_tg; bench_tg.set(e.value()); }
                            }
                        }
                        div { class: "form-group", style: "margin: 0;",
                            label { class: "form-label", "Runs per test" }
                            input { class: "form-input", placeholder: "3", value: bench_runs(),
                                oninput: move |e| { let mut bench_runs = bench_runs; bench_runs.set(e.value()); }
                            }
                        }
                    }
                }

                button {
                    class: "btn btn-start",
                    disabled: bench_running(),
                    onclick: run_bench,
                    if bench_running() { "\u{23F3} Running..." } else { "\u{26A1} Run Benchmark" }
                }

                if let Some(ref err) = *bench_error.read() {
                    div { class: "error-toast", style: "margin-top: 12px;", "{err}" }
                }
            }

            if !bench_results.read().is_empty() {
                div { class: "card", style: "margin-bottom: 20px;",
                    div { class: "card-title", "\u{1F4CA} Results" }
                    div { style: "overflow-x: auto;",
                        table { style: "width: 100%; border-collapse: collapse; font-size: 13px;",
                            thead {
                                tr { style: "border-bottom: 2px solid var(--color-hairline);",
                                    th { style: "padding: 8px 12px; font-weight: 600; color: var(--color-muted); text-align: left;", "PP" }
                                    th { style: "padding: 8px 12px; font-weight: 600; color: var(--color-muted); text-align: left;", "TG" }
                                    th { style: "padding: 8px 12px; font-weight: 600; color: var(--color-muted); text-align: left;", "PP Speed" }
                                    th { style: "padding: 8px 12px; font-weight: 600; color: var(--color-muted); text-align: left;", "TG Speed" }
                                }
                            }
                            tbody {
                                for (pp, tg, pp_spd, tg_spd) in bench_results.read().iter() {
                                    tr { style: "border-bottom: 1px solid var(--color-hairline);",
                                        td { style: "padding: 8px 12px; font-weight: 500;", "{pp}" }
                                        td { style: "padding: 8px 12px; font-weight: 500;", "{tg}" }
                                        td { style: "padding: 8px 12px;",
                                            span { style: "background: color-mix(in srgb, var(--color-success) 15%, transparent); color: var(--color-success); padding: 2px 8px; border-radius: 4px; font-weight: 600;", "{pp_spd}" }
                                        }
                                        td { style: "padding: 8px 12px;",
                                            span { style: "background: color-mix(in srgb, var(--color-brand-accent) 15%, transparent); color: var(--color-brand-accent); padding: 2px 8px; border-radius: 4px; font-weight: 600;", "{tg_spd}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !bench_output.read().is_empty() {
                div { class: "card",
                    div { class: "card-title", "Raw Output" }
                    pre { style: "white-space: pre-wrap; font-family: 'JetBrains Mono', monospace; font-size: 12px; color: var(--color-on-dark-soft); background: var(--color-surface-dark); padding: 14px; border-radius: 6px; max-height: 340px; overflow-y: auto;",
                        "{bench_output}"
                    }
                }
            }
        } else {
            div { class: "card", style: "margin-bottom: 20px; display: flex; flex-direction: column; gap: 12px;",
                div { class: "card-title", "Blind Evaluation Setup" }

                div { style: "display: flex; gap: 16px; flex-wrap: wrap;",
                    div { style: "flex: 1; min-width: 250px; display: flex; flex-direction: column; gap: 8px;",
                        div { style: "font-weight: 600; font-size: 13px; color: var(--color-ink);", "Model A" }
                        div { class: "form-group", style: "margin: 0;",
                            label { class: "form-label", "Base API URL" }
                            input { class: "form-input", value: model_a_url(), oninput: move |e| { let mut model_a_url = model_a_url; model_a_url.set(e.value()); } }
                        }
                        div { class: "form-group", style: "margin: 0;",
                            label { class: "form-label", "Model Identifier" }
                            input { class: "form-input", value: model_a_name(), oninput: move |e| { let mut model_a_name = model_a_name; model_a_name.set(e.value()); } }
                        }
                    }
                    div { style: "flex: 1; min-width: 250px; display: flex; flex-direction: column; gap: 8px;",
                        div { style: "font-weight: 600; font-size: 13px; color: var(--color-ink);", "Model B" }
                        div { class: "form-group", style: "margin: 0;",
                            label { class: "form-label", "Base API URL" }
                            input { class: "form-input", value: model_b_url(), oninput: move |e| { let mut model_b_url = model_b_url; model_b_url.set(e.value()); } }
                        }
                        div { class: "form-group", style: "margin: 0;",
                            label { class: "form-label", "Model Identifier" }
                            input { class: "form-input", value: model_b_name(), oninput: move |e| { let mut model_b_name = model_b_name; model_b_name.set(e.value()); } }
                        }
                    }
                }

                div { class: "form-group",
                    label { class: "form-label", "Evaluation Prompt" }
                    textarea { class: "form-textarea", style: "height: 80px;",
                        placeholder: "e.g., Write a rust function that parses JSON without external crates...",
                        value: prompt(),
                        oninput: move |e| { let mut prompt = prompt; prompt.set(e.value()); }
                    }
                }

                if let Some(ref err) = *error.read() { div { class: "error-toast", "{err}" } }

                button {
                    class: "btn btn-start",
                    onclick: run_eval,
                    disabled: loading(),
                    if loading() { "Evaluating..." } else { "\u{1F3AF} Run Blind Evaluation" }
                }
            }

            if !alpha_response.read().is_empty() || !beta_response.read().is_empty() {
                div { style: "display: flex; gap: 20px; flex-wrap: wrap; margin-bottom: 20px;",
                    div { style: "flex: 1; min-width: 300px;",
                        div { class: "card", style: "height: 100%; border-top: 4px solid var(--color-brand-accent);",
                            div { class: "card-title", "Model Alpha" }
                            pre { style: "white-space: pre-wrap; font-family: inherit; font-size: 13px; color: var(--color-body); line-height: 1.5; height: 350px; overflow-y: auto;", "{alpha_response}" }
                        }
                    }
                    div { style: "flex: 1; min-width: 300px;",
                        div { class: "card", style: "height: 100%; border-top: 4px solid var(--color-brand-accent);",
                            div { class: "card-title", "Model Beta" }
                            pre { style: "white-space: pre-wrap; font-family: inherit; font-size: 13px; color: var(--color-body); line-height: 1.5; height: 350px; overflow-y: auto;", "{beta_response}" }
                        }
                    }
                }
            }

            if !loading() && (!alpha_response.read().is_empty() || !beta_response.read().is_empty()) {
                div { class: "card", style: "text-align: center; display: flex; flex-direction: column; gap: 12px; align-items: center;",
                    div { class: "card-title", "Submit Your Vote" }
                    div { style: "display: flex; gap: 12px; margin-top: 6px;",
                        button { class: "btn btn-start", onclick: { let mut cast_vote = cast_vote.clone(); move |_| cast_vote(true) }, "Alpha is Better" }
                        button { class: "btn btn-start", onclick: { let mut cast_vote = cast_vote.clone(); move |_| cast_vote(false) }, "Beta is Better" }
                        button { class: "btn btn-ghost", onclick: { let mut revealed = revealed; move |_| revealed.set(true) }, "Reveal Identities" }
                    }
                    if revealed() {
                        div { style: "margin-top: 12px; font-weight: 600; color: var(--color-brand-accent); background: var(--color-surface-soft); padding: 10px 20px; border-radius: 6px; border: 1px solid var(--color-hairline);",
                            "Alpha = " strong { if alpha_is_a() { "{model_a_name()}" } else { "{model_b_name()}" } }
                            " | Beta = " strong { if alpha_is_a() { "{model_b_name()}" } else { "{model_a_name()}" } }
                        }
                    }
                    if !vote_result.read().is_empty() {
                        div { style: "font-size: 13px; color: var(--color-success); font-weight: 500;", "{vote_result}" }
                    }
                }
            }

            if !votes_log.read().is_empty() {
                div { class: "card", style: "margin-top: 20px;",
                    div { class: "card-title", "Evaluation History" }
                    ul { style: "list-style-type: decimal; margin-left: 20px;",
                        for log in votes_log.read().iter() {
                            li { style: "font-size: 12.5px; color: var(--color-body); margin-bottom: 4px;", "{log}" }
                        }
                    }
                }
            }
        }
    }
}

// ── Deep Research View ───────────────────────────────────────────────────────

#[component]
fn TabDeepResearch(config: Signal<ServerConfig>) -> Element {
    let query = use_signal(String::new);
    let is_running = use_signal(|| false);
    let log_content = use_signal(String::new);
    let current_report = use_signal(String::new);
    let selected_report = use_signal(String::new);
    let reports_list = use_signal(get_previous_reports);

    let start_research = move |_| {
        let mut is_running = is_running;
        let mut log_content = log_content;
        let mut current_report = current_report;
        let reports_list = reports_list;

        let q = query.read().trim().to_string();
        if q.is_empty() {
            return;
        }

        is_running.set(true);
        log_content.set("Initializing Deep Research agent...\n".to_string());
        current_report.set(String::new());

        let host = "127.0.0.1".to_string();
        let port = 8080; // Placeholder

        spawn(async move {
            let target_model = "llama3".to_string(); // Placeholder
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let report_filename = format!("report_{}.md", timestamp);
            let report_path = format!(
                "/home/notroot/.local/share/llama-manager/research/{}",
                report_filename
            );
            let log_path = "/home/notroot/.local/share/llama-manager/research/research_log.txt";

            let _ = std::fs::create_dir_all("/home/notroot/.local/share/llama-manager/research/");

            let mut cmd = std::process::Command::new("uv");
            cmd.arg("run")
                .arg("deep_research.py")
                .arg("--query")
                .arg(&q)
                .arg("--host")
                .arg(&host)
                .arg("--port")
                .arg(&port.to_string())
                .arg("--model")
                .arg(&target_model)
                .arg("--output")
                .arg(&report_path)
                .arg("--log-file")
                .arg(log_path);

            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        if let Ok(content) = std::fs::read_to_string(&report_path) {
                            let mut current_report = current_report;
                            current_report.set(content);
                        }
                    } else {
                        let err_msg = String::from_utf8_lossy(&output.stderr);
                        let mut log_content = log_content;
                        log_content
                            .set(format!("Research process exited with error:\n{}", err_msg));
                    }
                }
                Err(e) => {
                    let mut log_content = log_content;
                    log_content.set(format!("Failed to launch deep research process: {}", e));
                }
            }
            let mut is_running = is_running;
            is_running.set(false);
            let mut reports_list = reports_list;
            reports_list.set(get_previous_reports());
        });
    };

    let _log_poller = use_resource(move || {
        let is_running = is_running;
        let log_content = log_content;
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if is_running() {
                    let log_path = std::path::PathBuf::from(
                        "/home/notroot/.local/share/llama-manager/research/research_log.txt",
                    );
                    if log_path.exists() {
                        if let Ok(logs) = std::fs::read_to_string(&log_path) {
                            let mut log_content = log_content;
                            log_content.set(logs);
                        }
                    }
                }
            }
        }
    });

    let load_report = move |filename: String| {
        let mut current_report = current_report;
        let mut selected_report = selected_report;
        let path = format!(
            "/home/notroot/.local/share/llama-manager/research/{}",
            filename
        );
        if let Ok(content) = std::fs::read_to_string(&path) {
            current_report.set(content);
            selected_report.set(filename);
        }
    };

    rsx! {
        div { class: "section-title", "Deep Research Agent" }
        div { class: "section-desc", "Executes multi-step breadth-then-depth research loops crawling resources to compile comprehensive visual reports." }

        div {
            style: "display: flex; gap: 20px; flex-wrap: wrap; align-items: flex-start;",

            // Left column: Setup and Reports list
            div {
                style: "flex: 1; min-width: 300px; display: flex; flex-direction: column; gap: 16px;",

                // Research Input Card
                div { class: "card",
                    div { class: "card-title", "New Research Campaign" }
                    div { class: "form-group",
                        label { class: "form-label", "Research Goal / Query" }
                        textarea {
                            class: "form-textarea",
                            style: "height: 80px;",
                            placeholder: "e.g., Explain the advancements in Gemini 1.5 Flash architecture and cache caching mechanisms...",
                            value: query(),
                            oninput: move |e| {
                                let mut query = query;
                                query.set(e.value());
                            },
                        }
                    }
                    button {
                        class: "btn btn-start",
                        onclick: start_research,
                        disabled: is_running(),
                        if is_running() { "Researching..." } else { "Launch Deep Research" }
                    }
                }

                // Previous Reports Card
                div { class: "card",
                    div { class: "card-title", "Research Repository" }
                    if reports_list.read().is_empty() {
                        div { style: "text-align: center; color: var(--color-muted); font-style: italic; padding: 20px;",
                            "No reports in repository."
                        }
                    } else {
                        div {
                            style: "display: flex; flex-direction: column; gap: 6px; max-height: 300px; overflow-y: auto;",
                            for (filename, _) in reports_list.read().iter() {
                                button {
                                    key: "{filename}",
                                    class: if selected_report() == *filename { "btn btn-start btn-sm" } else { "btn btn-ghost btn-sm" },
                                    style: "text-align: left; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; width: 100%; display: block;",
                                    onclick: {
                                        let name = filename.clone();
                                        let load_report = load_report.clone();
                                        move |_| load_report(name.clone())
                                    },
                                    "{filename}"
                                }
                            }
                        }
                    }
                }
            }

            // Right column: Live log or Report viewer
            div {
                style: "flex: 2; min-width: 400px; display: flex; flex-direction: column; gap: 16px;",

                if is_running() {
                    div { class: "card",
                        div { class: "card-title", "Live Research Status Logs" }
                        pre {
                            style: "background: var(--color-surface-dark); color: var(--color-on-dark-soft); font-family: 'JetBrains Mono', monospace; font-size: 11.5px; padding: 12px; border-radius: 6px; height: 320px; overflow-y: auto; white-space: pre-wrap; word-break: break-all;",
                            "{log_content}"
                        }
                    }
                } else if !current_report.read().is_empty() {
                    div { class: "card",
                        style: "border-top: 4px solid var(--color-brand-accent);",
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--color-hairline); padding-bottom: 8px; margin-bottom: 12px;",
                            div { class: "card-title", style: "margin: 0;", "Synthesized Research Report" }
                            span { style: "font-size: 11px; color: var(--color-muted);", "{selected_report}" }
                        }

                        div {
                            style: "max-height: 600px; overflow-y: auto; padding-right: 8px;",
                            {render_markdown(&current_report.read())}
                        }
                    }
                } else {
                    div { class: "card",
                        style: "text-align: center; color: var(--color-muted); padding: 80px 20px;",
                        div { style: "font-size: 40px; margin-bottom: 12px;", "🔍" }
                        div { style: "font-weight: 500;", "Deep Research Repository" }
                        div { style: "font-size: 12px; margin-top: 4px;", "Launch a campaign on the left or select an archived report to read." }
                    }
                }
            }
        }
    }
}

fn get_previous_reports() -> Vec<(String, String)> {
    let dir = std::path::PathBuf::from("/home/notroot/.local/share/llama-manager/research/");
    let _ = std::fs::create_dir_all(&dir);
    let mut reports = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if filename.starts_with("report_") {
                        reports.push((filename.to_string(), path.to_string_lossy().to_string()));
                    }
                }
            }
        }
    }
    reports.sort_by(|a, b| b.0.cmp(&a.0));
    reports
}

fn render_markdown(text: &str) -> Element {
    let mut in_code_block = false;
    let mut code_content = String::new();
    let mut code_lang = String::new();

    let mut rendered_elements = Vec::new();

    for line in text.lines() {
        if line.starts_with("```") {
            if in_code_block {
                in_code_block = false;
                let content = code_content.clone();
                let lang = code_lang.clone();
                rendered_elements.push(rsx! {
                    div {
                        style: "background: var(--color-surface-dark); color: var(--color-on-dark-soft); font-family: monospace; padding: 12px; border-radius: 6px; margin: 10px 0; overflow-x: auto; font-size: 13px;",
                        div { style: "font-size: 10px; color: var(--color-muted); text-transform: uppercase; margin-bottom: 6px; border-bottom: 1px solid var(--color-overlay-white-1); padding-bottom: 4px;",
                            { if lang.is_empty() { "code".to_string() } else { lang } }
                        }
                        pre { "{content}" }
                    }
                });
                code_content.clear();
                code_lang.clear();
            } else {
                in_code_block = true;
                code_lang = line.trim_start_matches("```").trim().to_string();
            }
            continue;
        }

        if in_code_block {
            code_content.push_str(line);
            code_content.push('\n');
            continue;
        }

        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            let heading = trimmed[2..].to_string();
            rendered_elements.push(rsx! {
                h1 { style: "font-size: 22px; font-weight: 700; color: var(--color-ink); margin: 20px 0 10px 0; border-bottom: 1px solid var(--color-hairline); padding-bottom: 6px;",
                    "{heading}"
                }
            });
        } else if trimmed.starts_with("## ") {
            let heading = trimmed[3..].to_string();
            rendered_elements.push(rsx! {
                h2 { style: "font-size: 18px; font-weight: 600; color: var(--color-ink); margin: 16px 0 8px 0;",
                    "{heading}"
                }
            });
        } else if trimmed.starts_with("### ") {
            let heading = trimmed[4..].to_string();
            rendered_elements.push(rsx! {
                h3 { style: "font-size: 15px; font-weight: 600; color: var(--color-ink); margin: 12px 0 6px 0;",
                    "{heading}"
                }
            });
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let li_text = trimmed[2..].to_string();
            rendered_elements.push(rsx! {
                ul { style: "margin: 4px 0 4px 20px; list-style-type: disc;",
                    li { style: "font-size: 13.5px; color: var(--color-body); line-height: 1.5;", "{li_text}" }
                }
            });
        } else if trimmed.starts_with("|") && trimmed.ends_with("|") {
            rendered_elements.push(rsx! {
                div { style: "font-family: monospace; font-size: 12.5px; background: var(--color-surface-soft); padding: 4px 10px; border-left: 3px solid var(--color-brand-accent); margin: 2px 0;",
                    "{trimmed}"
                }
            });
        } else if !trimmed.is_empty() {
            let para = trimmed.to_string();
            rendered_elements.push(rsx! {
                p { style: "font-size: 13.5px; color: var(--color-body); margin: 8px 0; line-height: 1.5; text-align: justify;",
                    "{para}"
                }
            });
        }
    }

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 4px;",
            for el in rendered_elements {
                {el}
            }
        }
    }
}

// ── GAM Memory Simulation Structs and Component ─────────────────────────────

#[derive(Clone, PartialEq, Debug)]
struct EpisodicLog {
    text: String,
    timestamp: String,
}

#[derive(Clone, PartialEq, Debug)]
struct SimNode {
    id: String,
    label: String,
    x: f64,
    y: f64,
}

#[derive(Clone, PartialEq, Debug)]
struct SimEdge {
    source: String,
    target: String,
}

#[component]
fn GamMemorySimulation(config: Signal<ServerConfig>) -> Element {
    let mut episodic_buffer = use_signal(|| {
        vec![
            EpisodicLog {
                text: "Session initiated: connected to model server".to_string(),
                timestamp: "23:15:00".to_string(),
            },
            EpisodicLog {
                text: "Executed search for local memory context".to_string(),
                timestamp: "23:15:05".to_string(),
            },
            EpisodicLog {
                text: "User queried: 'Optimize CUDA configurations'".to_string(),
                timestamp: "23:15:10".to_string(),
            },
        ]
    });
    let mut sim_divergence_threshold = use_signal(|| 0.40_f64);
    let mut calculated_divergence_score = use_signal(|| 0.15_f64);
    let mut new_log_input = use_signal(String::new);
    let mut anim_consolidating = use_signal(|| false);

    let mut global_semantic_nodes = use_signal(|| {
        vec![
            SimNode {
                id: "project".to_string(),
                label: "Project Core".to_string(),
                x: 270.0,
                y: 210.0,
            },
            SimNode {
                id: "user_preferences".to_string(),
                label: "User Preferences".to_string(),
                x: 130.0,
                y: 110.0,
            },
            SimNode {
                id: "tech_stack".to_string(),
                label: "Tech Stack (Rust/Dioxus)".to_string(),
                x: 410.0,
                y: 110.0,
            },
        ]
    });

    let mut global_semantic_edges = use_signal(|| {
        vec![
            SimEdge {
                source: "user_preferences".to_string(),
                target: "project".to_string(),
            },
            SimEdge {
                source: "project".to_string(),
                target: "tech_stack".to_string(),
            },
        ]
    });

    let add_log = move |_| {
        let text = new_log_input.read().trim().to_string();
        if text.is_empty() {
            return;
        }
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        episodic_buffer.write().push(EpisodicLog {
            text,
            timestamp: now,
        });
        let current_score = *calculated_divergence_score.read();
        calculated_divergence_score.set((current_score + 0.12).min(1.0));
        new_log_input.set(String::new());
    };

    let trigger_consolidation = move |_| {
        if episodic_buffer.read().is_empty() {
            return;
        }
        anim_consolidating.set(true);

        let node_label = if let Some(last_log) = episodic_buffer.read().last() {
            let t = last_log.text.trim();
            let clean = if t.len() > 22 {
                format!("{}...", &t[..19])
            } else {
                t.to_string()
            };
            if clean.to_lowercase().contains("cuda") {
                "CUDA Optimization".to_string()
            } else if clean.to_lowercase().contains("search") {
                "Search Context".to_string()
            } else if clean.to_lowercase().contains("model") {
                "Model Configuration".to_string()
            } else {
                clean
            }
        } else {
            "Consolidated Node".to_string()
        };

        let node_id = format!("node_{}", chrono::Local::now().timestamp_millis());

        let node_count = global_semantic_nodes.read().len() as f64;
        let angle = node_count * 2.0 * std::f64::consts::PI / 6.0;
        let radius = 120.0;
        let x = 270.0 + radius * angle.cos();
        let y = 210.0 + radius * angle.sin();

        let new_node = SimNode {
            id: node_id.clone(),
            label: node_label,
            x,
            y,
        };
        global_semantic_nodes.write().push(new_node);
        global_semantic_edges.write().push(SimEdge {
            source: "project".to_string(),
            target: node_id,
        });

        episodic_buffer.write().clear();
        calculated_divergence_score.set(0.05);

        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            anim_consolidating.set(false);
        });
    };

    rsx! {
        div {
            style: "display: flex; gap: 20px; flex-wrap: wrap; width: 100%;",

            // Left Column: Episodic Buffer
            div {
                style: "flex: 1.2; min-width: 320px; display: flex; flex-direction: column; gap: 16px;",
                div { class: "card", style: "flex: 1; display: flex; flex-direction: column; padding: 18px;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--color-hairline); padding-bottom: 8px; margin-bottom: 14px;",
                        span { style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-muted);", "📋 Episodic Buffer (Event Graph)" }
                        span { class: "badge", style: "background: var(--color-badge-neutral-bg); color: var(--color-badge-neutral-text); font-size: 11px;", "{episodic_buffer.read().len()} Transient Events" }
                    }

                    // Log Entries List
                    div {
                        style: "flex: 1; overflow-y: auto; max-height: 220px; display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; padding-right: 4px; min-height: 120px;",
                        if episodic_buffer.read().is_empty() {
                            div {
                                style: "text-align: center; color: var(--color-muted); font-style: italic; padding: 30px 10px; font-size: 12.5px;",
                                "Episodic buffer is clean. Add some log entries below to simulate agent actions."
                            }
                        } else {
                            for (idx, log) in episodic_buffer.read().iter().enumerate() {
                                div {
                                    key: "{idx}",
                                    style: "display: flex; gap: 10px; padding: 8px; border-left: 2px solid var(--color-hairline); background: var(--color-surface-soft); border-radius: 0 4px 4px 0; font-size: 12.5px;",
                                    span { style: "font-family: monospace; font-size: 11px; color: var(--color-muted); padding-top: 1.5px;", "[{log.timestamp}]" }
                                    span { style: "color: var(--color-body);", "{log.text}" }
                                }
                            }
                        }
                    }

                    // Divergence Meter
                    div {
                        style: "background: var(--color-canvas); border: 1px solid var(--color-hairline); border-radius: 6px; padding: 12px; margin-bottom: 16px;",
                        div {
                            style: "display: flex; justify-content: space-between; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--color-ink);",
                            span { "Semantic Divergence Score" }
                            span {
                                color: if *calculated_divergence_score.read() >= *sim_divergence_threshold.read() { "var(--color-error)" } else { "var(--color-brand-accent)" },
                                "{calculated_divergence_score():.2} / {sim_divergence_threshold():.2}"
                            }
                        }
                        // Progress Bar
                        div {
                            style: "width: 100%; height: 8px; background: var(--color-surface-strong); border-radius: 4px; overflow: hidden; position: relative;",
                            div {
                                style: format!("height: 100%; width: {}%; background: {}; transition: width 0.4s ease, background 0.3s;",
                                    (calculated_divergence_score() * 100.0).min(100.0),
                                    if *calculated_divergence_score.read() >= *sim_divergence_threshold.read() { "var(--color-error)" } else if *calculated_divergence_score.read() >= *sim_divergence_threshold.read() * 0.7 { "var(--color-warning)" } else { "var(--color-success)" }
                                )
                            }
                        }
                        if *calculated_divergence_score.read() >= *sim_divergence_threshold.read() {
                            div {
                                style: "font-size: 11px; color: var(--color-error); font-weight: 600; margin-top: 6px; display: flex; align-items: center; gap: 4px;",
                                "⚡ Divergence limit crossed! Consolidation phase recommended."
                            }
                        } else {
                            div { style: "font-size: 11px; color: var(--color-muted); margin-top: 4px;", "Score increments as passing interactions accumulate in the buffer." }
                        }
                    }

                    // Simulation Controls / Inputs
                    div {
                        style: "display: flex; flex-direction: column; gap: 12px; border-top: 1px solid var(--color-hairline); padding-top: 14px;",

                        // Slider Input
                        div {
                            style: "display: flex; flex-direction: column; gap: 4px;",
                            label { style: "font-size: 11.5px; font-weight: 600; color: var(--color-body);", "Consolidation Divergence Threshold" }
                            div {
                                style: "display: flex; align-items: center; gap: 10px;",
                                input {
                                    type: "range",
                                    min: "0.1",
                                    max: "0.9",
                                    step: "0.05",
                                    style: "flex: 1; cursor: pointer;",
                                    value: sim_divergence_threshold().to_string(),
                                    oninput: move |e| {
                                        if let Ok(v) = e.value().parse::<f64>() {
                                            sim_divergence_threshold.set(v);
                                        }
                                    }
                                }
                                span { style: "font-size: 12px; font-family: monospace; font-weight: bold; width: 30px;", "{sim_divergence_threshold():.2}" }
                            }
                        }

                        // Text Entry Input
                        div {
                            style: "display: flex; gap: 8px;",
                            input {
                                class: "form-input",
                                style: "flex: 1; font-size: 12.5px; height: 32px;",
                                placeholder: "Type a sample log entry or action...",
                                value: new_log_input(),
                                oninput: move |e| new_log_input.set(e.value()),
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        let text = new_log_input.read().trim().to_string();
                                        if !text.is_empty() {
                                            let now = chrono::Local::now().format("%H:%M:%S").to_string();
                                            episodic_buffer.write().push(EpisodicLog { text, timestamp: now });
                                            let current_score = *calculated_divergence_score.read();
                                            calculated_divergence_score.set((current_score + 0.12).min(1.0));
                                            new_log_input.set(String::new());
                                        }
                                    }
                                }
                            }
                            button {
                                class: "btn btn-start btn-sm",
                                style: "height: 32px; padding: 0 12px;",
                                onclick: add_log,
                                "Add Log"
                            }
                        }

                        // Trigger Consolidation
                        button {
                            class: "btn btn-start",
                            style: "width: 100%; font-weight: 700; margin-top: 4px; display: flex; justify-content: center; align-items: center; gap: 8px;",
                            disabled: episodic_buffer.read().is_empty() || anim_consolidating(),
                            onclick: trigger_consolidation,
                            if anim_consolidating() { "⚡ Consolidating..." } else { "🧬 Trigger Semantic Consolidation" }
                        }
                    }
                }
            }

            // Right Column: Global Semantic Graph Visualizer
            div {
                style: "flex: 1.8; min-width: 400px; display: flex; flex-direction: column;",
                div { class: "card", style: "height: 100%; display: flex; flex-direction: column; padding: 18px; position: relative;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                        span { style: "font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-muted);", "🕸️ Global topic associative network" }
                        div { style: "display: flex; gap: 8px; font-size: 10px;",
                            span { style: "display: flex; align-items: center; gap: 4px;",
                                span { style: "display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: var(--color-ink);" }
                                "Core Node"
                            }
                            span { style: "display: flex; align-items: center; gap: 4px;",
                                span { style: "display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: var(--color-brand-accent);" }
                                "Consolidated"
                            }
                        }
                    }

                    // Interactive SVG view for semantic graph
                    div {
                        style: "position: relative; width: 100%; height: 420px; border-radius: 8px; overflow: hidden; border: 1px solid var(--color-hairline); background: var(--color-canvas);",
                        svg {
                            width: "100%",
                            height: "100%",
                            view_box: "0 0 540 420",

                            defs {
                                pattern {
                                    id: "sim-grid",
                                    width: "24",
                                    height: "24",
                                    pattern_units: "userSpaceOnUse",
                                    path {
                                        d: "M 24 0 L 0 0 0 24",
                                        fill: "none",
                                        stroke: "rgba(99, 102, 241, 0.04)",
                                        stroke_width: "1",
                                    }
                                }
                            }

                            rect {
                                width: "100%",
                                height: "100%",
                                fill: "url(#sim-grid)",
                            }

                            // Draw Edges
                            for edge in global_semantic_edges.read().iter() {
                                if let (Some(s_node), Some(t_node)) = (
                                    global_semantic_nodes.read().iter().find(|n| n.id == edge.source),
                                    global_semantic_nodes.read().iter().find(|n| n.id == edge.target)
                                ) {
                                    line {
                                        x1: "{s_node.x}",
                                        y1: "{s_node.y}",
                                        x2: "{t_node.x}",
                                        y2: "{t_node.y}",
                                        stroke: "rgba(99, 102, 241, 0.4)",
                                        stroke_width: "2",
                                        stroke_dasharray: "4,4",
                                    }
                                }
                            }

                            // Draw Nodes
                            for node in global_semantic_nodes.read().iter() {
                                {
                                    let is_core = node.id == "project" || node.id == "user_preferences" || node.id == "tech_stack";
                                    let node_fill = if is_core { "#1e293b" } else { "#6366f1" };
                                    let radius = if node.id == "project" { 14.0 } else if is_core { 10.0 } else { 8.5 };

                                    rsx! {
                                        g {
                                            key: "{node.id}",
                                            circle {
                                                cx: "{node.x}",
                                                cy: "{node.y}",
                                                r: "{radius}",
                                                fill: "{node_fill}",
                                                stroke: "#ffffff",
                                                stroke_width: "2",
                                                style: "transition: r 0.2s;",
                                            }
                                            text {
                                                x: "{node.x}",
                                                y: "{node.y - radius - 6.0}",
                                                text_anchor: "middle",
                                                font_size: "10.5px",
                                                font_weight: if is_core { "700" } else { "500" },
                                                fill: if is_core { "#0f172a" } else { "#475569" },
                                                "{node.label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Overlay consolidation animation
                        if anim_consolidating() {
                            div {
                                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: var(--color-overlay-4); display: flex; flex-direction: column; align-items: center; justify-content: center; z-index: 100;",
                                div {
                                    style: "border: 4px solid var(--color-overlay-2); border-top: 4px solid var(--color-brand-accent); border-radius: 50%; width: 40px; height: 40px; animation: spin 1s linear infinite; margin-bottom: 12px;"
                                }
                                span { style: "font-size: 13.5px; font-weight: 700; color: var(--color-primary-active);", "Consolidating Event Graph..." }
                                span { style: "font-size: 11.5px; color: var(--color-muted); margin-top: 4px;", "Merging summarized items into Global network" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TabAppSettings(
    config: Signal<ServerConfig>,
    search_target: Signal<String>,
    scan_trigger: Signal<u64>,
) -> Element {
    let mut input_text = use_signal(String::new);
    let mut new_theme_name = use_signal(String::new);

    let browse_dir = move |_| {
        let mut config = config;
        let mut scan_trigger = scan_trigger;
        spawn(async move {
            if let Ok(Some(path)) =
                tokio::task::spawn_blocking(move || rfd::FileDialog::new().pick_folder()).await
            {
                let path_str = path.to_string_lossy().to_string();
                let mut current_dirs = config.read().model_scan_dirs.clone();
                if !current_dirs.contains(&path_str) {
                    current_dirs.push(path_str);
                    config.write().model_scan_dirs = current_dirs;
                    *scan_trigger.write() += 1;
                }
            }
        });
    };

    // Always reads directly from ui_* working copy
    let get_resolved_values = |cfg: &ServerConfig| {
        (
            cfg.ui_background_color.clone(),
            cfg.ui_text_color.clone(),
            cfg.ui_accent_color.clone(),
            cfg.ui_card_bg.clone(),
            cfg.ui_sidebar_bg.clone(),
            cfg.ui_border_color.clone(),
            cfg.ui_font_family.clone(),
            cfg.ui_transparency,
            cfg.ui_blur,
            cfg.ui_blur_intensity,
        )
    };

    // Check if current ui_* values differ from the named theme (unsaved edits indicator)
    let theme_is_modified = {
        let cfg = config.read();
        let name = cfg.theme_name.clone();
        if name == "default" || name == "custom" {
            false
        } else if let Some(preset) = THEME_PRESETS.iter().find(|p| p.name == name) {
            cfg.ui_background_color != preset.background_color
                || cfg.ui_text_color != preset.text_color
                || cfg.ui_accent_color != preset.accent_color
                || cfg.ui_card_bg != preset.card_bg
                || cfg.ui_sidebar_bg != preset.sidebar_bg
                || cfg.ui_border_color != preset.border_color
                || cfg.ui_font_family != preset.font_family
                || (cfg.ui_transparency - preset.transparency).abs() > 0.001
                || cfg.ui_blur != preset.blur
                || cfg.ui_blur_intensity != preset.blur_intensity
        } else if let Some(custom) = cfg.custom_themes.iter().find(|t| t.name == name) {
            cfg.ui_background_color != custom.background_color
                || cfg.ui_text_color != custom.text_color
                || cfg.ui_accent_color != custom.accent_color
                || cfg.ui_card_bg != custom.card_bg
                || cfg.ui_sidebar_bg != custom.sidebar_bg
                || cfg.ui_border_color != custom.border_color
                || cfg.ui_font_family != custom.font_family
                || (cfg.ui_transparency - custom.transparency).abs() > 0.001
                || cfg.ui_blur != custom.blur
                || cfg.ui_blur_intensity != custom.blur_intensity
        } else {
            false
        }
    };

    rsx! {
        div { class: "section-title", "App Settings" }
        div { class: "section-desc", "Configure Llama-Manager client settings, custom theme styling, window transparency, frosted glass panels, search engine integration, and model scan paths." }

        // Card 1: UI Theme & Aesthetics
        div { class: "card",
            div { class: "card-title", "UI Theme & Aesthetics" }

            div { class: "form-row",
                div { class: "form-group",
                    label { r#for: "form-theme-preset", class: "form-label", "Theme Preset" }
                    select {
                        id: "form-theme-preset",
                        class: "form-input",
                        value: config.read().theme_name.clone(),
                        onchange: move |e: Event<FormData>| {
                            let theme = e.value();
                            let mut cfg = config.write();
                            cfg.theme_name = theme.clone();
                            if theme == "default" {
                                let defaults = ServerConfig::default();
                                cfg.ui_background_color = defaults.ui_background_color;
                                cfg.ui_text_color = defaults.ui_text_color;
                                cfg.ui_accent_color = defaults.ui_accent_color;
                                cfg.ui_card_bg = defaults.ui_card_bg;
                                cfg.ui_sidebar_bg = defaults.ui_sidebar_bg;
                                cfg.ui_border_color = defaults.ui_border_color;
                                cfg.ui_font_family = defaults.ui_font_family;
                                cfg.ui_transparency = defaults.ui_transparency;
                                cfg.ui_blur = defaults.ui_blur;
                                cfg.ui_blur_intensity = defaults.ui_blur_intensity;
                            } else if let Some(preset) = THEME_PRESETS.iter().find(|p| p.name == theme) {
                                apply_preset_to_config(&mut cfg, preset);
                            } else {
                                let custom = cfg.custom_themes.iter().find(|t| t.name == theme).cloned();
                                if let Some(custom) = custom {
                                    apply_custom_to_config(&mut cfg, &custom);
                                }
                            }
                        },
                        option { value: "default", "Default Theme" }
                        for preset in THEME_PRESETS {
                            option { value: preset.name, "{preset.label}" }
                        }
                        for custom in config.read().custom_themes.iter() {
                            option { value: "{custom.name}", "Saved: {custom.name}" }
                        }
                        if config.read().theme_name == "custom" {
                            option { value: "custom", "Custom (Unsaved)" }
                        }
                    }
                    if theme_is_modified {
                        div { style: "margin-top: 6px; display: flex; align-items: center; gap: 6px;",
                            span { style: "width: 6px; height: 6px; border-radius: 50%; background: var(--color-brand-accent); display: inline-block; flex-shrink: 0;" }
                            span { style: "font-size: 11px; color: var(--color-brand-accent); font-weight: 600;", "Unsaved changes — save below to keep" }
                        }
                    }
                    div { class: "form-hint", "Select a preset or your saved theme. Editing any value customizes it without losing the preset name." }
                }

                div { class: "form-group",
                    label { r#for: "form-ui-font-family", class: "form-label", "Font Family" }
                    select {
                        id: "form-ui-font-family",
                        class: "form-input",
                        value: config.read().ui_font_family.clone(),
                        onchange: move |e: Event<FormData>| {
                            let font = e.value();
                            let mut cfg = config.write();
                            cfg.ui_font_family = font;
                        },
                        option { value: "Inter", "Inter" }
                        option { value: "JetBrains Mono", "JetBrains Mono" }
                        option { value: "Roboto", "Roboto" }
                        option { value: "system-ui", "System Default" }
                    }
                    div { class: "form-hint", "Primary typography style used across text and headers." }
                }
            }

            div { class: "form-row", style: "margin-top: 12px;",
                div { class: "form-group",
                    label { r#for: "form-ui-transparency", class: "form-label", "Window Transparency ({config.read().ui_transparency * 100.0:.0}%)" }
                    input {
                        id: "form-ui-transparency",
                        class: "form-input",
                        r#type: "range",
                        min: "0.0",
                        max: "1.0",
                        step: "0.05",
                        value: config.read().ui_transparency.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f32>() {
                                let mut cfg = config.write();
                                                cfg.ui_transparency = v;
                            }
                        }
                    }
                    div { class: "form-hint", "How much of the desktop shows through the whole window." }
                }

                div { class: "form-group", style: "display: flex; flex-direction: column; justify-content: center;",
                    div { class: "toggle-row",
                        div { class: "toggle-info",
                            div { class: "toggle-name", "Frosted Glass Panels" }
                            div { class: "toggle-desc", "Give panels a frosted translucent surface. Off = thin see-through glass." }
                        }
                        div {
                            id: "form-ui-blur",
                            class: if config.read().ui_blur { "toggle-switch on" } else { "toggle-switch" },
                            onclick: move |_| {
                                let mut cfg = config.write();
                                                let current = cfg.ui_blur;
                                cfg.ui_blur = !current;
                            },
                            div { class: "toggle-thumb" }
                        }
                    }
                }
            }

            div { class: "form-row", style: "margin-top: 12px;",
                div { class: "form-group",
                    label { r#for: "form-ui-blur-intensity", class: "form-label", "Glass Frost Strength ({config.read().ui_blur_intensity}/64)" }
                    input {
                        id: "form-ui-blur-intensity",
                        class: "form-input",
                        r#type: "range",
                        min: "0",
                        max: "64",
                        step: "1",
                        value: config.read().ui_blur_intensity.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                let mut cfg = config.write();
                                                cfg.ui_blur_intensity = v;
                            }
                        }
                    }
                    div { class: "form-hint", "How opaque/frosted the panels are. Only active if Frosted Glass Panels is enabled." }
                }

                div { class: "form-group",
                    label { r#for: "form-app-log-level", class: "form-label", "Application Log Level" }
                    select {
                        id: "form-app-log-level",
                        class: "form-input",
                        value: config.read().app_log_level.clone(),
                        onchange: move |e: Event<FormData>| {
                            config.write().app_log_level = e.value();
                        },
                        option { value: "DEBUG", "DEBUG (All logs)" }
                        option { value: "INFO", "INFO (Standard info)" }
                        option { value: "WARN", "WARN (Warnings only)" }
                        option { value: "ERROR", "ERROR (Errors only)" }
                    }
                    div { class: "form-hint", "Tiered console logs verbosity for debugging and auditing." }
                }
            }

            // Theme Color Pickers Row
            div { style: "margin-top: 18px; padding-top: 14px; border-top: 1px solid var(--color-hairline);",
                div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;",
                    div { class: "card-title", style: "font-size: 13px; margin: 0;", "Customize Colors" }
                    div { style: "font-size: 11px; color: var(--color-muted);", "Editing: {config.read().theme_name}" }
                }

                div { class: "form-row",
                    div { class: "form-group",
                        label { r#for: "form-color-tint", class: "form-label", "Background Tint" }
                        div { class: "input-group",
                            input {
                                id: "form-color-tint",
                                class: "form-input",
                                r#type: "color",
                                value: config.read().ui_background_color.clone(),
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_background_color = e.value();
                                }
                            }
                            input {
                                class: "form-input",
                                r#type: "text",
                                style: "width: 90px; font-size: 12px;",
                                value: config.read().ui_background_color.clone(),
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_background_color = e.value();
                                }
                            }
                        }
                    }

                    div { class: "form-group",
                        label { r#for: "form-color-text", class: "form-label", "Text Color" }
                        div { class: "input-group",
                            input {
                                id: "form-color-text",
                                class: "form-input",
                                r#type: "color",
                                value: config.read().ui_text_color.clone(),
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_text_color = e.value();
                                }
                            }
                            input {
                                class: "form-input",
                                r#type: "text",
                                style: "width: 90px; font-size: 12px;",
                                value: config.read().ui_text_color.clone(),
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_text_color = e.value();
                                }
                            }
                        }
                    }

                    div { class: "form-group",
                        label { r#for: "form-color-accent", class: "form-label", "Accent Color" }
                        div { class: "input-group",
                            input {
                                id: "form-color-accent",
                                class: "form-input",
                                r#type: "color",
                                value: config.read().ui_accent_color.clone(),
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_accent_color = e.value();
                                }
                            }
                            input {
                                class: "form-input",
                                r#type: "text",
                                style: "width: 90px; font-size: 12px;",
                                value: config.read().ui_accent_color.clone(),
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_accent_color = e.value();
                                }
                            }
                        }
                    }
                }

                div { class: "form-row", style: "margin-top: 12px;",
                    div { class: "form-group",
                        label { r#for: "form-color-border", class: "form-label", "Border / Hairline Color" }
                        div { class: "input-group",
                            input {
                                id: "form-color-border",
                                class: "form-input",
                                r#type: "color",
                                value: if config.read().ui_border_color.starts_with("rgba") { "#475569".to_string() } else { config.read().ui_border_color.clone() },
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_border_color = e.value();
                                }
                            }
                            input {
                                class: "form-input",
                                r#type: "text",
                                style: "width: 140px; font-size: 12px;",
                                value: config.read().ui_border_color.clone(),
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_border_color = e.value();
                                }
                            }
                        }
                    }

                    div { class: "form-group",
                        label { r#for: "form-color-card", class: "form-label", "Card Background Color" }
                        div { class: "input-group",
                            input {
                                id: "form-color-card",
                                class: "form-input",
                                r#type: "color",
                                value: if config.read().ui_card_bg.starts_with("rgba") { "#1e293b".to_string() } else { config.read().ui_card_bg.clone() },
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_card_bg = e.value();
                                }
                            }
                            input {
                                class: "form-input",
                                r#type: "text",
                                style: "width: 140px; font-size: 12px;",
                                value: config.read().ui_card_bg.clone(),
                                oninput: move |e: Event<FormData>| {
                                    let mut cfg = config.write();
                                                        cfg.ui_card_bg = e.value();
                                }
                            }
                        }
                    }
                }

                div { class: "form-group", style: "margin-top: 12px;",
                    label { r#for: "form-color-sidebar", class: "form-label", "Sidebar Background CSS Color/Gradient" }
                    input {
                        id: "form-color-sidebar",
                        class: "form-input",
                        r#type: "text",
                        value: config.read().ui_sidebar_bg.clone(),
                        oninput: move |e: Event<FormData>| {
                            let mut cfg = config.write();
                                        cfg.ui_sidebar_bg = e.value();
                        }
                    }
                    div { class: "form-hint", "Accepts solid color hex or CSS gradient formats (e.g. linear-gradient(135deg, rgba(255,255,255,0.04) 0%, rgba(255,255,255,0.01) 100%))" }
                }
            }

            // Save theme sub-card
            div { style: "margin-top: 18px; padding: 16px; background: var(--color-surface-soft); border: 1px solid var(--color-hairline); border-radius: 12px;",
                div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px;",
                    div { class: "card-title", style: "font-size: 12px; margin: 0; color: var(--color-brand-accent);", "Save as Named Theme" }
                    if theme_is_modified {
                        span { style: "font-size: 11px; background: var(--color-badge-info-bg); color: var(--color-badge-info-text); padding: 2px 8px; border-radius: 20px; font-weight: 600;", "Modified" }
                    }
                }
                div { style: "font-size: 11.5px; color: var(--color-muted); margin-bottom: 12px;",
                    "Give your current customization a name to save it. You can overwrite existing names."
                }
                div { style: "display: flex; gap: 10px; align-items: center;",
                    input {
                        class: "form-input",
                        style: "flex: 1;",
                        placeholder: "e.g. My Cyberpunk, Dark Office...",
                        value: new_theme_name(),
                        onfocus: move |_| {
                            if new_theme_name.read().is_empty() {
                                let current = config.read().theme_name.clone();
                                if current != "default" && current != "custom" {
                                    new_theme_name.set(current);
                                }
                            }
                        },
                        oninput: move |e: Event<FormData>| new_theme_name.set(e.value()),
                    }
                    button {
                        class: "btn btn-start",
                        style: "padding: 8px 18px; min-height: 38px; margin: 0; white-space: nowrap;",
                        disabled: new_theme_name.read().trim().is_empty() || new_theme_name.read().trim() == "default" || new_theme_name.read().trim() == "custom",
                        onclick: move |_| {
                            let name = new_theme_name.read().trim().to_string();
                            if !name.is_empty() && name != "default" && name != "custom" {
                                let mut cfg = config.write();
                                let (bg, fg, acc, card, sbar, border, font, trans, bl, bl_int) = get_resolved_values(&cfg);
                                let new_theme = CustomTheme {
                                    name: name.clone(),
                                    background_color: bg,
                                    text_color: fg,
                                    accent_color: acc,
                                    card_bg: card,
                                    sidebar_bg: sbar,
                                    border_color: border,
                                    font_family: font,
                                    transparency: trans,
                                    blur: bl,
                                    blur_intensity: bl_int,
                                };
                                if let Some(pos) = cfg.custom_themes.iter().position(|t| t.name == name) {
                                    cfg.custom_themes[pos] = new_theme;
                                } else {
                                    cfg.custom_themes.push(new_theme);
                                }
                                cfg.theme_name = name;
                                new_theme_name.set(String::new());
                            }
                        },
                        "Save Theme"
                    }
                }

                // List saved custom themes
                if !config.read().custom_themes.is_empty() {
                    div { style: "margin-top: 16px; display: flex; flex-direction: column; gap: 8px;",
                        div { style: "font-size: 10.5px; color: var(--color-muted); font-weight: 700; text-transform: uppercase; letter-spacing: 0.07em; margin-bottom: 2px;", "Saved Themes" }
                        for custom in config.read().custom_themes.clone().iter() {
                            div {
                                key: "{custom.name}",
                                style: "display: flex; justify-content: space-between; align-items: center; padding: 9px 12px; background: var(--color-overlay-1); border-radius: 8px; border: 1px solid var(--color-hairline); cursor: pointer;",
                                onclick: {
                                    let name = custom.name.clone();
                                    move |_| {
                                        let mut cfg = config.write();
                                        cfg.theme_name = name.clone();
                                        let custom = cfg.custom_themes.iter().find(|t| t.name == name).cloned();
                                        if let Some(custom) = custom {
                                            apply_custom_to_config(&mut cfg, &custom);
                                        }
                                    }
                                },
                                div { style: "display: flex; align-items: center; gap: 10px;",
                                    // Color swatches
                                    div { style: "display: flex; gap: 3px;",
                                        div { style: "width: 12px; height: 12px; border-radius: 3px; background: {custom.background_color}; border: 1px solid var(--color-hairline-soft);" }
                                        div { style: "width: 12px; height: 12px; border-radius: 3px; background: {custom.accent_color}; border: 1px solid var(--color-hairline-soft);" }
                                        div { style: "width: 12px; height: 12px; border-radius: 3px; background: {custom.text_color}; border: 1px solid var(--color-hairline-soft);" }
                                    }
                                    span { style: "font-size: 13px; font-weight: 600; color: var(--color-ink);", "{custom.name}" }
                                    if config.read().theme_name == custom.name {
                                        span { style: "font-size: 10px; background: var(--color-badge-info-bg); color: var(--color-badge-info-text); padding: 1px 7px; border-radius: 20px; font-weight: 700;", "Active" }
                                    }
                                }
                                div { style: "display: flex; gap: 6px;",
                                    button {
                                        style: "background: var(--color-badge-error-bg); border: 1px solid var(--color-badge-error-border); cursor: pointer; color: var(--color-badge-error-text); font-size: 11px; padding: 3px 10px; border-radius: 6px; transition: all 0.2s;",
                                        onclick: {
                                            let name = custom.name.clone();
                                            move |e| {
                                                e.stop_propagation();
                                                let mut cfg = config.write();
                                                if let Some(pos) = cfg.custom_themes.iter().position(|t| t.name == name) {
                                                    cfg.custom_themes.remove(pos);
                                                }
                                                if cfg.theme_name == name {
                                                    cfg.theme_name = "default".to_string();
                                                    let defaults = ServerConfig::default();
                                                    cfg.ui_background_color = defaults.ui_background_color;
                                                    cfg.ui_text_color = defaults.ui_text_color;
                                                    cfg.ui_accent_color = defaults.ui_accent_color;
                                                    cfg.ui_card_bg = defaults.ui_card_bg;
                                                    cfg.ui_sidebar_bg = defaults.ui_sidebar_bg;
                                                    cfg.ui_border_color = defaults.ui_border_color;
                                                    cfg.ui_font_family = defaults.ui_font_family;
                                                    cfg.ui_transparency = defaults.ui_transparency;
                                                    cfg.ui_blur = defaults.ui_blur;
                                                    cfg.ui_blur_intensity = defaults.ui_blur_intensity;
                                                }
                                            }
                                        },
                                        "Delete"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Card 2: Sidebar Navigation Favorites
        div { class: "card",
            div { class: "card-title", "Sidebar Navigation & Favorites" }
            div { class: "form-hint", style: "margin-bottom: 12px;", "Choose which sections appear in the 'Favorites' group at the top of your sidebar for quick access." }

            div {
                style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 10px;",
                for tab in &[
                    Tab::Chat, Tab::Planner, Tab::Monitor, Tab::Mcp, Tab::Agents,
                    Tab::Model, Tab::Library, Tab::Download, Tab::Instances,
                    Tab::Server, Tab::Context, Tab::Gpu, Tab::Performance, Tab::Sampling, Tab::Advanced, Tab::Api,
                    Tab::Calendar, Tab::Todos, Tab::QuickNotes, Tab::Compare, Tab::DeepResearch
                ] {
                    div {
                        key: "fav-mgr-{tab.as_str()}",
                        style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: var(--color-surface-soft); border: 1px solid var(--color-hairline); border-radius: 8px;",
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            span { style: "font-size: 14px;", {tab.icon()} }
                            span { style: "font-size: 12.5px; font-weight: 500; color: var(--color-ink);", {tab.label()} }
                        }
                        button {
                            style: "background: none; border: none; cursor: pointer; font-size: 16px; transition: transform 0.2s;",
                            class: if config.read().sidebar_favorites.contains(&tab.as_str().to_string()) { "sidebar-fav-btn-active" } else { "" },
                            onclick: {
                                let t = *tab;
                                move |_| {
                                    let mut current = config.read().sidebar_favorites.clone();
                                    let t_str = t.as_str().to_string();
                                    if let Some(pos) = current.iter().position(|x| x == &t_str) {
                                        current.remove(pos);
                                    } else {
                                        current.push(t_str);
                                    }
                                    config.write().sidebar_favorites = current;
                                }
                            },
                            if config.read().sidebar_favorites.contains(&tab.as_str().to_string()) { "★" } else { "☆" }
                        }
                    }
                }
            }
        }

        // Card 3: Model Scan Directories
        div { class: "card",
            div { class: "card-title", "Scan Directories" }
            div { class: "form-group",
                label { r#for: "form-scan-directories-1", class: "form-label", "Directories to Scan" }
                div { class: "input-group",
                    div {
                        class: "tag-input-container form-input",
                        style: "display: flex; flex-wrap: wrap; align-items: center; gap: 6px; padding: 6px 10px; min-height: 38px; cursor: text;",
                        for (idx, dir) in config.read().model_scan_dirs.iter().enumerate() {
                            div {
                                key: "{dir}",
                                class: "tag-badge",
                                span { {dir.clone()} }
                                button {
                                    r#type: "button",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        config.write().model_scan_dirs.remove(idx);
                                        *scan_trigger.write() += 1;
                                    },
                                    "\u{2715}"
                                }
                            }
                        }
                        input {
                            id: "form-scan-directories-1",
                            autofocus: search_target.read().as_str() == "form-scan-directories-1",
                            r#type: "text",
                            style: "border: none; outline: none; background: transparent; color: var(--color-ink); font-size: 14px; flex: 1; min-width: 150px; padding: 2px 0;",
                            value: input_text(),
                            placeholder: if config.read().model_scan_dirs.is_empty() { "Type directory path and press Enter or comma..." } else { "" },
                            oninput: move |e: Event<FormData>| {
                                let val = e.value();
                                if val.contains(',') {
                                    let parts: Vec<String> = val.split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();

                                    let last_is_incomplete = !val.ends_with(',');
                                    let mut to_add = parts.clone();
                                    let mut remainder = String::new();
                                    if last_is_incomplete && !parts.is_empty() {
                                        remainder = to_add.pop().unwrap();
                                    }

                                    if !to_add.is_empty() {
                                        let mut current = config.read().model_scan_dirs.clone();
                                        let mut added = false;
                                        for p in to_add {
                                            if !current.contains(&p) {
                                                current.push(p);
                                                added = true;
                                            }
                                        }
                                        if added {
                                            config.write().model_scan_dirs = current;
                                            *scan_trigger.write() += 1;
                                        }
                                    }
                                    input_text.set(remainder);
                                } else {
                                    input_text.set(val);
                                }
                            },
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Enter {
                                    let val = input_text.read().trim().to_string();
                                    if !val.is_empty() {
                                        let mut current = config.read().model_scan_dirs.clone();
                                        if !current.contains(&val) {
                                            current.push(val);
                                            config.write().model_scan_dirs = current;
                                            *scan_trigger.write() += 1;
                                        }
                                    }
                                    input_text.set(String::new());
                                } else if e.key() == Key::Backspace && input_text.read().is_empty() {
                                    let mut current = config.read().model_scan_dirs.clone();
                                    if !current.is_empty() {
                                        current.pop();
                                        config.write().model_scan_dirs = current;
                                        *scan_trigger.write() += 1;
                                    }
                                }
                            },
                            onblur: move |_| {
                                let val = input_text.read().trim().to_string();
                                if !val.is_empty() {
                                    let mut current = config.read().model_scan_dirs.clone();
                                    if !current.contains(&val) {
                                        current.push(val.clone());
                                        config.write().model_scan_dirs = current;
                                        *scan_trigger.write() += 1;
                                    }
                                }
                                input_text.set(String::new());
                            }
                        }
                    }
                    button { class: "btn-browse", onclick: browse_dir, "Browse & Add" }
                }
                div { class: "form-hint", "List of directories to search for models. Type paths and separate with commas or Enter." }
            }
        }

        // Card 4: Deep Research Search Engine Integration
        div { class: "card",
            div { class: "card-title", "Search Engine Integration" }
            div { class: "form-group",
                label { r#for: "form-searxng-service-url-1", class: "form-label", "SearXNG Service URL" }
                input {
                    id: "form-searxng-service-url-1",
                    autofocus: search_target.read().as_str() == "form-searxng-service-url-1",
                    class: "form-input",
                    value: config.read().searxng_url.clone(),
                    placeholder: "http://localhost:8888",
                    oninput: move |e: Event<FormData>| config.write().searxng_url = e.value(),
                }
                div { class: "form-hint", "SearXNG instance URL used for the Deep Research search tool and automatic model library enrichment." }
            }
        }
    }
}
