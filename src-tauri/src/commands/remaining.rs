use std::path::PathBuf;
use std::process::Stdio;
use tauri::{AppHandle, Emitter, State, Manager};
use tokio::process::Command;
use shared::ipc::{
    ScannedModel, KanbanTask, PlannerState, MonitorState, AgentStatus, AgentActivityEvent,
    CalendarEvent, CalendarState, LlamaInstance, DownloadStatus, BenchmarkResult, BenchmarkOutput,
    ResearchStatus, ResearchReportInfo, OptimizationSuggestion, Memory, EventStatus,
};
use crate::state::AppState;
use crate::config_io::config_dir;
use crate::library;
use crate::agent::AgentContext;

// ── Model Library ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn library_get_index() -> Vec<ScannedModel> {
    let index_path = config_dir().join("model_index.json");
    library::load_index(&index_path.to_string_lossy())
}

#[tauri::command]
pub async fn library_scan(state: State<'_, AppState>) -> Result<Vec<ScannedModel>, String> {
    let dirs = {
        let cfg = state.config.lock().unwrap();
        if cfg.model_scan_dirs.is_empty() {
            let mut auto_dirs = Vec::new();
            let base = "/mnt/modelsext/models";
            let subdirs = [
                "3D", "all", "asr", "audio", "coder", "dflash", "diffusion", "embedding",
                "embodied", "image", "infographic", "mtp", "multilingual", "multimodal",
                "ocr", "osworld", "privacy", "research", "reshoot", "text", "timeseries",
                "tts", "ui", "uncensored", "video", "vision", "web", "world",
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

    if dirs.is_empty() {
        return Ok(library_get_index());
    }

    let existing = library_get_index();
    let updated = tokio::task::spawn_blocking(move || {
        let scanned = library::scan_directories(&dirs);
        library::merge_indexes(scanned, existing)
    })
    .await
    .map_err(|e| e.to_string())?;

    let index_path = config_dir().join("model_index.json");
    library::save_index(&index_path.to_string_lossy(), &updated)?;
    Ok(updated)
}

#[tauri::command]
pub async fn library_enrich_single(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<ScannedModel>, String> {
    let mut index = library_get_index();
    let idx = index.iter().position(|m| m.path == path)
        .ok_or_else(|| "Model not found in index".to_string())?;

    let searx_url = state.config.lock().unwrap().searxng_url.clone();
    let port = state.config.lock().unwrap().port;

    index[idx].status = "enriching".to_string();
    let index_path = config_dir().join("model_index.json");
    library::save_index(&index_path.to_string_lossy(), &index)?;

    let mut model = index[idx].clone();
    let result = library::enrich_model(&mut model, &searx_url, port).await;

    // Reload index to apply update
    let mut index = library_get_index();
    if let Some(m_idx) = index.iter().position(|m| m.path == path) {
        match result {
            Ok(_) => {
                index[m_idx] = model;
            }
            Err(e) => {
                model.status = "failed".to_string();
                index[m_idx] = model;
                return Err(format!("Enrichment failed: {}", e));
            }
        }
        library::save_index(&index_path.to_string_lossy(), &index)?;
    }
    Ok(index)
}

#[tauri::command]
pub async fn library_enrich_all(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let index = library_get_index();
    let pending_paths: Vec<String> = index.iter()
        .filter(|m| m.status == "pending_enrichment")
        .map(|m| m.path.clone())
        .collect();

    if pending_paths.is_empty() {
        return Ok(());
    }

    let searx_url = state.config.lock().unwrap().searxng_url.clone();
    let port = state.config.lock().unwrap().port;
    let index_path = config_dir().join("model_index.json");

    tokio::spawn(async move {
        for path in pending_paths {
            // Update to enriching
            let mut index = library::load_index(&index_path.to_string_lossy());
            if let Some(idx) = index.iter().position(|m| m.path == path) {
                index[idx].status = "enriching".to_string();
                let _ = library::save_index(&index_path.to_string_lossy(), &index);
            }
            let _ = app.emit("library://status", format!("Enriching {}...", path));

            let mut model = ScannedModel {
                path: path.clone(),
                filename: std::path::Path::new(&path).file_name().unwrap_or_default().to_string_lossy().to_string(),
                size_bytes: 0,
                clean_name: "".into(),
                use_case: "Unknown".into(),
                hf_link: "".into(),
                github_link: "".into(),
                size_info: "".into(),
                status: "enriching".into(),
                family: "".into(),
                version: "".into(),
                tags: vec![],
            };

            // Reload model data from index
            let index = library::load_index(&index_path.to_string_lossy());
            if let Some(m) = index.iter().find(|m| m.path == path) {
                model = m.clone();
            }

            let result = library::enrich_model(&mut model, &searx_url, port).await;

            let mut index = library::load_index(&index_path.to_string_lossy());
            if let Some(idx) = index.iter().position(|m| m.path == path) {
                match result {
                    Ok(_) => {
                        index[idx] = model;
                    }
                    Err(_) => {
                        index[idx].status = "failed".to_string();
                    }
                }
                let _ = library::save_index(&index_path.to_string_lossy(), &index);
            }
        }
        let _ = app.emit("library://status", "Enrichment complete.");
    });

    Ok(())
}

// ── Planner / Kanban ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn planner_load() -> PlannerState {
    let path = config_dir().join("planner_tasks.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<PlannerState>(&content) {
                return state;
            }
        }
    }
    // Default PlannerState
    let mut default_state = PlannerState::default();
    default_state.tasks.push(KanbanTask {
        id: "task-1".to_string(),
        title: "Optimize Llama Server Parameters".to_string(),
        summary: "Tune micro-batch size and GPU layers for hardware acceleration".to_string(),
        assigned_agent: "ConfigOptimizer".to_string(),
        status: shared::ipc::TaskStatus::Todo,
        priority: "High".to_string(),
        created_at: "2026-06-03".to_string(),
    });
    default_state.tasks.push(KanbanTask {
        id: "task-2".to_string(),
        title: "Internet Research on Hermes-Agent".to_string(),
        summary: "Search repositories and documentation for core implementation examples".to_string(),
        assigned_agent: "WebResearcher".to_string(),
        status: shared::ipc::TaskStatus::InProgress,
        priority: "Medium".to_string(),
        created_at: "2026-06-03".to_string(),
    });
    let _ = planner_save(default_state.tasks.clone());
    default_state
}

#[tauri::command]
pub fn planner_save(tasks: Vec<KanbanTask>) -> Result<(), String> {
    let path = config_dir().join("planner_tasks.json");
    let state = PlannerState { tasks };
    let content = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

// ── Agent Monitor ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn monitor_load() -> MonitorState {
    let path = config_dir().join("agent_activities.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<MonitorState>(&content) {
                return state;
            }
        }
    }
    // Default initial monitor state
    let mut default_state = MonitorState::default();
    default_state.agents.push(AgentStatus {
        name: "ConfigOptimizer".to_string(),
        current_action: "Idle".to_string(),
        status: "Idle".to_string(),
        last_active: "Just now".to_string(),
    });
    default_state.agents.push(AgentStatus {
        name: "WebResearcher".to_string(),
        current_action: "Scraping LLM specifications".to_string(),
        status: "Active".to_string(),
        last_active: "Just now".to_string(),
    });
    default_state.events.push(AgentActivityEvent {
        timestamp: "22:20:10".to_string(),
        agent_name: "ConfigOptimizer".to_string(),
        message: "Finished analyzing optimization profiles".to_string(),
    });
    default_state.events.push(AgentActivityEvent {
        timestamp: "22:23:01".to_string(),
        agent_name: "WebResearcher".to_string(),
        message: "Started DuckDuckGo query: 'hermes-agent python codebase'".to_string(),
    });
    let _ = monitor_save(default_state.clone());
    default_state
}

pub fn monitor_save(state: MonitorState) -> Result<(), String> {
    let path = config_dir().join("agent_activities.json");
    let content = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn monitor_clear_events() -> Result<MonitorState, String> {
    let mut state = monitor_load();
    state.events.clear();
    monitor_save(state.clone())?;
    Ok(state)
}

// ── Calendar ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn calendar_load() -> CalendarState {
    let path = config_dir().join("calendar.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<CalendarState>(&content) {
                return state;
            }
        }
    }
    CalendarState::default()
}

#[tauri::command]
pub fn calendar_save(events: Vec<CalendarEvent>) -> Result<(), String> {
    let path = config_dir().join("calendar.json");
    let state = CalendarState { events };
    let content = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn calendar_add_event(event: CalendarEvent) -> Result<CalendarState, String> {
    let mut state = calendar_load();
    state.events.push(event);
    calendar_save(state.events.clone())?;
    Ok(state)
}

#[tauri::command]
pub fn calendar_delete_event(id: String) -> Result<CalendarState, String> {
    let mut state = calendar_load();
    state.events.retain(|e| e.id != id);
    calendar_save(state.events.clone())?;
    Ok(state)
}

pub fn log_monitor_event_internal(agent_name: &str, status: &str, current_action: &str, message: &str) {
    let mut state = monitor_load();
    
    if let Some(agent) = state.agents.iter_mut().find(|a| a.name == agent_name) {
        agent.status = status.to_string();
        agent.current_action = current_action.to_string();
        agent.last_active = "Just now".to_string();
    } else {
        state.agents.push(AgentStatus {
            name: agent_name.to_string(),
            status: status.to_string(),
            current_action: current_action.to_string(),
            last_active: "Just now".to_string(),
        });
    }
    
    let now = chrono::Local::now().to_rfc3339();
    state.events.push(AgentActivityEvent {
        timestamp: now,
        agent_name: agent_name.to_string(),
        message: message.to_string(),
    });
    
    if state.events.len() > 100 {
        state.events.drain(0..(state.events.len() - 100));
    }
    
    let _ = monitor_save(state);
}

pub fn spawn_calendar_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            
            let mut state = calendar_load();
            let mut updated = false;
            
            let now = chrono::Local::now();
            let now_str = now.format("%Y-%m-%dT%H:%M").to_string();
            
            for event in &mut state.events {
                if event.status == EventStatus::Pending && event.time <= now_str {
                    event.status = EventStatus::Firing;
                    updated = true;
                    
                    let app_clone = app.clone();
                    let prompt = event.prompt.clone();
                    let event_id = event.id.clone();
                    let event_title = event.title.clone();
                    
                    tauri::async_runtime::spawn(async move {
                        let app_state = app_clone.state::<AppState>();
                        let cfg = app_state.config.lock().unwrap().clone();
                        
                        let agent_id = crate::util::new_id("agent");
                        let handle = crate::state::AgentHandle::new(agent_id.clone());
                        app_state.register_agent(handle.clone());
                        
                        let (host, port, model) = if cfg.override_calendar.enabled {
                            let m = if cfg.override_calendar.model.is_empty() {
                                if cfg.model_alias.is_empty() { cfg.model_path.clone() } else { cfg.model_alias.clone() }
                            } else {
                                cfg.override_calendar.model.clone()
                            };
                            (cfg.override_calendar.host.clone(), cfg.override_calendar.port, m)
                        } else {
                            let m = if cfg.model_alias.is_empty() { cfg.model_path.clone() } else { cfg.model_alias.clone() };
                            (cfg.host.clone(), cfg.port, m)
                        };

                        let ctx = AgentContext {
                            app: app_clone.clone(),
                            model,
                            host,
                            port,
                            searxng_url: (!cfg.searxng_url.is_empty()).then_some(cfg.searxng_url),
                            config_dir: config_dir(),
                        };
                        
                        let start_msg = format!("Calendar Event '{}' triggered prompt: '{}'", event_title, prompt);
                        log_monitor_event_internal(&agent_id, "Active", &format!("Running: {}", event_title), &start_msg);
                        
                        let output = crate::agent::run_agent(ctx, handle, None, "CalendarTrigger".into(), prompt, 0).await;
                        app_clone.state::<AppState>().remove_agent(&agent_id);
                        
                        let mut state = calendar_load();
                        if let Some(evt) = state.events.iter_mut().find(|e| e.id == event_id) {
                            evt.status = EventStatus::Completed;
                            evt.result = Some(output.clone());
                            let _ = calendar_save(state.events.clone());
                        }
                        
                        let done_msg = format!("Calendar Event '{}' finished successfully.", event_title);
                        log_monitor_event_internal(&agent_id, "Idle", "Idle", &done_msg);
                    });
                }
            }
            
            if updated {
                let _ = calendar_save(state.events);
            }
        }
    });
}


// ── Downloader ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn download_get_targets() -> String {
    let path = config_dir().join("t");
    std::fs::read_to_string(path).unwrap_or_default()
}

#[tauri::command]
pub fn download_save_targets(targets: String) -> Result<(), String> {
    let path = config_dir().join("t");
    std::fs::write(path, targets).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_start(state: State<'_, AppState>, targets: String) -> Result<(), String> {
    download_save_targets(targets.clone())?;

    let cfg = state.config.lock().unwrap().clone();
    let models_root = if cfg.model_dir.is_empty() {
        "/mnt/modelsext/models".to_string()
    } else {
        cfg.model_dir.clone()
    };

    // Create the download script (dw.sh)
    let script_path = config_dir().join("dw.sh");
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
    script.push_str(&format!("MODELS_ROOT=\"{}\"\n", models_root));
    script.push_str("mkdir -p \"$MODELS_ROOT\"\n");
    script.push_str("cd \"$MODELS_ROOT\"\n\n");

    let mut current_category = "mtp".to_string();
    let mut current_subfolder: Option<String> = None;

    for line in targets.lines() {
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

        // Create target directory in the script
        script.push_str(&format!("mkdir -p \"$MODELS_ROOT/{}\"\n", base_dir));

        if trimmed.ends_with(".git") || trimmed.starts_with("https://github.com") {
            let mut repo_name = trimmed.split('/').last().unwrap_or("").to_string();
            if repo_name.ends_with(".git") {
                repo_name = repo_name[..repo_name.len() - 4].to_string();
            }
            script.push_str(&format!(
                "git clone \"{}\" \"$MODELS_ROOT/{}/{}\"\n",
                trimmed, base_dir, repo_name
            ));
        } else if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
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
                "if command -v aria2c &> /dev/null; then\n  aria2c -c -x 16 -s 16 -o \"{}\" -d \"$MODELS_ROOT/{}\" \"{}\"\nelse\n  wget -c -O \"$MODELS_ROOT/{}/{}\" \"{}\"\nfi\n",
                target_name, base_dir, trimmed, base_dir, target_name, trimmed
            ));
        } else {
            let repo_name = trimmed.split('/').last().unwrap_or("");
            let target_dir = repo_name.to_lowercase();
            script.push_str(&format!(
                "hf download \"{}\" --local-dir \"$MODELS_ROOT/{}/{}\"\n",
                trimmed, base_dir, target_dir
            ));
        }
    }

    script.push_str(
        "\n# Keep tmux window open for 10 seconds after completion to see the final logs\n",
    );
    script.push_str("sleep 10\n");

    std::fs::write(&script_path, script).map_err(|e| e.to_string())?;

    // Make script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).map_err(|e| e.to_string())?;
    }

    // Spawn download in tmux session 'hf_downloads'
    let _ = Command::new("tmux")
        .args(&["kill-session", "-t", "hf_downloads"])
        .output()
        .await;

    let child = Command::new("tmux")
        .args(&["new-session", "-d", "-s", "hf_downloads", "bash", &script_path.to_string_lossy()])
        .spawn()
        .map_err(|e| format!("Failed to spawn tmux session: {}", e))?;

    // Let process run in background
    tokio::spawn(async move {
        let mut c = child;
        let _ = c.wait().await;
    });

    Ok(())
}

#[tauri::command]
pub async fn download_cancel() -> Result<(), String> {
    let output = Command::new("tmux")
        .args(&["kill-session", "-t", "hf_downloads"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn download_status() -> Result<DownloadStatus, String> {
    let has_session_out = Command::new("tmux")
        .args(&["has-session", "-t", "hf_downloads"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let is_downloading = has_session_out.status.success();
    let mut log = String::new();

    if is_downloading {
        let capture_out = Command::new("tmux")
            .args(&["capture-pane", "-pt", "hf_downloads"])
            .output()
            .await
            .map_err(|e| e.to_string())?;
        log = String::from_utf8_lossy(&capture_out.stdout).to_string();
    }

    Ok(DownloadStatus { is_downloading, log })
}

// ── Instance Monitor ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn instances_detect() -> Result<Vec<LlamaInstance>, String> {
    let common_ports = vec![8080, 8000, 8001, 8002, 8003, 8004, 5000, 5001, 5002];
    let client = reqwest::Client::new();
    let mut instances = Vec::new();

    for port in common_ports {
        let url = format!("http://127.0.0.1:{}/v1/models", port);
        match tokio::time::timeout(
            std::time::Duration::from_millis(300),
            client.get(&url).send(),
        )
        .await
        {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    if let Ok(body) = response.text().await {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
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
    Ok(instances)
}

// ── MCP Tools ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn mcp_get_registry() -> Result<serde_json::Value, String> {
    let path = config_dir().join("mcp_registry.json");
    if path.exists() {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(serde_json::json!({
            "mcp_servers": {}
        }))
    }
}

#[tauri::command]
pub fn mcp_save_registry(registry: serde_json::Value) -> Result<(), String> {
    let path = config_dir().join("mcp_registry.json");
    let content = serde_json::to_string_pretty(&registry).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
}

// ── Deep Research ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn deep_research_start(
    state: State<'_, AppState>,
    query: String,
    host: String,
    port: u16,
    model: String,
) -> Result<(), String> {
    let mut child_guard = state.deep_research_child.lock().unwrap();
    if child_guard.is_some() {
        return Err("Deep research campaign is already running.".into());
    }

    let research_dir = config_dir().join("research");
    let _ = std::fs::create_dir_all(&research_dir);

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let report_filename = format!("report_{}.md", timestamp);
    let report_path = research_dir.join(report_filename);
    let log_path = research_dir.join("research_log.txt");

    let cfg = state.config.lock().unwrap().clone();
    let mut cmd = Command::new("uv");
    cmd.arg("run")
        .arg("deep_research.py")
        .arg("--query")
        .arg(&query)
        .arg("--host")
        .arg(&host)
        .arg("--port")
        .arg(&port.to_string())
        .arg("--model")
        .arg(&model)
        .arg("--output")
        .arg(&report_path)
        .arg("--log-file")
        .arg(&log_path);

    if !cfg.searxng_url.is_empty() {
        cmd.arg("--searxng-url").arg(&cfg.searxng_url);
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let child = cmd.spawn().map_err(|e| format!("Failed to spawn deep research: {}", e))?;
    *child_guard = Some(child);

    Ok(())
}

fn resolve_target_backend(cfg: &shared::ServerConfig, usecase: &str) -> (String, u16, String) {
    let ovr = match usecase {
        "planner" => Some(&cfg.override_planner),
        "calendar" => Some(&cfg.override_calendar),
        "memory" => Some(&cfg.override_memory),
        "research" => Some(&cfg.override_research),
        "compare" => Some(&cfg.override_compare),
        _ => None,
    };
    if let Some(o) = ovr {
        if o.enabled {
            let model = if o.model.is_empty() {
                if cfg.model_alias.is_empty() { cfg.model_path.clone() } else { cfg.model_alias.clone() }
            } else {
                o.model.clone()
            };
            return (o.host.clone(), o.port, model);
        }
    }
    let model = if cfg.model_alias.is_empty() { cfg.model_path.clone() } else { cfg.model_alias.clone() };
    (cfg.host.clone(), cfg.port, model)
}

async fn process_research_report_and_save(cfg: &shared::ServerConfig) -> Result<(), String> {
    let research_dir = config_dir().join("research");
    let mut reports = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&research_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if filename.starts_with("report_") {
                        reports.push(path);
                    }
                }
            }
        }
    }
    reports.sort();
    let Some(latest_report_path) = reports.last() else {
        return Err("No research reports found to process.".to_string());
    };
    
    let report_content = std::fs::read_to_string(latest_report_path)
        .map_err(|e| format!("Failed to read latest report: {}", e))?;
        
    if report_content.trim().is_empty() {
        return Err("Latest report content is empty.".to_string());
    }
    
    let (host, port, model) = resolve_target_backend(cfg, "research");
    
    let url = format!("http://{}:{}/v1/chat/completions", host, port);
    let system_prompt = "You are a knowledge extraction system. Analyze the provided research report and split it into logical, independent knowledge nodes. For each node, extract a concise title, a list of 2-4 tags, and the detailed markdown content. Respond with ONLY a valid JSON array of objects containing 'title', 'tags' (array of strings), and 'content' (detailed markdown text). Do not include markdown JSON wrapping, quotes, or conversational filler.";
    let user_prompt = format!("Deep Research Report Content:\n\n{}", report_content);
    
    let payload = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ],
        "temperature": 0.2,
    });
    
    let client = reqwest::Client::new();
    let res = client.post(&url).json(&payload).send().await
        .map_err(|e| format!("Failed to call LLM for research processing: {}", e))?;
        
    if !res.status().is_success() {
        return Err(format!("LLM returned error status: {}", res.status()));
    }
    
    #[derive(serde::Deserialize)]
    struct Choice {
        message: serde_json::Value,
    }
    #[derive(serde::Deserialize)]
    struct Response {
        choices: Vec<Choice>,
    }
    
    let v: Response = res.json().await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;
        
    let Some(choice) = v.choices.first() else {
        return Err("LLM returned empty choices.".to_string());
    };
    
    let content = choice.message["content"].as_str().unwrap_or("").trim();
    let content_clean = content.trim_start_matches("```json").trim_end_matches("```").trim().to_string();
    
    #[derive(serde::Deserialize)]
    struct ExtractedNode {
        title: String,
        tags: Vec<String>,
        content: String,
    }
    
    let nodes: Vec<ExtractedNode> = serde_json::from_str(&content_clean)
        .map_err(|e| format!("Failed to parse extracted nodes JSON: {}\nRaw: {}", e, content_clean))?;
        
    let now = chrono::Local::now().to_rfc3339();
    
    for (idx, node) in nodes.into_iter().enumerate() {
        let mem_id = format!("mem-research-{}-{}", chrono::Local::now().format("%Y%m%d%H%M%S"), idx);
        let mem = shared::ipc::Memory {
            id: mem_id,
            title: node.title,
            created_at: now.clone(),
            tags: node.tags,
            links: vec![],
            scope: "project".to_string(),
            content: node.content,
        };
        if let Err(e) = obsidian_memory_save(mem) {
            tracing::error!("Failed to save research node memory: {}", e);
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn deep_research_status(state: State<'_, AppState>) -> Result<ResearchStatus, String> {
    let mut child_guard = state.deep_research_child.lock().unwrap();
    let is_running = if let Some(child) = child_guard.as_mut() {
        match child.try_wait() {
            Ok(None) => true,
            _ => {
                *child_guard = None;
                let cfg = state.config.lock().unwrap().clone();
                tokio::spawn(async move {
                    if let Err(e) = process_research_report_and_save(&cfg).await {
                        tracing::error!("Failed to process research report: {}", e);
                    }
                });
                false
            }
        }
    } else {
        false
    };

    let log_path = config_dir().join("research").join("research_log.txt");
    let log = std::fs::read_to_string(log_path).unwrap_or_default();

    // Try to read last report
    let research_dir = config_dir().join("research");
    let mut last_report = String::new();
    if let Ok(entries) = std::fs::read_dir(research_dir) {
        let mut reports: Vec<PathBuf> = entries.flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().map_or(false, |ext| ext == "md"))
            .collect();
        reports.sort();
        if let Some(last) = reports.last() {
            last_report = std::fs::read_to_string(last).unwrap_or_default();
        }
    }

    Ok(ResearchStatus {
        is_running,
        log,
        report: last_report,
    })
}

#[tauri::command]
pub async fn deep_research_cancel(state: State<'_, AppState>) -> Result<(), String> {
    let child = {
        let mut child_guard = state.deep_research_child.lock().unwrap();
        child_guard.take()
    };
    if let Some(mut child) = child {
        let _ = child.kill().await;
        Ok(())
    } else {
        Err("No active research campaign found.".into())
    }
}

#[tauri::command]
pub fn deep_research_list_reports() -> Result<Vec<ResearchReportInfo>, String> {
    let research_dir = config_dir().join("research");
    let _ = std::fs::create_dir_all(&research_dir);
    let mut reports = Vec::new();
    if let Ok(entries) = std::fs::read_dir(research_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if filename.starts_with("report_") {
                        reports.push(ResearchReportInfo {
                            filename: filename.to_string(),
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }
    reports.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(reports)
}

#[tauri::command]
pub fn deep_research_read_report(filename: String) -> Result<String, String> {
    let path = config_dir().join("research").join(filename);
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

// ── Model Comparison / llama-benchy ──────────────────────────────────────────

#[tauri::command]
pub async fn compare_run_bench(
    state: State<'_, AppState>,
    url: String,
    model: String,
    pp: String,
    tg: String,
    runs: String,
) -> Result<BenchmarkOutput, String> {
    let local_path = config_dir().join(".venv").join("bin").join("llama-benchy");
    let mut cmd = if local_path.exists() {
        Command::new(local_path)
    } else {
        Command::new("llama-benchy")
    };
    cmd.arg("--base-url").arg(&url)
       .arg("--model").arg(&model)
       .arg("--runs").arg(&runs);
    for p in pp.split_whitespace() { cmd.arg("--pp").arg(p); }
    for t in tg.split_whitespace() { cmd.arg("--tg").arg(t); }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    // Scope child_guard to drop it before await
    {
        let mut child_guard = state.benchmark_child.lock().unwrap();
        if child_guard.is_some() {
            return Err("Benchmark is already running.".into());
        }
        let child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "llama-benchy not found. Initialize the virtual environment and install llama-benchy: python3 -m venv .venv && .venv/bin/pip install llama-benchy".to_string()
            } else {
                format!("Failed to run llama-benchy: {}", e)
            }
        })?;
        *child_guard = Some(child);
    }

    // Wait for the process to finish by taking it out of state
    let child = state.benchmark_child.lock().unwrap().take();
    let output_res = match child {
        Some(c) => c.wait_with_output().await,
        None => return Err("Benchmark was cancelled or not started".to_string()),
    };

    match output_res {
        Ok(output) => {
            let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
            let raw = if stderr_str.is_empty() {
                stdout_str.clone()
            } else {
                format!("{}\n\nSTDERR:\n{}", stdout_str, stderr_str)
            };

            let mut results = Vec::new();
            for line in stdout_str.lines() {
                let l = line.trim();
                if l.contains("pp=") && l.contains("tg_speed") {
                    let extract = |prefix: &str| -> String {
                        l.split_whitespace().find(|s| s.starts_with(prefix))
                            .map(|s| s.trim_start_matches(prefix).to_string())
                            .unwrap_or_else(|| "?".to_string())
                    };
                    results.push(BenchmarkResult {
                        pp: extract("pp="),
                        tg: extract("tg="),
                        pp_speed: extract("pp_speed="),
                        tg_speed: extract("tg_speed="),
                    });
                }
            }

            Ok(BenchmarkOutput { raw, results })
        }
        Err(e) => Err(format!("Benchmark run failed: {}", e))
    }
}

#[tauri::command]
pub async fn compare_run_eval(
    url: String,
    model: String,
    prompt: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url_chat = format!("{}/chat/completions", url.trim_end_matches('/'));
    let payload = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.7
    });

    let res = client.post(&url_chat).json(&payload).send().await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = res.status();
    if status.is_success() {
        let json: serde_json::Value = res.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;
        if let Some(txt) = json["choices"][0]["message"]["content"].as_str() {
            return Ok(txt.to_string());
        }
        return Err("Unexpected response format".to_string());
    }
    Err(format!("Status error: {}", status))
}

// ── Agent Memory ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn memory_get() -> Result<Vec<String>, String> {
    let path = config_dir().join("agent_memory.json");
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let lessons: Vec<String> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(lessons)
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
pub fn memory_clear() -> Result<(), String> {
    let path = config_dir().join("agent_memory.json");
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

#[tauri::command]
pub fn server_suggest_optimizations(config: shared::ServerConfig) -> Vec<OptimizationSuggestion> {
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4);
    let mut suggestions = Vec::new();

    if config.threads == 0 {
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
    } else if config.threads > cpu_count && cpu_count > 0 {
        suggestions.push(OptimizationSuggestion {
            key: "threads".into(),
            label: "Worker threads (-t)".into(),
            current: config.threads.to_string(),
            recommended: cpu_count.to_string(),
            reason: "Limit thread count to available CPUs to prevent oversubscription.".into(),
            selected: false,
        });
    }

    let recommended_batch = if config.threads > 0 {
        std::cmp::max(1, config.threads / 2)
    } else {
        std::cmp::max(1, cpu_count / 2)
    };
    if config.threads_batch == 0 {
        suggestions.push(OptimizationSuggestion {
            key: "threads_batch".into(),
            label: "Batch threads (-tb)".into(),
            current: "auto".into(),
            recommended: recommended_batch.to_string(),
            reason: "Use fewer batch threads than worker threads for balanced throughput and lower latency.".into(),
            selected: false,
        });
    }

    if config.threads_http == 0 {
        suggestions.push(OptimizationSuggestion {
            key: "threads_http".into(),
            label: "HTTP threads".into(),
            current: "auto".into(),
            recommended: std::cmp::max(2, cpu_count / 2).to_string(),
            reason: "Allocate API server workers without overwhelming the host.".into(),
            selected: false,
        });
    }

    if !config.cont_batching {
        suggestions.push(OptimizationSuggestion {
            key: "cont_batching".into(),
            label: "Continuous batching (-cb)".into(),
            current: "off".into(),
            recommended: "on".into(),
            reason: "Continuous batching improves request throughput for many clients.".into(),
            selected: false,
        });
    }

    if !config.fit {
        suggestions.push(OptimizationSuggestion {
            key: "fit".into(),
            label: "Fit memory (--fit)".into(),
            current: "off".into(),
            recommended: "on".into(),
            reason: "Enable memory fitting to reduce peak RAM usage for large models.".into(),
            selected: false,
        });
    }

    if !config.flash_attn {
        suggestions.push(OptimizationSuggestion {
            key: "flash_attn".into(),
            label: "Flash attention (-fa)".into(),
            current: "off".into(),
            recommended: "on".into(),
            reason: "Use flash attention when supported for faster execution.".into(),
            selected: false,
        });
    }

    if config.parallel <= 1 && cpu_count > 1 {
        suggestions.push(OptimizationSuggestion {
            key: "parallel".into(),
            label: "Parallel requests (-np)".into(),
            current: config.parallel.to_string(),
            recommended: "2".into(),
            reason: "Enable a small amount of request parallelism to improve throughput.".into(),
            selected: false,
        });
    }

    if config.ubatch_size > config.batch_size && config.batch_size > 0 {
        suggestions.push(OptimizationSuggestion {
            key: "ubatch_size".into(),
            label: "Micro-batch size (-ub)".into(),
            current: config.ubatch_size.to_string(),
            recommended: config.batch_size.to_string(),
            reason: "Micro-batch size (-ub) should not exceed physical batch size (-b). Aligning them prevents execution issues.".into(),
            selected: false,
        });
    }

    if config.ctx_size >= 16384
        && (config.cache_type_k == shared::config::CacheType::F16 || config.cache_type_v == shared::config::CacheType::F16)
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

    if !config.draft_model.is_empty() && config.draft_gpu_layers <= 0 && config.gpu_layers > 0 {
        suggestions.push(OptimizationSuggestion {
            key: "draft_gpu_layers".into(),
            label: "Draft GPU Layers".into(),
            current: "0".into(),
            recommended: "99".into(),
            reason: "Draft model should be offloaded to GPU to match main model GPU usage and speed up speculative decoding.".into(),
            selected: false,
        });
    }

    if config.no_warmup {
        suggestions.push(OptimizationSuggestion {
            key: "no_warmup".into(),
            label: "Disable Warmup Skip".into(),
            current: "skipped".into(),
            recommended: "false".into(),
            reason: "Warmup runs check tensors and load cache, reducing latency for the user's first query.".into(),
            selected: false,
        });
    }

    if config.no_mmap {
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

// ── Obsidian & Graphical Agent Memory (GAM) ──────────────────────────────────

fn get_memories_dirs() -> (PathBuf, PathBuf) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/notroot".to_string());
    let global_dir = PathBuf::from(home).join(".local/share/llama-manager/memories");
    let project_dir = config_dir().join(".llama-manager-memories");
    (global_dir, project_dir)
}

fn parse_memory(file_content: &str) -> Result<Memory, String> {
    let mut mem = Memory::default();
    if !file_content.starts_with("---") {
        return Err("Missing frontmatter prefix".to_string());
    }
    let parts: Vec<&str> = file_content.split("---\n").collect();
    if parts.len() < 3 {
        return Err("Invalid frontmatter structure".to_string());
    }
    let frontmatter = parts[1];
    let content = parts[2..].join("---\n");
    mem.content = content;

    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("id:") {
            mem.id = line["id:".len()..].trim().to_string();
        } else if line.starts_with("title:") {
            mem.title = line["title:".len()..].trim().to_string();
        } else if line.starts_with("created_at:") {
            mem.created_at = line["created_at:".len()..].trim().to_string();
        } else if line.starts_with("scope:") {
            mem.scope = line["scope:".len()..].trim().to_string();
        }
    }

    let mut current_section = "";
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("tags:") {
            current_section = "tags";
        } else if trimmed.starts_with("links:") {
            current_section = "links";
        } else if trimmed.starts_with("-") {
            let item = trimmed.trim_start_matches('-').trim().to_string();
            if !item.is_empty() {
                if current_section == "tags" {
                    mem.tags.push(item);
                } else if current_section == "links" {
                    mem.links.push(item);
                }
            }
        } else if line.starts_with("  -") {
            let item = trimmed.trim_start_matches('-').trim().to_string();
            if !item.is_empty() {
                if current_section == "tags" {
                    mem.tags.push(item);
                } else if current_section == "links" {
                    mem.links.push(item);
                }
            }
        }
    }
    
    Ok(mem)
}

fn serialize_memory(mem: &Memory) -> String {
    let mut s = String::new();
    s.push_str("---\n");
    s.push_str(&format!("id: {}\n", mem.id));
    s.push_str(&format!("title: {}\n", mem.title.replace('\n', " ")));
    s.push_str(&format!("created_at: {}\n", mem.created_at));
    s.push_str("tags:\n");
    for t in &mem.tags {
        s.push_str(&format!("  - {}\n", t));
    }
    s.push_str("links:\n");
    for l in &mem.links {
        s.push_str(&format!("  - {}\n", l));
    }
    s.push_str(&format!("scope: {}\n", mem.scope));
    s.push_str("---\n");
    s.push_str(&mem.content);
    s
}

fn load_memories_from_dir(dir: &std::path::Path, scope: &str) -> Vec<Memory> {
    let mut list = Vec::new();
    if !dir.exists() {
        return list;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(mut mem) = parse_memory(&content) {
                        mem.scope = scope.to_string();
                        list.push(mem);
                    }
                }
            }
        }
    }
    list
}

#[tauri::command]
pub fn obsidian_memories_get() -> Result<Vec<Memory>, String> {
    let (global_dir, project_dir) = get_memories_dirs();
    let mut global_mems = load_memories_from_dir(&global_dir, "global");
    let mut project_mems = load_memories_from_dir(&project_dir, "project");
    
    global_mems.append(&mut project_mems);
    global_mems.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(global_mems)
}

#[tauri::command]
pub fn obsidian_memory_save(mem: Memory) -> Result<(), String> {
    let (global_dir, project_dir) = get_memories_dirs();
    let dir = if mem.scope == "global" {
        &global_dir
    } else {
        &project_dir
    };
    let _ = std::fs::create_dir_all(dir);
    let path = dir.join(format!("{}.md", mem.id));
    let content = serialize_memory(&mem);
    std::fs::write(&path, content).map_err(|e| format!("Failed to write memory file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn obsidian_memory_delete(id: String, scope: String) -> Result<(), String> {
    let (global_dir, project_dir) = get_memories_dirs();
    let dir = if scope == "global" {
        &global_dir
    } else {
        &project_dir
    };
    let path = dir.join(format!("{}.md", id));
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| format!("Failed to delete memory file: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_skills_and_agents() -> Result<Vec<shared::ipc::SkillOrAgentFile>, String> {
    use std::path::PathBuf;
    let mut results = Vec::new();

    // 1. Scan Workspace
    let workspace_path = PathBuf::from("/home/notroot/Work/llama-manager");
    scan_dir_for_skills_and_agents(&workspace_path, "Workspace", &mut results);

    // 2. Scan Gemini Skills
    let gemini_skills = PathBuf::from("/home/notroot/.gemini/skills");
    if gemini_skills.exists() {
        scan_dir_for_skills_and_agents(&gemini_skills, "Gemini Skills", &mut results);
    }

    // 3. Scan Gemini Plugins
    let gemini_plugins = PathBuf::from("/home/notroot/.gemini/config/plugins");
    if gemini_plugins.exists() {
        scan_dir_for_skills_and_agents(&gemini_plugins, "Gemini Plugins", &mut results);
    }

    Ok(results)
}

fn scan_dir_for_skills_and_agents(
    dir: &std::path::Path,
    source: &str,
    results: &mut Vec<shared::ipc::SkillOrAgentFile>,
) {
    if !dir.exists() || !dir.is_dir() {
        return;
    }

    let ignore_names = ["node_modules", "target", ".git", "legacy", ".venv", "dist", "build"];

    fn walk(
        p: &std::path::Path,
        source: &str,
        results: &mut Vec<shared::ipc::SkillOrAgentFile>,
        ignore: &[&str],
        depth: usize,
    ) {
        if depth > 5 {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(p) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if ignore.contains(&name) {
                    continue;
                }

                if path.is_file() {
                    let lower_name = name.to_lowercase();
                    let is_skill = lower_name == "skill.md" || path.extension().map_or(false, |ext| ext == "mdc");
                    let is_agents = lower_name == "agents.md"
                        || lower_name == "claude.md"
                        || lower_name == ".cursorrules"
                        || lower_name == ".windsurfrules"
                        || lower_name == ".copilotrules"
                        || lower_name == "copilot-instructions.md"
                        || lower_name == ".ai-instructions"
                        || lower_name == ".agentrules"
                        || lower_name == ".clauderc"
                        || lower_name == ".codexrules"
                        || lower_name == "codex.json"
                        || lower_name == ".codex"
                        || lower_name == ".antigravityrules"
                        || lower_name == ".antigravity"
                        || lower_name == ".hermesrules"
                        || lower_name == "hermes.json"
                        || lower_name == ".openclawrules"
                        || lower_name == "openclaw.json";
                    if is_skill || is_agents {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            results.push(shared::ipc::SkillOrAgentFile {
                                name: name.to_string(),
                                path: path.to_string_lossy().to_string(),
                                is_skill,
                                content,
                                source: source.to_string(),
                            });
                        }
                    }
                } else if path.is_dir() {
                    walk(&path, source, results, ignore, depth + 1);
                }
            }
        }
    }

    walk(dir, source, results, &ignore_names, 0);
}


