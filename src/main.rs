#![allow(non_snake_case)]

mod config;
mod library;

use config::*;
use dioxus::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

// ─────────────────────────────────────────────────────────────────────────────
// Tabs
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Chat,
    Model,
    Library,
    Download,
    Server,
    Context,
    Gpu,
    Performance,
    Sampling,
    Advanced,
    Api,
}

impl Tab {
    fn label(&self) -> &str {
        match self {
            Self::Chat => "Chat",
            Self::Model => "Model",
            Self::Library => "Model Index",
            Self::Download => "Downloader",
            Self::Server => "Server",
            Self::Context => "Context",
            Self::Gpu => "GPU & Memory",
            Self::Performance => "Performance",
            Self::Sampling => "Sampling",
            Self::Advanced => "Advanced",
            Self::Api => "API & Security",
        }
    }
    fn icon(&self) -> &str {
        match self {
            Self::Chat => "\u{1F4AC}",        // 💬
            Self::Model => "\u{1F4E6}",       // 📦
            Self::Library => "\u{1F4DA}",     // 📚
            Self::Download => "\u{1F4E5}",    // 📥
            Self::Server => "\u{1F310}",       // 🌐
            Self::Context => "\u{1F4CF}",      // 📏
            Self::Gpu => "\u{1F3AE}",          // 🎮
            Self::Performance => "\u{26A1}",   // ⚡
            Self::Sampling => "\u{1F3B2}",     // 🎲
            Self::Advanced => "\u{1F527}",     // 🔧
            Self::Api => "\u{1F512}",          // 🔒
        }
    }
    fn all() -> &'static [Tab] {
        &[
            Tab::Chat,
            Tab::Model,
            Tab::Library,
            Tab::Download,
            Tab::Server,
            Tab::Context,
            Tab::Gpu,
            Tab::Performance,
            Tab::Sampling,
            Tab::Advanced,
            Tab::Api,
        ]
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
        ConfigSearchEntry { label: "llama-server path".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-llama-server-path-1".into() },
        ConfigSearchEntry { label: "Model File (.gguf)".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-model-file-gguf-1".into() },
        ConfigSearchEntry { label: "Model Alias".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-model-alias-1".into() },
        ConfigSearchEntry { label: "Model Directory".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-model-directory-models-dir-1".into() },
        ConfigSearchEntry { label: "Model URL".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-model-url-1".into() },
        ConfigSearchEntry { label: "HuggingFace Repo".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-huggingface-repo-1".into() },
        ConfigSearchEntry { label: "HuggingFace File".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-huggingface-file-1".into() },
        ConfigSearchEntry { label: "HuggingFace Token".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-huggingface-token-1".into() },
        ConfigSearchEntry { label: "Chat Template".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-chat-template-1".into() },
        ConfigSearchEntry { label: "System Prompt".into(), section: "Model".into(), tab: Tab::Model, target_id: "form-system-prompt-1".into() },
        ConfigSearchEntry { label: "Host".into(), section: "Server".into(), tab: Tab::Server, target_id: "form-host-1".into() },
        ConfigSearchEntry { label: "Port".into(), section: "Server".into(), tab: Tab::Server, target_id: "form-port-1".into() },
        ConfigSearchEntry { label: "Timeout".into(), section: "Server".into(), tab: Tab::Server, target_id: "form-server-timeout-s-1".into() },
        ConfigSearchEntry { label: "HTTP Threads".into(), section: "Server".into(), tab: Tab::Server, target_id: "form-http-threads-1".into() },
        ConfigSearchEntry { label: "Context Size".into(), section: "Context".into(), tab: Tab::Context, target_id: "form-context-size-c-1".into() },
        ConfigSearchEntry { label: "Predict".into(), section: "Context".into(), tab: Tab::Context, target_id: "form-max-predict-n-1".into() },
        ConfigSearchEntry { label: "Batch Size".into(), section: "Context".into(), tab: Tab::Context, target_id: "form-batch-size-b-1".into() },
        ConfigSearchEntry { label: "Micro Batch Size".into(), section: "Context".into(), tab: Tab::Context, target_id: "form-micro-batch-size-ub-1".into() },
        ConfigSearchEntry { label: "Parallel Sequences".into(), section: "Context".into(), tab: Tab::Context, target_id: "form-parallel-sequences-np-1".into() },
        ConfigSearchEntry { label: "GPU Layers".into(), section: "GPU & Memory".into(), tab: Tab::Gpu, target_id: "form-gpu-layers-ngl-1".into() },
        ConfigSearchEntry { label: "Main GPU".into(), section: "GPU & Memory".into(), tab: Tab::Gpu, target_id: "form-main-gpu-mg-1".into() },
        ConfigSearchEntry { label: "Split Mode".into(), section: "GPU & Memory".into(), tab: Tab::Gpu, target_id: "form-split-mode-sm-1".into() },
        ConfigSearchEntry { label: "Tensor Split".into(), section: "GPU & Memory".into(), tab: Tab::Gpu, target_id: "form-tensor-split-ts-1".into() },
        ConfigSearchEntry { label: "Cache Type K".into(), section: "GPU & Memory".into(), tab: Tab::Gpu, target_id: "form-k-cache-type-ctk-1".into() },
        ConfigSearchEntry { label: "Cache Type V".into(), section: "GPU & Memory".into(), tab: Tab::Gpu, target_id: "form-v-cache-type-ctv-1".into() },
        ConfigSearchEntry { label: "Threads".into(), section: "Performance".into(), tab: Tab::Performance, target_id: "form-threads-t-1".into() },
        ConfigSearchEntry { label: "Threads Batch".into(), section: "Performance".into(), tab: Tab::Performance, target_id: "form-batch-threads-tb-1".into() },
        ConfigSearchEntry { label: "Flash Attention".into(), section: "Performance".into(), tab: Tab::Performance, target_id: "".into() },
        ConfigSearchEntry { label: "No Warmup".into(), section: "Performance".into(), tab: Tab::Performance, target_id: "".into() },
        ConfigSearchEntry { label: "Check Tensors".into(), section: "Performance".into(), tab: Tab::Performance, target_id: "".into() },
        ConfigSearchEntry { label: "Temperature".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-temperature-temp-1".into() },
        ConfigSearchEntry { label: "Seed".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-seed-s-1".into() },
        ConfigSearchEntry { label: "Top K".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-top-k-top-k-1".into() },
        ConfigSearchEntry { label: "Top P".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-top-p-top-p-1".into() },
        ConfigSearchEntry { label: "Min P".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-min-p-min-p-1".into() },
        ConfigSearchEntry { label: "Repeat Penalty".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-repeat-penalty-1".into() },
        ConfigSearchEntry { label: "Presence Penalty".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-presence-penalty-1".into() },
        ConfigSearchEntry { label: "Frequency Penalty".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-frequency-penalty-1".into() },
        ConfigSearchEntry { label: "Grammar".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-grammar-bnf-1".into() },
        ConfigSearchEntry { label: "Grammar File".into(), section: "Sampling".into(), tab: Tab::Sampling, target_id: "form-grammar-file-1".into() },
        ConfigSearchEntry { label: "Rope Scaling".into(), section: "Advanced".into(), tab: Tab::Advanced, target_id: "form-scaling-type-1".into() },
        ConfigSearchEntry { label: "RoPE Freq Base".into(), section: "Advanced".into(), tab: Tab::Advanced, target_id: "form-freq-base-1".into() },
        ConfigSearchEntry { label: "RoPE Freq Scale".into(), section: "Advanced".into(), tab: Tab::Advanced, target_id: "form-freq-scale-1".into() },
        ConfigSearchEntry { label: "Yarn Orig Ctx".into(), section: "Advanced".into(), tab: Tab::Advanced, target_id: "form-yarn-orig-ctx-1".into() },
        ConfigSearchEntry { label: "Yarn Ext Factor".into(), section: "Advanced".into(), tab: Tab::Advanced, target_id: "form-extension-factor-1".into() },
        ConfigSearchEntry { label: "Yarn Attn Factor".into(), section: "Advanced".into(), tab: Tab::Advanced, target_id: "form-attention-factor-1".into() },
        ConfigSearchEntry { label: "Yarn Beta Fast".into(), section: "Advanced".into(), tab: Tab::Advanced, target_id: "form-beta-fast-1".into() },
        ConfigSearchEntry { label: "Yarn Beta Slow".into(), section: "Advanced".into(), tab: Tab::Advanced, target_id: "form-beta-slow-1".into() },
        ConfigSearchEntry { label: "API Key".into(), section: "API & Security".into(), tab: Tab::Api, target_id: "form-api-key-api-key-1".into() },
        ConfigSearchEntry { label: "API Key File".into(), section: "API & Security".into(), tab: Tab::Api, target_id: "form-api-key-file-api-key-file-1".into() },
        ConfigSearchEntry { label: "Metrics".into(), section: "API & Security".into(), tab: Tab::Api, target_id: "".into() },
        ConfigSearchEntry { label: "Slots".into(), section: "API & Security".into(), tab: Tab::Api, target_id: "".into() },
        ConfigSearchEntry { label: "Slot Save Path".into(), section: "API & Security".into(), tab: Tab::Api, target_id: "form-slot-save-path-1".into() },
        ConfigSearchEntry { label: "Log Format".into(), section: "API & Security".into(), tab: Tab::Api, target_id: "form-log-format-1".into() },
        ConfigSearchEntry { label: "Verbose Logging".into(), section: "API & Security".into(), tab: Tab::Api, target_id: "".into() },
        ConfigSearchEntry { label: "Model Scan Directories".into(), section: "Model Index".into(), tab: Tab::Library, target_id: "form-scan-directories-1".into() },
        ConfigSearchEntry { label: "SearxNG URL".into(), section: "Model Index".into(), tab: Tab::Library, target_id: "form-searxng-service-url-1".into() },
        ConfigSearchEntry { label: "Chat".into(), section: "Navigation".into(), tab: Tab::Chat, target_id: "".into() },
        ConfigSearchEntry { label: "Downloader".into(), section: "Navigation".into(), tab: Tab::Download, target_id: "".into() },
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// CSS
// ─────────────────────────────────────────────────────────────────────────────

const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap');

* { margin: 0; padding: 0; box-sizing: border-box; }

:root {
    --color-primary: #111111;
    --color-primary-active: #242424;
    --color-primary-disabled: #e5e7eb;
    --color-ink: #111111;
    --color-body: #374151;
    --color-muted: #6b7280;
    --color-muted-soft: #898989;
    --color-hairline: #e5e7eb;
    --color-hairline-soft: #f3f4f6;
    --color-canvas: #ffffff;
    --color-surface-soft: #f8f9fa;
    --color-surface-card: #f5f5f5;
    --color-surface-strong: #e5e7eb;
    --color-surface-dark: #101010;
    --color-surface-dark-elevated: #1a1a1a;
    --color-on-primary: #ffffff;
    --color-on-dark: #ffffff;
    --color-on-dark-soft: #a1a1aa;
    --color-brand-accent: #3b82f6;
    --color-success: #10b981;
    --color-warning: #f59e0b;
    --color-error: #ef4444;
}

body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: var(--color-canvas);
    color: var(--color-body);
    overflow: hidden;
    height: 100vh;
}

.app { display: flex; flex-direction: column; height: 100vh; }

/* ── Focus States ────────────────────────────────────────────────── */
.btn:focus-visible,
.form-input:focus-visible,
.form-select:focus-visible,
.form-textarea:focus-visible,
.sidebar-item:focus-visible,
[role="button"]:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
}

/* ── Top Bar ─────────────────────────────────────────────────────── */
.top-bar {
    display: flex; align-items: center; justify-content: space-between;
    flex-wrap: wrap; gap: 12px;
    padding: 12px 24px;
    background: var(--color-canvas);
    border-bottom: 1px solid var(--color-hairline);
    min-height: 64px;
}
.top-title {
    display: flex; align-items: center; gap: 12px;
    font-size: 20px; font-weight: 600; letter-spacing: -0.04em;
    color: var(--color-ink);
}
.top-title span { font-size: 24px; }
.top-right { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }

.search-group {
    position: relative;
    display: inline-flex;
    align-items: center;
    width: min(420px, 100%);
}
.search-input {
    width: 100%; padding: 10px 14px;
    border: 1px solid var(--color-hairline);
    border-radius: 9999px;
    background: var(--color-canvas);
    color: var(--color-ink);
    font-size: 14px;
    transition: border-color 0.2s, box-shadow 0.2s;
    font-family: 'Inter', sans-serif;
}
.search-input:focus {
    border-color: var(--color-brand-accent);
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.12);
    outline: none;
}
.search-results {
    background: var(--color-surface-card);
    border: 1px solid var(--color-hairline);
    border-radius: 16px;
    box-shadow: 0 20px 40px rgba(15, 23, 42, 0.07);
    overflow: hidden;
}
.search-results-header {
    padding: 16px 18px;
    border-bottom: 1px solid var(--color-hairline);
    font-size: 13px; font-weight: 600; color: var(--color-muted);
}
.search-results-list {
    display: flex; flex-direction: column;
}
.search-result-item {
    display: flex; align-items: center; justify-content: space-between;
    gap: 14px;
    padding: 14px 18px;
    text-decoration: none;
    color: inherit;
    transition: background 0.15s;
}
.search-result-item:hover {
    background: var(--color-surface-soft);
}
.search-result-label {
    display: flex; flex-direction: column;
    gap: 4px;
    min-width: 0;
}
.search-result-title {
    font-size: 14px; font-weight: 600; color: var(--color-ink);
}
.search-result-meta {
    font-size: 12px; color: var(--color-muted);
}
.search-chip {
    display: inline-flex; align-items: center; justify-content: center;
    padding: 6px 10px; border-radius: 9999px;
    background: var(--color-surface-soft); color: var(--color-muted);
    font-size: 12px; white-space: nowrap;
}

.status-badge {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 14px; border-radius: 9999px;
    font-size: 12px; font-weight: 600; letter-spacing: 0.02em;
    border: 1px solid transparent;
}
.status-badge.running { 
    background: rgba(16,185,129,0.08); 
    color: var(--color-success); 
    border-color: rgba(16,185,129,0.15);
}
.status-badge.stopped { 
    background: rgba(107,114,128,0.08); 
    color: var(--color-muted); 
    border-color: rgba(107,114,128,0.15);
}
.status-dot {
    width: 8px; height: 8px; border-radius: 50%;
}
.status-dot.running {
    background: var(--color-success);
    box-shadow: 0 0 8px rgba(16,185,129,0.5);
    animation: pulse 2s infinite;
}
.status-dot.stopped { background: var(--color-muted); }
@keyframes pulse { 0%,100%{opacity:1;} 50%{opacity:0.4;} }

/* ── Buttons ─────────────────────────────────────────────────────── */
.btn {
    padding: 8px 16px; border: none; border-radius: 8px;
    font-size: 14px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s, transform 0.1s;
    display: inline-flex; align-items: center; gap: 8px;
    outline: none;
    font-family: 'Inter', sans-serif;
}
.btn:active { transform: scale(0.98); }

.btn-start {
    background: var(--color-primary);
    color: var(--color-on-primary);
}
.btn-start:hover {
    background: var(--color-primary-active);
}
.btn-start:disabled {
    background: var(--color-primary-disabled);
    color: var(--color-muted);
    cursor: not-allowed;
}

.btn-stop {
    background: var(--color-error);
    color: var(--color-on-primary);
}
.btn-stop:hover {
    background: #dc2626;
}

.btn-ghost {
    background: var(--color-canvas);
    color: var(--color-ink);
    border: 1px solid var(--color-hairline);
}
.btn-ghost:hover {
    background: var(--color-surface-soft);
}

.btn-sm { padding: 6px 12px; font-size: 13px; }

.btn-browse {
    padding: 8px 14px; background: var(--color-canvas); color: var(--color-body);
    border: 1px solid var(--color-hairline); border-radius: 8px; cursor: pointer;
    font-size: 13px; font-weight: 500; font-family: 'Inter', sans-serif;
    white-space: nowrap; transition: background-color 0.2s;
    outline: none;
}
.btn-browse:hover { background: var(--color-surface-soft); }

/* ── Main area ───────────────────────────────────────────────────── */
.main { display: flex; flex: 1; overflow: hidden; }

/* ── Sidebar ─────────────────────────────────────────────────────── */
.sidebar {
    width: 220px; min-width: 220px;
    background: var(--color-surface-card);
    border-right: 1px solid var(--color-hairline);
    display: flex; flex-direction: column;
    padding: 16px 8px;
    gap: 4px;
}
.sidebar-item {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 16px; cursor: pointer;
    transition: all 0.15s; color: var(--color-muted);
    border-radius: 6px;
    font-size: 14px; font-weight: 500;
    user-select: none;
    outline: none;
    background: transparent;
    border: none;
    width: 100%;
    text-align: left;
}
.sidebar-item:hover { background: var(--color-hairline-soft); color: var(--color-ink); }
.sidebar-item.active {
    background: var(--color-canvas); color: var(--color-ink);
    font-weight: 600;
    box-shadow: 0 1px 3px rgba(0,0,0,0.05);
    border: 1px solid var(--color-hairline);
}
.sidebar-icon { font-size: 16px; width: 20px; text-align: center; }

/* ── Content ─────────────────────────────────────────────────────── */
.content {
    flex: 1; overflow-y: auto; padding: 32px;
    display: flex; flex-direction: column; gap: 24px;
    background: var(--color-canvas);
}
.content::-webkit-scrollbar { width: 8px; }
.content::-webkit-scrollbar-track { background: transparent; }
.content::-webkit-scrollbar-thumb { background: var(--color-hairline); border-radius: 4px; }

.section-title {
    font-size: 24px; font-weight: 600; letter-spacing: -0.04em; color: var(--color-ink); margin-bottom: 6px;
    font-family: 'Inter', sans-serif;
}
.section-desc {
    font-size: 14px; color: var(--color-muted); margin-bottom: 12px; line-height: 1.5;
}

/* ── Cards ────────────────────────────────────────────────────────── */
.card {
    background: var(--color-surface-card);
    border: 1px solid var(--color-hairline);
    border-radius: 12px; padding: 24px;
    display: flex; flex-direction: column; gap: 16px;
}
.card-title {
    font-size: 12px; font-weight: 600; color: var(--color-muted);
    text-transform: uppercase; letter-spacing: 0.05em;
    margin-bottom: 4px;
}

/* ── Form  ────────────────────────────────────────────────────────── */
.form-group { display: flex; flex-direction: column; gap: 6px; margin-bottom: 16px; }
.form-group:last-child { margin-bottom: 0; }
.form-label {
    font-size: 14px; font-weight: 500;
    color: var(--color-ink);
}
.form-hint { font-size: 12px; color: var(--color-muted); margin-top: 2px; line-height: 1.4; }

.form-input, .form-select {
    width: 100%; padding: 10px 14px;
    background: var(--color-canvas); border: 1px solid var(--color-hairline);
    border-radius: 8px; color: var(--color-ink); font-size: 14px;
    transition: border-color 0.2s, box-shadow 0.2s; outline: none;
    font-family: 'Inter', sans-serif;
}
.form-input:focus, .form-select:focus, .tag-input-container:focus-within {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px rgba(17,17,17,0.05);
}
.form-input::placeholder { color: var(--color-muted-soft); }

.form-textarea {
    width: 100%; padding: 10px 14px; min-height: 100px; resize: vertical;
    background: var(--color-canvas); border: 1px solid var(--color-hairline);
    border-radius: 8px; color: var(--color-ink); font-size: 14px;
    font-family: 'JetBrains Mono', monospace;
    outline: none; transition: border-color 0.2s, box-shadow 0.2s;
}
.form-textarea:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px rgba(17,17,17,0.05);
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
    background: rgba(239, 68, 68, 0.15);
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
    background: var(--color-surface-card);
    border: 1px solid var(--color-hairline);
    border-radius: 14px;
    overflow: hidden;
    box-shadow: 0 14px 40px rgba(15, 23, 42, 0.05);
}
.log-panel .log-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 16px; background: var(--color-canvas);
    border-bottom: 1px solid var(--color-hairline);
}
.log-panel .log-title {
    font-size: 12px; font-weight: 700; color: var(--color-ink);
    text-transform: uppercase; letter-spacing: 0.12em;
}
.log-panel .log-content {
    flex: 1; overflow-y: auto; padding: 14px 16px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px; line-height: 1.7;
    background: var(--color-canvas);
}
.log-panel .log-content::-webkit-scrollbar { width: 8px; }
.log-panel .log-content::-webkit-scrollbar-track { background: transparent; }
.log-panel .log-content::-webkit-scrollbar-thumb { background: rgba(148, 163, 184, 0.65); border-radius: 4px; }
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
.cmd-preview::-webkit-scrollbar-thumb { background: #222222; border-radius: 3px; }

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
    background: rgba(239,68,68,0.05); border: 1px solid rgba(239,68,68,0.2);
    color: var(--color-error); border-radius: 8px; padding: 12px 16px;
    font-size: 14px; display: flex; align-items: center; gap: 8px;
    font-weight: 500;
}
.lora-row {
    display: flex; align-items: flex-end; gap: 12px;
    padding: 12px 0; border-bottom: 1px solid var(--color-hairline);
    flex-wrap: wrap;
}
.lora-row:last-child { border-bottom: none; }
.table-row-hover:hover { background: var(--color-surface-soft); }
"#;

// ─────────────────────────────────────────────────────────────────────────────
// Entry Point
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    dioxus::launch(App);
}

// ─────────────────────────────────────────────────────────────────────────────
// App
// ─────────────────────────────────────────────────────────────────────────────

fn get_default_config_path() -> std::path::PathBuf {
    std::path::PathBuf::from("/home/notroot/Work/llama-manager")
}

fn load_default_config() -> ServerConfig {
    let dir = get_default_config_path();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("config.json");
    if path.exists() {
        if let Ok(cfg) = ServerConfig::load_from_file(&path.to_string_lossy()) {
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

#[component]
fn App() -> Element {
    // ── State ───────────────────────────────────────────────────────
    let mut config = use_signal(load_default_config);
    let mut show_config_modal = use_signal(|| false);

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

    // Auto-save config to ~/.local/share/llama-manager/config.json whenever it changes
    use_effect(move || {
        let current_config = config.read().clone();
        let dir = get_default_config_path();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.json");
        match current_config.save_to_file(&path.to_string_lossy()) {
            Ok(_) => {
                println!("[MANAGER] Auto-saved config to: {:?}", path);
            }
            Err(e) => {
                eprintln!("[MANAGER] Failed to auto-save config: {}", e);
            }
        }
    });
    let mut active_tab = use_signal(|| Tab::Chat);
    let mut logs = use_signal(Vec::<String>::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut available_models = use_signal(Vec::<String>::new);
    let mut selected_model = use_signal(String::new);
    let mut loading_models = use_signal(|| false);

    let mut search_query = use_signal(|| String::new());
    let mut search_results = use_signal(|| Vec::<ConfigSearchEntry>::new());
    let search_entries = config_search_entries();

    let mut scanned_models = use_signal(|| {
        let index_path = get_default_config_path().join("model_index.json");
        library::load_index(&index_path.to_string_lossy())
    });
    let mut index_status = use_signal(|| "Idle".to_string());
    let scan_trigger = use_signal(|| 0u64);

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
            let mut last_scan_time = std::time::Instant::now() - std::time::Duration::from_secs(600);
            
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
                                "3D", "all", "asr", "audio", "coder", "dflash", "diffusion", "embedding",
                                "embodied", "image", "infographic", "mtp", "multilingual", "multimodal",
                                "ocr", "osworld", "privacy", "research", "reshoot", "text", "timeseries",
                                "tts", "ui", "uncensored", "video", "vision", "web", "world"
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
                        }).await.unwrap_or_default();
                        
                        let index_path = get_default_config_path().join("model_index.json");
                        let _ = library::save_index(&index_path.to_string_lossy(), &updated);
                        scanned_models.set(updated);
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
                    let server_active = client.get(&ping_url)
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
                                let _ = library::save_index(&index_path.to_string_lossy(), &current);
                                scanned_models.set(current);
                            }

                            let mut model_to_enrich = scanned_models.read()[first_idx].clone();
                            index_status.set(format!("Enriching metadata for {}\u{2026}", model_to_enrich.filename));
                            
                            let searx_url = config.read().searxng_url.clone();
                            let result = library::enrich_model(&mut model_to_enrich, &searx_url, port).await;
                            
                            let mut current = scanned_models.read().clone();
                            match result {
                                Ok(_) => {
                                    index_status.set(format!("Enriched: {}", model_to_enrich.filename));
                                    current[first_idx] = model_to_enrich;
                                }
                                Err(e) => {
                                    index_status.set(format!("Enrichment failed for {}: {}", model_to_enrich.filename, e));
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
                if in_models_section && line.contains("srv   load_models:") && !line.contains("Available models") {
                    if let Some(model_name) = line.split("srv   load_models:").nth(1).map(|s| s.trim()) {
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
            if cfg.model_path.is_empty() && cfg.hf_repo.is_empty() && cfg.model_url.is_empty() && cfg.model_dir.is_empty() {
                error_msg.set(Some(
                    "Please provide a model path, HuggingFace repo, model directory, or model URL.".into(),
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

    // ── Render ──────────────────────────────────────────────────────
    rsx! {
        style { {CSS} }
        if show_config_modal() {
            div {
                style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(5, 5, 8, 0.85); backdrop-filter: blur(8px); display: flex; align-items: center; justify-content: center; z-index: 1000;",
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
            div { class: "top-bar",
                div { class: "top-title",
                    span { aria_hidden: "true", "\u{1F999}" }
                    "Llama-Manager"
                }
                div { class: "search-group",
                    input {
                        class: "search-input",
                        placeholder: "Search config entries…",
                        value: search_query(),
                        oninput: on_search_input,
                    }
                }
                div { class: "top-right",
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
            }

            // ── Body (sidebar + content) ────────────────────────────
            div { class: "main",

                // ── Sidebar ─────────────────────────────────────────
                div {
                    class: "sidebar",
                    style: "width: {sidebar_width}px; min-width: {sidebar_width}px;",
                    for tab in Tab::all() {
                        button {
                            class: if active_tab() == *tab { "sidebar-item active" } else { "sidebar-item" },
                            onclick: {
                                let t = *tab;
                                move |_| active_tab.set(t)
                            },
                            span { class: "sidebar-icon", aria_hidden: "true", {tab.icon()} }
                            {tab.label()}
                        }
                    }
                }

                // Vertical Resizer Handle
                div {
                    class: if drag_state() == DragType::Sidebar { "resizer-v dragging" } else { "resizer-v" },
                    onmousedown: on_sidebar_resizer_mousedown,
                }

                // ── Content area ────────────────────────────────────
                div { class: "content",
                    // Error toast
                    if let Some(ref msg) = *error_msg.read() {
                        div { class: "error-toast",
                            "\u{26A0}\u{FE0F} "
                            {msg.clone()}
                        }
                    }

                    if !search_matches.is_empty() {
                        div { class: "search-results",
                            div { class: "search-results-header", "Search results" }
                            div { class: "search-results-list",
                                for result in search_matches.into_iter() {
                                    a {
                                        class: "search-result-item",
                                        href: format!("#{}", result.target_id),
                                        onclick: move |_| {
                                            active_tab.set(result.tab);
                                            search_results.set(Vec::new());
                                            search_query.set(String::new());
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

                    match active_tab() {
                        Tab::Chat        => rsx! { TabChat        { config } },
                        Tab::Model       => rsx! { TabModel       { config } },
                        Tab::Library     => rsx! { TabLibrary     { config, scanned_models, index_status, scan_trigger } },
                        Tab::Download    => rsx! { TabDownload    { } },
                        Tab::Server      => rsx! { TabServer      { config } },
                        Tab::Context     => rsx! { TabContext      { config } },
                        Tab::Gpu         => rsx! { TabGpu         { config } },
                        Tab::Performance => rsx! { TabPerformance { config } },
                        Tab::Sampling    => rsx! { TabSampling    { config } },
                        Tab::Advanced    => rsx! { TabAdvanced    { config } },
                        Tab::Api         => rsx! { TabApi         { config } },
                    }

                    // Command preview
                    div { class: "card",
                        div { class: "card-title", "Command Preview" }
                        div { class: "cmd-preview", {config.read().command_preview()} }
                    }

                    // Available models
                    if is_running && !available_models.read().is_empty() {
                        div { class: "card",
                            div { class: "card-title", "Available Models" }
                            if loading_models() {
                                div { class: "form-hint", "Loading models\u{2026}" }
                            } else {
                                div { class: "form-group",
                                    label { r#for: "form-select-a-model-to-load-1", class: "form-label", "Select a model to load" }
                                    select {
                    id: "form-select-a-model-to-load-1",
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
            div {
                class: if drag_state() == DragType::LogPanel { "resizer-h dragging" } else { "resizer-h" },
                onmousedown: on_log_resizer_mousedown,
            }

            // ── Output panel integrated into main UI ───────────────
            div {
                class: "log-panel",
                style: "min-height: {log_panel_height}px;",
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

// ═════════════════════════════════════════════════════════════════════════════
//  Tab Components
// ═════════════════════════════════════════════════════════════════════════════

// ── Model ────────────────────────────────────────────────────────────────────

#[component]
fn TabModel(config: Signal<ServerConfig>) -> Element {
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
            if let Ok(Some(path)) = tokio::task::spawn_blocking(move || {
                rfd::FileDialog::new()
                    .pick_folder()
            })
            .await
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
            reason: format!("Use all available CPU cores ({}) for better throughput.", cpu_count),
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

    suggestions
}

fn apply_server_optimization(cfg: &mut ServerConfig, key: &str, value: &str) {
    match key {
        "threads" => {
            if let Ok(v) = value.parse::<u32>() { cfg.threads = v; }
        }
        "threads_batch" => {
            if let Ok(v) = value.parse::<u32>() { cfg.threads_batch = v; }
        }
        "threads_http" => {
            if let Ok(v) = value.parse::<u32>() { cfg.threads_http = v; }
        }
        "cont_batching" => {
            cfg.cont_batching = value.eq_ignore_ascii_case("on") || value.eq_ignore_ascii_case("enabled") || value.eq_ignore_ascii_case("true");
        }
        "fit" => {
            cfg.fit = value.eq_ignore_ascii_case("on") || value.eq_ignore_ascii_case("enabled") || value.eq_ignore_ascii_case("true");
        }
        "flash_attn" => {
            cfg.flash_attn = value.eq_ignore_ascii_case("on") || value.eq_ignore_ascii_case("enabled") || value.eq_ignore_ascii_case("true");
        }
        "parallel" => {
            if let Ok(v) = value.parse::<u32>() { cfg.parallel = v; }
        }
        _ => {}
    }
}

#[component]
fn TabServer(config: Signal<ServerConfig>) -> Element {
    let mut suggestions = use_signal(Vec::<OptimizationSuggestion>::new);
    let analyze = move |_| {
        suggestions.set(suggest_server_optimizations(&config.read().clone()));
    };

    let apply_selected = move |_| {
        let selected = suggestions.read().iter().filter(|s| s.selected).cloned().collect::<Vec<_>>();
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

        div { class: "card",
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
    }
}

// ── Context & Batching ───────────────────────────────────────────────────────

#[component]
fn TabContext(config: Signal<ServerConfig>) -> Element {
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
fn TabGpu(config: Signal<ServerConfig>) -> Element {
    let mut gpu_info = use_signal(|| None::<GpuInfo>);

    let _gpu_poller = use_resource(move || {
        async move {
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
        }
    });

    let vram_pct = gpu_info().map(|info| {
        if info.mem_total > 0.0 {
            info.mem_used / info.mem_total * 100.0
        } else {
            0.0
        }
    }).unwrap_or(0.0);

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
fn TabPerformance(config: Signal<ServerConfig>) -> Element {
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
fn TabSampling(config: Signal<ServerConfig>) -> Element {
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
fn TabAdvanced(config: Signal<ServerConfig>) -> Element {
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
fn TabApi(config: Signal<ServerConfig>) -> Element {
    rsx! {
        div { class: "section-title", "API & Security" }
        div { class: "section-desc", "Authentication, monitoring endpoints, and logging configuration." }

        div { class: "card",
            div { class: "card-title", "Authentication" }
            div { class: "form-group",
                label { r#for: "form-api-key-api-key-1", class: "form-label", "API Key (--api-key)" }
                input {
                    id: "form-api-key-api-key-1",
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
) -> Element {
    let mut input_text = use_signal(String::new);
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

    let browse_dir = move |_| {
        let mut config = config;
        let mut scan_trigger = scan_trigger;
        spawn(async move {
            if let Ok(Some(path)) = tokio::task::spawn_blocking(move || {
                rfd::FileDialog::new().pick_folder()
            })
            .await
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

    let force_scan = move |_| {
        *scan_trigger.write() += 1;
    };

    let is_family_collapsed = move |fam: &str| {
        collapsed_families.read().contains(&fam.to_string())
    };

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

    let is_version_collapsed = move |ver: &str| {
        collapsed_versions.read().contains(&ver.to_string())
    };

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
            button {
                class: "btn btn-ghost btn-sm",
                style: "color: var(--color-warning); border-color: var(--color-hairline);",
                onclick: enrich_all_models,
                "\u{26A1} Enrich All Models"
            }
        }

        if library_view_mode() == "index" {
            // Directories Configuration
            div { class: "card",
            div { class: "card-title", "Scan Directories" }

            div { class: "form-group",
                label { r#for: "form-scan-directories-1", class: "form-label", "Directories to Scan" }
                div { class: "input-group",
                    div {
                        class: "tag-input-container form-input",
                        style: "display: flex; flex-wrap: wrap; align-items: center; gap: 6px; padding: 6px 10px; min-height: 38px; cursor: text;",
                        onclick: move |_| {
                            // Focus can be handled naturally by the user clicking the transparent input
                        },
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

            // SearXNG URL configuration
            div { class: "form-group", style: "margin-top: 14px;",
                label { r#for: "form-searxng-service-url-1", class: "form-label", "SearXNG Service URL" }
                input {
                    id: "form-searxng-service-url-1",
                    class: "form-input",
                    value: config.read().searxng_url.clone(),
                    placeholder: "http://localhost:8888",
                    oninput: move |e: Event<FormData>| config.write().searxng_url = e.value(),
                }
                div { class: "form-hint", "SearXNG URL used to fetch internet search results for models." }
            }

            // Actions & Status
            div { style: "display: flex; align-items: center; justify-content: space-between; margin-top: 18px; padding-top: 14px; border-top: 1px solid #232640;",
                div { class: "form-hint", "Status: " span { style: "color: #a78bfa; font-weight: 600;", "{index_status()}" } }
                button { class: "btn btn-ghost btn-sm", onclick: force_scan, "\u{1F50E} Scan & Index Now" }
            }
        }

        // Indexed Models List
        div { class: "card",
            div { class: "card-title", "Indexed Models ({scanned_models.read().len()})" }

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
                                                                                                                    "background: rgba(59, 130, 246, 0.08); color: #1d4ed8; border: 1px solid rgba(59, 130, 246, 0.15);"
                                                                                                                } else {
                                                                                                                    match tag.as_str() {
                                                                                                                        "GGUF" => "background: rgba(139, 92, 246, 0.08); color: #6d28d9; border: 1px solid rgba(139, 92, 246, 0.15);",
                                                                                                                        "Safetensors" => "background: rgba(59, 130, 246, 0.08); color: #1d4ed8; border: 1px solid rgba(59, 130, 246, 0.15);",
                                                                                                                        "Uncensored" => "background: rgba(239, 68, 68, 0.08); color: #b91c1c; border: 1px solid rgba(239, 68, 68, 0.15);",
                                                                                                                        "Thinking" => "background: rgba(245, 158, 11, 0.08); color: #b45309; border: 1px solid rgba(245, 158, 11, 0.15);",
                                                                                                                        "Abliterated" => "background: rgba(249, 115, 22, 0.08); color: #c2410c; border: 1px solid rgba(249, 115, 22, 0.15);",
                                                                                                                        _ => "background: rgba(107, 114, 128, 0.08); color: #374151; border: 1px solid rgba(107, 114, 128, 0.15);"
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
                                                                                                        "Text Generation" => "rgba(124, 92, 252, 0.15); color: #c084fc",
                                                                                                        "Text Embedding" => "rgba(59, 130, 246, 0.15); color: #60a5fa",
                                                                                                        "OCR" => "rgba(234, 179, 8, 0.15); color: #facc15",
                                                                                                        "ASR" => "rgba(34, 197, 94, 0.15); color: #4ade80",
                                                                                                        "TTS" => "rgba(236, 72, 153, 0.15); color: #f472b6",
                                                                                                        "Image Generation" => "rgba(249, 115, 22, 0.15); color: #ffedd5",
                                                                                                        "Video Generation" => "rgba(6, 182, 212, 0.15); color: #22d3ee",
                                                                                                        "Coding" => "rgba(239, 68, 68, 0.15); color: #f87171",
                                                                                                        _ => "rgba(100, 116, 139, 0.15); color: #cbd5e1"
                                                                                                    };
                                                                                                    format!("padding: 2px 6px; border-radius: 10px; font-size: 10.5px; font-weight: 500; background: {}", bg)
                                                                                                },
                                                                                                {m.use_case.clone()}
                                                                                            }
                                                                                        }

                                                                                        // Column 3: Size
                                                                                        td { style: "padding: 10px 6px;",
                                                                                            div { style: "font-weight: 500; color: #cbd5e1;", {m.size_info.clone()} }
                                                                                        }

                                                                                        // Column 4: Links
                                                                                        td { style: "padding: 10px 6px;",
                                                                                            div { style: "display: flex; gap: 6px;",
                                                                                                if !m.hf_link.is_empty() {
                                                                                                    a {
                                                                                                        href: "{m.hf_link}",
                                                                                                        target: "_blank",
                                                                                                        style: "color: #3b82f6; text-decoration: none; display: inline-flex; align-items: center; gap: 2px;",
                                                                                                        "\u{1F917} HF"
                                                                                                    }
                                                                                                }
                                                                                                if !m.github_link.is_empty() {
                                                                                                    a {
                                                                                                        href: "{m.github_link}",
                                                                                                        target: "_blank",
                                                                                                        style: "color: #10b981; text-decoration: none; display: inline-flex; align-items: center; gap: 2px;",
                                                                                                        "\u{1F431} Git"
                                                                                                    }
                                                                                                }
                                                                                                if m.hf_link.is_empty() && m.github_link.is_empty() {
                                                                                                    span { style: "color: #4a5568; font-style: italic; font-size: 11px;", "None" }
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
                                                                                                    span { style: "font-size: 10.5px; color: #eab308; font-style: italic;", "Enriching\u{2026}" }
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
                let completed = enriched + failed;
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
                                                                        "enriching" => "background: rgba(139, 92, 246, 0.08); color: #6d28d9; border: 1px solid rgba(139, 92, 246, 0.15);",
                                                                        "failed" => "background: rgba(239, 68, 68, 0.08); color: #b91c1c; border: 1px solid rgba(239, 68, 68, 0.15);",
                                                                        _ => "background: rgba(245, 158, 11, 0.08); color: #b45309; border: 1px solid rgba(245, 158, 11, 0.15);"
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
                    format!("{}{}", filename[..last_dash].to_lowercase(), &filename[last_dash..])
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

fn parse_tmux_progress(lines: &[String], targets: &[TargetItem]) -> (Option<usize>, f32, String, String) {
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
                return Some(after.split_whitespace().next().unwrap_or("Unknown").to_string());
            }
        }

        if let Some(start) = lower.find("remaining") {
            let after = line[start..].split(':').nth(1).unwrap_or("").trim();
            if !after.is_empty() {
                return Some(after.split_whitespace().next().unwrap_or("Unknown").to_string());
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

            if search_terms.iter().any(|term| !term.is_empty() && lower_line.contains(term)) {
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
            script.push_str(&format!("git clone {} ./{}/{}\n", trimmed, base_dir, repo_name));
        } else if trimmed.starts_with("http") {
            let parts: Vec<&str> = trimmed.split('/').collect();
            let repo = if parts.len() > 4 { parts[4] } else { "unknown" };
            let filename = parts.last().unwrap_or(&"");

            let target_name = if filename.starts_with("mmproj") {
                let prefix = repo.to_lowercase().replace("-gguf", "");
                format!("{}-{}", prefix, filename)
            } else {
                if let Some(last_dash) = filename.rfind('-') {
                    format!("{}{}", filename[..last_dash].to_lowercase(), &filename[last_dash..])
                } else {
                    filename.to_lowercase()
                }
            };

            script.push_str(&format!("aria2c -c -x 16 -s 16 -o ./{}/{} {}\n", base_dir, target_name, trimmed));
        } else {
            let repo_name = trimmed.split('/').last().unwrap_or("");
            let target_dir = repo_name.to_lowercase();
            script.push_str(&format!("hf download {} --local-dir ./{}/{}\n", trimmed, base_dir, target_dir));
        }
    }

    script.push_str("\n# Keep tmux window open for 10 seconds after completion to see the final logs\n");
    script.push_str("sleep 10\n");
    script
}

#[component]
fn TabDownload() -> Element {
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
                }).await.unwrap_or(false);

                is_downloading.set(has_session);

                if has_session {
                    // Capture tmux pane output
                    let output = tokio::task::spawn_blocking(move || {
                        std::process::Command::new("tmux")
                            .current_dir(get_default_config_path())
                            .args(&["capture-pane", "-pt", "hf_downloads"])
                            .output()
                    }).await;

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
            }).await;

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
            }).await;

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
                            style: "background: #0f172a; color: #38bdf8; font-family: 'JetBrains Mono', monospace; font-size: 11px; padding: 12px; border-radius: 8px; overflow-y: auto; height: 180px; white-space: pre-wrap; word-break: break-all; border: 1px solid #1e293b; line-height: 1.4; text-align: left; margin-top: 8px;",
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
    mut messages: Signal<Vec<(String, String)>>,
    mut current_input: Signal<String>,
    mut is_generating: Signal<bool>,
    mut chat_error: Signal<Option<String>>,
) {
    let text = current_input.read().trim().to_string();
    if text.is_empty() || is_generating() {
        return;
    }

    messages.write().push(("user".to_string(), text.clone()));
    current_input.set(String::new());
    is_generating.set(true);
    chat_error.set(None);

    let port = config.read().port;
    let history = messages.read().clone();

    spawn(async move {
        let target_model = library::get_target_model(port, "").await;
        let llm_url = format!("http://127.0.0.1:{}/v1/chat/completions", port);

        let mut req_messages = Vec::new();
        for (role, content) in &history {
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
                        if let Some(content) = chat_res.choices
                            .and_then(|c| c.first().cloned())
                            .and_then(|c| c.message)
                            .and_then(|m| m.content)
                        {
                            messages.write().push(("assistant".to_string(), content));
                        } else {
                            chat_error.set(Some("Invalid response format received from model server.".to_string()));
                        }
                    } else {
                        chat_error.set(Some("Failed to parse JSON response from model server.".to_string()));
                    }
                } else {
                    chat_error.set(Some(format!("Model server returned error status: {}", res.status())));
                }
            }
            Err(e) => {
                chat_error.set(Some(format!("Failed to connect to model server: {}", e)));
            }
        }
        is_generating.set(false);
    });
}

#[component]
fn TabChat(config: Signal<ServerConfig>) -> Element {
    let mut messages = use_signal(Vec::<(String, String)>::new);
    let mut current_input = use_signal(String::new);
    let is_generating = use_signal(|| false);
    let mut chat_error = use_signal(|| None::<String>);
    let mut server_status = use_signal(|| false);

    let _server_checker = use_resource(move || {
        async move {
            loop {
                let port = config.read().port;
                let client = reqwest::Client::new();
                let url = format!("http://127.0.0.1:{}/v1/models", port);
                let online = match client.get(&url).timeout(std::time::Duration::from_secs(1)).send().await {
                    Ok(res) => res.status().is_success(),
                    Err(_) => false,
                };
                server_status.set(online);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    });

    let clear_chat = move |_| {
        messages.write().clear();
        chat_error.set(None);
    };

    rsx! {
        div { class: "section-title", "Interactive Model Chat" }
        div { class: "section-desc", "Have live conversations with the loaded model. Start the model server from the Server tab to begin." }

        if !server_status() {
            div {
                style: "background: var(--color-warning); color: #fff; padding: 12px; border-radius: 8px; margin-bottom: 16px; font-size: 13.5px; display: flex; align-items: center; justify-content: space-between;",
                span { "⚠️ Model server is offline. Please make sure a model is configured and click 'Start Server' in the Server tab to chat." }
            }
        }

        if let Some(ref err) = *chat_error.read() {
            div { class: "error-toast", style: "margin-bottom: 16px;",
                "❌ {err}"
            }
        }

        div {
            style: "display: flex; flex-direction: column; height: 500px; background: var(--color-surface-soft); border: 1px solid var(--color-hairline); border-radius: 12px; overflow: hidden;",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: var(--color-canvas); border-bottom: 1px solid var(--color-hairline);",
                span { style: "font-weight: 600; color: var(--color-ink);", "💬 Chat Session" }
                button {
                    class: "btn btn-outline",
                    style: "padding: 4px 12px; font-size: 12px;",
                    disabled: messages.read().is_empty(),
                    onclick: clear_chat,
                    "Clear Conversation"
                }
            }

            div {
                style: "flex-grow: 1; overflow-y: auto; padding: 16px; display: flex; flex-direction: column; gap: 12px;",
                if messages.read().is_empty() {
                    div {
                        style: "margin: auto; text-align: center; color: var(--color-muted); max-width: 320px;",
                        div { style: "font-size: 32px; margin-bottom: 8px;", "💬" }
                        div { style: "font-weight: 600; margin-bottom: 4px; color: var(--color-ink);", "Start a Conversation" }
                        div { style: "font-size: 12.5px; line-height: 1.4;", "Type a message below to query the running Llama model server in real time." }
                    }
                } else {
                    for (i, (role, text)) in messages.read().iter().enumerate() {
                        div {
                            key: "{i}",
                            style: if role == "user" { "display: flex; justify-content: flex-end;" } else { "display: flex; justify-content: flex-start;" },
                            div {
                                style: if role == "user" {
                                    "background: var(--color-brand-accent); color: #fff; padding: 10px 14px; border-radius: 14px 14px 2px 14px; max-width: 80%; font-size: 13.5px; line-height: 1.4; text-align: left;"
                                } else {
                                    "background: var(--color-canvas); color: var(--color-ink); padding: 10px 14px; border-radius: 14px 14px 14px 2px; max-width: 80%; font-size: 13.5px; line-height: 1.4; text-align: left; border: 1px solid var(--color-hairline);"
                                },
                                {text.clone()}
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
                            perform_send(config, messages, current_input, is_generating, chat_error);
                        }
                    }
                }
                button {
                    class: "btn btn-primary",
                    style: "padding: 8px 20px;",
                    disabled: !server_status() || is_generating() || current_input.read().trim().is_empty(),
                    onclick: move |_| {
                        perform_send(config, messages, current_input, is_generating, chat_error);
                    },
                    "Send"
                }
            }
        }
    }
}
