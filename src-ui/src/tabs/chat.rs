//! Chat tab — the reference pattern for the migration.
//!
//! Plain mode streams completion tokens over `chat://event`. Agent mode drives the
//! revamped engine via `agent_start` and renders the live plan, step/tool trace,
//! sub-agent activity, and human approval prompts from `agent://event`. All OS
//! work is in the backend; this component only renders and dispatches IPC.

use std::sync::atomic::{AtomicU64, Ordering};

use gloo_timers::callback::Interval;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use shared::ipc::{
    AGENT_EVENT, AgentEvent, AgentRequest, ApprovalDecision, ApprovalRequest, CHAT_EVENT,
    ChatEvent, ChatMessage, ChatRequest, PlanStatus, PlanStep,
};
use wasm_bindgen_futures::spawn_local;

use crate::components::Spinner;
use crate::state::{AppCtx, Tab};
use crate::{api, ipc};

/// Monotonic id correlating a send with its streamed events.
static STREAM_SEQ: AtomicU64 = AtomicU64::new(0);
fn next_stream_id() -> String {
    format!("s{}", STREAM_SEQ.fetch_add(1, Ordering::Relaxed))
}

#[derive(Clone)]
struct Msg {
    role: String,
    text: String,
}

#[component]
pub fn ChatTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    // ── Reactive state ────────────────────────────────────────────────────
    let messages = RwSignal::new(
        ctx.chat_history.get_untracked().into_iter().map(|m| Msg {
            role: m.role,
            text: m.content,
        }).collect::<Vec<_>>()
    );
    let input = RwSignal::new(String::new());
    let generating = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let models = RwSignal::new(Vec::<String>::new());
    let online = RwSignal::new(false);
    let selected_model = RwSignal::new(String::new());
    let use_agent = RwSignal::new(false);

    // Helper to persist history
    let save_history = move |current_msgs: Vec<Msg>| {
        let chat_msgs: Vec<ChatMessage> = current_msgs.into_iter()
            .map(|m| ChatMessage {
                role: m.role,
                content: m.text,
            })
            .collect();
        ctx.chat_history.set(chat_msgs.clone());
        spawn_local(async move {
            if let Err(e) = api::chat_save_history(chat_msgs).await {
                tracing::error!("Failed to save chat history: {e}");
            }
        });
    };

    // Sync global to local model selection
    Effect::new(move |_| {
        if let Some(m) = ctx.routed_model.get() {
            selected_model.set(m);
        }
    });

    let on_model_change = {
        let ctx = ctx.clone();
        move |val: String| {
            selected_model.set(val.clone());
            ctx.routed_model.set(Some(val.clone()));

            let val_c = val.clone();
            spawn_local(async move {
                if let Ok(scanned) = api::library_get_index().await {
                    if let Some(m) = scanned.into_iter().find(|m| m.filename == val_c) {
                        ctx.config.update(move |c| c.model_path = m.path.clone());
                        if let Ok(_) = ctx.save_async().await {
                            // Restart server if it is running
                            if let Ok(running) = api::server_status().await {
                                if running {
                                    let _ = api::server_stop().await;
                                    let _ = api::server_start().await;
                                }
                            }
                        }
                    }
                }
            });
        }
    };

    // Plain-chat stream correlation + agent run state.
    let current_stream = RwSignal::new(String::new());
    let current_agent = RwSignal::new(None::<String>);
    let plan = RwSignal::new(Vec::<PlanStep>::new());
    let trace = RwSignal::new(Vec::<String>::new());
    let approvals = RwSignal::new(Vec::<(String, ApprovalRequest)>::new());

    // Append a streamed delta to the in-flight assistant bubble.
    let append_assistant = move |delta: &str| {
        messages.update(|m| {
            if let Some(last) = m.last_mut() {
                if last.role == "assistant" {
                    last.text.push_str(delta);
                }
            }
        });
    };

    // Resolve the chat target: a routed instance, else the local server port.
    let target = move || match ctx.routed_instance.get() {
        Some((h, p)) => (h, p),
        None => ("127.0.0.1".to_string(), ctx.config.get().port),
    };

    // ── Backend event subscriptions (registered once) ─────────────────────
    ipc::listen::<ChatEvent, _>(CHAT_EVENT, move |ev| match ev {
        ChatEvent::Token { stream_id, delta } => {
            if stream_id == current_stream.get_untracked() {
                append_assistant(&delta);
            }
        }
        ChatEvent::Done { stream_id } => {
            if stream_id == current_stream.get_untracked() {
                generating.set(false);
                save_history(messages.get_untracked());
            }
        }
        ChatEvent::Error { stream_id, message } => {
            if stream_id == current_stream.get_untracked() {
                error.set(Some(message));
                generating.set(false);
                save_history(messages.get_untracked());
            }
        }
    });

    ipc::listen::<AgentEvent, _>(AGENT_EVENT, move |ev| {
        let is_root = |id: &str| current_agent.get_untracked().as_deref() == Some(id);
        match ev {
            AgentEvent::Started {
                agent_id,
                parent,
                role,
                task,
            } => {
                // The first parentless agent is the root run we stream to the bubble.
                if parent.is_none() && current_agent.get_untracked().is_none() {
                    current_agent.set(Some(agent_id));
                }
                trace.update(|t| t.push(format!("▶ {role} started: {task}")));
            }
            AgentEvent::Plan { steps, .. } => plan.set(steps),
            AgentEvent::PlanUpdate {
                step_id, status, ..
            } => plan.update(|p| {
                for s in p.iter_mut() {
                    if s.id == step_id {
                        s.status = status;
                    }
                }
            }),
            AgentEvent::Step { index, thought, .. } => {
                trace.update(|t| t.push(format!("💭 step {}: {}", index + 1, thought)))
            }
            AgentEvent::Token { agent_id, delta } => {
                if is_root(&agent_id) {
                    append_assistant(&delta);
                } else {
                    trace.update(|t| t.push(format!("  ↳ {delta}")));
                }
            }
            AgentEvent::ToolCall { call, .. } => {
                trace.update(|t| t.push(format!("🔧 {}({})", call.tool, call.args)))
            }
            AgentEvent::ToolResult { result, .. } => {
                let mut o = result.output;
                if o.len() > 200 {
                    o.truncate(200);
                    o.push('…');
                }
                trace.update(|t| t.push(format!("{} {}", if result.ok { "✅" } else { "⚠️" }, o)));
            }
            AgentEvent::ApprovalRequest { agent_id, request } => {
                approvals.update(|a| a.push((agent_id, request)))
            }
            AgentEvent::SubAgentSpawned { role, task, .. } => {
                trace.update(|t| t.push(format!("🌱 spawned {role}: {task}")))
            }
            AgentEvent::Log { message, .. } => trace.update(|t| t.push(message)),
            AgentEvent::Done {
                agent_id,
                final_text,
            } => {
                if is_root(&agent_id) {
                    if !final_text.is_empty() {
                        messages.update(|m| {
                            if let Some(last) = m.last_mut() {
                                if last.role == "assistant" && last.text.is_empty() {
                                    last.text = final_text;
                                }
                            }
                        });
                    }
                    generating.set(false);
                    save_history(messages.get_untracked());
                } else {
                    trace.update(|t| t.push("✔ subagent done".to_string()));
                }
            }
            AgentEvent::Error { agent_id, message } => {
                if is_root(&agent_id) {
                    error.set(Some(message));
                    generating.set(false);
                    save_history(messages.get_untracked());
                } else {
                    trace.update(|t| t.push(format!("subagent error: {message}")));
                }
            }
        }
    });

    // ── Model/online polling (every 2s) ───────────────────────────────────
    let poll_once = move || {
        let (h, p) = target();
        spawn_local(async move {
            match api::chat_list_models(h, p).await {
                Ok(list) => {
                    online.set(list.online);
                    if list.online {
                        let use_library_models = list.models.iter().any(|m| m == "multimodal" || m == "text")
                            || !ctx.config.get_untracked().model_dir.is_empty()
                            || list.models.is_empty();

                        let mut final_models = list.models;
                        if use_library_models {
                            if let Ok(scanned) = api::library_get_index().await {
                                let filenames: Vec<String> = scanned.into_iter()
                                    .map(|m| m.filename)
                                    .collect();
                                if !filenames.is_empty() {
                                    final_models = filenames;
                                }
                            }
                        }

                        // Preserving the selected/routed model so polling doesn't reset it
                        if let Some(ref rm) = ctx.routed_model.get_untracked() {
                            if !final_models.contains(rm) {
                                final_models.push(rm.clone());
                            }
                        }

                        if !final_models
                            .iter()
                            .any(|m| *m == selected_model.get_untracked())
                        {
                            let m = final_models.first().cloned().unwrap_or_default();
                            selected_model.set(m.clone());
                            if ctx.routed_model.get_untracked().is_none() && !m.is_empty() {
                                ctx.routed_model.set(Some(m));
                            }
                        }
                        models.set(final_models);
                    } else {
                        models.set(vec![]);
                        if let Some(rm) = ctx.routed_model.get_untracked() {
                            selected_model.set(rm);
                        } else {
                            selected_model.set(String::new());
                        }
                    }
                }
                Err(_) => online.set(false),
            }
        });
    };
    poll_once();
    Interval::new(2000, poll_once).forget();

    // ── Actions ───────────────────────────────────────────────────────────
    let send = move || {
        let text = input.get_untracked().trim().to_string();
        if text.is_empty() || generating.get_untracked() || !online.get_untracked() {
            return;
        }
        error.set(None);

        // 1. Intercept slash commands
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        let cmd_raw = parts[0];
        let cmd = cmd_raw.to_lowercase();
        let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

        if cmd == "/clear" {
            messages.set(vec![]);
            save_history(vec![]);
            error.set(None);
            plan.set(vec![]);
            trace.set(vec![]);
            approvals.set(vec![]);
            input.set(String::new());
            return;
        }

        if cmd == "/help" {
            messages.update(|m| {
                m.push(Msg {
                    role: "user".into(),
                    text: text.clone(),
                });
                m.push(Msg {
                    role: "assistant".into(),
                    text: "Here are the available slash commands:\n\n\
                           - `/clear` - Clear chat history\n\
                           - `/help` - Show this help message\n\
                           - `/agent <task>` - Start agent mode for the given task\n\
                           - `/research <query>` - Start Deep Research on the given query and switch to the Deep Research tab\n\
                           - `/skills` - List available agent skills/rules\n\
                           - `/skills <name>` - Load a specific skill/agent file into the model's active context\n\
                           - `/mcp` - List connected MCP servers and their available tools\n\
                           - `/todo <task>` - Add a new task to your todo list\n\
                           - `/planner <task>` - Create a new planner/Kanban task and switch to the Task Planner tab".into(),
                });
            });
            save_history(messages.get_untracked());
            input.set(String::new());
            return;
        }

        if cmd == "/research" {
            if arg.is_empty() {
                messages.update(|m| {
                    m.push(Msg { role: "user".into(), text: text.clone() });
                    m.push(Msg { role: "assistant".into(), text: "Usage: `/research <query>`".into() });
                });
                save_history(messages.get_untracked());
                input.set(String::new());
                return;
            }
            messages.update(|m| {
                m.push(Msg {
                    role: "user".into(),
                    text: text.clone(),
                });
                m.push(Msg {
                    role: "assistant".into(),
                    text: format!("Deep research started for: \"{}\". Switching to the Deep Research panel...", arg),
                });
            });
            save_history(messages.get_untracked());
            input.set(String::new());

            let query_str = arg.to_string();
            let (h, p, m) = ctx.resolve_target("research");
            spawn_local(async move {
                let _ = api::deep_research_start(query_str, h, p, m).await;
            });
            ctx.active_tab.set(Tab::DeepResearch);
            return;
        }

        if cmd == "/planner" {
            if arg.is_empty() {
                messages.update(|m| {
                    m.push(Msg { role: "user".into(), text: text.clone() });
                    m.push(Msg { role: "assistant".into(), text: "Usage: `/planner <task>`".into() });
                });
                save_history(messages.get_untracked());
                input.set(String::new());
                return;
            }
            messages.update(|m| {
                m.push(Msg {
                    role: "user".into(),
                    text: text.clone(),
                });
                m.push(Msg {
                    role: "assistant".into(),
                    text: format!("Planner task created: \"{}\". Switching to the Task Planner tab...", arg),
                });
            });
            save_history(messages.get_untracked());
            input.set(String::new());

            let title = arg.to_string();
            let now = js_sys::Date::new_0();
            let created_at = format!("{}-{:02}-{:02}",
                now.get_full_year() as i32,
                (now.get_month() as i32) + 1,
                now.get_date() as i32);
            let id = format!("task-{}", js_sys::Date::now() as u64);

            spawn_local(async move {
                if let Ok(planner_state) = api::planner_load().await {
                    let mut tasks = planner_state.tasks;
                    tasks.push(shared::ipc::KanbanTask {
                        id,
                        title,
                        summary: String::new(),
                        assigned_agent: String::new(),
                        status: shared::ipc::TaskStatus::Todo,
                        priority: "Medium".to_string(),
                        created_at,
                    });
                    let _ = api::planner_save(tasks).await;
                }
            });
            ctx.active_tab.set(Tab::Planner);
            return;
        }

        if cmd == "/todo" {
            if arg.is_empty() {
                messages.update(|m| {
                    m.push(Msg { role: "user".into(), text: text.clone() });
                    m.push(Msg { role: "assistant".into(), text: "Usage: `/todo <task>`".into() });
                });
                save_history(messages.get_untracked());
                input.set(String::new());
                return;
            }
            messages.update(|m| {
                m.push(Msg {
                    role: "user".into(),
                    text: text.clone(),
                });
            });
            input.set(String::new());
            generating.set(true);

            let t = arg.to_string();
            let id = format!("t{}", js_sys::Date::now() as u64);
            spawn_local(async move {
                if let Ok(mut list) = api::todos_get().await {
                    list.push(shared::ipc::TodoItem { id, text: t.clone(), done: false });
                    if let Ok(_) = api::todos_set(list).await {
                        messages.update(|m| {
                            m.push(Msg {
                                role: "assistant".into(),
                                text: format!("Todo added: \"{}\"", t),
                            });
                        });
                        save_history(messages.get_untracked());
                    } else {
                        messages.update(|m| {
                            m.push(Msg {
                                role: "assistant".into(),
                                text: "Failed to save todo item.".into(),
                            });
                        });
                    }
                } else {
                    messages.update(|m| {
                        m.push(Msg {
                            role: "assistant".into(),
                            text: "Failed to retrieve todo list.".into(),
                        });
                    });
                }
                generating.set(false);
            });
            return;
        }

        if cmd == "/mcp" {
            messages.update(|m| {
                m.push(Msg {
                    role: "user".into(),
                    text: text.clone(),
                });
            });
            input.set(String::new());
            generating.set(true);

            spawn_local(async move {
                match api::mcp_get_registry().await {
                    Ok(val) => {
                        let mut output = String::from("### Connected MCP Servers\n\n");
                        let mut found = false;
                        if let Some(mcp_servers) = val.get("mcpServers").and_then(|v| v.as_object()) {
                            for (name, server_val) in mcp_servers {
                                found = true;
                                let cmd = server_val.get("command").and_then(|v| v.as_str()).unwrap_or("");
                                let tools = server_val.get("tools")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                                    .unwrap_or_default();

                                output.push_str(&format!("* **{}**\n  - Command: `{}`\n", name, cmd));
                                if !tools.is_empty() {
                                    output.push_str("  - Tools: ");
                                    let tools_formatted = tools.iter().map(|t| format!("`{}`", t)).collect::<Vec<_>>().join(", ");
                                    output.push_str(&tools_formatted);
                                    output.push_str("\n");
                                } else {
                                    output.push_str("  - Tools: None configured\n");
                                }
                            }
                        }
                        if !found {
                            output.push_str("No MCP servers configured.");
                        }
                        messages.update(|m| {
                            m.push(Msg {
                                role: "assistant".into(),
                                text: output,
                            });
                        });
                    }
                    Err(e) => {
                        messages.update(|m| {
                            m.push(Msg {
                                role: "assistant".into(),
                                text: format!("Failed to retrieve MCP registry: {}", e),
                            });
                        });
                    }
                }
                save_history(messages.get_untracked());
                generating.set(false);
            });
            return;
        }

        if cmd == "/skills" {
            messages.update(|m| {
                m.push(Msg {
                    role: "user".into(),
                    text: text.clone(),
                });
            });
            input.set(String::new());
            generating.set(true);

            let name_filter = arg.to_string();
            spawn_local(async move {
                match api::list_skills_and_agents().await {
                    Ok(list) => {
                        if name_filter.is_empty() {
                            let mut output = String::from("### Loaded Skills & Agents\n\n");
                            if list.is_empty() {
                                output.push_str("No loaded skills or agent files found.");
                            } else {
                                for f in &list {
                                    let type_label = if f.is_skill { "Skill" } else { "Agent" };
                                    output.push_str(&format!("* **{}** ({}) - `{}`\n  - Source: {}\n", f.name, type_label, f.path, f.source));
                                }
                                output.push_str("\nTo load a specific skill into the active chat context, use: `/skills <name>`");
                            }
                            messages.update(|m| {
                                m.push(Msg {
                                    role: "assistant".into(),
                                    text: output,
                                });
                            });
                        } else {
                            let match_lower = name_filter.to_lowercase();
                            let matching = list.into_iter().find(|f| f.name.to_lowercase().contains(&match_lower) || f.path.to_lowercase().contains(&match_lower));

                            if let Some(f) = matching {
                                messages.update(|m| {
                                    m.push(Msg {
                                        role: "system".into(),
                                        text: format!("Injected Context from Skill '{}' ({}):\n\n{}", f.name, f.path, f.content),
                                    });
                                    m.push(Msg {
                                        role: "assistant".into(),
                                        text: format!("Loaded skill/agent file **{}** (`{}`) into active context. The model can now use its instructions.", f.name, f.path),
                                    });
                                });
                            } else {
                                messages.update(|m| {
                                    m.push(Msg {
                                        role: "assistant".into(),
                                        text: format!("Could not find any skill or agent file matching \"{}\". Use `/skills` to list all loaded files.", name_filter),
                                    });
                                });
                            }
                        }
                    }
                    Err(e) => {
                        messages.update(|m| {
                            m.push(Msg {
                                role: "assistant".into(),
                                text: format!("Failed to retrieve skills and agent files: {}", e),
                            });
                        });
                    }
                }
                save_history(messages.get_untracked());
                generating.set(false);
            });
            return;
        }

        let mut is_agent = use_agent.get_untracked();
        let mut task_text = text.clone();

        if cmd == "/agent" {
            if arg.is_empty() {
                messages.update(|m| {
                    m.push(Msg { role: "user".into(), text: text.clone() });
                    m.push(Msg { role: "assistant".into(), text: "Usage: `/agent <task>`".into() });
                });
                save_history(messages.get_untracked());
                input.set(String::new());
                return;
            }
            use_agent.set(true);
            is_agent = true;
            task_text = arg.to_string();
        }

        messages.update(|m| {
            m.push(Msg {
                role: "user".into(),
                text: text.clone(),
            });
            // Placeholder bubble we stream the response into.
            m.push(Msg {
                role: "assistant".into(),
                text: String::new(),
            });
        });
        save_history(messages.get_untracked());
        input.set(String::new());
        generating.set(true);

        let (host, port) = target();
        let model = selected_model.get_untracked();

        if is_agent {
            plan.set(vec![]);
            trace.set(vec![]);
            approvals.set(vec![]);
            current_agent.set(None);
            let req = AgentRequest {
                host,
                port,
                model,
                task: task_text,
            };
            spawn_local(async move {
                match api::agent_start(req).await {
                    Ok(id) => {
                        if current_agent.get_untracked().is_none() {
                            current_agent.set(Some(id));
                        }
                    }
                    Err(e) => {
                        error.set(Some(e));
                        generating.set(false);
                    }
                }
            });
        } else {
            let sid = next_stream_id();
            current_stream.set(sid.clone());
            // History minus the trailing empty assistant placeholder.
            let history: Vec<ChatMessage> = messages
                .get_untracked()
                .into_iter()
                .filter(|m| !(m.role == "assistant" && m.text.is_empty()))
                .map(|m| ChatMessage {
                    role: m.role,
                    content: m.text,
                })
                .collect();
            let req = ChatRequest {
                stream_id: sid,
                host,
                port,
                model,
                messages: history,
                temperature: 0.7,
            };
            spawn_local(async move {
                if let Err(e) = api::chat_send(req).await {
                    error.set(Some(e));
                    generating.set(false);
                }
            });
        }
    };

    let run_suggestion = move |prompt_text: String| {
        input.set(prompt_text);
        send();
    };

    let clear = move |_| {
        messages.set(vec![]);
        save_history(vec![]);
        error.set(None);
        plan.set(vec![]);
        trace.set(vec![]);
        approvals.set(vec![]);
    };

    let cancel = move |_| {
        if let Some(id) = current_agent.get_untracked() {
            spawn_local(async move {
                let _ = api::agent_cancel(id).await;
            });
            generating.set(false);
            save_history(messages.get_untracked());
        }
    };

    let respond = move |agent_id: String, call_id: String, approved: bool| {
        approvals.update(|a| a.retain(|(_, r)| r.call_id != call_id));
        spawn_local(async move {
            let _ = api::agent_approve(ApprovalDecision {
                agent_id,
                call_id,
                approved,
            })
            .await;
        });
    };

    // ── View ──────────────────────────────────────────────────────────────
    view! {
        <div class="chat">
            <div class="chat-header glass">
                <div style="display: flex; align-items: center; gap: var(--s-sm);">
                    <span class="chat-title" style="font-size: 16px; display: flex; align-items: center; gap: 6px;">
                        "💬"
                        {move || match ctx.routed_instance.get() {
                            Some((h, p)) => format!("Connected: http://{}:{}", h, p),
                            None => "Local Server".to_string()
                        }}
                    </span>
                </div>
                <div class="chat-controls">
                    <select
                        class="model-select"
                        prop:value=move || {
                            let _ = models.get();
                            selected_model.get()
                        }
                        on:change=move |e| on_model_change(event_target_value(&e))
                    >
                        {move || {
                            models
                                .get()
                                .into_iter()
                                .map(|m| {
                                    let label = m.clone();
                                    view! { <option value=m>{label}</option> }
                                })
                                .collect_view()
                        }}
                    </select>
                    <label class="agent-toggle">
                        <input
                            type="checkbox"
                            prop:checked=move || use_agent.get()
                            on:change=move |e| use_agent.set(event_target_checked(&e))
                        />
                        "🤖 Agent Mode"
                    </label>
                    <button
                        class="btn ghost"
                        on:click=move |_| ctx.immersive.set(!ctx.immersive.get_untracked())
                    >
                        {move || if ctx.immersive.get() { "✨ Exit" } else { "✨ Immersive" }}
                    </button>
                    {move || {
                        generating
                            .get()
                            .then(|| view! { <button class="btn ghost" on:click=cancel>"⛔ Cancel"</button> })
                    }}
                    <button class="btn ghost" on:click=clear>"Clear"</button>
                </div>
            </div>

            {move || {
                (!online.get())
                    .then(|| {
                        if ctx.server_running.get() {
                            view! {
                                <div class="banner info">
                                    "⏳ Model server is initializing or loading the model. Please wait..."
                                </div>
                            }
                        } else {
                            view! {
                                <div class="banner warn">
                                    "⚠️ Model server offline. Start a server to chat."
                                </div>
                            }
                        }
                    })
            }}
            {move || {
                error.get().map(|e| view! { <div class="toast error">"❌ "{e}</div> })
            }}

            <div class="messages" style="flex: 1; min-height: 0; overflow-y: auto;">
                {move || {
                    let msgs = messages.get();
                    if msgs.is_empty() {
                        view! {
                            <div class="welcome-container">
                                <h1 class="welcome-title">"Where should we start?"</h1>
                                <div class="suggestions-grid">
                                    <div class="suggestion-card" on:click=move |_| run_suggestion("Summarize my quick notes and list key topics.".to_string())>
                                        <span class="suggestion-icon">"📝"</span>
                                        <div class="suggestion-text">
                                            <div class="suggestion-header">"Summarize Notes"</div>
                                            <div class="suggestion-desc">"Summarize quick notes and list key topics."</div>
                                        </div>
                                    </div>
                                    <div class="suggestion-card" on:click=move |_| run_suggestion("Look at my task planner backlog and suggest priority tasks.".to_string())>
                                        <span class="suggestion-icon">"📋"</span>
                                        <div class="suggestion-text">
                                            <div class="suggestion-header">"Review Planner"</div>
                                            <div class="suggestion-desc">"Review backlog and suggest priority tasks."</div>
                                        </div>
                                    </div>
                                    <div class="suggestion-card" on:click=move |_| run_suggestion("Check my calendar events and list them.".to_string())>
                                        <span class="suggestion-icon">"📅"</span>
                                        <div class="suggestion-text">
                                            <div class="suggestion-header">"Check Calendar"</div>
                                            <div class="suggestion-desc">"Review calendar and list upcoming events."</div>
                                        </div>
                                    </div>
                                    <div class="suggestion-card" on:click=move |_| run_suggestion("Check the active model server and report its status and parameters.".to_string())>
                                        <span class="suggestion-icon">"🔍"</span>
                                        <div class="suggestion-text">
                                            <div class="suggestion-header">"Model Status"</div>
                                            <div class="suggestion-desc">"Check active model server status."</div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        msgs.into_iter()
                            .map(|m| {
                                let cls = if m.role == "user" {
                                    "msg user"
                                } else if m.role == "system" {
                                    "msg system"
                                } else {
                                    "msg assistant"
                                };
                                view! {
                                    <div class=cls>
                                        <div class="bubble">{m.text}</div>
                                    </div>
                                }
                            })
                            .collect_view().into_any()
                    }
                }}
                {move || {
                    generating
                        .get()
                        .then(|| {
                            view! {
                                <div class="thinking">
                                    <Spinner/>
                                    "Working…"
                                </div>
                            }
                        })
                }}
            </div>

            // Agent plan
            {move || {
                (!plan.get().is_empty())
                    .then(|| {
                        view! {
                            <div class="panel glass" style="margin-bottom: var(--s-sm);">
                                <div class="panel-title">"📋 Plan"</div>
                                <ul class="plan">
                                    {move || {
                                        plan.get()
                                            .into_iter()
                                            .map(|s| {
                                                let icon = match s.status {
                                                    PlanStatus::Done => "✅",
                                                    PlanStatus::Running => "⏳",
                                                    PlanStatus::Failed => "❌",
                                                    PlanStatus::Pending => "•",
                                                };
                                                view! {
                                                    <li>
                                                        <span class="plan-icon">{icon}</span>
                                                        {s.description}
                                                    </li>
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </ul>
                            </div>
                        }
                    })
            }}

            // Approval prompts (L4)
            {move || {
                (!approvals.get().is_empty())
                    .then(|| {
                        view! {
                            <div class="panel approval glass" style="margin-bottom: var(--s-sm);">
                                <div class="panel-title">"⚠️ Approval required"</div>
                                {move || {
                                    approvals
                                        .get()
                                        .into_iter()
                                        .map(|(agent_id, req)| {
                                            let (a1, a2) = (agent_id.clone(), agent_id);
                                            let (c1, c2) = (req.call_id.clone(), req.call_id.clone());
                                            view! {
                                                <div class="approval-item">
                                                    <div class="approval-tool">
                                                        <strong>{req.tool.clone()}</strong>
                                                        <code>{req.args.to_string()}</code>
                                                    </div>
                                                    <div class="approval-actions">
                                                        <button
                                                            class="btn allow"
                                                            on:click=move |_| respond(a1.clone(), c1.clone(), true)
                                                        >
                                                            "Allow"
                                                        </button>
                                                        <button
                                                            class="btn deny"
                                                            on:click=move |_| respond(a2.clone(), c2.clone(), false)
                                                        >
                                                            "Deny"
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>
                        }
                    })
            }}

            // Agent trace
            {move || {
                (!trace.get().is_empty())
                    .then(|| {
                        view! {
                            <details class="panel trace glass" open=true style="margin-bottom: var(--s-sm);">
                                <summary>
                                    {move || format!("🧭 Agent trace ({} events)", trace.get().len())}
                                </summary>
                                <div class="trace-lines">
                                    {move || {
                                        trace
                                            .get()
                                            .into_iter()
                                            .map(|l| view! { <div class="trace-line">{l}</div> })
                                            .collect_view()
                                    }}
                                </div>
                            </details>
                        }
                    })
            }}

            <div class="modern-composer glass">
                <input
                    class="composer-input"
                    type="text"
                    placeholder=move || {
                        if online.get() {
                            "Type a message, reference notes/planner/calendar or click suggestion..."
                        } else {
                            "Start the server first…"
                        }
                    }
                    prop:value=move || input.get()
                    prop:disabled=move || !online.get() || generating.get()
                    on:input=move |e| input.set(event_target_value(&e))
                    on:keydown=move |e: KeyboardEvent| {
                        if e.key() == "Enter" {
                            send();
                        }
                    }
                />
                <button
                    class="composer-send-btn"
                    prop:disabled=move || !online.get() || generating.get()
                    on:click=move |_| send()
                >
                    {move || if generating.get() {
                        view! { <span style="font-size: 12px;">"⏳"</span> }.into_any()
                    } else {
                        view! { <span style="font-size: 16px; font-weight: bold;">"↑"</span> }.into_any()
                    }}
                </button>
            </div>
        </div>
    }
}
