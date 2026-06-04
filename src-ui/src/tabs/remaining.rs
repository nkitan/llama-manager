//! Implementations of the remaining tabs: Library, Downloader, Instances,
//! Agents & Memory, MCP Tools, Monitor, Planner, Calendar, Compare, and Deep Research.
//! Talk to the backend through `crate::api`.

use leptos::prelude::*;
use leptos::ev::KeyboardEvent;
use shared::ipc::{
    AgentRequest, ScannedModel, KanbanTask, MonitorState, CalendarEvent, CalendarState,
    LlamaInstance, BenchmarkOutput, ResearchStatus, ResearchReportInfo,
    TaskStatus, EventStatus, Memory, AgentEvent, AGENT_EVENT,
};
use wasm_bindgen_futures::spawn_local;
use gloo_timers::callback::Interval;

use crate::{api, ipc};
use crate::components::{Card, PageHeader};

use crate::state::{AppCtx, Tab};

// ── 1. LibraryTab / Model Index ──────────────────────────────────────────────
#[component]
pub fn LibraryTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let index = RwSignal::new(Vec::<ScannedModel>::new());
    let search = RwSignal::new(String::new());
    let status_text = RwSignal::new(String::new());
    let collapsed_families = RwSignal::new(std::collections::HashSet::<String>::new());
    let collapsed_versions = RwSignal::new(std::collections::HashSet::<String>::new());

    let load_data = move || {
        spawn_local(async move {
            if let Ok(idx) = api::library_get_index().await {
                index.set(idx);
            }
        });
    };

    // Load data on mount
    load_data();

    let scan = move |_| {
        status_text.set("Scanning directories...".to_string());
        spawn_local(async move {
            match api::library_scan().await {
                Ok(idx) => {
                    index.set(idx);
                    status_text.set("Scan complete.".to_string());
                }
                Err(e) => {
                    status_text.set(format!("Scan failed: {}", e));
                }
            }
        });
    };

    let enrich_single = move |path: String| {
        status_text.set(format!("Enriching model: {}...", path));
        let path_clone = path.clone();
        spawn_local(async move {
            match api::library_enrich_single(path_clone).await {
                Ok(idx) => {
                    index.set(idx);
                    status_text.set("Enrichment complete.".to_string());
                }
                Err(e) => {
                    status_text.set(format!("Enrichment failed: {}", e));
                }
            }
        });
    };

    let enrich_all = move |_| {
        status_text.set("Enriching all pending models in background...".to_string());
        spawn_local(async move {
            if let Err(e) = api::library_enrich_all().await {
                status_text.set(format!("Failed to start enrich all: {}", e));
            }
        });
    };

    let toggle_family = move |fam: String| {
        collapsed_families.update(|set| {
            if set.contains(&fam) {
                set.remove(&fam);
            } else {
                set.insert(fam);
            }
        });
    };

    let toggle_version = move |ver_key: String| {
        collapsed_versions.update(|set| {
            if set.contains(&ver_key) {
                set.remove(&ver_key);
            } else {
                set.insert(ver_key);
            }
        });
    };

    view! {
        <div class="page">
            <PageHeader title="Model Index" desc="Discover and enrich local model files and HuggingFace metadata."/>
            
            <Card title="Controls">
                <div class="row-actions">
                    <button class="btn primary" on:click=scan>"Scan Directories"</button>
                    <button class="btn secondary" on:click=enrich_all>"Enrich All Pending"</button>
                    <button class="btn ghost" on:click=move |_| load_data()>"Refresh List"</button>
                </div>
                {move || {
                    let text = status_text.get();
                    (!text.is_empty()).then(|| view! {
                        <div class="field-hint" style="margin-top: 10px; font-weight: 500;">
                            {text}
                        </div>
                    })
                }}
            </Card>

            <Card title="Scanned Models">
                <input
                    class="input"
                    type="text"
                    placeholder="Search by filename, family, tag, or use-case..."
                    style="margin-bottom: 16px;"
                    prop:value=move || search.get()
                    on:input=move |e| search.set(event_target_value(&e))
                />

                <div style="display: flex; flex-direction: column; gap: 8px;">
                    {move || {
                        let query = search.get().to_lowercase();
                        let filtered: Vec<ScannedModel> = index.get().into_iter().filter(|m| {
                            m.filename.to_lowercase().contains(&query) ||
                            m.clean_name.to_lowercase().contains(&query) ||
                            m.family.to_lowercase().contains(&query) ||
                            m.use_case.to_lowercase().contains(&query) ||
                            m.tags.iter().any(|t| t.to_lowercase().contains(&query))
                        }).collect();

                        let mut unique_families = Vec::new();
                        for m in &filtered {
                            let fam = if m.family.is_empty() { "Other".to_string() } else { m.family.clone() };
                            if !unique_families.contains(&fam) {
                                unique_families.push(fam);
                            }
                        }
                        unique_families.sort_by_key(|f| f.to_lowercase());

                        if unique_families.is_empty() {
                            return view! {
                                <div class="field-hint" style="text-align: center; padding: 20px;">
                                    "No models found. Try scanning your model directories."
                                </div>
                            }.into_any();
                        }

                        unique_families.into_iter().map(|family| {
                            let family_clone = family.clone();
                            let family_models: Vec<ScannedModel> = filtered.iter()
                                .filter(|m| {
                                    let fam = if m.family.is_empty() { "Other".to_string() } else { m.family.clone() };
                                    fam == family
                                })
                                .cloned()
                                .collect();
                            
                            let is_collapsed = collapsed_families.get().contains(&family);
                            let toggle_fam = family.clone();
                            
                            let mut unique_versions = Vec::new();
                            for m in &family_models {
                                let ver = if m.version.is_empty() { "Unknown".to_string() } else { m.version.clone() };
                                if !unique_versions.contains(&ver) {
                                    unique_versions.push(ver);
                                }
                            }
                            unique_versions.sort_by_key(|v| v.to_lowercase());

                            view! {
                                <div style="display: flex; flex-direction: column; gap: 6px; margin-bottom: 8px;">
                                    // Family Header
                                    <div 
                                        class="family-header"
                                        on:click=move |_| toggle_family(toggle_fam.clone())
                                    >
                                        <div style="display: flex; align-items: center; gap: 8px;">
                                            <span style="font-size: 11px; color: var(--primary);">{if is_collapsed { "▶" } else { "▼" }}</span>
                                            <span>"📁 " {family_clone.clone()} " Family"</span>
                                        </div>
                                        <span style="font-size: 11px; color: var(--muted); background: var(--surface-soft); padding: 2px 8px; border-radius: 12px; font-weight: 600; border: 1px solid var(--hairline);">
                                            {family_models.len()} " files"
                                        </span>
                                    </div>

                                    // Family Content
                                    {move || (!is_collapsed).then(|| {
                                        unique_versions.clone().into_iter().map(|version| {
                                            let version_clone = version.clone();
                                            let ver_key = format!("{}:{}", family_clone, version);
                                            let is_ver_collapsed = collapsed_versions.get().contains(&ver_key);
                                            let toggle_ver_key = ver_key.clone();
                                            
                                            let version_models: Vec<ScannedModel> = family_models.iter()
                                                .filter(|m| {
                                                    let ver = if m.version.is_empty() { "Unknown".to_string() } else { m.version.clone() };
                                                    ver == version
                                                })
                                                .cloned()
                                                .collect();

                                            view! {
                                                <div style="display: flex; flex-direction: column; gap: 6px; margin-left: 16px;">
                                                    // Version Header
                                                    <div 
                                                        class="version-header"
                                                        on:click=move |_| toggle_version(toggle_ver_key.clone())
                                                    >
                                                        <div style="display: flex; align-items: center; gap: 8px;">
                                                            <span style="font-size: 10px; color: var(--muted);">{if is_ver_collapsed { "▶" } else { "▼" }}</span>
                                                            <span>"↳ 🗄️ " {version_clone}</span>
                                                        </div>
                                                        <span style="font-size: 10.5px; color: var(--muted); font-weight: 500;">
                                                            {version_models.len()} " files"
                                                        </span>
                                                    </div>

                                                    // Version Content
                                                    {move || (!is_ver_collapsed).then(|| {
                                                        view! {
                                                            <div class="version-content">
                                                                <table class="model-table">
                                                                    <thead>
                                                                        <tr>
                                                                            <th>"Model Name / File"</th>
                                                                            <th>"Use-case"</th>
                                                                            <th>"Size"</th>
                                                                            <th>"Links"</th>
                                                                            <th style="text-align: right;">"Action"</th>
                                                                        </tr>
                                                                    </thead>
                                                                    <tbody>
                                                                        {version_models.clone().into_iter().map(|m| {
                                                                            let m_path = m.path.clone();
                                                                            let size_gb = (m.size_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
                                                                            let use_model_path = m.path.clone();
                                                                            
                                                                            let use_model = {
                                                                                let p = use_model_path.clone();
                                                                                let fname = m.filename.clone();
                                                                                move |_| {
                                                                                    let p = p.clone();
                                                                                    ctx.config.update(move |c| c.model_path = p.clone());
                                                                                    ctx.routed_model.set(Some(fname.clone()));
                                                                                    status_text.set("Active model updated!".to_string());
                                                                                    ctx.active_tab.set(Tab::Chat);

                                                                                    // Restart server if it is running
                                                                                    let ctx = ctx.clone();
                                                                                    spawn_local(async move {
                                                                                        if let Ok(_) = ctx.save_async().await {
                                                                                            if let Ok(running) = api::server_status().await {
                                                                                                if running {
                                                                                                    let _ = api::server_stop().await;
                                                                                                    let _ = api::server_start().await;
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                }
                                                                            };

                                                                            let p = m_path.clone();
                                                                            let enrich_click = move |_| {
                                                                                enrich_single(p.clone());
                                                                            };

                                                                            let is_active_row = {
                                                                                let p_check = m_path.clone();
                                                                                let filename_check = m.filename.clone();
                                                                                move || {
                                                                                    if let Some(routed) = ctx.routed_model.get() {
                                                                                        if routed == p_check || routed == filename_check {
                                                                                            return true;
                                                                                        }
                                                                                    }
                                                                                    let cfg = ctx.config.get();
                                                                                    cfg.model_path == p_check || (!cfg.model_alias.is_empty() && cfg.model_alias == filename_check)
                                                                                }
                                                                            };

                                                                            let is_active_badge = {
                                                                                let p_check = m_path.clone();
                                                                                let filename_check = m.filename.clone();
                                                                                move || {
                                                                                    if let Some(routed) = ctx.routed_model.get() {
                                                                                        if routed == p_check || routed == filename_check {
                                                                                            return true;
                                                                                        }
                                                                                    }
                                                                                    let cfg = ctx.config.get();
                                                                                    cfg.model_path == p_check || (!cfg.model_alias.is_empty() && cfg.model_alias == filename_check)
                                                                                }
                                                                            };

                                                                            view! {
                                                                                <tr class:active-model-row=is_active_row>
                                                                                    <td style="padding: 8px 6px; max-width: 280px; word-break: break-all;">
                                                                                        <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px; flex-wrap: wrap;">
                                                                                            {if m.clean_name.is_empty() || m.clean_name == "Unknown" { m.filename.clone() } else { m.clean_name.clone() }}
                                                                                            <span style="font-size: 9px; background: var(--surface-soft); color: var(--primary); padding: 1px 5px; border-radius: 4px; font-weight: 600; text-transform: uppercase; border: 1px solid var(--hairline);">
                                                                                                {m.version.clone()}
                                                                                            </span>
                                                                                            {move || is_active_badge().then(|| view! {
                                                                                                <span class="status-pill online" style="font-size: 9px; padding: 2px 6px; border-radius: 4px; font-weight: 600; text-transform: uppercase; border: none;">
                                                                                                    "Active"
                                                                                                </span>
                                                                                            })}
                                                                                        </div>
                                                                                        <div style="font-size: 10px; color: var(--muted); font-family: var(--font-mono); margin-top: 2px;">
                                                                                            {m.filename.clone()}
                                                                                        </div>
                                                                                        {(!m.tags.is_empty()).then(|| view! {
                                                                                            <div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px;">
                                                                                                {m.tags.iter().map(|tag| view! {
                                                                                                    <span class="status-pill sm" style="font-size: 9px; padding: 1px 4px; border-radius: 3px; background: var(--surface-soft); border-color: var(--hairline); color: var(--body);">
                                                                                                        {tag.clone()}
                                                                                                    </span>
                                                                                                }).collect_view()}
                                                                                            </div>
                                                                                        })}
                                                                                    </td>
                                                                                    <td style="padding: 8px 6px;">
                                                                                        {(!m.use_case.is_empty()).then(|| {
                                                                                            let bg = match m.use_case.as_str() {
                                                                                                "Text Generation" => "color-mix(in srgb, var(--primary) 12%, transparent); color: var(--primary);",
                                                                                                "Coding" => "color-mix(in srgb, #ef4444 12%, transparent); color: #ef4444;",
                                                                                                "Text Embedding" => "color-mix(in srgb, #3b82f6 12%, transparent); color: #3b82f6;",
                                                                                                _ => "var(--surface-soft); color: var(--muted);"
                                                                                            };
                                                                                            view! {
                                                                                                <span style=format!("padding: 2px 6px; border-radius: 10px; font-size: 10px; font-weight: 600; background: {}", bg)>
                                                                                                    {m.use_case.clone()}
                                                                                                </span>
                                                                                            }
                                                                                        })}
                                                                                    </td>
                                                                                    <td style="padding: 8px 6px; font-weight: 500; color: var(--muted); white-space: nowrap;">
                                                                                        {format!("{:.2} GB", size_gb)}
                                                                                    </td>
                                                                                    <td style="padding: 8px 6px;">
                                                                                        <div style="display: flex; gap: 8px; font-size: 11px;">
                                                                                            {(!m.hf_link.is_empty()).then(|| view! {
                                                                                                <a href=m.hf_link.clone() target="_blank" style="color: var(--primary); text-decoration: underline;">"HF"</a>
                                                                                            })}
                                                                                            {(!m.github_link.is_empty()).then(|| view! {
                                                                                                <a href=m.github_link.clone() target="_blank" style="color: var(--accent); text-decoration: underline;">"Git"</a>
                                                                                            })}
                                                                                        </div>
                                                                                    </td>
                                                                                    <td style="padding: 8px 6px; text-align: right;">
                                                                                        <div style="display: flex; gap: 6px; justify-content: flex-end; align-items: center;">
                                                                                            {(m.status != "enriching" && m.status != "enriched").then(|| {
                                                                                                view! {
                                                                                                    <button 
                                                                                                        class="btn secondary sm"
                                                                                                        style="font-size: 10.5px; padding: 2px 6px; height: 24px;"
                                                                                                        on:click=enrich_click.clone()
                                                                                                    >
                                                                                                        "Enrich"
                                                                                                    </button>
                                                                                                }
                                                                                            })}
                                                                                            <button 
                                                                                                class="btn primary sm" 
                                                                                                style="font-size: 10.5px; padding: 2px 6px; height: 24px;"
                                                                                                on:click=use_model
                                                                                            >
                                                                                                "Use Model"
                                                                                            </button>
                                                                                        </div>
                                                                                    </td>
                                                                                </tr>
                                                                            }
                                                                        }).collect_view()}
                                                                    </tbody>
                                                                </table>
                                                            </div>
                                                        }
                                                    })}
                                                </div>
                                            }
                                        }).collect_view()
                                    })}
                                </div>
                            }
                        }).collect_view().into_any()
                    }}
                </div>
            </Card>
        </div>
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TargetItem {
    name: String,
    line: String,
}

fn parse_input_targets(input: &str) -> Vec<TargetItem> {
    let mut targets = Vec::new();
    let mut _current_category = "mtp".to_string();
    let mut _current_subfolder: Option<String> = None;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.ends_with(':') {
            _current_category = trimmed[..trimmed.len() - 1].trim().to_string();
            _current_subfolder = None;
            continue;
        }
        if trimmed.starts_with('-') {
            _current_subfolder = Some(trimmed[1..].trim().to_string());
            continue;
        }

        let display_name = if trimmed.starts_with("http") {
            trimmed.split('/').last().unwrap_or(trimmed).to_string()
        } else if trimmed.ends_with(".git") || trimmed.starts_with("https://github") {
            let repo = trimmed.split('/').last().unwrap_or(trimmed);
            format!("git: {}", repo.trim_end_matches(".git"))
        } else {
            let repo_name = trimmed.split('/').last().unwrap_or("");
            format!("HF: {}", repo_name)
        };

        targets.push(TargetItem { name: display_name, line: trimmed.to_string() });
    }
    targets
}

fn parse_tmux_progress(log: &str, targets: &[TargetItem]) -> (Option<usize>, f32, String, String) {
    let lines: Vec<&str> = log.lines().collect();
    let mut active_idx: Option<usize> = None;
    let mut percent = 0.0f32;
    let mut speed = "Pending\u{2026}".to_string();
    let mut eta = "Unknown".to_string();

    // Find active target by matching recent log lines to target names
    'outer: for line in lines.iter().rev() {
        let lower = line.to_lowercase();
        for (idx, target) in targets.iter().enumerate().rev() {
            let kw = target.line.to_lowercase();
            let search_terms = vec![
                kw.clone(),
                target.name.to_lowercase(),
                target.line.rsplit('/').next().unwrap_or("").to_lowercase(),
            ];
            if search_terms.iter().any(|t| !t.is_empty() && lower.contains(t.as_str())) {
                active_idx = Some(idx);
                break 'outer;
            }
        }
    }

    // Extract percent/speed/ETA from recent lines
    for line in lines.iter().rev() {
        if let Some(pct) = extract_dl_percentage(line) {
            percent = pct;
            if let Some(s) = extract_dl_speed(line) { speed = s; }
            if let Some(e) = extract_dl_eta(line) { eta = e; }
            break;
        }
        if line.contains("Receiving objects:") || line.contains("Resolving deltas:") {
            if let Some(pct) = extract_dl_percentage(line) { percent = pct; }
            if let Some(s) = extract_dl_speed(line) { speed = s; }
            eta = "Cloning\u{2026}".to_string();
            break;
        }
    }

    (active_idx, percent, speed, eta)
}

fn extract_dl_percentage(line: &str) -> Option<f32> {
    if let Some(pct_pos) = line.find('%') {
        let before = &line[..pct_pos];
        let num_start = before.rfind(|c: char| !c.is_ascii_digit() && c != '.').map_or(0, |i| i + 1);
        return before[num_start..].parse::<f32>().ok();
    }
    None
}

fn extract_dl_speed(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    if let Some(dl_idx) = lower.find("dl:") {
        let sub = line[dl_idx + 3..].trim();
        let end = sub.find(']').or_else(|| sub.find(' ')).unwrap_or(sub.len());
        let sp = sub[..end].trim();
        if !sp.is_empty() { return Some(sp.to_string()); }
    }
    for token in line.split_whitespace().rev() {
        let t = token.trim_end_matches(|c: char| c == ',' || c == ']' || c == ')');
        if t.ends_with("/s") || t.ends_with("b/s") || t.ends_with("B/s") {
            return Some(t.to_string());
        }
    }
    None
}

fn extract_dl_eta(line: &str) -> Option<String> {
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
    if let Some(open_bracket) = line.find('[') {
        if let Some(close_bracket) = line.find(']') {
            let sub = &line[open_bracket + 1 .. close_bracket];
            if let Some(less_than) = sub.find('<') {
                if let Some(comma) = sub.find(',') {
                    if less_than < comma {
                        let eta_val = sub[less_than + 1 .. comma].trim();
                        if !eta_val.is_empty() {
                            return Some(eta_val.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}


#[component]
pub fn DownloadTab() -> impl IntoView {
    let targets_text = RwSignal::new(String::new());
    let is_downloading = RwSignal::new(false);
    let log_data = RwSignal::new(String::new());
    let show_raw_logs = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());

    let load_targets = move || {
        spawn_local(async move {
            if let Ok(t) = api::download_get_targets().await {
                targets_text.set(t);
            }
        });
    };

    let check_status = move || {
        spawn_local(async move {
            if let Ok(status) = api::download_status().await {
                is_downloading.set(status.is_downloading);
                if !status.log.is_empty() {
                    log_data.set(status.log);
                }
            }
        });
    };

    load_targets();
    check_status();

    let check_status_cb = check_status;
    let poller = Interval::new(1000, move || {
        check_status_cb();
    });
    poller.forget();

    let start = move |_| {
        error_msg.set(String::new());
        let t = targets_text.get_untracked();
        spawn_local(async move {
            match api::download_start(t).await {
                Ok(_) => is_downloading.set(true),
                Err(e) => error_msg.set(format!("Failed to start: {e}")),
            }
        });
    };

    let cancel = move |_| {
        spawn_local(async move {
            match api::download_cancel().await {
                Ok(_) => is_downloading.set(false),
                Err(e) => error_msg.set(format!("Failed to cancel: {e}")),
            }
        });
    };

    view! {
        <div class="page">
            <PageHeader title="Download Manager" desc="Model and repository downloader. Edit the targets file, generate dw.sh, and monitor live progress via tmux session 'hf_downloads'."/>

            {move || {
                let err = error_msg.get();
                (!err.is_empty()).then(|| view! {
                    <div style="padding:10px 14px;background:#ef444420;border:1px solid #ef4444;border-radius:var(--r-md);color:#ef4444;font-size:13px;">{err}</div>
                })
            }}

            <div style="display:flex;gap:20px;flex-wrap:wrap;min-height:0;">
                // ── Left: Target editor ────────────────────────────────────────
                <div style="flex:1 1 420px;display:flex;flex-direction:column;gap:16px;">
                    <Card title="Download Targets ('t' file format)">
                        <div style="font-size:12px;color:var(--muted);margin-bottom:8px;line-height:1.5;">
                            "Format: Category headers end with ':', subfolder lines start with '-', then one model/URL per line."
                        </div>
                        <textarea
                            class="notes-area"
                            style="min-height:360px;font-family:var(--font-mono);font-size:12.5px;resize:vertical;"
                            placeholder="# Example\ntext:\n- stable\nfacebook/opt-125m\n\nmultimodal:\nhttps://example.com/model.gguf\n\nrepos:\nhttps://github.com/user/repo.git"
                            prop:value=move || targets_text.get()
                            on:input=move |e| targets_text.set(event_target_value(&e))
                            disabled=move || is_downloading.get()
                        ></textarea>
                        <div style="display:flex;gap:10px;margin-top:12px;flex-wrap:wrap;">
                            <button class="btn primary" style="flex:1;"
                                disabled=move || is_downloading.get()
                                on:click=start
                            >
                                {move || if is_downloading.get() { "\u{23f3} Generating & Downloading\u{2026}" } else { "\u{2b07} Generate dw.sh & Start" }}
                            </button>
                            <button class="btn danger"
                                disabled=move || !is_downloading.get()
                                on:click=cancel
                            >"\u{23f9} Cancel"</button>
                        </div>
                        <div style="margin-top:8px;font-size:11.5px;color:var(--muted);">
                            "Tip: attach to live session with "
                            <code style="background:var(--surface-soft);padding:2px 6px;border-radius:4px;">
                                "tmux attach -t hf_downloads"
                            </code>
                        </div>
                    </Card>
                </div>

                // ── Right: Live monitor ────────────────────────────────────────
                <div style="flex:1 1 380px;display:flex;flex-direction:column;gap:16px;">
                    <Card title="Live Download Monitor">
                        // Status bar
                        <div style="display:flex;align-items:center;gap:8px;margin-bottom:14px;">
                            <div style=move || format!("width:9px;height:9px;border-radius:50%;flex-shrink:0;background:{};",
                                if is_downloading.get() { "#10b981" } else { "var(--muted)" }
                            )></div>
                            <span style="font-weight:600;font-size:13px;">
                                {move || if is_downloading.get() { "Downloading in 'hf_downloads' tmux session" } else { "Idle / Completed" }}
                            </span>
                        </div>

                        // Per-target progress list
                        <div style="display:flex;flex-direction:column;gap:10px;max-height:360px;overflow-y:auto;padding-right:2px;">
                            {move || {
                                let targets = parse_input_targets(&targets_text.get());
                                let log = log_data.get();
                                let (active_idx, percent, speed, eta) = parse_tmux_progress(&log, &targets);

                                if targets.is_empty() {
                                    return view! {
                                        <div style="text-align:center;padding:20px;color:var(--muted);font-style:italic;font-size:13px;">
                                            "No targets defined. Edit the file on the left."
                                        </div>
                                    }.into_any();
                                }

                                targets.into_iter().enumerate().map(|(idx, target)| {
                                    let is_active = is_downloading.get() && active_idx == Some(idx);
                                    let is_done = if is_downloading.get() {
                                        active_idx.map_or(false, |a| idx < a)
                                    } else {
                                        !log.is_empty()
                                    };

                                    let (bar_width, bar_color, label) = if is_active {
                                        (format!("{}%", percent as u32), "var(--primary)", format!("{}%", percent as u32))
                                    } else if is_done {
                                        ("100%".to_string(), "#10b981", "\u{2705} Done".to_string())
                                    } else {
                                        ("0%".to_string(), "var(--muted)", "\u{23f3} Pending".to_string())
                                    };

                                    view! {
                                        <div style="background:var(--surface-soft);border:1px solid var(--hairline);border-radius:var(--r-md);padding:10px 12px;display:flex;flex-direction:column;gap:7px;">
                                            <div style="display:flex;justify-content:space-between;align-items:center;">
                                                <span style="font-weight:600;font-size:12.5px;color:var(--ink);word-break:break-all;flex:1;margin-right:8px;">{target.name.clone()}</span>
                                                <span style=format!("font-size:11px;font-weight:600;white-space:nowrap;color:{};",
                                                    if is_active { "var(--primary)" } else if is_done { "#10b981" } else { "var(--muted)" }
                                                )>{label}</span>
                                            </div>
                                            // Progress bar
                                            <div style="width:100%;height:5px;background:var(--surface-strong);border-radius:9999px;overflow:hidden;">
                                                <div style=format!("height:100%;width:{};background:{};border-radius:9999px;transition:width 0.4s ease;", bar_width, bar_color)></div>
                                            </div>
                                            {if is_active {
                                                view! {
                                                    <div style="display:flex;justify-content:space-between;font-size:11px;color:var(--muted);">
                                                        <span>{format!("Speed: {speed}")}</span>
                                                        <span>{format!("ETA: {eta}")}</span>
                                                    </div>
                                                }.into_any()
                                            } else { view!{<span></span>}.into_any() }}
                                        </div>
                                    }
                                }).collect_view().into_any()
                            }}
                        </div>

                        // Collapsible raw log
                        {move || if is_downloading.get() || !log_data.get().is_empty() {
                            view! {
                                <div style="margin-top:12px;">
                                    <button class="btn secondary sm" style="width:100%;font-size:11.5px;"
                                        on:click=move |_| show_raw_logs.update(|v| *v = !*v)
                                    >
                                        {move || if show_raw_logs.get() { "Hide Raw Terminal Output" } else { "Show Raw Terminal Output" }}
                                    </button>
                                    {move || if show_raw_logs.get() {
                                        view! {
                                            <div class="log-console" style="height:180px;margin-top:8px;font-size:11px;background:var(--surface-soft);">
                                                <pre class="log-line" style="margin:0;">{log_data.get()}</pre>
                                            </div>
                                        }.into_any()
                                    } else { view!{<span></span>}.into_any() }}
                                </div>
                            }.into_any()
                        } else { view!{<span></span>}.into_any() }}
                    </Card>
                </div>
            </div>
        </div>
    }
}

// ── 3. InstancesTab / Instances ──────────────────────────────────────────────
#[component]
pub fn InstancesTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let detected = RwSignal::new(Vec::<LlamaInstance>::new());
    let is_scanning = RwSignal::new(false);

    let scan = move || {
        is_scanning.set(true);
        spawn_local(async move {
            if let Ok(list) = api::instances_detect().await {
                detected.set(list);
            }
            is_scanning.set(false);
        });
    };

    scan();

    let set_route = move |host: String, port: u16, default_model: Option<String>| {
        ctx.routed_instance.set(Some((host, port)));
        ctx.routed_model.set(default_model);
    };

    let reset_route = move |_| {
        ctx.routed_instance.set(None);
        ctx.routed_model.set(None);
    };

    let render_override = move |label: &'static str,
                               get_ovr: fn(&shared::ServerConfig) -> &shared::ModelOverride,
                               update_ovr: fn(&mut shared::ServerConfig, shared::ModelOverride)| {
        let get_ovr_val = move || {
            let cfg = ctx.config.get();
            get_ovr(&cfg).clone()
        };

        let on_toggle = move |e| {
            let checked = event_target_checked(&e);
            ctx.update_cfg(move |cfg| {
                let mut curr = get_ovr(cfg).clone();
                curr.enabled = checked;
                update_ovr(cfg, curr);
            });
        };

        let on_host = move |e| {
            let val = event_target_value(&e);
            ctx.update_cfg(move |cfg| {
                let mut curr = get_ovr(cfg).clone();
                curr.host = val.clone();
                update_ovr(cfg, curr);
            });
        };

        let on_port = move |e| {
            let val = event_target_value(&e).parse::<u16>().unwrap_or(8080);
            ctx.update_cfg(move |cfg| {
                let mut curr = get_ovr(cfg).clone();
                curr.port = val;
                update_ovr(cfg, curr);
            });
        };

        let on_model = move |e| {
            let val = event_target_value(&e);
            ctx.update_cfg(move |cfg| {
                let mut curr = get_ovr(cfg).clone();
                curr.model = val.clone();
                update_ovr(cfg, curr);
            });
        };

        view! {
            <div style="display: flex; flex-direction: column; gap: 6px; padding: 12px; border: 1px solid var(--hairline); border-radius: var(--r-md); background: rgba(255,255,255,0.01);">
                <div style="display: flex; align-items: center; justify-content: space-between;">
                    <span style="font-weight: 600; font-size: 13.5px; color: var(--ink);">{label}</span>
                    <label class="agent-toggle">
                        <input type="checkbox" prop:checked=move || get_ovr_val().enabled on:change=on_toggle/>
                        "Override Active"
                    </label>
                </div>
                {move || get_ovr_val().enabled.then(|| {
                    let ovr = get_ovr_val();
                    view! {
                        <div style="display: grid; grid-template-columns: 2fr 1fr 2fr; gap: 8px; margin-top: 6px;">
                            <div class="field" style="margin: 0;">
                                <label class="field-label" style="font-size: 11px; margin-bottom: 2px;">"Host"</label>
                                <input class="input" style="height: 28px; font-size: 12px; padding: 0 8px;" type="text" prop:value=ovr.host.clone() on:input=on_host/>
                            </div>
                            <div class="field" style="margin: 0;">
                                <label class="field-label" style="font-size: 11px; margin-bottom: 2px;">"Port"</label>
                                <input class="input" style="height: 28px; font-size: 12px; padding: 0 8px;" type="number" prop:value=ovr.port on:input=on_port/>
                            </div>
                            <div class="field" style="margin: 0;">
                                <label class="field-label" style="font-size: 11px; margin-bottom: 2px;">"Model Override"</label>
                                <input class="input" style="height: 28px; font-size: 12px; padding: 0 8px;" type="text" placeholder="e.g. llama3" prop:value=ovr.model.clone() on:input=on_model/>
                            </div>
                        </div>
                    }
                })}
            </div>
        }
    };

    view! {
        <div class="page">
            <PageHeader title="Instances" desc="Scan running model server instances and route front-end requests."/>

            <div style="display: flex; gap: var(--s-lg); flex-wrap: wrap; align-items: start; width: 100%;">
                <div style="flex: 1 1 360px; display: flex; flex-direction: column; gap: var(--s-lg); max-width: 100%;">
                    <Card title="Active Target Routing">
                        <div class="todo-item" style="padding: 14px; flex-direction: column; align-items: stretch; gap: 8px;">
                            <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                                <div>
                                    <div style="font-weight: 600; font-size: 15px; color: var(--ink);">
                                        {move || match ctx.routed_instance.get() {
                                            Some((h, p)) => format!("Routed to Remote Target: http://{}:{}", h, p),
                                            None => "Routed to: Local launch-config server".to_string()
                                        }}
                                    </div>
                                    <div style="font-size: 13px; font-weight: 500; color: var(--primary); margin-top: 4px;">
                                        {move || match ctx.routed_model.get() {
                                            Some(m) => format!("Active Model: {}", m),
                                            None => "Active Model: Default local model".to_string()
                                        }}
                                    </div>
                                    <div class="field-hint" style="margin-top: 4px;">
                                        "Determines where the Chat and agent modules send requests."
                                    </div>
                                </div>
                                {move || ctx.routed_instance.get().is_some().then(|| view! {
                                    <button class="btn secondary sm" on:click=reset_route>"Reset to Local"</button>
                                })}
                            </div>
                        </div>
                    </Card>

                    <Card title="Local Port Scan">
                        <div class="row-actions" style="margin-bottom: 16px;">
                            <button class="btn primary" disabled=move || is_scanning.get() on:click=move |_| scan()>
                                {move || if is_scanning.get() { "Scanning..." } else { "Scan Common Ports" }}
                            </button>
                        </div>

                        <div style="display: flex; flex-direction: column; gap: 10px;">
                            {move || {
                                let list = detected.get();
                                if list.is_empty() {
                                    return view! {
                                        <div class="field-hint" style="text-align: center; padding: 20px;">
                                            {if is_scanning.get() { "Searching for endpoints..." } else { "No active server instances detected." }}
                                        </div>
                                    }.into_any();
                                }

                                list.into_iter().map(|inst| {
                                    let host = inst.hostname.clone();
                                    let port = inst.port;
                                    let is_routed_active = Signal::derive(move || {
                                        if let Some((rh, rp)) = ctx.routed_instance.get() {
                                            rh == host && rp == port
                                        } else {
                                            false
                                        }
                                    });

                                    let h = inst.hostname.clone();
                                    let models_clone = inst.models.clone();
                                    let first_model = models_clone.first().cloned();

                                    view! {
                                        <div class="todo-item" style="padding: 12px; flex-direction: column; align-items: stretch; gap: 8px;">
                                            <div style="display: flex; align-items: center; justify-content: space-between; width: 100%;">
                                                <div>
                                                    <div style="font-weight: 600; display: flex; align-items: center; gap: 8px;">
                                                        <span>{format!("http://{}:{}", inst.hostname, inst.port)}</span>
                                                        <span style="font-size: 11px; background: #10b981; color: white; padding: 2px 6px; border-radius: var(--r-pill);">
                                                            "Online"
                                                        </span>
                                                    </div>
                                                    <div class="field-hint" style="margin-top: 4px;">
                                                        {format!("Loaded models: {}", inst.models.join(", "))}
                                                    </div>
                                                </div>
                                                {
                                                    let h_for_click = h.clone();
                                                    view! {
                                                        <button
                                                            class="btn sm"
                                                            class:primary=move || !is_routed_active.get()
                                                            class:secondary=move || is_routed_active.get()
                                                            disabled=move || is_routed_active.get()
                                                            on:click=move |_| set_route(h_for_click.clone(), port, first_model.clone())
                                                        >
                                                            {move || if is_routed_active.get() { "Routed" } else { "Route / Attach" }}
                                                        </button>
                                                    }
                                                }
                                            </div>

                                            {(!models_clone.is_empty()).then(|| {
                                                let h_c = h.clone();
                                                let m_list = models_clone.clone();
                                                view! {
                                                    <div style="border-top: 1px dashed var(--hairline); padding-top: 8px; display: flex; align-items: center; justify-content: space-between;">
                                                        <div style="display: flex; align-items: center; gap: 8px;">
                                                            <span style="font-size: 12px; color: var(--muted); font-weight: 500;">"Served Model:"</span>
                                                            <select
                                                                class="model-select"
                                                                style="height: 26px; padding: 0 4px; font-size: 12px; line-height: 1;"
                                                                prop:value=move || {
                                                                    if is_routed_active.get() {
                                                                        ctx.routed_model.get().unwrap_or_default()
                                                                    } else {
                                                                        String::new()
                                                                    }
                                                                }
                                                                on:change=move |e| {
                                                                    let selected = event_target_value(&e);
                                                                    ctx.routed_instance.set(Some((h_c.clone(), port)));
                                                                    ctx.routed_model.set(Some(selected));
                                                                }
                                                            >
                                                                <option value="" disabled=true>"-- Select Model --"</option>
                                                                {m_list.into_iter().map(|m| {
                                                                    let label = m.clone();
                                                                    view! { <option value=m>{label}</option> }
                                                                }).collect_view()}
                                                            </select>
                                                        </div>
                                                        {move || (is_routed_active.get()).then(|| view! {
                                                            <span style="font-size: 11px; color: var(--primary); font-weight: 600; display: flex; align-items: center; gap: 4px;">
                                                                "● Active Connection"
                                                            </span>
                                                        })}
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    }
                                }).collect_view().into_any()
                            }}
                        </div>
                    </Card>
                </div>

                <div style="flex: 1 1 360px; display: flex; flex-direction: column; gap: var(--s-lg); max-width: 100%;">
                    <Card title="Target overrides per Tool & Subagent">
                        <div style="display: flex; flex-direction: column; gap: 12px;">
                            {render_override("Task Planner & Subagents",
                                             |c| &c.override_planner,
                                             |c, val| c.override_planner = val)}
                            {render_override("Calendar Scheduled Actions",
                                             |c| &c.override_calendar,
                                             |c, val| c.override_calendar = val)}
                            {render_override("Quick Notes / Memory Copilot",
                                             |c| &c.override_memory,
                                             |c, val| c.override_memory = val)}
                            {render_override("Deep Research Agent",
                                             |c| &c.override_research,
                                             |c, val| c.override_research = val)}
                            {render_override("Compare Tab / llama-benchy",
                                             |c| &c.override_compare,
                                             |c, val| c.override_compare = val)}
                        </div>
                    </Card>
                </div>
            </div>
        </div>
    }
}

// ── 4. AgentsTab / Memory & Self-Learning ────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
struct GraphNode {
    id: String,
    title: String,
    scope: String,
    x: f64,
    y: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct GraphEdge {
    source: String,
    target: String,
}

fn compute_node_positions(
    global_mems: &[Memory],
    project_mems: &[Memory],
    width: f64,
    height: f64,
) -> Vec<GraphNode> {
    let mut nodes = Vec::new();
    let total = global_mems.len() + project_mems.len();
    if total == 0 {
        return nodes;
    }
    let mut idx = 0;
    for m in global_mems.iter().chain(project_mems.iter()) {
        let angle = idx as f64 * 2.0 * std::f64::consts::PI / total as f64;
        let r = 120.0;
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
    nodes
}

#[derive(Clone, Debug, PartialEq)]
struct EpisodicLog {
    text: String,
    timestamp: String,
}

#[derive(Clone, Debug, PartialEq)]
struct SimNode {
    id: String,
    label: String,
    x: f64,
    y: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct SimEdge {
    source: String,
    target: String,
}

#[component]
fn GamMemorySimulation() -> impl IntoView {
    let episodic_buffer = RwSignal::new(vec![
        EpisodicLog { text: "Session initiated: connected to model server".to_string(), timestamp: "23:15:00".to_string() },
        EpisodicLog { text: "Executed search for local memory context".to_string(), timestamp: "23:15:05".to_string() },
        EpisodicLog { text: "User queried: 'Optimize CUDA configurations'".to_string(), timestamp: "23:15:10".to_string() },
    ]);
    let sim_divergence_threshold = RwSignal::new(0.40_f64);
    let calculated_divergence_score = RwSignal::new(0.15_f64);
    let new_log_input = RwSignal::new(String::new());
    let anim_consolidating = RwSignal::new(false);

    let global_semantic_nodes = RwSignal::new(vec![
        SimNode { id: "project".into(), label: "Project Core".into(), x: 270.0, y: 210.0 },
        SimNode { id: "user_prefs".into(), label: "User Preferences".into(), x: 130.0, y: 110.0 },
        SimNode { id: "tech_stack".into(), label: "Tech Stack (Rust/Tauri)".into(), x: 410.0, y: 110.0 },
    ]);
    let global_semantic_edges = RwSignal::new(vec![
        SimEdge { source: "user_prefs".into(), target: "project".into() },
        SimEdge { source: "project".into(), target: "tech_stack".into() },
    ]);

    let add_log = move || {
        let text = new_log_input.get_untracked().trim().to_string();
        if text.is_empty() { return; }
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        episodic_buffer.update(|buf| buf.push(EpisodicLog { text, timestamp: now }));
        calculated_divergence_score.update(|s| *s = (*s + 0.12).min(1.0));
        new_log_input.set(String::new());
    };

    let trigger_consolidation = move || {
        let buf = episodic_buffer.get_untracked();
        if buf.is_empty() { return; }
        anim_consolidating.set(true);
        let node_label = if let Some(last_log) = buf.last() {
            let t = last_log.text.trim();
            let clean = if t.len() > 22 { format!("{}...", &t[..19]) } else { t.to_string() };
            if clean.to_lowercase().contains("cuda") { "CUDA Optimization".into() }
            else if clean.to_lowercase().contains("search") { "Search Context".into() }
            else { clean }
        } else { "Consolidated Node".to_string() };

        let node_id = format!("node_{}", chrono::Local::now().timestamp_millis());
        let node_count = global_semantic_nodes.get_untracked().len() as f64;
        let angle = node_count * 2.0 * std::f64::consts::PI / 6.0;
        let radius = 110.0;
        let x = 270.0 + radius * angle.cos();
        let y = 210.0 + radius * angle.sin();

        global_semantic_nodes.update(|nodes| nodes.push(SimNode { id: node_id.clone(), label: node_label, x, y }));
        global_semantic_edges.update(|edges| edges.push(SimEdge { source: "project".into(), target: node_id }));
        episodic_buffer.set(Vec::new());
        calculated_divergence_score.set(0.05);

        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(800).await;
            anim_consolidating.set(false);
        });
    };

    view! {
        <div style="display: flex; gap: 16px; flex-wrap: wrap; width: 100%;">
            <div style="flex: 1.2; min-width: 320px; display: flex; flex-direction: column; gap: 16px;">
                <Card title="📋 Episodic Buffer (Event Graph)">
                    <div style="overflow-y: auto; max-height: 200px; display: flex; flex-direction: column; gap: 6px; margin-bottom: 16px; min-height: 120px;">
                        {move || {
                            let list = episodic_buffer.get();
                            if list.is_empty() {
                                view! { <div style="text-align: center; color: var(--muted); font-style: italic; padding: 20px 0;">"Episodic buffer is clean."</div> }.into_any()
                            } else {
                                list.into_iter().enumerate().map(|(_idx, log)| {
                                    view! {
                                        <div style="display: flex; gap: 8px; padding: 6px 10px; border-left: 2px solid var(--accent); background: var(--surface-soft); border-radius: 0 var(--r-sm) var(--r-sm) 0; font-size: 12.5px;">
                                            <span style="font-family: var(--font-mono); opacity: 0.6; font-size: 11px;">{format!("[{}]", log.timestamp)}</span>
                                            <span style="color: var(--body);">{log.text}</span>
                                        </div>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </div>
                    <div style="background: var(--surface-soft); border: 1px solid var(--hairline); border-radius: var(--r-md); padding: 12px; margin-bottom: 16px;">
                        <div style="display: flex; justify-content: space-between; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--ink);">
                            <span>"Semantic Divergence Score"</span>
                            {move || {
                                let score = calculated_divergence_score.get();
                                let limit = sim_divergence_threshold.get();
                                let color = if score >= limit { "#ef4444" } else { "var(--accent)" };
                                view! { <span style=format!("color: {}; font-weight: bold;", color)>{format!("{:.2} / {:.2}", score, limit)}</span> }
                            }}
                        </div>
                        <div style="width: 100%; height: 8px; background: rgba(0,0,0,0.1); border-radius: var(--r-pill); overflow: hidden;">
                            <div style=move || {
                                let score = calculated_divergence_score.get();
                                let limit = sim_divergence_threshold.get();
                                let color = if score >= limit { "#ef4444" } else if score >= limit * 0.7 { "#f59e0b" } else { "#10b981" };
                                format!("height: 100%; width: {}%; background: {}; transition: width 0.3s ease;", (score * 100.0).min(100.0), color)
                            }></div>
                        </div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 12px;">
                        <div style="display: flex; gap: 8px;">
                            <input class="input" type="text" style="flex: 1; height: 32px; font-size: 13px;" placeholder="Add simulated agent event..."
                                prop:value=move || new_log_input.get()
                                on:input=move |e| new_log_input.set(event_target_value(&e))
                                on:keydown=move |e| { if e.key() == "Enter" { add_log(); } }
                            />
                            <button class="btn primary sm" on:click=move |_| add_log()>"Add"</button>
                        </div>
                        <button class="btn primary" style="width: 100%; font-weight: bold;"
                            prop:disabled=move || episodic_buffer.get().is_empty() || anim_consolidating.get()
                            on:click=move |_| trigger_consolidation()
                        >
                            {move || if anim_consolidating.get() { "Consolidating..." } else { "🧬 Trigger Consolidation" }}
                        </button>
                    </div>
                </Card>
            </div>
            <div style="flex: 1.8; min-width: 400px;">
                <Card title="🕸️ Topic Associative Graph">
                    <div style="width: 100%; height: 380px; border-radius: var(--r-lg); overflow: hidden; border: 1px solid var(--hairline); background: var(--surface-soft);">
                        <svg width="100%" height="100%" viewBox="0 0 540 420">
                            {move || {
                                let nodes = global_semantic_nodes.get();
                                let edges = global_semantic_edges.get();
                                edges.into_iter().map(|edge| {
                                    if let (Some(s), Some(t)) = (nodes.iter().find(|n| n.id == edge.source), nodes.iter().find(|n| n.id == edge.target)) {
                                        view! { <line x1=s.x y1=s.y x2=t.x y2=t.y stroke="rgba(99, 102, 241, 0.4)" stroke-width="2" stroke-dasharray="4,4"/> }.into_any()
                                    } else { view! {}.into_any() }
                                }).collect_view()
                            }}
                            {move || {
                                let nodes = global_semantic_nodes.get();
                                nodes.into_iter().map(|node| {
                                    view! {
                                        <g>
                                            <circle cx=node.x cy=node.y r="8" fill=if node.id == "project" { "var(--primary)" } else { "#10b981" } stroke="#fff" stroke-width="1.5"/>
                                            <text x=node.x y=node.y - 12.0 text-anchor="middle" font-size="11px" fill="var(--ink)">{node.label}</text>
                                        </g>
                                    }
                                }).collect_view()
                            }}
                        </svg>
                    </div>
                </Card>
            </div>
        </div>
    }
}

// Which sub-section to show: "memory" | "skills"
#[derive(Clone, Copy, PartialEq)]
enum AgentView { Memory, Skills }

#[derive(Clone, Debug)]
pub struct TreeItem {
    pub name: String,
    pub full_path: Option<String>,
    pub is_skill: bool,
    pub content: String,
    pub children: Vec<TreeItem>,
}

fn get_relative_path(path: &str, source: &str) -> String {
    let prefix = match source {
        "Workspace" => "/home/notroot/Work/llama-manager",
        "Gemini Skills" => "/home/notroot/.gemini/skills",
        "Gemini Plugins" => "/home/notroot/.gemini/config/plugins",
        _ => "",
    };
    if !prefix.is_empty() && path.starts_with(prefix) {
        path[prefix.len()..].trim_start_matches('/').to_string()
    } else {
        path.to_string()
    }
}

fn insert_into_tree(tree: &mut Vec<TreeItem>, segments: &[&str], full_path: &str, is_skill: bool, content: &str) {
    if segments.is_empty() {
        return;
    }
    let segment = segments[0];
    if segments.len() == 1 {
        // File node
        if !tree.iter().any(|item| item.name == segment && item.full_path.is_some()) {
            tree.push(TreeItem {
                name: segment.to_string(),
                full_path: Some(full_path.to_string()),
                is_skill,
                content: content.to_string(),
                children: Vec::new(),
            });
        }
    } else {
        // Directory node
        let dir_idx = match tree.iter().position(|item| item.name == segment && item.full_path.is_none()) {
            Some(idx) => idx,
            None => {
                tree.push(TreeItem {
                    name: segment.to_string(),
                    full_path: None,
                    is_skill: false,
                    content: String::new(),
                    children: Vec::new(),
                });
                tree.len() - 1
            }
        };
        insert_into_tree(&mut tree[dir_idx].children, &segments[1..], full_path, is_skill, content);
    }
}

fn sort_tree(tree: &mut Vec<TreeItem>) {
    tree.sort_by(|a, b| {
        let a_is_dir = a.full_path.is_none();
        let b_is_dir = b.full_path.is_none();
        if a_is_dir != b_is_dir {
            b_is_dir.cmp(&a_is_dir) // Directories first
        } else {
            a.name.cmp(&b.name)
        }
    });
    for item in tree.iter_mut() {
        sort_tree(&mut item.children);
    }
}

#[component]
fn TreeViewNode(
    item: TreeItem,
    depth: usize,
    selected_skill: RwSignal<Option<shared::ipc::SkillOrAgentFile>>,
    expanded_nodes: RwSignal<std::collections::HashSet<String>>,
    node_key: String,
) -> impl IntoView {
    let is_dir = item.full_path.is_none();
    let name = item.name.clone();
    let full_path = item.full_path.clone().unwrap_or_default();
    let is_skill = item.is_skill;
    
    let key = if node_key.is_empty() {
        name.clone()
    } else {
        format!("{}/{}", node_key, name)
    };
    
    let key_for_view1 = key.clone();
    let key_for_view2 = key.clone();
    let key_for_children = key.clone();
    let is_expanded_1 = move || expanded_nodes.get().contains(&key_for_view1);
    let is_expanded_2 = move || expanded_nodes.get().contains(&key_for_view2);
    
    let key_for_toggle = key.clone();
    let toggle_expand = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        let k = key_for_toggle.clone();
        expanded_nodes.update(|set| {
            if set.contains(&k) {
                set.remove(&k);
            } else {
                set.insert(k);
            }
        });
    };
    
    let full_path_for_click = full_path.clone();
    let name_for_click = name.clone();
    let content_for_click = item.content.clone();
    let is_skill_for_click = item.is_skill;
    let on_file_click = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        let path = full_path_for_click.clone();
        let name_val = name_for_click.clone();
        let content_val = content_for_click.clone();
        let is_sk_val = is_skill_for_click;
        selected_skill.update(|sel| {
            if sel.as_ref().map_or(false, |s| s.path == path) {
                *sel = None;
            } else {
                *sel = Some(shared::ipc::SkillOrAgentFile {
                    name: name_val,
                    path,
                    is_skill: is_sk_val,
                    content: content_val,
                    source: String::new(),
                });
            }
        });
    };
    
    let padding_left = format!("{}px", depth * 16 + 12);
    let name_for_view = name.clone();
    let children_val = item.children.clone();
    
    view! {
        <div style="display:flex;flex-direction:column;">
            {if is_dir {
                let key_for_child_loop = key_for_children.clone();
                let name_for_dir_view = name_for_view.clone();
                let children_for_dir = children_val.clone();
                view! {
                    <div
                        style=format!("display:flex;align-items:center;padding:6px 12px 6px {};cursor:pointer;user-select:none;font-weight:600;font-size:12.5px;color:var(--ink);gap:6px;transition:background 0.1s;", padding_left)
                        class="tree-dir-node"
                        on:click=toggle_expand
                    >
                        <span style="font-size:11px;color:var(--muted);width:12px;text-align:center;">
                            {move || if is_expanded_1() { "▼" } else { "▶" }}
                        </span>
                        <span style="font-size:14px;">"📁"</span>
                        <span>{name_for_dir_view}</span>
                    </div>
                    {move || is_expanded_2().then(|| {
                        let k = key_for_child_loop.clone();
                        let children = children_for_dir.clone();
                        view! {
                            <div style="display:flex;flex-direction:column;">
                                {children.into_iter().map(|child| {
                                    view! {
                                        <TreeViewNode
                                            item=child
                                            depth=depth + 1
                                            selected_skill=selected_skill
                                            expanded_nodes=expanded_nodes
                                            node_key=k.clone()
                                        />
                                    }
                                }).collect_view()}
                            </div>
                        }
                    })}
                }.into_any()
            } else {
                let file_path = full_path.clone();
                let is_selected = move || selected_skill.get().as_ref().map_or(false, |s| s.path == file_path);
                let name_for_file_view = name_for_view.clone();
                
                view! {
                    <div
                        style=move || format!("display:flex;align-items:center;padding:6px 12px 6px {};cursor:pointer;user-select:none;font-size:12.5px;gap:8px;transition:background 0.1s;border-bottom:1px solid var(--hairline);{}",
                            padding_left,
                            if is_selected() { "background:color-mix(in srgb, var(--primary) 10%, transparent);" } else { "background:transparent;" }
                        )
                        class="tree-file-node"
                        on:click=on_file_click
                    >
                        <span style="width:12px;"></span>
                        <span style="font-size:14px;">{if is_skill { "📋" } else { "🤖" }}</span>
                        <span style="flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{name_for_file_view.clone()}</span>
                        <span style=format!("font-size:9px;padding:1px 5px;border-radius:9999px;font-weight:600;{}",
                            if is_skill { "background:#6366f120;color:#6366f1;" } else { "background:#f59e0b20;color:#f59e0b;" })>
                            {if is_skill { "SKILL" } else { "AGENTS" }}
                        </span>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
pub fn AgentsTab() -> impl IntoView {
    let agent_view = RwSignal::new(AgentView::Memory);
    let show_gam_simulation = RwSignal::new(false);
    let memories = RwSignal::new(Vec::<Memory>::new());
    let selected_mem = RwSignal::new(None::<Memory>);
    let show_new_form = RwSignal::new(false);

    let edit_id = RwSignal::new(String::new());
    let edit_title = RwSignal::new(String::new());
    let edit_scope = RwSignal::new("project".to_string());
    let edit_tags = RwSignal::new(String::new());
    let edit_links = RwSignal::new(Vec::<String>::new());
    let edit_content = RwSignal::new(String::new());

    let search_query = RwSignal::new(String::new());
    let expanded_nodes = RwSignal::new(std::collections::HashSet::<String>::new());

    let load_memories = move || {
        spawn_local(async move {
            if let Ok(mems) = api::obsidian_memories_get().await {
                memories.set(mems);
            }
        });
    };
    load_memories();

    let save_memory = move || {
        let id = edit_id.get_untracked();
        let title = edit_title.get_untracked().trim().to_string();
        let title = if title.is_empty() { "Untitled Memory".to_string() } else { title };
        let scope = edit_scope.get_untracked();
        let content = edit_content.get_untracked();
        let tags = edit_tags.get_untracked().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let links = edit_links.get_untracked();

        let mem = Memory { id, title, created_at: chrono::Local::now().to_rfc3339(), tags, links, scope, content };
        spawn_local(async move {
            if api::obsidian_memory_save(mem).await.is_ok() {
                show_new_form.set(false);
                selected_mem.set(None);
                load_memories();
            }
        });
    };

    let delete_memory = move || {
        let id = edit_id.get_untracked();
        let scope = edit_scope.get_untracked();
        spawn_local(async move {
            if api::obsidian_memory_delete(id, scope).await.is_ok() {
                show_new_form.set(false);
                selected_mem.set(None);
                load_memories();
            }
        });
    };

    let clear_all_memories = move |_| {
        let list = memories.get_untracked();
        spawn_local(async move {
            for m in list {
                let _ = api::obsidian_memory_delete(m.id, m.scope).await;
            }
            show_new_form.set(false);
            selected_mem.set(None);
            load_memories();
        });
    };

    let filtered_memories = move || {
        let q = search_query.get().to_lowercase();
        let list = memories.get();
        list.into_iter()
            .filter(|m| q.is_empty() || m.title.to_lowercase().contains(&q) || m.content.to_lowercase().contains(&q))
            .collect::<Vec<_>>()
    };

    let graph_node_positions = move || {
        let list = memories.get();
        let global_mems: Vec<_> = list.iter().filter(|m| m.scope == "global").cloned().collect();
        let project_mems: Vec<_> = list.iter().filter(|m| m.scope == "project").cloned().collect();
        compute_node_positions(&global_mems, &project_mems, 540.0, 420.0)
    };

    let graph_edges = move || {
        let list = memories.get();
        let mut edges = Vec::new();
        let id_set: std::collections::HashSet<_> = list.iter().map(|m| m.id.clone()).collect();
        for m in list {
            for link in m.links {
                if id_set.contains(&link) && id_set.contains(&m.id) {
                    edges.push(GraphEdge { source: m.id.clone(), target: link });
                }
            }
        }
        edges
    };

    let agent_output = RwSignal::new(String::new());
    let agent_running = RwSignal::new(false);
    let agent_prompt = RwSignal::new(String::new());
    let ctx = expect_context::<crate::state::AppCtx>();

    // ── Agent Skills / AGENTS.md ──────────────────────────────────────────────
    let skill_files = RwSignal::new(Vec::<shared::ipc::SkillOrAgentFile>::new());
    let skills_loading = RwSignal::new(false);
    let selected_skill = RwSignal::new(None::<shared::ipc::SkillOrAgentFile>);
    let skills_search = RwSignal::new(String::new());
    let skills_error = RwSignal::new(String::new());

    let load_skills = move || {
        skills_loading.set(true);
        spawn_local(async move {
            match api::list_skills_and_agents().await {
                Ok(files) => {
                    skill_files.set(files);
                    skills_loading.set(false);
                }
                Err(e) => {
                    skills_error.set(format!("Failed to load skills: {e}"));
                    skills_loading.set(false);
                }
            }
        });
    };

    let dispatch_agent_on_memory = move || {
        let title = edit_title.get_untracked();
        let content = edit_content.get_untracked();
        let instruction = agent_prompt.get_untracked().trim().to_string();
        let task = if instruction.is_empty() {
            format!("Analyze, expand, and summarize this memory note titled \"{title}\":\n\n{content}")
        } else {
            format!("For memory note titled \"{title}\", perform this command: {instruction}\n\nExisting content:\n{content}")
        };
        agent_running.set(true);
        agent_output.set("⏳ Dispatching agent copilot…".to_string());
        let (host, port, model) = ctx.resolve_target("memory");
        spawn_local(async move {
            let req = AgentRequest { host, port, model, task };
            match api::agent_start(req).await {
                Ok(id) => agent_output.set(format!("Agent started: {id}. Watch the Monitor tab, or wait for output here. Once finished, use the controls below to insert/append.")),
                Err(e) => agent_output.set(format!("Error starting agent: {e}")),
            }
            agent_running.set(false);
        });
    };

    // Calculate backlinks (which notes link to this selected note)
    let backlinks = move || {
        let cur_id = edit_id.get();
        let list = memories.get();
        if cur_id.is_empty() {
            return Vec::new();
        }
        list.into_iter()
            .filter(|m| m.id != cur_id && m.links.contains(&cur_id))
            .collect::<Vec<_>>()
    };

    let append_agent_output = move |_| {
        let output = agent_output.get_untracked();
        if !output.is_empty() && !output.starts_with('⏳') && !output.starts_with('E') {
            let mut current = edit_content.get_untracked();
            current.push_str(&format!("\n\n### Agent Summary\n{}", output));
            edit_content.set(current);
            agent_output.set(String::new());
        }
    };

    // Parallelism check warning block
    let show_multi_agent_warn = move || {
        let cfg = ctx.config.get();
        if cfg.parallel < 2 {
            view! {
                <div class="parallelism-warn" style="margin: 0 0 12px 0; border-radius: 4px;">
                    <span>"ℹ️"</span>
                    <div>
                        <strong>"Single-threaded runtime check:"</strong>
                        " Model server parallelism is set to 1. Concurrent agents or sub-agents will run serially. To unleash multi-agent flows, increase 'Parallel' slots in the Server tab and reduce model size if needed."
                    </div>
                </div>
            }.into_any()
        } else {
            view! {}.into_any()
        }
    };

    view! {
        <div style="display:flex;flex-direction:column;height:100%;padding:var(--s-md);gap:var(--s-md);overflow:hidden;">
            // Top bar with view switcher tabs
            <div style="display:flex;align-items:center;justify-content:space-between;flex-shrink:0;flex-wrap:wrap;gap:8px;">
                <div>
                    <div style="font-size:18px;font-weight:700;color:var(--ink);letter-spacing:-0.3px;">
                        "Memory & Agent Knowledge Base"
                    </div>
                    <div style="font-size:12px;color:var(--muted);margin-top:2px;">
                        {move || match agent_view.get() {
                            AgentView::Memory => format!("{} memory nodes across all scopes", memories.get().len()),
                            AgentView::Skills => format!("{} skills and agent files loaded", skill_files.get().len()),
                        }}
                    </div>
                </div>
                <div style="display:flex;gap:6px;align-items:center;flex-wrap:wrap;">
                    // View switcher pill
                    <div style="display:flex;background:var(--surface-card);border:1px solid var(--hairline);border-radius:var(--r-md);padding:3px;gap:2px;">
                        <button
                            style=move || format!("padding:4px 12px;border-radius:4px;border:none;cursor:pointer;font-size:12.5px;font-weight:600;{}",
                                if agent_view.get() == AgentView::Memory {
                                    "background:var(--primary);color:var(--on-primary);"
                                } else {
                                    "background:transparent;color:var(--body);"
                                })
                            on:click=move |_| agent_view.set(AgentView::Memory)
                        >"🧠 Memory"</button>
                        <button
                            style=move || format!("padding:4px 12px;border-radius:4px;border:none;cursor:pointer;font-size:12.5px;font-weight:600;{}",
                                if agent_view.get() == AgentView::Skills {
                                    "background:var(--primary);color:var(--on-primary);"
                                } else {
                                    "background:transparent;color:var(--body);"
                                })
                            on:click=move |_| {
                                agent_view.set(AgentView::Skills);
                                if skill_files.get_untracked().is_empty() {
                                    load_skills();
                                }
                            }
                        >"🗺 AGENTS.md / agent-skills"</button>
                    </div>

                    {move || if agent_view.get() == AgentView::Memory {
                        view! {
                            <button class="btn secondary sm"
                                on:click=move |_| {
                                    let new_id = format!("mem-{}", js_sys::Date::now() as u64);
                                    edit_id.set(new_id);
                                    edit_title.set("Untitled".to_string());
                                    edit_scope.set("project".to_string());
                                    edit_tags.set(String::new());
                                    edit_links.set(Vec::new());
                                    edit_content.set(String::new());
                                    show_new_form.set(true);
                                    selected_mem.set(None);
                                }
                            >"＋ New Node"</button>
                            <button class="btn secondary sm"
                                on:click=move |_| show_gam_simulation.update(|v| *v = !*v)
                            >
                                {move || if show_gam_simulation.get() { "📂 Workspace" } else { "🕸 Graph View" }}
                            </button>
                            <button class="btn danger sm"
                                prop:disabled=move || memories.get().is_empty()
                                on:click=clear_all_memories
                            >"🗑 Clear All"</button>
                        }.into_any()
                    } else {
                        view! {
                            <button class="btn secondary sm"
                                on:click=move |_| load_skills()
                            >
                                {move || if skills_loading.get() { "⏳ Scanning…" } else { "↻ Rescan" }}
                            </button>
                        }.into_any()
                    }}
                </div>
            </div>

            // Warn if parallelism is low (only in memory view)
            {move || if agent_view.get() == AgentView::Memory {
                view! { <div>{show_multi_agent_warn}</div> }.into_any()
            } else { view!{}.into_any() }}

            // Main content: switch between Memory workspace and Skills viewer
            {move || if agent_view.get() == AgentView::Skills {
                // ── Skills & AGENTS.md Viewer ─────────────────────────────
                let files = skill_files.get();
                let query = skills_search.get().to_lowercase();

                let sources: Vec<String> = {
                    let mut seen = std::collections::HashSet::new();
                    let mut v = Vec::new();
                    for f in &files {
                        if seen.insert(f.source.clone()) {
                            v.push(f.source.clone());
                        }
                    }
                    v
                };

                view! {
                    <div style="display:flex;flex-direction:column;gap:12px;flex:1;min-height:0;">
                        // Error
                        {move || {
                            let err = skills_error.get();
                            (!err.is_empty()).then(|| view! {
                                <div style="padding:10px 14px;background:#ef444420;border:1px solid #ef4444;border-radius:var(--r-md);color:#ef4444;font-size:13px;flex-shrink:0;">{err}</div>
                            })
                        }}

                        <div style="display:flex;gap:16px;flex-wrap:wrap;flex:1;min-height:0;overflow:hidden;">
                            // Left Column: Tree Viewer
                            <div style="flex:1 1 320px;display:flex;flex-direction:column;gap:12px;overflow-y:auto;max-height:100%;padding-right:4px;">
                                // Search bar
                                <input class="input" type="text" placeholder="🔍 Search skills, agents, keywords…"
                                    prop:value=move || skills_search.get()
                                    on:input=move |e| skills_search.set(event_target_value(&e))
                                />

                                {if files.is_empty() && !skills_loading.get() {
                                    view! {
                                        <div style="text-align:center;padding:40px;color:var(--muted);flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;">
                                            <div style="font-size:36px;margin-bottom:12px;">"🗺"</div>
                                            <div style="font-weight:600;margin-bottom:6px;">"No skills or agent files found"</div>
                                            <div style="font-size:12px;">"Click 'Rescan' to search for SKILL.md and AGENTS.md files in your workspace, ~/.gemini, and other agent directories."</div>
                                        </div>
                                    }.into_any()
                                } else {
                                    sources.clone().into_iter().map(|source| {
                                        let source2 = source.clone();
                                        let filtered: Vec<shared::ipc::SkillOrAgentFile> = files.iter()
                                            .filter(|f| f.source == source)
                                            .filter(|f| query.is_empty()
                                                || f.name.to_lowercase().contains(&query)
                                                || f.path.to_lowercase().contains(&query)
                                                || f.content.to_lowercase().contains(&query))
                                            .cloned()
                                            .collect();

                                        if filtered.is_empty() {
                                            return view!{}.into_any();
                                        }

                                        let mut root_items = Vec::new();
                                        for file in &filtered {
                                            let rel_path = get_relative_path(&file.path, &file.source);
                                            let segments: Vec<&str> = rel_path.split('/').collect();
                                            insert_into_tree(&mut root_items, &segments, &file.path, file.is_skill, &file.content);
                                        }
                                        sort_tree(&mut root_items);

                                        view! {
                                            <div style="background:var(--surface-card);border:1px solid var(--hairline);border-radius:var(--r-lg);overflow:hidden;margin-bottom:12px;">
                                                // Source header
                                                <div style="padding:10px 16px;background:var(--surface-soft);border-bottom:1px solid var(--hairline);display:flex;align-items:center;gap:8px;">
                                                    <span style="font-size:14px;">{if source2.contains("Gemini") { "⚡" } else if source2.contains("Plugin") { "🔌" } else { "📁" }}</span>
                                                    <span style="font-weight:700;font-size:13px;color:var(--ink);">{source.clone()}</span>
                                                    <span style="font-size:11px;color:var(--muted);margin-left:auto;">{format!("{} file{}", filtered.len(), if filtered.len() == 1 { "" } else { "s" })}</span>
                                                </div>
                                                // File Tree
                                                <div style="display:flex;flex-direction:column;gap:0;padding:8px 0;">
                                                    {root_items.into_iter().map(|item| {
                                                        view! {
                                                            <TreeViewNode
                                                                item=item
                                                                depth=0
                                                                selected_skill=selected_skill
                                                                expanded_nodes=expanded_nodes
                                                                node_key=source2.clone()
                                                            />
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        }.into_any()
                                    }).collect_view().into_any()
                                }}
                            </div>

                            // Right Column: Detail preview of the selected skill
                            {move || {
                                let sel = selected_skill.get();
                                if let Some(file) = sel {
                                    let name = file.name.clone();
                                    let path = file.path.clone();
                                    let content = file.content.clone();
                                    view! {
                                        <div style="flex:1.5 1 420px;display:flex;flex-direction:column;max-height:100%;overflow-y:auto;gap:12px;">
                                            <Card title=format!("Preview: {}", name)>
                                                <div style="font-size:11px;color:var(--muted);word-break:break-all;margin-bottom:10px;">
                                                    {path}
                                                </div>
                                                <div style="flex:1;overflow-y:auto;background:var(--canvas);border:1px solid var(--hairline);border-radius:var(--r-md);padding:14px;font-family:var(--font-mono);font-size:12px;line-height:1.6;white-space:pre-wrap;word-break:break-all;color:var(--body);max-height:560px;">
                                                    {content}
                                                </div>
                                            </Card>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div style="flex:1.5 1 420px;display:flex;align-items:center;justify-content:center;border:1px dashed var(--hairline);border-radius:var(--r-lg);background:var(--surface-soft);padding:30px;color:var(--muted);font-style:italic;text-align:center;min-height:200px;">
                                            <div>
                                                <div style="font-size:24px;margin-bottom:8px;">"📄"</div>
                                                <div>"Select a skill or agent rule file from the tree on the left to preview its content."</div>
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                }.into_any()
            } else if show_gam_simulation.get() {
                view! { <GamMemorySimulation/> }.into_any()
            } else {
                view! {
                    // Obsidian-style workspace
                    <div class="memory-workspace">
                        // Left sidebar: file list
                        <div class="memory-sidebar">
                            <div class="memory-sidebar-header">
                                <div class="memory-sidebar-title">"Knowledge Nodes"</div>
                                <input class="input memory-sidebar-search" type="text"
                                    placeholder="🔍 Filter nodes…"
                                    prop:value=move || search_query.get()
                                    on:input=move |e| search_query.set(event_target_value(&e))
                                />
                            </div>
                            <div class="memory-list">
                                // Global scope
                                <div class="memory-scope-section">
                                    <div class="memory-scope-label">"🌐 Global"</div>
                                    {move || {
                                        let list = filtered_memories();
                                        let globals: Vec<_> = list.into_iter().filter(|m| m.scope == "global").collect();
                                        if globals.is_empty() {
                                            view! { <div style="font-size:11px;color:var(--muted);padding:4px 8px;font-style:italic;">"Empty"</div> }.into_any()
                                        } else {
                                            globals.into_iter().map(|m| {
                                                let m2 = m.clone();
                                                let is_sel = move || selected_mem.get().map_or(false, |s| s.id == m2.id) || (show_new_form.get() && edit_id.get() == m2.id);
                                                view! {
                                                    <div class=move || if is_sel() { "memory-item active" } else { "memory-item" }
                                                        on:click=move |_| {
                                                            let m = m.clone();
                                                            edit_id.set(m.id.clone());
                                                            edit_title.set(m.title.clone());
                                                            edit_scope.set(m.scope.clone());
                                                            edit_tags.set(m.tags.join(", "));
                                                            edit_links.set(m.links.clone());
                                                            edit_content.set(m.content.clone());
                                                            selected_mem.set(Some(m));
                                                            show_new_form.set(false);
                                                        }
                                                    >
                                                        <span class="memory-item-icon">"📄"</span>
                                                        <span class="memory-item-title">{m2.title.clone()}</span>
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    }}
                                </div>
                                // Project scope
                                <div class="memory-scope-section">
                                    <div class="memory-scope-label">"📂 Project"</div>
                                    {move || {
                                        let list = filtered_memories();
                                        let projs: Vec<_> = list.into_iter().filter(|m| m.scope == "project").collect();
                                        if projs.is_empty() {
                                            view! { <div style="font-size:11px;color:var(--muted);padding:4px 8px;font-style:italic;">"Empty"</div> }.into_any()
                                        } else {
                                            projs.into_iter().map(|m| {
                                                let m2 = m.clone();
                                                let is_sel = move || selected_mem.get().map_or(false, |s| s.id == m2.id) || (show_new_form.get() && edit_id.get() == m2.id);
                                                view! {
                                                    <div class=move || if is_sel() { "memory-item active" } else { "memory-item" }
                                                        on:click=move |_| {
                                                            let m = m.clone();
                                                            edit_id.set(m.id.clone());
                                                            edit_title.set(m.title.clone());
                                                            edit_scope.set(m.scope.clone());
                                                            edit_tags.set(m.tags.join(", "));
                                                            edit_links.set(m.links.clone());
                                                            edit_content.set(m.content.clone());
                                                            selected_mem.set(Some(m));
                                                            show_new_form.set(false);
                                                        }
                                                    >
                                                        <span class="memory-item-icon">"📝"</span>
                                                        <span class="memory-item-title">{m2.title.clone()}</span>
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    }}
                                </div>
                            </div>
                            <div class="memory-sidebar-footer">
                                <button class="btn secondary sm" style="width:100%;font-size:11px;"
                                    on:click=move |_| load_memories()
                                >"↻ Refresh"</button>
                            </div>
                        </div>

                        // Main editor area
                        <div class="memory-editor">
                            {move || if selected_mem.get().is_some() || show_new_form.get() {
                                view! {
                                    // Toolbar
                                    <div class="memory-editor-toolbar">
                                        <button class="btn primary sm" on:click=move |_| save_memory()>"💾 Save"</button>
                                        {move || if selected_mem.get().is_some() {
                                            view! {
                                                <button class="btn danger sm" on:click=move |_| delete_memory()>"Delete"</button>
                                            }.into_any()
                                        } else { view! {}.into_any() }}
                                        <div style="flex:1;"></div>
                                        <button class="btn secondary sm" on:click=move |_| { selected_mem.set(None); show_new_form.set(false); }>
                                            "✕ Close"
                                        </button>
                                    </div>
                                    // Meta row
                                    <div class="memory-editor-meta">
                                        <div class="memory-meta-item">
                                            <span class="memory-meta-label">"Scope"</span>
                                            <select class="input" style="height:26px;font-size:12px;padding:0 6px;width:100px;"
                                                prop:value=move || edit_scope.get()
                                                on:change=move |e| edit_scope.set(event_target_value(&e))
                                            >
                                                <option value="project">"📂 Project"</option>
                                                <option value="global">"🌐 Global"</option>
                                            </select>
                                        </div>
                                        <div class="memory-meta-item">
                                            <span class="memory-meta-label">"Tags"</span>
                                            <input class="input" style="height:26px;font-size:12px;padding:0 8px;width:120px;"
                                                placeholder="rust, research, …"
                                                prop:value=move || edit_tags.get()
                                                on:input=move |e| edit_tags.set(event_target_value(&e))
                                            />
                                        </div>
                                        // Tags preview chips
                                        <div style="display:flex;flex-wrap:wrap;gap:3px;align-items:center;">
                                            {move || {
                                                edit_tags.get().split(',')
                                                    .map(|t| t.trim().to_string())
                                                    .filter(|t| !t.is_empty())
                                                    .map(|tag| view! { <span class="memory-tag-chip">{tag}</span> })
                                                    .collect_view()
                                            }}
                                        </div>
                                    </div>

                                    // Copilot Agent block
                                    <div style="background:var(--surface-soft);border-bottom:1px solid var(--hairline);padding:8px 16px;display:flex;flex-direction:column;gap:6px;">
                                        <div style="display:flex;gap:8px;align-items:center;">
                                            <span style="font-size:11px;font-weight:700;color:var(--muted);text-transform:uppercase;">"🤖 Copilot Agent Instruction"</span>
                                            <div style="flex:1;"></div>
                                            {move || if agent_running.get() {
                                                view! { <span class="agent-assist-spinner"></span> }.into_any()
                                            } else {
                                                view! {
                                                    <button class="btn primary sm" style="font-size:11px;height:24px;padding:0 8px;"
                                                        on:click=move |_| dispatch_agent_on_memory()
                                                    >"Ask Agent"</button>
                                                }.into_any()
                                            }}
                                        </div>
                                        <input class="input" style="height:28px;font-size:12px;"
                                            placeholder="Instruct the agent (e.g. 'summarize', 'find actions', leave blank for default analysis)"
                                            prop:value=move || agent_prompt.get()
                                            on:input=move |e| agent_prompt.set(event_target_value(&e))
                                        />
                                        {move || if !agent_output.get().is_empty() {
                                            view! {
                                                <div style="background:var(--canvas);border:1px solid var(--hairline);border-radius:4px;padding:8px;font-size:11.5px;max-height:100px;overflow-y:auto;white-space:pre-wrap;color:var(--body);margin-top:4px;font-family:var(--font-mono);">
                                                    {agent_output.get()}
                                                </div>
                                                {if !agent_output.get().starts_with('⏳') && !agent_output.get().starts_with('E') {
                                                    view! {
                                                        <div style="display:flex;gap:6px;margin-top:4px;">
                                                            <button class="btn ghost sm" style="font-size:10px;padding:2px 6px;" on:click=append_agent_output>"＋ Append to Note"</button>
                                                            <button class="btn ghost sm" style="font-size:10px;padding:2px 6px;color:#ef4444;" on:click=move |_| agent_output.set(String::new())>"✕ Dismiss"</button>
                                                        </div>
                                                    }.into_any()
                                                } else { view! {}.into_any() }}
                                            }.into_any()
                                        } else { view! {}.into_any() }}
                                    </div>

                                    // Title
                                    <input class="memory-title-input"
                                        placeholder="Untitled Memory"
                                        prop:value=move || edit_title.get()
                                        on:input=move |e| edit_title.set(event_target_value(&e))
                                    />
                                    // Content
                                    <textarea class="memory-content-area"
                                        placeholder="Write Markdown here…"
                                        prop:value=move || edit_content.get()
                                        on:input=move |e| edit_content.set(event_target_value(&e))
                                    ></textarea>

                                    // Backlinks panel
                                    <div style="border-top:1px solid var(--hairline);padding:10px 16px;background:var(--surface-soft);max-height:120px;overflow-y:auto;flex-shrink:0;">
                                        <div style="font-size:11px;font-weight:700;color:var(--muted);text-transform:uppercase;margin-bottom:6px;">"🔗 Backlinks"</div>
                                        <div style="display:flex;flex-wrap:wrap;gap:6px;">
                                            {move || {
                                                let bl_list = backlinks();
                                                if bl_list.is_empty() {
                                                    view! { <span style="font-size:11px;color:var(--muted);font-style:italic;">"No notes link to this node"</span> }.into_any()
                                                } else {
                                                    bl_list.into_iter().map(|other| {
                                                        let o = other.clone();
                                                        view! {
                                                            <span class="memory-tag-chip" style="cursor:pointer;background:var(--canvas);border:1px solid var(--hairline);"
                                                                on:click=move |_| {
                                                                    let o = o.clone();
                                                                    edit_id.set(o.id.clone());
                                                                    edit_title.set(o.title.clone());
                                                                    edit_scope.set(o.scope.clone());
                                                                    edit_tags.set(o.tags.join(", "));
                                                                    edit_links.set(o.links.clone());
                                                                    edit_content.set(o.content.clone());
                                                                    selected_mem.set(Some(o));
                                                                    show_new_form.set(false);
                                                                }
                                                            >
                                                                {other.title}
                                                            </span>
                                                        }
                                                    }).collect_view().into_any()
                                                }
                                            }}
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div style="flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;color:var(--muted);">
                                        <div style="font-size:48px;opacity:0.3;">"📓"</div>
                                        <div style="font-size:14px;font-weight:600;">"Select a note or create a new one"</div>
                                        <div style="font-size:12px;">"Your agent's accumulated knowledge lives here."</div>
                                        <button class="btn primary" style="margin-top:8px;"
                                            on:click=move |_| {
                                                let new_id = format!("mem-{}", js_sys::Date::now() as u64);
                                                edit_id.set(new_id);
                                                edit_title.set("Untitled".to_string());
                                                edit_scope.set("project".to_string());
                                                edit_tags.set(String::new());
                                                edit_links.set(Vec::new());
                                                edit_content.set(String::new());
                                                show_new_form.set(true);
                                                selected_mem.set(None);
                                            }
                                        >"＋ New Memory Node"</button>
                                    </div>
                                }.into_any()
                            }}
                        </div>

                        // Right graph panel (Obsidian style)
                        <div class="memory-graph-panel">
                            <div class="memory-graph-header">"🕸 Knowledge Graph"</div>
                            <div style="flex:1;position:relative;overflow:hidden;">
                                <svg width="100%" height="100%" viewBox="0 0 540 420" style="display:block;">
                                    <defs>
                                        <pattern id="graph-grid-mini" width="20" height="20" patternUnits="userSpaceOnUse">
                                            <path d="M 20 0 L 0 0 0 20" fill="none" stroke="rgba(100,116,139,0.07)" stroke-width="1"/>
                                        </pattern>
                                        <filter id="node-glow" x="-30%" y="-30%" width="160%" height="160%">
                                            <feGaussianBlur stdDeviation="3.5" result="blur" />
                                            <feComposite in="SourceGraphic" in2="blur" operator="over" />
                                        </filter>
                                    </defs>
                                    <rect width="100%" height="100%" fill="url(#graph-grid-mini)"/>
                                    {move || {
                                        let nodes = graph_node_positions();
                                        let edges = graph_edges();
                                        let sel_id = selected_mem.get().as_ref().map(|s| s.id.clone()).unwrap_or_default();
                                        
                                        let edges_view = edges.into_iter().map(|edge| {
                                            if let (Some(s), Some(t)) = (nodes.iter().find(|n| n.id == edge.source), nodes.iter().find(|n| n.id == edge.target)) {
                                                let is_hi = edge.source == sel_id || edge.target == sel_id;
                                                view! {
                                                    <line
                                                        x1={s.x} y1={s.y}
                                                        x2={t.x} y2={t.y}
                                                        stroke=if is_hi { "var(--primary)" } else { "rgba(148,163,184,0.35)" }
                                                        stroke-width=if is_hi { "2.0" } else { "1.2" }
                                                        stroke-dasharray=if is_hi { "0" } else { "4,3" }
                                                    />
                                                }.into_any()
                                            } else { view! {}.into_any() }
                                        }).collect_view();

                                        let all_mems = memories.get();
                                        let nodes_view = nodes.into_iter().map(|node| {
                                            let nid = node.id.clone();
                                            let is_sel = nid == sel_id;
                                            let fill = if node.scope == "global" {
                                                if is_sel { "var(--accent)" } else { "#6366f1" }
                                            } else {
                                                if is_sel { "#f59e0b" } else { "#a78bfa" }
                                            };
                                            let r = if is_sel { 9.0f64 } else { 6.0 };
                                            let all = all_mems.clone();
                                            view! {
                                                <g style="cursor:pointer;"
                                                    on:click=move |_| {
                                                        if let Some(mem) = all.iter().find(|m| m.id == nid) {
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
                                                >
                                                    <circle cx={node.x} cy={node.y} r={r} fill={fill}
                                                        filter=if is_sel { "url(#node-glow)" } else { "none" }
                                                        stroke=if is_sel { "#ffffff" } else { "transparent" } stroke-width="1.5"
                                                    />
                                                    <text x={node.x} y={node.y - 12.0}
                                                        text-anchor="middle" font-size="10px" font-weight=if is_sel { "700" } else { "400" } fill="var(--ink)"
                                                    >{node.title}</text>
                                                </g>
                                            }
                                        }).collect_view();
                                        view! {
                                            {edges_view}
                                            {nodes_view}
                                        }
                                    }}
                                </svg>
                            </div>
                            // Links panel for selected node
                            {move || if let Some(sel) = selected_mem.get() {
                                let all = memories.get();
                                let cur_id = sel.id.clone();
                                  let others: Vec<_> = all.into_iter().filter(|m| m.id != cur_id).collect();
                                view! {
                                    <div style="border-top:1px solid var(--hairline);padding:10px 12px;max-height:140px;overflow-y:auto;background:var(--surface-soft);">
                                        <div class="memory-meta-label" style="margin-bottom:6px;">"Linked Nodes"</div>
                                        {if others.is_empty() {
                                            view! { <div style="font-size:11px;color:var(--muted);font-style:italic;">"No other nodes"</div> }.into_any()
                                        } else {
                                            others.into_iter().map(|other| {
                                                let o_id = other.id.clone();
                                                let o_id2 = o_id.clone();
                                                let o_id3 = o_id.clone();
                                                let is_linked = move || edit_links.get().contains(&o_id2);
                                                let toggle_link = move |e| {
                                                    let checked = event_target_checked(&e);
                                                    edit_links.update(|links| {
                                                        if checked { if !links.contains(&o_id3) { links.push(o_id3.clone()); } }
                                                        else { links.retain(|l| l != &o_id3); }
                                                    });
                                                };
                                                view! {
                                                    <label style="display:flex;align-items:center;gap:5px;font-size:11.5px;cursor:pointer;color:var(--body);padding:2px 0;">
                                                        <input type="checkbox" prop:checked=is_linked on:change=toggle_link />
                                                        {other.title}
                                                    </label>
                                                }
                                            }).collect_view().into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else { view! {}.into_any() }}
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
// ── 5. McpTab / MCP Tools Registry ───────────────────────────────────────────
#[component]
pub fn McpTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let raw_config = RwSignal::new(String::new());
    let status_text = RwSignal::new(String::new());
    let show_raw_editor = RwSignal::new(false);
    let tools_collapsed = ctx.tools_collapsed;

    let load_config = move || {
        spawn_local(async move {
            if let Ok(val) = api::mcp_get_registry().await {
                raw_config.set(serde_json::to_string_pretty(&val).unwrap_or_default());
            }
        });
    };
    load_config();

    let save_config = move |_| {
        status_text.set("Saving…".to_string());
        match serde_json::from_str::<serde_json::Value>(&raw_config.get_untracked()) {
            Ok(json_val) => {
                spawn_local(async move {
                    match api::mcp_save_registry(json_val).await {
                        Ok(_) => status_text.set("Saved ✓".to_string()),
                        Err(e) => status_text.set(format!("Error: {e}")),
                    }
                });
            }
            Err(e) => status_text.set(format!("Invalid JSON: {e}")),
        }
    };

    let parsed_servers = move || {
        let raw = raw_config.get();
        let mut servers = Vec::new();
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(serde_json::Value::Object(mcp_servers)) = map.get("mcpServers") {
                for (name, server_val) in mcp_servers {
                    let cmd = server_val.get("command").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let args = server_val.get("args")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|val| val.as_str()).map(|s| s.to_string()).collect::<Vec<_>>().join(" "))
                        .unwrap_or_default();
                    let env_keys: Vec<String> = server_val.get("env")
                        .and_then(|v| v.as_object())
                        .map(|obj| obj.keys().cloned().collect())
                        .unwrap_or_default();
                    let tools: Vec<String> = server_val.get("tools")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                        .unwrap_or_default();
                    servers.push((name.clone(), cmd, args, env_keys, tools));
                }
            }
        }
        servers
    };

    view! {
        <div class="page">
            <PageHeader title="MCP Tools" desc="Model Context Protocol server registry and configuration."/>

            // Status
            {move || if !status_text.get().is_empty() {
                view! {
                    <div class="field-hint" style="padding:8px 12px;background:var(--surface-card);border:1px solid var(--hairline);border-radius:var(--r-md);">
                        {status_text.get()}
                    </div>
                }.into_any()
            } else { view! {}.into_any() }}

            // Actions bar
            <div class="row-actions">
                <button class="btn primary" on:click=save_config>"💾 Save Registry"</button>
                <button class="btn secondary sm"
                    on:click=move |_| show_raw_editor.update(|v| *v = !*v)
                >
                    {move || if show_raw_editor.get() { "📋 Card View" } else { "⚙ Raw JSON" }}
                </button>
            </div>

            {move || if show_raw_editor.get() {
                view! {
                    <Card title="Raw MCP Registry JSON">
                        <textarea class="notes-area" style="min-height:400px;font-size:12.5px;"
                            prop:value=move || raw_config.get()
                            on:input=move |e| raw_config.set(event_target_value(&e))
                        ></textarea>
                    </Card>
                }.into_any()
            } else {
                view! {
                    <div style="display: flex; flex-direction: column; gap: 20px;">
                        {{
                            view! {
                            <div class="card" style="padding:0;overflow:hidden;">
                                <button
                                    style="width:100%;display:flex;align-items:center;justify-content:space-between;padding:14px 16px;background:transparent;border:none;cursor:pointer;font-weight:600;font-size:14px;color:var(--ink);text-align:left;gap:8px;"
                                    on:click=move |_| tools_collapsed.update(|v| *v = !*v)
                                >
                                    <span>"🔧 Built-in Agent Tools"</span>
                                    <span style="font-size:11px;color:var(--muted);font-weight:400;">
                                        {move || if tools_collapsed.get() { "(collapsed — click to expand)" } else { "(click to collapse)" }}
                                    </span>
                                    <span style=move || format!("margin-left:auto;transition:transform 0.2s;transform:rotate({}deg);", if tools_collapsed.get() { -90 } else { 0 })>
                                        "▾"
                                    </span>
                                </button>
                                {move || if !tools_collapsed.get() {
                                    view! {
                                        <div style="padding:0 16px 16px;border-top:1px solid var(--hairline);">
                                        <div class="field-hint" style="margin:12px 0;">
                                            "These core tools are built directly into the agent runner, providing autonomous capabilities to inspect/update your planner, todos, notes, calendar, and model memory."
                                        </div>
                            <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 10px;">
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"📋"</span> <span>"get_todos"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Retrieve current user todo checklist."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"📋"</span> <span>"add_todo"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Add a new todo item to the checklist."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"📋"</span> <span>"complete_todo"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Mark a todo item as completed."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"📝"</span> <span>"read_notes"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Read global and project notes content."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"📝"</span> <span>"write_note"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Create or update a note with content."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"📅"</span> <span>"get_calendar_events"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Retrieve scheduled calendar events."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"📅"</span> <span>"add_calendar_event"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Add a scheduled event to the calendar."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"📅"</span> <span>"delete_calendar_event"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Delete a calendar event by ID."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"🗂"</span> <span>"get_planner_tasks"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Read tasks on the Kanban board."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"🗂"</span> <span>"add_planner_task"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Create a new Kanban task card."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"🗂"</span> <span>"update_planner_task"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Update a Kanban task card's details/status."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"🗂"</span> <span>"delete_planner_task"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Delete a task card from the planner."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"🔍"</span> <span>"web_search"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Query SearXNG search index."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"📄"</span> <span>"web_scrape"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Extract text content from any URL."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"💻"</span> <span>"run_command"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Execute bash commands on host (Sensitive)."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"🔌"</span> <span>"call_mcp_tool"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Call external Model Context Protocol tool."</div>
                                </div>
                                <div class="mcp-card" style="padding: 12px; display: flex; flex-direction: column; gap: 4px; background: var(--surface-soft);">
                                    <div style="font-weight: 600; color: var(--ink); display: flex; align-items: center; gap: 6px;">
                                        <span>"🧠"</span> <span>"spawn_subagent"</span>
                                    </div>
                                    <div style="font-size: 11px; color: var(--muted);">"Delegate a subtask to an autonomous sub-agent."</div>
                                </div>
                            </div>
                        </div>
                        }.into_any()
                        } else { view!{}.into_any() }}
                    </div>
                    }
                    }}
                        
                        <div style="display: flex; flex-direction: column; gap: 8px;">
                            <div style="font-size: 14px; font-weight: 600; color: var(--ink); margin-bottom: 4px;">"Active MCP Servers"</div>
                            <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:12px;">
                                {move || {
                                    let servers = parsed_servers();
                                    if servers.is_empty() {
                                        view! {
                                            <div style="grid-column:1/-1;text-align:center;padding:32px;color:var(--muted);font-style:italic;">
                                                "No MCP servers configured. Switch to Raw JSON editor to add servers."
                                            </div>
                                        }.into_any()
                                    } else {
                                        servers.into_iter().map(|(name, cmd, args, env_keys, tools)| {
                                            view! {
                                                <div class="mcp-card">
                                                    <div class="mcp-card-name">
                                                        <span style="font-size:18px;">"⚙"</span>
                                                        {name.clone()}
                                                        <span class="mcp-status-badge running">"Running"</span>
                                                    </div>
                                                    <div class="mcp-card-cmd" title={format!("{cmd} {args}")}>{cmd.clone()} " "{args.clone()}</div>
                                                    {if !env_keys.is_empty() {
                                                        view! {
                                                            <div style="font-size:11px;color:var(--muted);">
                                                                "Env: "
                                                                {env_keys.iter().cloned().map(|k| view! { <span class="mcp-tool-chip">{k}</span> }).collect_view()}
                                                            </div>
                                                        }.into_any()
                                                    } else { view! {}.into_any() }}
                                                    {if !tools.is_empty() {
                                                        view! {
                                                            <div>
                                                                <div style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;color:var(--muted);margin-bottom:4px;">"Tools"</div>
                                                                <div style="display:flex;flex-wrap:wrap;">
                                                                    {tools.iter().cloned().map(|t| view! { <span class="mcp-tool-chip">"🔧 "{t}</span> }).collect_view()}
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    } else { view! {}.into_any() }}
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ── 6. MonitorTab / Agent Monitor ────────────────────────────────────────────
#[component]
pub fn MonitorTab() -> impl IntoView {
    let state = RwSignal::new(MonitorState::default());
    let ctx = expect_context::<crate::state::AppCtx>();
    let new_task_input = RwSignal::new(String::new());
    let dispatch_running = RwSignal::new(false);

    let load_state = move || {
        spawn_local(async move {
            if let Ok(s) = api::monitor_load().await {
                state.set(s);
            }
        });
    };
    load_state();

    let clear_logs = move |_| {
        spawn_local(async move {
            if let Ok(s) = api::monitor_clear_events().await {
                state.set(s);
            }
        });
    };

    let poller = Interval::new(2000, move || load_state());
    poller.forget();

    let dispatch_task = move || {
        let task = new_task_input.get_untracked().trim().to_string();
        if task.is_empty() { return; }
        let (host, port, model) = ctx.resolve_target("planner");
        dispatch_running.set(true);
        spawn_local(async move {
            let req = AgentRequest { host, port, model, task };
            let _ = api::agent_start(req).await;
            dispatch_running.set(false);
        });
        new_task_input.set(String::new());
    };

    view! {
        <div class="page">
            <PageHeader title="Agent Monitor" desc="Real-time activity and operational state of running agents."/>

            <div class="monitor-stats-grid">
                <div class="monitor-stat-card">
                    <div class="monitor-stat-value">{move || state.get().agents.len()}</div>
                    <div class="monitor-stat-label">"Registered"</div>
                </div>
                <div class="monitor-stat-card">
                    <div class="monitor-stat-value">
                        {move || state.get().agents.iter().filter(|a| a.status == "Active").count()}
                    </div>
                    <div class="monitor-stat-label">"Active"</div>
                </div>
                <div class="monitor-stat-card">
                    <div class="monitor-stat-value">{move || state.get().events.len()}</div>
                    <div class="monitor-stat-label">"Events"</div>
                </div>
                <div class="monitor-stat-card">
                    <div class="monitor-stat-value">
                        {move || ctx.config.get().parallel}
                    </div>
                    <div class="monitor-stat-label">"Parallel Slots"</div>
                </div>
            </div>

            {move || {
                let cfg = ctx.config.get();
                let active = state.get().agents.iter().filter(|a| a.status == "Active").count();
                if cfg.parallel < 2 && active > 1 {
                    view! {
                        <div class="parallelism-warn">
                            <span>"⚠️"</span>
                            <div>
                                <strong>{format!("{active} agents active, but parallel = {}.", cfg.parallel)}</strong>
                                " Increase parallel slots in Server → Context and use a smaller/quantized model."
                            </div>
                        </div>
                    }.into_any()
                } else { view! {}.into_any() }
            }}

            <Card title="Dispatch Agent Task">
                <div style="display:flex;gap:var(--s-sm);align-items:center;">
                    <input class="input" style="flex:1;" type="text"
                        placeholder="Enter a task for the agent…"
                        prop:value=move || new_task_input.get()
                        on:input=move |e| new_task_input.set(event_target_value(&e))
                        on:keydown=move |e: KeyboardEvent| { if e.key() == "Enter" { dispatch_task(); } }
                    />
                    <button class="btn primary" prop:disabled=move || dispatch_running.get() on:click=move |_| dispatch_task()>
                        {move || if dispatch_running.get() { "Starting…" } else { "🤖 Dispatch" }}
                    </button>
                </div>
                <div class="field-hint" style="margin-top:4px;">"The agent will run on the configured model."</div>
            </Card>

            <Card title="Active Agent Processes">
                <div style="display:flex;flex-direction:column;gap:8px;">
                    {move || {
                        let agents = state.get().agents;
                        if agents.is_empty() {
                            view! {
                                <div style="text-align:center;padding:20px;color:var(--muted);font-size:13px;font-style:italic;">
                                    "No registered agents. Dispatch a task above to start one."
                                </div>
                            }.into_any()
                        } else {
                            agents.into_iter().map(|a| {
                                let dot_class = match a.status.as_str() {
                                    "Active"  => "monitor-agent-status-dot active",
                                    "Offline" => "monitor-agent-status-dot offline",
                                    _         => "monitor-agent-status-dot idle",
                                };
                                let (sc, sb) = match a.status.as_str() {
                                    "Active"  => ("#fb923c", "rgba(251,146,60,0.1)"),
                                    "Offline" => ("#6b7280", "rgba(107,114,128,0.1)"),
                                    _         => ("#10b981", "rgba(16,185,129,0.1)"),
                                };
                                view! {
                                    <div class="monitor-agent-card">
                                        <div class={dot_class}></div>
                                        <div style="flex:1;min-width:0;">
                                            <div style="font-weight:600;font-size:13px;color:var(--ink);display:flex;align-items:center;gap:6px;">
                                                {a.name}
                                                <span style=format!("font-size:10px;font-weight:700;padding:1px 6px;border-radius:3px;color:{sc};background:{sb};border:1px solid {sc};")>
                                                    {a.status}
                                                </span>
                                            </div>
                                            <div style="font-size:12px;color:var(--muted);margin-top:2px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">
                                                {a.current_action}
                                            </div>
                                        </div>
                                        <span style="font-size:11px;color:var(--muted);flex-shrink:0;">{a.last_active}</span>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </Card>

            <Card title="Execution Timeline">
                <div class="row-actions" style="margin-bottom:10px;justify-content:flex-end;">
                    <button class="btn secondary sm" on:click=clear_logs>"Clear Log"</button>
                </div>
                <div class="log-console" style="height:280px;padding:6px;">
                    <div class="monitor-timeline-entry" style="border-bottom:1px solid var(--hairline);margin-bottom:4px;padding-bottom:4px;">
                        <span style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;color:var(--muted);">"Timestamp"</span>
                        <span style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;color:var(--muted);">"Agent"</span>
                        <span style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;color:var(--muted);">"Message"</span>
                    </div>
                    {move || {
                        let events = state.get().events;
                        if events.is_empty() {
                            view! {
                                <div style="text-align:center;padding:20px;color:var(--muted);font-size:12px;font-style:italic;">
                                    "Timeline is empty."
                                </div>
                            }.into_any()
                        } else {
                            events.into_iter().rev().map(|e| {
                                let ts = e.timestamp.get(11..19).unwrap_or(&e.timestamp).to_string();
                                view! {
                                    <div class="monitor-timeline-entry">
                                        <span class="monitor-timeline-ts">{ts}</span>
                                        <span class="monitor-timeline-agent">{e.agent_name}</span>
                                        <span style="font-size:12px;color:var(--body);word-break:break-word;">{e.message}</span>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </Card>
        </div>
    }
}

// ── 7. PlannerTab / Kanban ───────────────────────────────────────────────────
#[component]
pub fn PlannerTab() -> impl IntoView {
    let ctx = expect_context::<crate::state::AppCtx>();
    let tasks = RwSignal::new(Vec::<KanbanTask>::new());

    // Task Composer Fields
    let add_title    = RwSignal::new(String::new());
    let add_summary  = RwSignal::new(String::new());
    let add_agent    = RwSignal::new(String::new());
    let add_priority = RwSignal::new("Medium".to_string());
    let add_status   = RwSignal::new("todo".to_string());

    // Per-card running-agent UI
    let running_card = RwSignal::new(None::<String>);
    let running_agent_id = RwSignal::new(None::<String>);
    let agent_output = RwSignal::new(String::new());
    let show_info = RwSignal::new(false);

    let load_tasks = move || {
        spawn_local(async move {
            if let Ok(state) = api::planner_load().await {
                tasks.set(state.tasks);
            }
        });
    };
    load_tasks();

    // Listen to agent events to stream progress inside the planner
    ipc::listen::<AgentEvent, _>(AGENT_EVENT, move |ev| {
        let active_id = running_agent_id.get_untracked();
        if let Some(ref _target_id) = active_id {
            match ev {
                AgentEvent::Token { agent_id, delta } => {
                    if Some(&agent_id) == active_id.as_ref() {
                        agent_output.update(|val| val.push_str(&delta));
                    }
                }
                AgentEvent::Step { thought, .. } => {
                    agent_output.update(|val| val.push_str(&format!("\n💭 {}\n", thought)));
                }
                AgentEvent::ToolCall { call, .. } => {
                    agent_output.update(|val| val.push_str(&format!("\n🔧 {}({})\n", call.tool, call.args)));
                }
                AgentEvent::ToolResult { result, .. } => {
                    let mut o = result.output.clone();
                    if o.len() > 100 {
                        o.truncate(100);
                        o.push_str("...");
                    }
                    agent_output.update(|val| val.push_str(&format!("  {} {}\n", if result.ok { "✅" } else { "⚠️" }, o)));
                }
                AgentEvent::Done { agent_id, final_text } => {
                    if Some(&agent_id) == active_id.as_ref() {
                        agent_output.set(format!("Done ✓\n\nFinal Answer:\n{}", final_text));
                        running_card.set(None);
                        running_agent_id.set(None);
                    }
                }
                AgentEvent::Error { agent_id, message } => {
                    if Some(&agent_id) == active_id.as_ref() {
                        agent_output.set(format!("❌ Error:\n{}", message));
                        running_card.set(None);
                        running_agent_id.set(None);
                    }
                }
                _ => {}
            }
        }
    });

    let save_tasks = move || {
        let list = tasks.get_untracked();
        spawn_local(async move {
            let _ = api::planner_save(list).await;
        });
    };

    let add_task = move || {
        let title = add_title.get_untracked().trim().to_string();
        let summary = add_summary.get_untracked().trim().to_string();
        if title.is_empty() { return; }
        let now = js_sys::Date::new_0();
        let created_at = format!("{}-{:02}-{:02}",
            now.get_full_year() as i32,
            (now.get_month() as i32) + 1,
            now.get_date() as i32);
        let status_raw = add_status.get_untracked();
        let status = TaskStatus::from_str(&status_raw);
        let new_task = KanbanTask {
            id: format!("task-{}", js_sys::Date::now() as u64),
            title,
            summary,
            assigned_agent: add_agent.get_untracked().trim().to_string(),
            status,
            priority: add_priority.get_untracked(),
            created_at,
        };
        tasks.update(|l| l.push(new_task));
        save_tasks();
        add_title.set(String::new());
        add_summary.set(String::new());
        add_agent.set(String::new());
    };

    let move_task = move |id: String, new_status: TaskStatus| {
        tasks.update(|list| {
            if let Some(t) = list.iter_mut().find(|t| t.id == id) {
                t.status = new_status;
            }
        });
        save_tasks();
    };

    let delete_task = move |id: String| {
        tasks.update(|list| { list.retain(|t| t.id != id); });
        save_tasks();
        if running_card.get_untracked().as_deref() == Some(&id) {
            running_card.set(None);
            running_agent_id.set(None);
            agent_output.set(String::new());
        }
    };

    // Dispatch an agent for a task's summary
    let dispatch_agent = move |task_id: String, task_prompt: String| {
        running_card.set(Some(task_id));
        agent_output.set("⏳ Dispatching agent…\n".to_string());
        let (host, port, model) = ctx.resolve_target("planner");
        spawn_local(async move {
            let req = AgentRequest { host, port, model, task: task_prompt };
            match api::agent_start(req).await {
                Ok(id) => {
                    running_agent_id.set(Some(id.clone()));
                    agent_output.update(|val| val.push_str(&format!("Agent started: {id}\nListening for events…\n")));
                }
                Err(e) => {
                    agent_output.set(format!("Error: {e}"));
                    running_card.set(None);
                }
            }
        });
    };

    // Parallelism check
    let show_parallel_warn = move || {
        let cfg = ctx.config.get();
        let active = tasks.get().into_iter().filter(|t| t.status == TaskStatus::InProgress).count();
        cfg.parallel < 2 && active > 1
    };

    let render_col = move |status: TaskStatus, col_label: &'static str, dot_color: &'static str| {
        let col_tasks: Vec<KanbanTask> = tasks.get().into_iter()
            .filter(|t| t.status == status)
            .collect();
        let count = col_tasks.len();
        let move_task_c   = move_task.clone();
        let delete_task_c = delete_task.clone();
        let dispatch_c    = dispatch_agent.clone();

        let next_status: Option<TaskStatus> = match status {
            TaskStatus::Backlog    => Some(TaskStatus::Todo),
            TaskStatus::Todo       => Some(TaskStatus::InProgress),
            TaskStatus::InProgress => Some(TaskStatus::Done),
            TaskStatus::Done       => None,
        };
        let prev_status: Option<TaskStatus> = match status {
            TaskStatus::Backlog    => None,
            TaskStatus::Todo       => Some(TaskStatus::Backlog),
            TaskStatus::InProgress => Some(TaskStatus::Todo),
            TaskStatus::Done       => Some(TaskStatus::InProgress),
        };

        view! {
            <div class="kanban-col">
                <div class="kanban-col-header">
                    <div style="display:flex;align-items:center;gap:6px;">
                        <div class="kanban-col-header-dot" style=format!("background:{dot_color};")></div>
                        {col_label}
                    </div>
                    <span class="kanban-col-count">{count}</span>
                </div>
                <div class="kanban-col-body">
                    {if col_tasks.is_empty() {
                        view! { <div class="kanban-empty">"Drop tasks here"</div> }.into_any()
                    } else {
                        col_tasks.into_iter().map(|t| {
                            let id1 = t.id.clone();
                            let id2 = t.id.clone();
                            let id_for_delete = t.id.clone();
                            let prompt = format!("Complete this task: {} — {}", t.title, t.summary);
                            let (priority_css, priority_color) = match t.priority.as_str() {
                                "High"   => ("color:#ef4444;border-color:#ef4444;", "#ef4444"),
                                "Low"    => ("color:#6b7280;border-color:#6b7280;", "#6b7280"),
                                _        => ("color:#f59e0b;border-color:#f59e0b;", "#f59e0b"),
                            };
                            let id_for_run1 = t.id.clone();
                            let is_running1 = move || running_card.get().as_deref() == Some(&id_for_run1);
                            let id_for_run2 = t.id.clone();
                            let is_running2 = move || running_card.get().as_deref() == Some(&id_for_run2);
                            view! {
                                <div class="kanban-card" style=format!("--priority-color:{priority_color};")>
                                    <div class="kanban-card-title">{t.title.clone()}</div>
                                    {if !t.summary.is_empty() {
                                        view! { <div class="kanban-card-summary">{t.summary.clone()}</div> }.into_any()
                                    } else { view! {}.into_any() }}
                                    <div style="margin-top: 8px; margin-bottom: 8px;">
                                        <button 
                                            class="btn sm primary" 
                                            style="width: 100%; height: 26px; font-size: 11px; display: flex; align-items: center; justify-content: center; gap: 4px;"
                                            prop:disabled=move || is_running1()
                                            on:click={
                                                let id = id1.clone();
                                                let p = prompt.clone();
                                                move |_| dispatch_c(id.clone(), p.clone())
                                            }
                                        >
                                            "⚡ Execute Now"
                                        </button>
                                    </div>
                                    <div class="kanban-card-meta">
                                        {if !t.assigned_agent.is_empty() {
                                            let a = t.assigned_agent.clone();
                                            view! {
                                                <span class="kanban-card-agent">"🤖 "{a}</span>
                                            }.into_any()
                                        } else {
                                            view! { <span class="kanban-card-agent">"—"</span> }.into_any()
                                        }}
                                        <div style="display:flex;align-items:center;gap:4px;">
                                            <span class="kanban-priority-badge" style=priority_css>{t.priority.clone()}</span>
                                            <span style="font-size:10px;color:var(--muted);">{t.created_at.clone()}</span>
                                        </div>
                                    </div>
                                    <div class="kanban-card-actions">
                                        {prev_status.clone().map(|ps| {
                                            let id = id1.clone();
                                            let ps_label = match ps { TaskStatus::Backlog => "← Backlog", TaskStatus::Todo => "← Todo", TaskStatus::InProgress => "← Reopen", TaskStatus::Done => "← Done" };
                                            view! {
                                                <button class="btn secondary sm" style="padding:0 6px;height:22px;font-size:10px;"
                                                    on:click=move |_| move_task_c(id.clone(), ps.clone())
                                                >{ps_label}</button>
                                            }.into_any()
                                        }).unwrap_or_else(|| view! {}.into_any())}
                                        {next_status.clone().map(|ns| {
                                            let id = id2.clone();
                                            let ns_label = match ns { TaskStatus::Todo => "Todo →", TaskStatus::InProgress => "Start →", TaskStatus::Done => "Done ✓", TaskStatus::Backlog => "Backlog" };
                                            view! {
                                                <button class="btn primary sm" style="padding:0 6px;height:22px;font-size:10px;"
                                                    on:click=move |_| move_task_c(id.clone(), ns.clone())
                                                >{ns_label}</button>
                                            }.into_any()
                                        }).unwrap_or_else(|| view! {}.into_any())}
                                        <div style="flex:1;"></div>
                                        // Agent dispatch button
                                        {move || if !is_running2() {
                                            let id = id1.clone();
                                            let p = prompt.clone();
                                            view! {
                                                <button class="btn ghost sm" style="padding:0 6px;height:22px;font-size:10px;"
                                                    title="Dispatch agent for this task"
                                                    on:click=move |_| dispatch_c(id.clone(), p.clone())
                                                >"🤖"</button>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <span class="agent-assist-spinner"></span>
                                            }.into_any()
                                        }}
                                        <button class="btn ghost sm" style="padding:0 6px;height:22px;color:#ef4444;font-size:10px;"
                                            on:click=move |_| delete_task_c(id_for_delete.clone())
                                        >"✕"</button>
                                    </div>
                                </div>
                            }
                        }).collect_view().into_any()
                    }}
                </div>
            </div>
        }
    };

    view! {
        <div class="page" style="max-width:1200px;">
            <PageHeader title="Task Planner" desc="Kanban board for agent and personal tasks."/>

            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px; margin-top:-8px;">
                <span style="font-size:12px; color:var(--muted);">"AI Agent Kanban checklist integration"</span>
                <button class="btn-info-toggle" on:click=move |_| show_info.update(|v| *v = !*v)>"i"</button>
            </div>

            {move || show_info.get().then(|| view! {
                <div class="reference-info-box">
                    <h4>"ℹ️ Referencing Planner Tasks in AI Agent"</h4>
                    "The AI agent can load, modify, and coordinate your Kanban task list via the planner tools. You can also run agents directly on a card using the 🤖 button, or interact with prompts like:"
                    <ul>
                        <li>"\"Show me my current backlog of tasks in the planner.\""</li>
                        <li>"\"Add a new high priority task 'Optimize search speed' to backlog.\""</li>
                        <li>"\"Move the task 'Fix memory leak' to In Progress.\""</li>
                        <li>"\"Tell me what tasks are currently in progress and summarize their status.\""</li>
                    </ul>
                </div>
            })}

            // Parallelism warning
            {move || if show_parallel_warn() {
                view! {
                    <div class="parallelism-warn">
                        <span>"⚠️"</span>
                        <div>
                            <strong>"Multiple tasks in progress, but parallelism is set to 1."</strong>
                            " Agents will queue and run serially. To run agents concurrently, increase "
                            <em>"Parallel"</em>" slots in the Server → Context tab and consider using a smaller/quantized model."
                        </div>
                    </div>
                }.into_any()
            } else { view! {}.into_any() }}

            // Quick-add form
            <Card title="New Task">
                <div style="display:grid;grid-template-columns:1fr 1fr 1fr 120px;gap:var(--s-sm);align-items:end;">
                    <div class="field" style="margin:0;">
                        <label class="field-label">"Title"</label>
                        <input class="input" type="text" placeholder="Task title…"
                            prop:value=move || add_title.get()
                            on:input=move |e| add_title.set(event_target_value(&e))
                            on:keydown=move |e: KeyboardEvent| { if e.key() == "Enter" { add_task(); } }
                        />
                    </div>
                    <div class="field" style="margin:0;">
                        <label class="field-label">"Summary"</label>
                        <input class="input" type="text" placeholder="Brief description…"
                            prop:value=move || add_summary.get()
                            on:input=move |e| add_summary.set(event_target_value(&e))
                        />
                    </div>
                    <div class="field" style="margin:0;">
                        <label class="field-label">"Agent"</label>
                        <input class="input" type="text" placeholder="e.g. Researcher"
                            prop:value=move || add_agent.get()
                            on:input=move |e| add_agent.set(event_target_value(&e))
                        />
                    </div>
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:6px;align-items:end;">
                        <div class="field" style="margin:0;">
                            <label class="field-label">"Priority"</label>
                            <select class="input" prop:value=move || add_priority.get() on:change=move |e| add_priority.set(event_target_value(&e))>
                                <option value="Low">"Low"</option>
                                <option value="Medium" selected>"Medium"</option>
                                <option value="High">"High"</option>
                            </select>
                        </div>
                        <div class="field" style="margin:0;">
                            <label class="field-label">"Column"</label>
                            <select class="input" prop:value=move || add_status.get() on:change=move |e| add_status.set(event_target_value(&e))>
                                <option value="backlog">"Backlog"</option>
                                <option value="todo" selected>"Todo"</option>
                                <option value="in_progress">"In Progress"</option>
                            </select>
                        </div>
                    </div>
                </div>
                <div class="row-actions" style="margin-top:var(--s-sm);">
                    <button class="btn primary" on:click=move |_| add_task()>"＋ Create Task"</button>
                    <span class="field-hint">{move || format!("{} tasks total", tasks.get().len())}</span>
                </div>
            </Card>

            // Agent output panel
            {move || if running_card.get().is_some() {
                view! {
                    <div class="agent-assist-panel">
                        <div class="agent-assist-header">
                            <span class="agent-assist-spinner"></span>
                            "Agent Running"
                        </div>
                        <div class="agent-assist-body">
                            <div class="agent-assist-output">{move || agent_output.get()}</div>
                            <div class="row-actions">
                                <button class="btn secondary sm" on:click=move |_| { running_card.set(None); agent_output.set(String::new()); }>"Dismiss"</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { view! {}.into_any() }}

            // Kanban Board
            <div class="kanban-board">
                {render_col(TaskStatus::Backlog,    "Backlog",     "#6b7280")}
                {render_col(TaskStatus::Todo,       "Todo",        "#6366f1")}
                {render_col(TaskStatus::InProgress, "In Progress", "#f59e0b")}
                {render_col(TaskStatus::Done,       "Done",        "#10b981")}
            </div>
        </div>
    }
}

// ── 8. CalendarTab / Calendar Triggers ───────────────────────────────────────
#[component]
pub fn CalendarTab() -> impl IntoView {
    use chrono::Datelike;

    let state = RwSignal::new(CalendarState::default());
    let calendar_view = RwSignal::new("month".to_string()); // "month" | "list"
    let expanded_event = RwSignal::new(None::<String>);

    // New Event Inputs
    let add_title = RwSignal::new(String::new());
    let add_time = RwSignal::new(String::new());
    let add_prompt = RwSignal::new(String::new());
    let add_note = RwSignal::new(String::new());
    let add_color = RwSignal::new("#6366f1".to_string());
    let show_info = RwSignal::new(false);

    // Current calendar month/year
    let today = chrono::Local::now();
    let view_year = RwSignal::new(today.year());
    let view_month = RwSignal::new(today.month()); // 1-12

    let load_events = move || {
        spawn_local(async move {
            if let Ok(st) = api::calendar_load().await {
                state.set(st);
            }
        });
    };

    load_events();

    let load_events_c = load_events.clone();
    let poller = Interval::new(3000, move || load_events_c());
    poller.forget();

    let add_event = move |_| {
        let title = add_title.get_untracked().trim().to_string();
        let time = add_time.get_untracked().trim().to_string();
        let prompt = add_prompt.get_untracked().trim().to_string();
        let note = add_note.get_untracked().trim().to_string();
        let color = add_color.get_untracked();

        if title.is_empty() || time.is_empty() || prompt.is_empty() {
            return;
        }

        let new_evt = CalendarEvent {
            id: format!("evt-{}", js_sys::Date::now() as u64),
            title,
            time,
            prompt,
            note: if note.is_empty() { None } else { Some(note) },
            color: Some(color),
            status: EventStatus::Pending,
            result: None,
        };

        spawn_local(async move {
            if let Ok(st) = api::calendar_add_event(new_evt).await {
                state.set(st);
            }
        });

        // Reset
        add_title.set(String::new());
        add_time.set(String::new());
        add_prompt.set(String::new());
        add_note.set(String::new());
    };

    let delete_event = move |id: String| {
        spawn_local(async move {
            if let Ok(st) = api::calendar_delete_event(id).await {
                state.set(st);
            }
        });
    };

    let days_in_month = move |year: i32, month: u32| -> u32 {
        let (y, m) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        chrono::NaiveDate::from_ymd_opt(y, m, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
            .day()
    };

    let first_weekday = move |year: i32, month: u32| -> usize {
        chrono::NaiveDate::from_ymd_opt(year, month, 1)
            .map(|d| d.weekday().num_days_from_monday() as usize)
            .unwrap_or(0)
    };

    view! {
        <div class="page">
            <PageHeader title="Calendar & AI Triggers" desc="Schedule automated prompts and triggers to run at specific times."/>

            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px; margin-top:-8px;">
                <span style="font-size:12px; color:var(--muted);">"AI Agent calendar scheduler integration"</span>
                <button class="btn-info-toggle" on:click=move |_| show_info.update(|v| *v = !*v)>"i"</button>
            </div>

            {move || show_info.get().then(|| view! {
                <div class="reference-info-box">
                    <h4>"ℹ️ Calendar AI Triggers & Referencing"</h4>
                    "The calendar system acts as a background automation engine. When a scheduled trigger fires, the system spawns an AI agent with the specified trigger prompt. The agent can use files and memory tools to act on your workspace."
                    <ul>
                        <li>"The agent will automatically run on the active model server."</li>
                        <li>"You can schedule tasks like: \"Check my project notes and extract todo items to my checklist\""</li>
                        <li>"You can schedule recurring reminders or prompt commands to run while you're offline."</li>
                        <li>"Triggers automatically update their execution status and record model outputs for your review."</li>
                    </ul>
                </div>
            })}

            // View toggle
            <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; flex-wrap: wrap; gap: 12px; width: 100%;">
                <div style="display: flex; border: 1px solid var(--color-border); border-radius: 20px; overflow: hidden; background: rgba(255,255,255,0.03); padding: 2px;">
                    <button
                        class=move || if calendar_view.get() == "month" { "btn primary sm" } else { "btn secondary sm" }
                        style="border-radius: 18px;"
                        on:click=move |_| calendar_view.set("month".to_string())
                    >
                        "📅 Month View"
                    </button>
                    <button
                        class=move || if calendar_view.get() == "list" { "btn primary sm" } else { "btn secondary sm" }
                        style="border-radius: 18px;"
                        on:click=move |_| calendar_view.set("list".to_string())
                    >
                        "📋 List View"
                    </button>
                </div>
                <div style="font-size: 12.5px; color: var(--color-text-body); opacity: 0.8;">
                    {move || format!("{} event(s) total", state.get().events.len())}
                </div>
            </div>

            <div style="display: flex; gap: var(--s-lg); flex-wrap: wrap; align-items: flex-start; width: 100%;">
                
                // Calendar Month grid or list view
                <div style="flex: 2.5; min-width: 450px; display: flex; flex-direction: column; gap: 16px;">
                    {move || {
                        if calendar_view.get() == "month" {
                            let y = view_year.get();
                            let m = view_month.get();
                            let days_count = days_in_month(y, m);
                            let first_wd = first_weekday(y, m);

                            let month_names = ["", "January", "February", "March", "April", "May", "June",
                                               "July", "August", "September", "October", "November", "December"];
                            let month_label = month_names.get(m as usize).copied().unwrap_or("");

                            let month_prefix = format!("{}-{:02}", y, m);
                            let month_events: Vec<CalendarEvent> = state.get().events.into_iter()
                                .filter(|e| e.time.starts_with(&month_prefix))
                                .collect();

                            let prev_month = move |_| {
                                if m == 1 {
                                    view_month.set(12);
                                    view_year.set(y - 1);
                                } else {
                                    view_month.set(m - 1);
                                }
                            };

                            let next_month = move |_| {
                                if m == 12 {
                                    view_month.set(1);
                                    view_year.set(y + 1);
                                } else {
                                    view_month.set(m + 1);
                                }
                            };

                            view! {
                                <Card title="">
                                    <div style="padding: 16px;">
                                        // Header
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                                            <button class="btn ghost sm" on:click=prev_month>"‹ Prev"</button>
                                            <span style="font-size: 16px; font-weight: 700; color: var(--color-text-header);">{month_label} " " {y}</span>
                                            <button class="btn ghost sm" on:click=next_month>"Next ›"</button>
                                        </div>

                                        // Weekday headers
                                        <div style="display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; margin-bottom: 8px;">
                                            {["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].into_iter().map(|day_name| {
                                                view! {
                                                    <div style="text-align: center; font-size: 11px; font-weight: 700; color: var(--color-muted); text-transform: uppercase; letter-spacing: 0.05em; padding: 4px;">
                                                        {day_name}
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>

                                        // Days Grid
                                        <div style="display: grid; grid-template-columns: repeat(7, 1fr); gap: 4px;">
                                            // Empty cell prefixes
                                            {
                                                (0..first_wd).map(|_| {
                                                    view! { <div style="min-height: 70px; border-radius: 6px;"></div> }
                                                }).collect_view()
                                            }

                                            // Days
                                            {
                                                let today_day = today.day();
                                                let today_month = today.month();
                                                let today_year = today.year();

                                                (1..=days_count).map(move |day| {
                                                    let day_str = format!("{}-{:02}-{:02}", y, m, day);
                                                    let day_events: Vec<CalendarEvent> = month_events.iter()
                                                        .filter(|e| e.time.starts_with(&day_str))
                                                        .cloned()
                                                        .collect();

                                                    let is_today = today_day == day && today_month == m && today_year == y;
                                                    let day_str_c = day_str.clone();
                                                    let day_str_c2 = day_str.clone();
                                                    let is_selected = move || add_time.get().starts_with(&day_str_c2);

                                                    view! {
                                                        <div
                                                            class="calendar-day-cell"
                                                            style=move || format!(
                                                                "min-height: 75px; border-radius: 6px; padding: 6px; border: 1px solid {}; background: {}; transition: all 0.2s; display: flex; flex-direction: column; cursor: pointer; transform: {};",
                                                                if is_today {
                                                                    "var(--accent)"
                                                                } else if is_selected() {
                                                                    "var(--primary)"
                                                                } else {
                                                                    "var(--hairline)"
                                                                },
                                                                if is_today {
                                                                    "color-mix(in srgb, var(--accent) 12%, transparent)"
                                                                } else if is_selected() {
                                                                    "color-mix(in srgb, var(--primary) 8%, transparent)"
                                                                } else {
                                                                    "transparent"
                                                                },
                                                                if is_selected() { "scale(0.97)" } else { "none" }
                                                            )
                                                            on:click=move |_| {
                                                                add_time.set(format!("{}T09:00", day_str_c));
                                                            }
                                                        >
                                                            <div style=format!(
                                                                "font-size: 12px; font-weight: {}; color: {}; margin-bottom: 4px;",
                                                                if is_today { "800" } else { "500" },
                                                                if is_today { "var(--accent)" } else { "var(--ink)" }
                                                            )>
                                                                {day}
                                                            </div>

                                                            <div style="display: flex; flex-direction: column; gap: 2px; overflow: hidden; flex: 1;">
                                                                {
                                                                    day_events.iter().take(3).map(|ev| {
                                                                        let id = ev.id.clone();
                                                                        let ev_title = ev.title.clone();
                                                                        let ev_title_render = ev_title.clone();
                                                                        let bg = ev.color.clone().unwrap_or_else(|| "#6366f1".to_string());
                                                                        view! {
                                                                            <div
                                                                                title=ev_title_render
                                                                                style=format!(
                                                                                    "font-size: 9px; padding: 2px 4px; border-radius: 3px; color: white; background: {}; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; cursor: pointer;",
                                                                                    bg
                                                                                )
                                                                                on:click=move |e| {
                                                                                    e.stop_propagation();
                                                                                    expanded_event.set(Some(id.clone()));
                                                                                }
                                                                            >
                                                                                {ev_title}
                                                                            </div>
                                                                        }
                                                                    }).collect_view()
                                                                }
                                                                {
                                                                    if day_events.len() > 3 {
                                                                        view! {
                                                                            <div style="font-size: 9px; color: var(--color-muted); padding-left: 2px; font-weight: 600;">
                                                                                {format!("+{} more", day_events.len() - 3)}
                                                                            </div>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {}.into_any()
                                                                    }
                                                                }
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()
                                            }
                                        </div>
                                    </div>
                                </Card>
                            }.into_any()
                        } else {
                            // List View
                            view! {
                                <Card title="Timeline List">
                                    <div style="display: flex; flex-direction: column; gap: 10px; padding: 12px;">
                                        {move || {
                                            let events = state.get().events;
                                            if events.is_empty() {
                                                view! { <div style="text-align: center; color: var(--color-muted); font-style: italic;">"No scheduled events."</div> }.into_any()
                                            } else {
                                                events.into_iter().map(|e| {
                                                    let id = e.id.clone();
                                                    let bg = e.color.clone().unwrap_or_else(|| "#6366f1".to_string());
                                                    let status_text = match e.status {
                                                        EventStatus::Pending => "Pending",
                                                        EventStatus::Firing => "Firing...",
                                                        EventStatus::Completed => "Completed",
                                                        EventStatus::Failed => "Failed",
                                                    };
                                                    view! {
                                                        <div
                                                            style=format!("border-left: 4px solid {}; border: 1px solid var(--color-border); border-radius: 6px; padding: 10px; cursor: pointer; background: rgba(255,255,255,0.01);", bg)
                                                            on:click=move |_| expanded_event.set(Some(id.clone()))
                                                        >
                                                            <div style="display: flex; justify-content: space-between; align-items: center;">
                                                                <span style="font-weight: 600; font-size: 13.5px; color: var(--color-text-header);">{e.title}</span>
                                                                <span style="font-size: 10px; opacity: 0.8;">{status_text}</span>
                                                            </div>
                                                            <div style="font-size: 11px; color: var(--color-muted); margin-top: 4px;">
                                                                {format!("Time: {} · Prompt: \"{}\"", e.time, e.prompt)}
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view().into_any()
                                            }
                                        }}
                                    </div>
                                </Card>
                            }.into_any()
                        }
                    }}
                </div>

                // Right Panel: Form + Expanded view
                <div style="flex: 1.5; min-width: 300px; display: flex; flex-direction: column; gap: 16px;">
                    
                    // Expanded event details
                    {move || {
                        if let Some(eid) = expanded_event.get() {
                            if let Some(ev) = state.get().events.into_iter().find(|e| e.id == eid) {
                                let id = ev.id.clone();
                                let _bg = ev.color.clone().unwrap_or_else(|| "#6366f1".to_string());
                                let status_style = match ev.status {
                                    EventStatus::Completed => "background: #10b981; color: white;",
                                    EventStatus::Firing => "background: #fb923c; color: white;",
                                    EventStatus::Failed => "background: #ef4444; color: white;",
                                    _ => "background: #8b5cf6; color: white;",
                                };
                                let delete_id = id.clone();
                                view! {
                                    <Card title="Event Details">
                                        <div style="padding: 12px; display: flex; flex-direction: column; gap: 10px;">
                                            <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                                                <div>
                                                    <div style="font-weight: 700; font-size: 15px; color: var(--color-text-header);">{ev.title}</div>
                                                    <div style="font-size: 12px; color: var(--color-muted); margin-top: 2px;">"🕒 " {ev.time}</div>
                                                </div>
                                                <span style=format!("font-size: 10px; padding: 2px 8px; border-radius: 4px; font-weight: 600; {}", status_style)>
                                                    {match ev.status {
                                                        EventStatus::Pending => "Pending",
                                                        EventStatus::Firing => "Firing",
                                                        EventStatus::Completed => "Completed",
                                                        EventStatus::Failed => "Failed",
                                                    }}
                                                </span>
                                            </div>

                                            {ev.note.as_ref().map(|n| view! {
                                                <div style="font-size: 12.5px; font-style: italic; background: rgba(255,255,255,0.02); padding: 8px; border-radius: 6px; border-left: 3px solid var(--color-accent);">
                                                    "📌 " {n.clone()}
                                                </div>
                                            })}

                                            <div style="background: rgba(0,0,0,0.15); padding: 8px; border-radius: 4px; font-size: 12px; border: 1px solid var(--color-border);">
                                                <strong>"Trigger Prompt:"</strong>
                                                <div style="margin-top: 4px; font-family: monospace; white-space: pre-wrap; word-break: break-all;">{ev.prompt}</div>
                                            </div>

                                            {ev.result.as_ref().map(|res| view! {
                                                <div style="background: rgba(0,0,0,0.25); padding: 8px; border-radius: 4px; font-size: 11px; max-height: 150px; overflow-y: auto;">
                                                    <strong>"Execution Output:"</strong>
                                                    <pre style="margin-top: 4px; font-family: monospace; white-space: pre-wrap; word-break: break-all; color: #a5b4fc;">{res.clone()}</pre>
                                                </div>
                                            })}

                                            <div style="display: flex; justify-content: space-between; margin-top: 8px;">
                                                <button class="btn ghost sm" on:click=move |_| expanded_event.set(None)>"Close Details"</button>
                                                <button class="btn secondary sm" style="color: #ef4444;" on:click=move |_| {
                                                    delete_event(delete_id.clone());
                                                    expanded_event.set(None);
                                                }>"Delete Event"</button>
                                            </div>
                                        </div>
                                    </Card>
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            }
                        } else {
                            view! {}.into_any()
                        }
                    }}

                    // Scheduler form
                    <Card title="Add Scheduled Trigger">
                        <div style="display: flex; flex-direction: column; gap: var(--s-sm); padding: 12px;">
                            <div class="field">
                                <label class="field-label">"Trigger Title"</label>
                                <input class="input" type="text" placeholder="e.g. Daily Sync Notes"
                                    prop:value=move || add_title.get()
                                    on:input=move |e| add_title.set(event_target_value(&e))
                                />
                            </div>

                            <div class="field">
                                <label class="field-label">"Target Date/Time"</label>
                                <input class="input" type="datetime-local"
                                    prop:value=move || add_time.get()
                                    on:input=move |e| add_time.set(event_target_value(&e))
                                />
                            </div>

                            <div class="field">
                                <label class="field-label">"Prompt Trigger Command"</label>
                                <input class="input" type="text" placeholder="Prompt instruction to fire..."
                                    prop:value=move || add_prompt.get()
                                    on:input=move |e| add_prompt.set(event_target_value(&e))
                                />
                            </div>

                            <div class="field">
                                <label class="field-label">"Optional Notes"</label>
                                <input class="input" type="text" placeholder="Notes for self..."
                                    prop:value=move || add_note.get()
                                    on:input=move |e| add_note.set(event_target_value(&e))
                                />
                            </div>

                            <div class="field">
                                <label class="field-label">"Color Pill"</label>
                                <div style="display: flex; gap: 8px; margin-top: 4px;">
                                    {["#6366f1", "#8b5cf6", "#10b981", "#ef4444", "#f59e0b"].into_iter().map(|color| {
                                        let c = color.to_string();
                                        let is_sel = move || add_color.get() == c;
                                        view! {
                                            <div
                                                style=move || format!(
                                                    "width: 20px; height: 20px; border-radius: 50%; background: {}; cursor: pointer; border: 2px solid {}; box-sizing: border-box;",
                                                    color,
                                                    if is_sel() { "white" } else { "transparent" }
                                                )
                                                on:click=move |_| add_color.set(color.to_string())
                                            ></div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>

                            <div class="row-actions" style="margin-top: 10px;">
                                <button class="btn primary" on:click=add_event>"Schedule Trigger"</button>
                            </div>
                        </div>
                    </Card>

                </div>

            </div>
        </div>
    }
}

// ── 9. CompareTab / Benchmarking & Evaluation ────────────────────────────────
#[component]
pub fn CompareTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let mode = RwSignal::new("bench"); // "bench" or "eval"

    // llama-benchy fields
    let bench_url = RwSignal::new(String::new());
    let bench_model = RwSignal::new(String::new());
    let bench_pp = RwSignal::new("512 1024 2048".to_string());
    let bench_tg = RwSignal::new("128 256".to_string());
    let bench_runs = RwSignal::new("5".to_string());
    let bench_output = RwSignal::new(None::<BenchmarkOutput>);
    let is_benching = RwSignal::new(false);
    let bench_error = RwSignal::new(String::new());

    // Prompt Evaluation fields
    let eval_url = RwSignal::new(String::new());
    let eval_model_a = RwSignal::new(String::new());
    let eval_model_b = RwSignal::new(String::new());
    let eval_prompt = RwSignal::new(String::new());
    let eval_res_a = RwSignal::new(String::new());
    let eval_res_b = RwSignal::new(String::new());
    let is_evaluating = RwSignal::new(false);
    let reveal_blind = RwSignal::new(false);
    let vote_status = RwSignal::new(String::new());

    // Initialize & Sync from global routed server or overrides
    Effect::new(move |_| {
        let (host, port, model) = ctx.resolve_target("compare");
        bench_url.set(format!("http://{}:{}", host, port));
        bench_model.set(model.clone());
        eval_url.set(format!("http://{}:{}", host, port));
        eval_model_a.set(model);
    });

    let run_benchmark = move |_| {
        is_benching.set(true);
        bench_error.set(String::new());
        bench_output.set(None);
        let url = bench_url.get_untracked();
        let model = bench_model.get_untracked();
        let pp = bench_pp.get_untracked();
        let tg = bench_tg.get_untracked();
        let runs = bench_runs.get_untracked();

        spawn_local(async move {
            match api::compare_run_bench(url, model, pp, tg, runs).await {
                Ok(out) => {
                    bench_output.set(Some(out));
                }
                Err(e) => {
                    bench_error.set(e);
                }
            }
            is_benching.set(false);
        });
    };

    let run_eval = move |_| {
        is_evaluating.set(true);
        reveal_blind.set(false);
        vote_status.set(String::new());
        eval_res_a.set("Running Model A evaluation...".to_string());
        eval_res_b.set("Running Model B evaluation...".to_string());

        let url = eval_url.get_untracked();
        let model_a = eval_model_a.get_untracked();
        let model_b = eval_model_b.get_untracked();
        let prompt = eval_prompt.get_untracked();

        let u1 = url.clone();
        let p1 = prompt.clone();
        spawn_local(async move {
            match api::compare_run_eval(u1, model_a, p1).await {
                Ok(res) => eval_res_a.set(res),
                Err(e) => eval_res_a.set(format!("Error: {}", e)),
            }
        });

        let u2 = url.clone();
        let p2 = prompt.clone();
        spawn_local(async move {
            match api::compare_run_eval(u2, model_b, p2).await {
                Ok(res) => eval_res_b.set(res),
                Err(e) => eval_res_b.set(format!("Error: {}", e)),
            }
            is_evaluating.set(false);
        });
    };

    view! {
        <div class="page">
            <PageHeader title="Compare Models" desc="Execute llama-benchy performance suites or blind side-by-side prompt testing."/>

            // Mode Selector Pill
            <div style="display: flex; border: 1px solid var(--hairline); border-radius: 20px; overflow: hidden; background: var(--surface-soft); padding: 2px; width: fit-content; margin-bottom: var(--s-lg);">
                <button
                    class=move || if mode.get() == "bench" { "btn primary sm" } else { "btn ghost sm" }
                    style="border-radius: 18px; margin: 0; padding: 4px 16px;"
                    on:click=move |_| mode.set("bench")
                >
                    "Performance Benchmark"
                </button>
                <button
                    class=move || if mode.get() == "eval" { "btn primary sm" } else { "btn ghost sm" }
                    style="border-radius: 18px; margin: 0; padding: 4px 16px;"
                    on:click=move |_| mode.set("eval")
                >
                    "Blind Prompt Test"
                </button>
            </div>

            {move || if mode.get() == "bench" {
                view! {
                    <Card title="Benchy Config">
                        <div style="display: grid; grid-template-columns: 2fr 1fr; gap: var(--s-md); margin-bottom: 12px;">
                            <div class="field">
                                <label class="field-label">"Target Endpoint URL"</label>
                                <input class="input" type="text" prop:value=move || bench_url.get()
                                    on:input=move |e| bench_url.set(event_target_value(&e))
                                />
                            </div>
                            <div class="field">
                                <label class="field-label">"Model Reference Name"</label>
                                <input class="input" type="text" placeholder="e.g. meta-llama-3"
                                    prop:value=move || bench_model.get()
                                    on:input=move |e| bench_model.set(event_target_value(&e))
                                />
                            </div>
                        </div>

                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: var(--s-md); margin-bottom: 12px;">
                            <div class="field">
                                <label class="field-label">"Prompt Processing (PP)"</label>
                                <input class="input" type="text" prop:value=move || bench_pp.get()
                                    on:input=move |e| bench_pp.set(event_target_value(&e))
                                />
                            </div>
                            <div class="field">
                                <label class="field-label">"Token Generation (TG)"</label>
                                <input class="input" type="text" prop:value=move || bench_tg.get()
                                    on:input=move |e| bench_tg.set(event_target_value(&e))
                                />
                            </div>
                            <div class="field">
                                <label class="field-label">"Bench Runs"</label>
                                <input class="input" type="number" prop:value=move || bench_runs.get()
                                    on:input=move |e| bench_runs.set(event_target_value(&e))
                                />
                            </div>
                        </div>

                        <div class="row-actions">
                            <button class="btn primary" disabled=move || is_benching.get() on:click=run_benchmark>
                                {move || if is_benching.get() { "Running Benchmark..." } else { "Run Performance Suite" }}
                            </button>
                        </div>
                    </Card>

                    {move || {
                        let err = bench_error.get();
                        (!err.is_empty()).then(|| view! {
                            <div class="banner warn" style="margin-top: 16px;">
                                {err}
                            </div>
                        })
                    }}

                    {move || {
                        bench_output.get().map(|out| {
                            view! {
                                <Card title="Benchmark Speeds">
                                    <table style="width: 100%; border-collapse: collapse; text-align: left; font-size: 13.5px; margin-bottom: 16px;">
                                        <thead>
                                            <tr style="border-bottom: 2px solid var(--hairline); color: var(--muted); font-weight: 600;">
                                                <th style="padding: 8px;">"PP Tokens"</th>
                                                <th style="padding: 8px;">"TG Tokens"</th>
                                                <th style="padding: 8px;">"PP Speed"</th>
                                                <th style="padding: 8px;">"TG Speed"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {out.results.into_iter().map(|res| {
                                                view! {
                                                    <tr style="border-bottom: 1px solid var(--hairline);">
                                                        <td style="padding: 8px;">{res.pp}</td>
                                                        <td style="padding: 8px;">{res.tg}</td>
                                                        <td style="padding: 8px; font-weight: 600; color: var(--primary);">{format!("{} t/s", res.pp_speed)}</td>
                                                        <td style="padding: 8px; font-weight: 600; color: var(--primary);">{format!("{} t/s", res.tg_speed)}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </Card>

                                <Card title="Raw Terminal Output">
                                    <div class="cmd-preview" style="max-height: 280px; overflow-y: auto;">
                                        <code>{out.raw}</code>
                                    </div>
                                </Card>
                            }
                        })
                    }}
                }.into_any()
            } else {
                view! {
                    <Card title="Blind Prompt Configuration">
                        <div style="display: grid; grid-template-columns: 2fr 1fr; gap: var(--s-md); margin-bottom: 12px;">
                            <div class="field">
                                <label class="field-label">"Endpoint API URL"</label>
                                <input class="input" type="text" prop:value=move || eval_url.get()
                                    on:input=move |e| eval_url.set(event_target_value(&e))
                                />
                            </div>
                        </div>
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-md); margin-bottom: 12px;">
                            <div class="field">
                                <label class="field-label">"Model A Name"</label>
                                <input class="input" type="text" placeholder="e.g. meta-llama-3"
                                    prop:value=move || eval_model_a.get()
                                    on:input=move |e| eval_model_a.set(event_target_value(&e))
                                />
                            </div>
                            <div class="field">
                                <label class="field-label">"Model B Name"</label>
                                <input class="input" type="text" placeholder="e.g. gemma-7b"
                                    prop:value=move || eval_model_b.get()
                                    on:input=move |e| eval_model_b.set(event_target_value(&e))
                                />
                            </div>
                        </div>
                        <div class="field">
                            <label class="field-label">"Evaluation Prompt"</label>
                            <textarea
                                class="notes-area"
                                style="min-height: 100px; font-family: inherit;"
                                placeholder="Enter prompt to execute side-by-side..."
                                prop:value=move || eval_prompt.get()
                                on:input=move |e| eval_prompt.set(event_target_value(&e))
                            ></textarea>
                        </div>
                        <div class="row-actions">
                            <button class="btn primary" disabled=move || is_evaluating.get() on:click=run_eval>
                                {move || if is_evaluating.get() { "Running Evaluations..." } else { "Run Blind Test" }}
                            </button>
                        </div>
                    </Card>

                    <div style="display: flex; gap: 16px; flex-wrap: wrap; margin-top: 16px; width: 100%;">
                        <div style="flex: 1 1 320px; max-width: 100%;">
                            <Card title="Model 1 Output">
                                <div class="log-console" style="height: 320px; font-family: inherit; font-size: 13.5px; background: var(--canvas); border-color: var(--hairline);">
                                    {move || eval_res_a.get()}
                                </div>
                                {move || reveal_blind.get().then(|| {
                                    let name = eval_model_a.get();
                                    view! { <div style="margin-top: 8px; font-weight: 600; font-size: 13px;">{format!("Model Name: {}", name)}</div> }
                                })}
                            </Card>
                        </div>

                        <div style="flex: 1 1 320px; max-width: 100%;">
                            <Card title="Model 2 Output">
                                <div class="log-console" style="height: 320px; font-family: inherit; font-size: 13.5px; background: var(--canvas); border-color: var(--hairline);">
                                    {move || eval_res_b.get()}
                                </div>
                                {move || reveal_blind.get().then(|| {
                                    let name = eval_model_b.get();
                                    view! { <div style="margin-top: 8px; font-weight: 600; font-size: 13px;">{format!("Model Name: {}", name)}</div> }
                                })}
                            </Card>
                        </div>
                    </div>

                    <div style="margin-top: 16px;">
                        <Card title="Judge responses">
                            <div class="row-actions">
                                <button class="btn secondary" disabled=move || is_evaluating.get() on:click=move |_| vote_status.set("Voted for Model 1!".to_string())>
                                    "Vote Model 1"
                                </button>
                                <button class="btn secondary" disabled=move || is_evaluating.get() on:click=move |_| vote_status.set("Voted for Model 2!".to_string())>
                                    "Vote Model 2"
                                </button>
                                <button class="btn primary" on:click=move |_| reveal_blind.set(true)>
                                    "Reveal Model Names"
                                </button>
                            </div>
                            {move || {
                                let text = vote_status.get();
                                (!text.is_empty()).then(|| view! {
                                    <div style="margin-top: 10px; font-weight: 600; color: #10b981;">{text}</div>
                                })
                            }}
                        </Card>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ── 10. DeepResearchTab ──────────────────────────────────────────────────────
#[component]
pub fn DeepResearchTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let query = RwSignal::new(String::new());
    let host = RwSignal::new(String::new());
    let port = RwSignal::new(String::new());
    let model = RwSignal::new(String::new());
    let available_models = RwSignal::new(Vec::<String>::new());
    let custom_model_revealed = RwSignal::new(false);

    // Initialize & Sync from global routed server or overrides
    Effect::new(move |_| {
        let (h, p, m) = ctx.resolve_target("research");
        host.set(h.clone());
        port.set(p.to_string());
        if model.get_untracked().is_empty() {
            model.set(m);
        }
        // Fetch available models from the server and local library
        let h2 = h;
        spawn_local(async move {
            let mut list_all = Vec::new();
            
            // 1. Chat models from active server
            if let Ok(list) = api::chat_list_models(h2, p).await {
                for m in list.models {
                    if !m.is_empty() && !list_all.contains(&m) {
                        list_all.push(m);
                    }
                }
            }
            
            // 2. Local models from library
            if let Ok(scanned) = api::library_get_index().await {
                for m in scanned {
                    if !m.clean_name.is_empty() && !list_all.contains(&m.clean_name) {
                        list_all.push(m.clean_name);
                    }
                    if !m.filename.is_empty() && !list_all.contains(&m.filename) {
                        list_all.push(m.filename);
                    }
                }
            }
            
            // 3. Standard fallbacks
            let fallbacks = vec![
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
                "gpt-4o".to_string(),
                "claude-3-5-sonnet".to_string(),
                "llama3".to_string(),
                "hermes-2-pro".to_string(),
            ];
            for f in fallbacks {
                if !list_all.contains(&f) {
                    list_all.push(f);
                }
            }
            
            available_models.set(list_all);
        });
    });

    let show_custom_input = move || {
        let cur = model.get();
        custom_model_revealed.get() || (!cur.is_empty() && !available_models.get().contains(&cur))
    };

    let select_value = move || {
        let cur = model.get();
        if available_models.get().contains(&cur) {
            cur
        } else if cur.is_empty() && !custom_model_revealed.get() {
            "".to_string()
        } else {
            "__custom__".to_string()
        }
    };

    let status = RwSignal::new(ResearchStatus {
        is_running: false,
        log: String::new(),
        report: String::new(),
    });

    let reports = RwSignal::new(Vec::<ResearchReportInfo>::new());
    let selected_report = RwSignal::new(String::new());
    let active_report_name = RwSignal::new(String::new());
    let status_text = RwSignal::new(String::new());

    let load_reports = move || {
        spawn_local(async move {
            if let Ok(list) = api::deep_research_list_reports().await {
                reports.set(list);
            }
        });
    };

    let check_status = move || {
        spawn_local(async move {
            if let Ok(st) = api::deep_research_status().await {
                status.set(st);
            }
        });
    };

    load_reports();
    check_status();

    // Poll status when running
    let check_status_cb = check_status;
    let poller = Interval::new(2000, move || {
        check_status_cb();
    });
    poller.forget();

    let start_campaign = move |_| {
        status_text.set("Launching deep research Python agent...".to_string());
        let q = query.get_untracked();
        let h = host.get_untracked();
        let p_parsed = port.get_untracked().parse::<u16>().unwrap_or(8080);
        let m = model.get_untracked();

        spawn_local(async move {
            match api::deep_research_start(q, h, p_parsed, m).await {
                Ok(_) => {
                    status_text.set("Research running in background.".to_string());
                    status.update(|s| s.is_running = true);
                }
                Err(e) => {
                    status_text.set(format!("Failed: {}", e));
                }
            }
        });
    };

    let cancel_campaign = move |_| {
        spawn_local(async move {
            if let Ok(_) = api::deep_research_cancel().await {
                status.update(|s| s.is_running = false);
                status_text.set("Research campaign cancelled.".to_string());
            }
        });
    };

    let read_report = move |filename: String| {
        let f = filename.clone();
        active_report_name.set(filename.clone());
        spawn_local(async move {
            if let Ok(content) = api::deep_research_read_report(f).await {
                selected_report.set(content);
            }
        });
    };

    view! {
        <div class="page">
            <PageHeader title="Deep Research Labs" desc="Conduct iterative research campaigns wrapped around deep_research.py."/>

            <div style="display: flex; flex-wrap: wrap; gap: 16px; align-items: start; width: 100%;">
                // Left Column: Inputs & Status
                <div style="flex: 1 1 320px; display: flex; flex-direction: column; gap: 16px; max-width: 100%;">
                    <Card title="Start Research Campaign">
                        <div class="field" style="margin-bottom: 12px;">
                            <label class="field-label">"Target Query / Research Topic"</label>
                            <textarea
                                class="notes-area"
                                style="min-height: 80px; font-family: inherit; font-size: 13px;"
                                placeholder="e.g. Research Llama 3 alignment updates and post-training methods."
                                prop:value=move || query.get()
                                on:input=move |e| query.set(event_target_value(&e))
                            ></textarea>
                        </div>


                        <div class="field" style="margin-bottom: 12px;">
                            <label class="field-label">"Model"</label>
                            <select class="input" style="height:34px;font-size:13px;"
                                prop:value=select_value
                                on:change=move |e| {
                                    let val = event_target_value(&e);
                                    if val == "__custom__" {
                                        custom_model_revealed.set(true);
                                        model.set(String::new());
                                    } else {
                                        custom_model_revealed.set(false);
                                        model.set(val);
                                    }
                                }
                            >
                                <option value="">"-- Select Model --"</option>
                                {move || {
                                    available_models.get().into_iter().map(|m| {
                                        let m_val = m.clone();
                                        view! { <option value=m_val>{m}</option> }
                                    }).collect_view()
                                }}
                                <option value="__custom__">"Custom Model..."</option>
                            </select>
                            {move || {
                                show_custom_input().then(|| {
                                    view! {
                                        <input class="input" style="height:32px;font-size:13px;margin-top:8px;" type="text" placeholder="Enter custom model name..."
                                            prop:value=move || model.get()
                                            on:input=move |e| model.set(event_target_value(&e))
                                        />
                                    }
                                })
                            }}
                        </div>

                        <div class="row-actions" style="margin-top: 8px; gap: 8px;">
                            <button class="btn primary sm" style="flex: 1;" disabled=move || status.get().is_running on:click=start_campaign>
                                {move || if status.get().is_running { "Running..." } else { "Launch Research" }}
                            </button>
                            <button class="btn danger sm" disabled=move || !status.get().is_running on:click=cancel_campaign>
                                "Cancel"
                            </button>
                        </div>

                        {move || {
                            let text = status_text.get();
                            (!text.is_empty()).then(|| view! {
                                <div class="field-hint" style="margin-top: 8px; font-weight: 500; font-size: 11px;">
                                    {text}
                                </div>
                            })
                        }}
                    </Card>

                    <Card title="Live Timeline Log">
                        <div class="log-console" style="height: 240px; font-size: 11.5px; border-radius: var(--r-md);">
                            {move || {
                                let l = status.get().log;
                                if l.is_empty() {
                                    view! { <span class="log-empty">"Logs will appear here once deep research launches."</span> }.into_any()
                                } else {
                                    view! { <pre class="log-line" style="margin: 0; font-family: var(--font-mono);">{l}</pre> }.into_any()
                                }
                            }}
                        </div>
                    </Card>
                </div>

                // Right Column: Consolidated Reports & Insights Card
                <div style="flex: 2 1 480px; display: flex; flex-direction: column; gap: 16px; min-width: 320px; max-width: 100%;">
                    <Card title="">
                        <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--hairline); padding-bottom: 12px; margin-bottom: 16px; flex-wrap: wrap; gap: 8px;">
                            <div style="display: flex; flex-direction: column;">
                                <span style="font-size: 15px; font-weight: 600; color: var(--ink);">"Research Reports & Insights"</span>
                                <span style="font-size: 12px; color: var(--muted);">"Select a saved markdown report to view deep analysis"</span>
                            </div>
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <select class="input" style="height: 32px; font-size: 13px; min-width: 180px; padding: 0 10px; border-radius: var(--r-md);"
                                    prop:value=move || active_report_name.get()
                                    on:change=move |e| {
                                        let val = event_target_value(&e);
                                        if !val.is_empty() {
                                            read_report(val);
                                        }
                                    }
                                >
                                    <option value="">"Select a report..."</option>
                                    {move || {
                                        reports.get().into_iter().map(|info| {
                                            let f = info.filename.clone();
                                            let f_val = f.clone();
                                            view! { <option value=f_val>{f}</option> }
                                        }).collect_view()
                                    }}
                                </select>
                                <button class="btn secondary sm" style="height: 32px; display: flex; align-items: center; gap: 4px; border-radius: var(--r-md);" on:click=move |_| load_reports()>
                                    <span>"🔄"</span> "Refresh"
                                </button>
                            </div>
                        </div>

                        <div class="log-console" style="height: 480px; font-family: inherit; font-size: 13.5px; background: var(--canvas); border-color: var(--hairline); overflow-y: auto; padding: 16px; border-radius: var(--r-md);">
                            {move || {
                                let r = selected_report.get();
                                if r.is_empty() {
                                    view! { 
                                        <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: var(--muted); font-style: italic; gap: 8px;">
                                            <span style="font-size: 28px;">"📄"</span>
                                            <span>"Select a report from the dropdown above to view its contents."</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <pre style="white-space: pre-wrap; word-break: break-all; margin: 0; font-family: inherit; line-height: 1.6; color: var(--body);">{r}</pre> }.into_any()
                                }
                            }}
                        </div>
                    </Card>
                </div>
            </div>
        </div>
    }
}
