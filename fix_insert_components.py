path = 'src-ui/src/tabs/remaining.rs'
with open(path, 'r') as f:
    content = f.read()
    lines = content.split('\n')

print(f"Total lines before: {len(lines)}")

# Delete orphaned MonitorTab fragment (lines 1251-1329, 1-indexed = indices 1250-1328 0-indexed)
# Keep lines[:1250] (includes closing brace of AgentsTab at line 1249) 
# and lines[1329:] (starts at "// ── 7. PlannerTab")
before = lines[:1250]
after = lines[1329:]

# The McpTab and MonitorTab components to insert
mcp_and_monitor = '''
// ── 5. McpTab / MCP Tools Registry ───────────────────────────────────────────
#[component]
pub fn McpTab() -> impl IntoView {
    let raw_config = RwSignal::new(String::new());
    let status_text = RwSignal::new(String::new());
    let show_raw_editor = RwSignal::new(false);

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
                                            <div class="mcp-card-cmd" title={format!("{cmd} {args}")}>{cmd} " "{args}</div>
                                            {if !env_keys.is_empty() {
                                                view! {
                                                    <div style="font-size:11px;color:var(--muted);">
                                                        "Env: "
                                                        {env_keys.iter().map(|k| view! { <span class="mcp-tool-chip">{k}</span> }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            } else { view! {}.into_any() }}
                                            {if !tools.is_empty() {
                                                view! {
                                                    <div>
                                                        <div style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;color:var(--muted);margin-bottom:4px;">"Tools"</div>
                                                        <div style="display:flex;flex-wrap:wrap;">
                                                            {tools.iter().map(|t| view! { <span class="mcp-tool-chip">"🔧 "{t}</span> }).collect_view()}
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

    let dispatch_task = move |_| {
        let task = new_task_input.get_untracked().trim().to_string();
        if task.is_empty() { return; }
        let cfg = ctx.config.get_untracked();
        let host  = cfg.host.clone();
        let port  = cfg.port;
        let model = if cfg.model_alias.is_empty() { cfg.model_path.clone() } else { cfg.model_alias.clone() };
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
                        on:keydown=move |e: KeyboardEvent| { if e.key() == "Enter" { dispatch_task(()); } }
                    />
                    <button class="btn primary" prop:disabled=move || dispatch_running.get() on:click=dispatch_task>
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
                                                <span style=format!("font-size:10px;font-weight:700;padding:1px 6px;border-radius:3px;color:{sc};background:{sb};border:1px solid {sc};">
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
'''.split('\n')

new_lines = before + mcp_and_monitor + after

print(f"Total lines after: {len(new_lines)}")

with open(path, 'w') as f:
    f.write('\n'.join(new_lines))

print("Done!")

# Verify structure
import subprocess
result = subprocess.run(['grep', '-n', '^pub fn \\|// ──', path], capture_output=True, text=True)
print(result.stdout[:2000])
