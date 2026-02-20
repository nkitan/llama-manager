#![allow(non_snake_case)]

mod config;

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
    Model,
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
            Self::Model => "Model",
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
            Self::Model => "\u{1F4E6}",       // 📦
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
            Tab::Model,
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

// ─────────────────────────────────────────────────────────────────────────────
// CSS
// ─────────────────────────────────────────────────────────────────────────────

const CSS: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; }

body {
    font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
    background: #0f1117;
    color: #e2e8f0;
    overflow: hidden;
    height: 100vh;
}

.app { display: flex; flex-direction: column; height: 100vh; }

/* ── Top Bar ─────────────────────────────────────────────────────── */
.top-bar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 24px;
    background: linear-gradient(135deg, #151827 0%, #1e2138 100%);
    border-bottom: 1px solid #2a2d4a;
    min-height: 56px;
}
.top-title {
    display: flex; align-items: center; gap: 12px;
    font-size: 18px; font-weight: 700; letter-spacing: -0.3px;
}
.top-title span { font-size: 26px; }
.top-right { display: flex; align-items: center; gap: 16px; }
.status-badge {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 14px; border-radius: 20px;
    font-size: 12px; font-weight: 600; letter-spacing: 0.3px;
}
.status-badge.running { background: rgba(34,197,94,0.15); color: #4ade80; }
.status-badge.stopped { background: rgba(100,116,139,0.15); color: #94a3b8; }
.status-dot {
    width: 8px; height: 8px; border-radius: 50%;
}
.status-dot.running {
    background: #22c55e;
    box-shadow: 0 0 8px rgba(34,197,94,0.6);
    animation: pulse 2s infinite;
}
.status-dot.stopped { background: #64748b; }
@keyframes pulse { 0%,100%{opacity:1;} 50%{opacity:0.4;} }

/* ── Buttons ─────────────────────────────────────────────────────── */
.btn {
    padding: 8px 20px; border: none; border-radius: 8px;
    font-size: 13px; font-weight: 600; cursor: pointer;
    transition: all 0.2s; display: inline-flex; align-items: center; gap: 8px;
    outline: none;
}
.btn:active { transform: scale(0.97); }
.btn-start {
    background: linear-gradient(135deg, #22c55e, #16a34a);
    color: white; box-shadow: 0 2px 12px rgba(34,197,94,0.3);
}
.btn-start:hover { filter: brightness(1.1); }
.btn-stop {
    background: linear-gradient(135deg, #ef4444, #dc2626);
    color: white; box-shadow: 0 2px 12px rgba(239,68,68,0.3);
}
.btn-stop:hover { filter: brightness(1.1); }
.btn-ghost {
    background: rgba(124,92,252,0.1); color: #a78bfa;
    border: 1px solid rgba(124,92,252,0.25);
}
.btn-ghost:hover { background: rgba(124,92,252,0.2); }
.btn-sm { padding: 6px 14px; font-size: 12px; }
.btn-browse {
    padding: 8px 14px; background: #2a2d4a; color: #cbd5e1;
    border: 1px solid #363a5c; border-radius: 8px; cursor: pointer;
    font-size: 12px; white-space: nowrap; transition: all 0.2s;
}
.btn-browse:hover { background: #363a5c; }

/* ── Main area ───────────────────────────────────────────────────── */
.main { display: flex; flex: 1; overflow: hidden; }

/* ── Sidebar ─────────────────────────────────────────────────────── */
.sidebar {
    width: 190px; min-width: 190px;
    background: #111322;
    border-right: 1px solid #232640;
    display: flex; flex-direction: column;
    padding: 12px 0;
}
.sidebar-item {
    display: flex; align-items: center; gap: 10px;
    padding: 10px 20px; cursor: pointer;
    transition: all 0.15s; color: #64748b;
    border-left: 3px solid transparent;
    font-size: 13px; font-weight: 500;
    user-select: none;
}
.sidebar-item:hover { background: #181b30; color: #cbd5e1; }
.sidebar-item.active {
    background: #1a1e38; color: #a78bfa;
    border-left-color: #7c5cfc;
}
.sidebar-icon { font-size: 16px; width: 22px; text-align: center; }

/* ── Content ─────────────────────────────────────────────────────── */
.content {
    flex: 1; overflow-y: auto; padding: 24px 28px;
    display: flex; flex-direction: column; gap: 20px;
}
.content::-webkit-scrollbar { width: 6px; }
.content::-webkit-scrollbar-track { background: transparent; }
.content::-webkit-scrollbar-thumb { background: #2a2d4a; border-radius: 3px; }

.section-title {
    font-size: 20px; font-weight: 700; margin-bottom: 4px;
}
.section-desc {
    font-size: 13px; color: #64748b; margin-bottom: 8px; line-height: 1.5;
}

/* ── Cards ────────────────────────────────────────────────────────── */
.card {
    background: #161829;
    border: 1px solid #232640;
    border-radius: 12px; padding: 20px;
}
.card-title {
    font-size: 11px; font-weight: 700; color: #64748b;
    text-transform: uppercase; letter-spacing: 0.8px;
    margin-bottom: 16px;
}

/* ── Form  ────────────────────────────────────────────────────────── */
.form-group { margin-bottom: 14px; }
.form-group:last-child { margin-bottom: 0; }
.form-label {
    display: block; font-size: 13px; font-weight: 500;
    color: #cbd5e1; margin-bottom: 6px;
}
.form-hint { font-size: 11px; color: #4a5568; margin-top: 4px; }
.form-input, .form-select {
    width: 100%; padding: 9px 13px;
    background: #0f1117; border: 1px solid #2a2d4a;
    border-radius: 8px; color: #e2e8f0; font-size: 13px;
    transition: border-color 0.2s; outline: none;
    font-family: inherit;
}
.form-input:focus, .form-select:focus { border-color: #7c5cfc; }
.form-input::placeholder { color: #3d4260; }
.form-textarea {
    width: 100%; padding: 9px 13px; min-height: 80px; resize: vertical;
    background: #0f1117; border: 1px solid #2a2d4a;
    border-radius: 8px; color: #e2e8f0; font-size: 13px;
    font-family: 'Cascadia Code', 'Fira Code', monospace;
    outline: none; transition: border-color 0.2s;
}
.form-textarea:focus { border-color: #7c5cfc; }
.form-row { display: flex; gap: 14px; }
.form-row > * { flex: 1; }
.input-group { display: flex; gap: 8px; }
.input-group .form-input { flex: 1; }

/* ── Toggles ──────────────────────────────────────────────────────── */
.toggle-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 11px 0; border-bottom: 1px solid #1e2040;
}
.toggle-row:last-child { border-bottom: none; }
.toggle-info { display: flex; flex-direction: column; gap: 2px; }
.toggle-name { font-size: 13px; color: #e2e8f0; font-weight: 500; }
.toggle-desc { font-size: 11px; color: #4a5568; }
.toggle-switch {
    width: 40px; height: 22px; border-radius: 11px;
    background: #2a2d4a; cursor: pointer;
    position: relative; transition: background 0.25s;
    flex-shrink: 0;
}
.toggle-switch.on { background: #7c5cfc; }
.toggle-thumb {
    position: absolute; width: 16px; height: 16px;
    background: white; border-radius: 50%;
    top: 3px; left: 3px;
    transition: transform 0.25s;
}
.toggle-switch.on .toggle-thumb { transform: translateX(18px); }

/* ── Log Panel ────────────────────────────────────────────────────── */
.log-panel {
    height: 200px; min-height: 100px;
    background: #0a0b14;
    border-top: 1px solid #232640;
    display: flex; flex-direction: column;
}
.log-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 6px 16px; background: #0f1020;
    border-bottom: 1px solid #1a1d30;
    font-size: 11px; color: #64748b; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.5px;
    user-select: none;
}
.log-content {
    flex: 1; overflow-y: auto; padding: 8px 16px;
    font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
    font-size: 11.5px; line-height: 1.7;
}
.log-content::-webkit-scrollbar { width: 5px; }
.log-content::-webkit-scrollbar-track { background: transparent; }
.log-content::-webkit-scrollbar-thumb { background: #1e2040; border-radius: 3px; }
.log-line { white-space: pre-wrap; word-break: break-all; }
.log-out { color: #94a3b8; }
.log-err { color: #fca5a5; }
.log-info { color: #7dd3fc; }
.log-empty { color: #2a2d4a; font-style: italic; padding: 16px 0; }

/* ── Command preview ──────────────────────────────────────────────── */
.cmd-preview {
    background: #0a0b14; border: 1px solid #232640;
    border-radius: 8px; padding: 12px 14px;
    font-family: 'Cascadia Code', 'Fira Code', monospace;
    font-size: 11.5px; line-height: 1.6;
    color: #94a3b8; word-break: break-all;
    max-height: 80px; overflow-y: auto;
}

/* ── Misc ─────────────────────────────────────────────────────────── */
.error-toast {
    background: rgba(239,68,68,0.12); border: 1px solid rgba(239,68,68,0.3);
    color: #fca5a5; border-radius: 8px; padding: 10px 16px;
    font-size: 13px; display: flex; align-items: center; gap: 8px;
}
.lora-row {
    display: flex; align-items: flex-end; gap: 8px;
    padding: 10px 0; border-bottom: 1px solid #1e2040;
}
.lora-row:last-child { border-bottom: none; }
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

#[component]
fn App() -> Element {
    // ── State ───────────────────────────────────────────────────────
    let mut config = use_signal(ServerConfig::default);
    let mut active_tab = use_signal(|| Tab::Model);
    let mut logs = use_signal(Vec::<String>::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut available_models = use_signal(Vec::<String>::new);
    let mut selected_model = use_signal(String::new);
    let mut loading_models = use_signal(|| false);

    // Server process + shared log buffer
    let mut server_child: Signal<Option<Arc<Mutex<Child>>>> = use_signal(|| None);
    let log_buffer: Signal<Arc<Mutex<Vec<String>>>> =
        use_signal(|| Arc::new(Mutex::new(Vec::new())));

    let is_running = server_child.read().is_some();

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

    // ── Render ──────────────────────────────────────────────────────
    rsx! {
        style { {CSS} }
        div { class: "app",

            // ── Top bar ─────────────────────────────────────────────
            div { class: "top-bar",
                div { class: "top-title",
                    span { "\u{1F999}" }
                    "Llama Manager"
                }
                div { class: "top-right",
                    // Save / Load
                    button { class: "btn btn-ghost btn-sm", onclick: save_config, "Save Config" }
                    button { class: "btn btn-ghost btn-sm", onclick: load_config, "Load Config" }
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
                div { class: "sidebar",
                    for tab in Tab::all() {
                        div {
                            class: if active_tab() == *tab { "sidebar-item active" } else { "sidebar-item" },
                            onclick: {
                                let t = *tab;
                                move |_| active_tab.set(t)
                            },
                            span { class: "sidebar-icon", {tab.icon()} }
                            {tab.label()}
                        }
                    }
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

                    match active_tab() {
                        Tab::Model       => rsx! { TabModel       { config } },
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
                                    label { class: "form-label", "Select a model to load" }
                                    select {
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

            // ── Log panel ───────────────────────────────────────────
            div { class: "log-panel",
                div { class: "log-header",
                    "Output"
                    button {
                        class: "btn-browse",
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
                label { class: "form-label", "llama-server path" }
                div { class: "input-group",
                    input {
                        class: "form-input",
                        value: config.read().exe_path.clone(),
                        placeholder: "llama-server",
                        oninput: move |e: Event<FormData>| config.write().exe_path = e.value(),
                    }
                    button { class: "btn-browse", onclick: pick_exe, "Browse" }
                }
                div { class: "form-hint", "Path to the llama-server binary. Use \"llama-server\" if it is on your PATH." }
            }
        }

        // Local model
        div { class: "card",
            div { class: "card-title", "Local Model" }
            div { class: "form-group",
                label { class: "form-label", "Model File (.gguf)" }
                div { class: "input-group",
                    input {
                        class: "form-input",
                        value: config.read().model_path.clone(),
                        placeholder: "C:\\models\\my-model.gguf",
                        oninput: move |e: Event<FormData>| config.write().model_path = e.value(),
                    }
                    button { class: "btn-browse", onclick: pick_model, "Browse" }
                }
            }
            div { class: "form-group",
                label { class: "form-label", "Model Alias" }
                input {
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
                label { class: "form-label", "Model Directory (--models-dir)" }
                div { class: "input-group",
                    input {
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
                label { class: "form-label", "Model URL" }
                input {
                    class: "form-input",
                    value: config.read().model_url.clone(),
                    placeholder: "https://example.com/model.gguf",
                    oninput: move |e: Event<FormData>| config.write().model_url = e.value(),
                }
            }
            div { class: "form-row",
                div { class: "form-group",
                    label { class: "form-label", "HuggingFace Repo" }
                    input {
                        class: "form-input",
                        value: config.read().hf_repo.clone(),
                        placeholder: "org/model-name-GGUF",
                        oninput: move |e: Event<FormData>| config.write().hf_repo = e.value(),
                    }
                }
                div { class: "form-group",
                    label { class: "form-label", "HuggingFace File" }
                    input {
                        class: "form-input",
                        value: config.read().hf_file.clone(),
                        placeholder: "model-Q4_K_M.gguf",
                        oninput: move |e: Event<FormData>| config.write().hf_file = e.value(),
                    }
                }
            }
            div { class: "form-group",
                label { class: "form-label", "HuggingFace Token" }
                input {
                    class: "form-input",
                    r#type: "password",
                    value: config.read().hf_token.clone(),
                    placeholder: "hf_...",
                    oninput: move |e: Event<FormData>| config.write().hf_token = e.value(),
                }
                div { class: "form-hint", "Required for gated models. Stored locally only." }
            }
        }

        // Chat / System Prompt
        div { class: "card",
            div { class: "card-title", "Chat Template & System Prompt" }
            div { class: "form-group",
                label { class: "form-label", "Chat Template" }
                input {
                    class: "form-input",
                    value: config.read().chat_template.clone(),
                    placeholder: "e.g. chatml, llama3, gemma ...",
                    oninput: move |e: Event<FormData>| config.write().chat_template = e.value(),
                }
                div { class: "form-hint", "Override the model's built-in chat template." }
            }
            div { class: "form-group",
                label { class: "form-label", "System Prompt" }
                textarea {
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

#[component]
fn TabServer(config: Signal<ServerConfig>) -> Element {
    rsx! {
        div { class: "section-title", "Server Settings" }
        div { class: "section-desc", "Network binding and connection management." }

        div { class: "card",
            div { class: "card-title", "Network" }
            div { class: "form-row",
                div { class: "form-group",
                    label { class: "form-label", "Host" }
                    input {
                        class: "form-input",
                        value: config.read().host.clone(),
                        placeholder: "127.0.0.1",
                        oninput: move |e: Event<FormData>| config.write().host = e.value(),
                    }
                    div { class: "form-hint", "Use 0.0.0.0 to expose to the network." }
                }
                div { class: "form-group",
                    label { class: "form-label", "Port" }
                    input {
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
                    label { class: "form-label", "Server Timeout (s)" }
                    input {
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
                    label { class: "form-label", "HTTP Threads" }
                    input {
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
                    label { class: "form-label", "Context Size (-c)" }
                    input {
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
                    label { class: "form-label", "Max Predict (-n)" }
                    input {
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
                    label { class: "form-label", "Batch Size (-b)" }
                    input {
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
                    label { class: "form-label", "Micro-Batch Size (-ub)" }
                    input {
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
                label { class: "form-label", "Parallel Sequences (-np)" }
                input {
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

#[component]
fn TabGpu(config: Signal<ServerConfig>) -> Element {
    rsx! {
        div { class: "section-title", "GPU & Memory" }
        div { class: "section-desc", "Control GPU offloading, memory mapping, and KV cache quantization." }

        div { class: "card",
            div { class: "card-title", "GPU Offload" }
            div { class: "form-row",
                div { class: "form-group",
                    label { class: "form-label", "GPU Layers (-ngl)" }
                    input {
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
                    label { class: "form-label", "Main GPU (-mg)" }
                    input {
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
                    label { class: "form-label", "Split Mode (-sm)" }
                    select {
                        class: "form-select",
                        value: config.read().split_mode.as_str().to_string(),
                        onchange: move |e: Event<FormData>| config.write().split_mode = SplitMode::from_str(&e.value()),
                        option { value: "layer", "Layer (default)" }
                        option { value: "row", "Row" }
                        option { value: "none", "None" }
                    }
                }
                div { class: "form-group",
                    label { class: "form-label", "Tensor Split (-ts)" }
                    input {
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
                    label { class: "form-label", "K Cache Type (-ctk)" }
                    select {
                        class: "form-select",
                        value: config.read().cache_type_k.as_str().to_string(),
                        onchange: move |e: Event<FormData>| config.write().cache_type_k = CacheType::from_str(&e.value()),
                        option { value: "f16", "f16 (default)" }
                        option { value: "q8_0", "q8_0 (saves ~50% KV VRAM)" }
                        option { value: "q4_0", "q4_0 (saves ~75% KV VRAM)" }
                    }
                }
                div { class: "form-group",
                    label { class: "form-label", "V Cache Type (-ctv)" }
                    select {
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
                    label { class: "form-label", "Threads (-t)" }
                    input {
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
                    label { class: "form-label", "Batch Threads (-tb)" }
                    input {
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
                    label { class: "form-label", "Temperature (--temp)" }
                    input {
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
                    label { class: "form-label", "Seed (-s)" }
                    input {
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
                    label { class: "form-label", "Top-K (--top-k)" }
                    input {
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
                    label { class: "form-label", "Top-P (--top-p)" }
                    input {
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
                label { class: "form-label", "Min-P (--min-p)" }
                input {
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
                    label { class: "form-label", "Repeat Penalty" }
                    input {
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
                    label { class: "form-label", "Presence Penalty" }
                    input {
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
                label { class: "form-label", "Frequency Penalty" }
                input {
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
                label { class: "form-label", "Grammar (BNF)" }
                textarea {
                    class: "form-textarea",
                    value: config.read().grammar.clone(),
                    placeholder: "root ::= ...",
                    oninput: move |e: Event<FormData>| config.write().grammar = e.value(),
                }
            }
            div { class: "form-group",
                label { class: "form-label", "Grammar File" }
                div { class: "input-group",
                    input {
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
                label { class: "form-label", "Scaling Type" }
                select {
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
                    label { class: "form-label", "Freq Base" }
                    input {
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
                    label { class: "form-label", "Freq Scale" }
                    input {
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
                        label { class: "form-label", "YaRN Orig Ctx" }
                        input {
                            class: "form-input",
                            r#type: "number",
                            value: config.read().yarn_orig_ctx.to_string(),
                            oninput: move |e: Event<FormData>| {
                                if let Ok(v) = e.value().parse::<u32>() { config.write().yarn_orig_ctx = v; }
                            },
                        }
                    }
                    div { class: "form-group",
                        label { class: "form-label", "Extension Factor" }
                        input {
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
                        label { class: "form-label", "Attention Factor" }
                        input {
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
                        label { class: "form-label", "Beta Fast" }
                        input {
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
                        label { class: "form-label", "Beta Slow" }
                        input {
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
                            label { class: "form-label", "Adapter Path" }
                            div { class: "input-group",
                                input {
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
                            label { class: "form-label", "Scale" }
                            input {
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
                        style: "align-self: center; color: #ef4444;",
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
                label { class: "form-label", "Draft Model (-md)" }
                div { class: "input-group",
                    input {
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
                    label { class: "form-label", "Draft GPU Layers (-ngld)" }
                    input {
                        class: "form-input",
                        r#type: "number",
                        value: config.read().draft_gpu_layers.to_string(),
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<i32>() { config.write().draft_gpu_layers = v; }
                        },
                    }
                }
                div { class: "form-group",
                    label { class: "form-label", "Draft Tokens (--draft)" }
                    input {
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
                    label { class: "form-label", "Pooling Type" }
                    select {
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
                label { class: "form-label", "API Key (--api-key)" }
                input {
                    class: "form-input",
                    r#type: "password",
                    value: config.read().api_key.clone(),
                    placeholder: "Optional bearer token",
                    oninput: move |e: Event<FormData>| config.write().api_key = e.value(),
                }
                div { class: "form-hint", "If set, clients must provide this key as a Bearer token." }
            }
            div { class: "form-group",
                label { class: "form-label", "API Key File (--api-key-file)" }
                div { class: "input-group",
                    input {
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
                label { class: "form-label", "Slot Save Path" }
                div { class: "input-group",
                    input {
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
                label { class: "form-label", "Log Format" }
                select {
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
