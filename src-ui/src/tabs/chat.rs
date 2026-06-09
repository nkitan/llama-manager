//! Chat tab — streams, agent mode, context overview.
//!
//! All streaming state (messages, generating, plan, trace, approvals) lives in
//! AppCtx so it survives tab switches. The ipc::listen handlers are registered
//! once in App; this component only dispatches and renders.

use std::sync::atomic::{AtomicU64, Ordering};

use gloo_timers::callback::Interval;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use shared::ipc::{
    AgentRequest, ApprovalDecision, ChatMessage, ChatRequest,
    PlanStatus,
};
use wasm_bindgen_futures::spawn_local;

use crate::components::Spinner;
use crate::state::{AppCtx, Tab};
use crate::api;

/// Monotonic id correlating a send with its streamed events.
static STREAM_SEQ: AtomicU64 = AtomicU64::new(0);
fn next_stream_id() -> String {
    format!("s{}", STREAM_SEQ.fetch_add(1, Ordering::Relaxed))
}

#[component]
pub fn ChatTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    // ── Persistent state from AppCtx (survives tab switches) ─────────────────
    let messages       = ctx.chat_messages;
    let generating     = ctx.chat_generating;
    let error          = ctx.chat_error;
    let current_stream = ctx.chat_current_stream;
    let current_agent  = ctx.chat_current_agent;
    let plan           = ctx.chat_plan;
    let trace          = ctx.chat_trace;
    let approvals      = ctx.chat_approvals;

    // ── Local tab-only state ─────────────────────────────────────────────────
    let input          = ctx.chat_draft;
    let models         = RwSignal::new(Vec::<String>::new());
    let online         = RwSignal::new(false);
    let selected_model = RwSignal::new(String::new());
    let use_agent      = RwSignal::new(false);

    // Autocomplete popup state
    let popup_open     = RwSignal::new(false);
    let active_index   = RwSignal::new(0usize);
    let skills_list    = RwSignal::new(Vec::<shared::ipc::SkillOrAgentFile>::new());

    spawn_local(async move {
        if let Ok(list) = api::list_skills_and_agents().await {
            skills_list.set(list);
        }
    });

    #[derive(Clone, Debug, PartialEq)]
    struct CmdOption {
        cmd: String,
        desc: String,
    }

    let built_ins = vec![
        CmdOption { cmd: "/clear".into(),    desc: "Clear chat history".into() },
        CmdOption { cmd: "/help".into(),     desc: "Show this help message".into() },
        CmdOption { cmd: "/agent".into(),    desc: "Start agent mode for the given task".into() },
        CmdOption { cmd: "/research".into(), desc: "Start Deep Research and switch to the Deep Research tab".into() },
        CmdOption { cmd: "/skills".into(),   desc: "List all loaded skills & agents".into() },
        CmdOption { cmd: "/mcp".into(),      desc: "List connected MCP servers and tools".into() },
        CmdOption { cmd: "/todo".into(),     desc: "Add a new task to your todo list".into() },
        CmdOption { cmd: "/planner".into(),  desc: "Create a new planner task and switch to the Task Planner tab".into() },
        CmdOption { cmd: "/agents".into(),   desc: "List available custom agents".into() },
    ];

    let skill_slug = move |file: &shared::ipc::SkillOrAgentFile| -> String {
        let path = std::path::Path::new(&file.path);
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        if filename == "skill.md" || filename == "agents.md" {
            if let Some(parent) = path.parent() {
                if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
                    if parent_name.to_lowercase() != "skills" && parent_name.to_lowercase() != "plugins" {
                        return parent_name.to_lowercase();
                    }
                }
            }
        }
        let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or(&file.name);
        let stem_clean = if stem.starts_with('.') { stem.trim_start_matches('.') } else { stem };
        stem_clean.to_lowercase().replace(" ", "-")
    };

    let get_skill_description = move |content: &str| -> String {
        let mut desc = String::new();
        let mut in_frontmatter = false;
        let mut frontmatter_lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "---" {
                if in_frontmatter { break; } else { in_frontmatter = true; continue; }
            }
            if in_frontmatter { frontmatter_lines.push(line); }
            else if !trimmed.is_empty() && desc.is_empty() { desc = trimmed.to_string(); }
        }
        for line in frontmatter_lines {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 && parts[0].trim().to_lowercase() == "description" {
                return parts[1].trim().trim_matches('"').trim_matches('\'').to_string();
            }
        }
        if desc.starts_with("# ") || desc.starts_with("## ") {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed != "---" {
                    desc = trimmed.to_string(); break;
                }
            }
        }
        if desc.len() > 150 { desc.truncate(147); desc.push_str("..."); }
        desc
    };

    let all_options = Memo::new(move |_| {
        let mut opts = built_ins.clone();
        for f in skills_list.get() {
            let slug = skill_slug(&f);
            let desc = get_skill_description(&f.content);
            opts.push(CmdOption { cmd: format!("/agent-skills:{}", slug), desc });
        }
        opts
    });

    let filtered_options = Memo::new(move |_| {
        let val = input.get().to_lowercase();
        if !val.starts_with('/') { return vec![]; }
        all_options.get().into_iter()
            .filter(|opt| opt.cmd.to_lowercase().starts_with(&val))
            .collect::<Vec<_>>()
    });

    Effect::new(move |_| {
        let len = filtered_options.get().len();
        let idx = active_index.get_untracked();
        if len == 0 { active_index.set(0); }
        else if idx >= len { active_index.set(len - 1); }
    });

    // Sync global to local model selection
    Effect::new(move |_| {
        if let Some(m) = ctx.routed_model.get() { selected_model.set(m); }
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

    // Resolve the chat target: a routed instance, else the local server port.
    let target = move || match ctx.routed_instance.get() {
        Some((h, p)) => (h, p),
        None => ("127.0.0.1".to_string(), ctx.config.get().port),
    };

    // ── Model/online polling (every 2s) ───────────────────────────────────────
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
                                let filenames: Vec<String> = scanned.into_iter().map(|m| m.filename).collect();
                                if !filenames.is_empty() { final_models = filenames; }
                            }
                        }
                        if let Some(ref rm) = ctx.routed_model.get_untracked() {
                            if !final_models.contains(rm) { final_models.push(rm.clone()); }
                        }
                        if !final_models.iter().any(|m| *m == selected_model.get_untracked()) {
                            let m = final_models.first().cloned().unwrap_or_default();
                            selected_model.set(m.clone());
                            if ctx.routed_model.get_untracked().is_none() && !m.is_empty() {
                                ctx.routed_model.set(Some(m));
                            }
                        }
                        models.set(final_models);
                    } else {
                        models.set(vec![]);
                        if let Some(rm) = ctx.routed_model.get_untracked() { selected_model.set(rm); }
                        else { selected_model.set(String::new()); }
                    }
                }
                Err(_) => online.set(false),
            }
        });
    };
    poll_once();
    Interval::new(2000, poll_once).forget();

    // ── Helpers ───────────────────────────────────────────────────────────────
    let save_history = move || {
        let msgs = messages.get_untracked();
        ctx.chat_history.set(msgs.clone());
        spawn_local(async move {
            if let Err(e) = api::chat_save_history(msgs).await {
                tracing::error!("chat_save_history: {e}");
            }
        });
    };

    // ── Actions ───────────────────────────────────────────────────────────────
    let send = move || {
        let text = input.get_untracked().trim().to_string();
        if text.is_empty() || generating.get_untracked() || !online.get_untracked() { return; }
        error.set(None);

        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        let cmd_raw = parts[0];
        let cmd = cmd_raw.to_lowercase();
        let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

        if cmd == "/clear" {
            messages.set(vec![]);
            ctx.chat_history.set(vec![]);
            spawn_local(async move { let _ = api::chat_save_history(vec![]).await; });
            error.set(None);
            plan.set(vec![]);
            trace.set(vec![]);
            approvals.set(vec![]);
            input.set(String::new());
            return;
        }

        if cmd == "/help" {
            messages.update(|m| {
                m.push(ChatMessage { role: "user".into(), content: text.clone() });
                m.push(ChatMessage { role: "assistant".into(), content: "\
                    Here are the available slash commands:\n\n\
                    - `/clear` — Clear chat history\n\
                    - `/help` — Show this help message\n\
                    - `/agent <task>` — Start agent mode for the given task\n\
                    - `/research <query>` — Start Deep Research on the given query\n\
                    - `/skills` — List available agent skills/rules\n\
                    - `/skills <name>` — Load a specific skill into active context\n\
                    - `/mcp` — List connected MCP servers and their tools\n\
                    - `/todo <task>` — Add a new task to your todo list\n\
                    - `/planner <task>` — Create a new planner/Kanban task".into() });
            });
            save_history();
            input.set(String::new());
            return;
        }

        if cmd == "/research" {
            if arg.is_empty() {
                messages.update(|m| {
                    m.push(ChatMessage { role: "user".into(), content: text.clone() });
                    m.push(ChatMessage { role: "assistant".into(), content: "Usage: `/research <query>`".into() });
                });
                save_history();
                input.set(String::new());
                return;
            }
            messages.update(|m| {
                m.push(ChatMessage { role: "user".into(), content: text.clone() });
                m.push(ChatMessage { role: "assistant".into(), content: format!("Deep research started for: \"{}\". Switching to the Deep Research panel…", arg) });
            });
            save_history();
            input.set(String::new());
            let query_str = arg.to_string();
            let (h, p, m) = ctx.resolve_target("research");
            spawn_local(async move { let _ = api::deep_research_start(query_str, h, p, m).await; });
            ctx.active_tab.set(Tab::DeepResearch);
            return;
        }

        if cmd == "/planner" {
            if arg.is_empty() {
                messages.update(|m| {
                    m.push(ChatMessage { role: "user".into(), content: text.clone() });
                    m.push(ChatMessage { role: "assistant".into(), content: "Usage: `/planner <task>`".into() });
                });
                save_history();
                input.set(String::new());
                return;
            }
            messages.update(|m| {
                m.push(ChatMessage { role: "user".into(), content: text.clone() });
                m.push(ChatMessage { role: "assistant".into(), content: format!("Planner task created: \"{}\". Switching to the Task Planner tab…", arg) });
            });
            save_history();
            input.set(String::new());
            let title = arg.to_string();
            let now = js_sys::Date::new_0();
            let created_at = format!("{}-{:02}-{:02}", now.get_full_year() as i32, (now.get_month() as i32) + 1, now.get_date() as i32);
            let id = format!("task-{}", js_sys::Date::now() as u64);
            spawn_local(async move {
                if let Ok(planner_state) = api::planner_load().await {
                    let mut tasks = planner_state.tasks;
                    tasks.push(shared::ipc::KanbanTask {
                        id, title, summary: String::new(), assigned_agent: String::new(),
                        status: shared::ipc::TaskStatus::Todo, priority: "Medium".to_string(), created_at,
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
                    m.push(ChatMessage { role: "user".into(), content: text.clone() });
                    m.push(ChatMessage { role: "assistant".into(), content: "Usage: `/todo <task>`".into() });
                });
                save_history();
                input.set(String::new());
                return;
            }
            messages.update(|m| m.push(ChatMessage { role: "user".into(), content: text.clone() }));
            input.set(String::new());
            generating.set(true);
            let t = arg.to_string();
            let id = format!("t{}", js_sys::Date::now() as u64);
            spawn_local(async move {
                if let Ok(mut list) = api::todos_get().await {
                    list.push(shared::ipc::TodoItem { id, text: t.clone(), done: false, priority: String::new(), due_date: None, tags: vec![] });
                    if let Ok(_) = api::todos_set(list).await {
                        messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: format!("Todo added: \"{}\"", t) }));
                        save_history();
                    } else {
                        messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: "Failed to save todo item.".into() }));
                    }
                } else {
                    messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: "Failed to retrieve todo list.".into() }));
                }
                generating.set(false);
            });
            return;
        }

        if cmd == "/mcp" {
            messages.update(|m| m.push(ChatMessage { role: "user".into(), content: text.clone() }));
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
                                let cmd_str = server_val.get("command").and_then(|v| v.as_str()).unwrap_or("");
                                let tools = server_val.get("tools").and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                                    .unwrap_or_default();
                                output.push_str(&format!("* **{}**\n  - Command: `{}`\n", name, cmd_str));
                                if !tools.is_empty() {
                                    output.push_str("  - Tools: ");
                                    output.push_str(&tools.iter().map(|t| format!("`{}`", t)).collect::<Vec<_>>().join(", "));
                                    output.push('\n');
                                } else {
                                    output.push_str("  - Tools: None configured\n");
                                }
                            }
                        }
                        if !found { output.push_str("No MCP servers configured."); }
                        messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: output }));
                    }
                    Err(e) => messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: format!("Failed to retrieve MCP registry: {}", e) })),
                }
                save_history();
                generating.set(false);
            });
            return;
        }

        if cmd == "/skills" {
            messages.update(|m| m.push(ChatMessage { role: "user".into(), content: text.clone() }));
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
                            messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: output }));
                        } else {
                            let match_lower = name_filter.to_lowercase();
                            let matching = list.into_iter().find(|f| f.name.to_lowercase().contains(&match_lower) || f.path.to_lowercase().contains(&match_lower));
                            if let Some(f) = matching {
                                messages.update(|m| {
                                    m.push(ChatMessage { role: "system".into(), content: format!("Injected Context from Skill '{}' ({}):\n\n{}", f.name, f.path, f.content) });
                                    m.push(ChatMessage { role: "assistant".into(), content: format!("Loaded skill/agent file **{}** (`{}`) into active context.", f.name, f.path) });
                                });
                            } else {
                                messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: format!("Could not find any skill or agent file matching \"{}\". Use `/skills` to list all.", name_filter) }));
                            }
                        }
                    }
                    Err(e) => messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: format!("Failed to retrieve skills and agent files: {}", e) })),
                }
                save_history();
                generating.set(false);
            });
            return;
        }

        if cmd == "/agents" {
            messages.update(|m| m.push(ChatMessage { role: "user".into(), content: text.clone() }));
            input.set(String::new());
            generating.set(true);
            spawn_local(async move {
                match api::list_skills_and_agents().await {
                    Ok(list) => {
                        let mut output = String::from("### Available Custom Agents & Instructions\n\n");
                        let mut found = false;
                        for f in &list {
                            if !f.is_skill {
                                found = true;
                                output.push_str(&format!("* **{}** - `{}`\n  - Source: {}\n", f.name, f.path, f.source));
                            }
                        }
                        if !found { output.push_str("No custom agent files loaded."); }
                        messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: output }));
                    }
                    Err(e) => messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: format!("Failed to retrieve agents: {}", e) })),
                }
                save_history();
                generating.set(false);
            });
            return;
        }

        if cmd.starts_with("/agent-skills:") {
            let target_slug = cmd.strip_prefix("/agent-skills:").unwrap_or("").to_lowercase();
            messages.update(|m| m.push(ChatMessage { role: "user".into(), content: text.clone() }));
            input.set(String::new());
            generating.set(true);
            spawn_local(async move {
                match api::list_skills_and_agents().await {
                    Ok(list) => {
                        let matching = list.into_iter().find(|f| {
                            let slug = f.name.to_lowercase().replace(" ", "-");
                            let path = std::path::Path::new(&f.path);
                            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
                            let derived_slug = if filename == "skill.md" || filename == "agents.md" {
                                if let Some(parent) = path.parent() {
                                    if let Some(pn) = parent.file_name().and_then(|n| n.to_str()) {
                                        if pn.to_lowercase() != "skills" && pn.to_lowercase() != "plugins" {
                                            pn.to_lowercase()
                                        } else { slug }
                                    } else { slug }
                                } else { slug }
                            } else {
                                let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or(&f.name);
                                let stem_clean = if stem.starts_with('.') { stem.trim_start_matches('.') } else { stem };
                                stem_clean.to_lowercase().replace(" ", "-")
                            };
                            derived_slug == target_slug
                        });
                        if let Some(f) = matching {
                            messages.update(|m| {
                                m.push(ChatMessage { role: "system".into(), content: format!("Injected Context from Skill '{}' ({}):\n\n{}", f.name, f.path, f.content) });
                                m.push(ChatMessage { role: "assistant".into(), content: format!("Loaded skill/agent file **{}** (`{}`) into active context.", f.name, f.path) });
                            });
                        } else {
                            messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: format!("Could not find any skill matching \"{}\".", target_slug) }));
                        }
                    }
                    Err(e) => messages.update(|m| m.push(ChatMessage { role: "assistant".into(), content: format!("Failed to retrieve skills: {}", e) })),
                }
                save_history();
                generating.set(false);
            });
            return;
        }

        let mut is_agent = use_agent.get_untracked();
        let mut task_text = text.clone();

        if cmd == "/agent" {
            if arg.is_empty() {
                messages.update(|m| {
                    m.push(ChatMessage { role: "user".into(), content: text.clone() });
                    m.push(ChatMessage { role: "assistant".into(), content: "Usage: `/agent <task>`".into() });
                });
                save_history();
                input.set(String::new());
                return;
            }
            use_agent.set(true);
            is_agent = true;
            task_text = arg.to_string();
        }

        messages.update(|m| {
            m.push(ChatMessage { role: "user".into(), content: text.clone() });
            m.push(ChatMessage { role: "assistant".into(), content: String::new() });
        });
        save_history();
        input.set(String::new());
        generating.set(true);

        let (host, port) = target();
        let model = selected_model.get_untracked();

        if is_agent {
            plan.set(vec![]);
            trace.set(vec![]);
            approvals.set(vec![]);
            current_agent.set(None);
            let req = AgentRequest { host, port, model, task: task_text };
            spawn_local(async move {
                match api::agent_start(req).await {
                    Ok(id) => {
                        if current_agent.get_untracked().is_none() { current_agent.set(Some(id)); }
                    }
                    Err(e) => { error.set(Some(e)); generating.set(false); }
                }
            });
        } else {
            let sid = next_stream_id();
            current_stream.set(sid.clone());
            let history: Vec<ChatMessage> = messages.get_untracked().into_iter()
                .filter(|m| !(m.role == "assistant" && m.content.is_empty()))
                .collect();
            let req = ChatRequest { stream_id: sid, host, port, model, messages: history, temperature: 0.7 };
            spawn_local(async move {
                if let Err(e) = api::chat_send(req).await { error.set(Some(e)); generating.set(false); }
            });
        }
    };

    let run_suggestion = move |prompt_text: String| {
        input.set(prompt_text);
        send();
    };

    let clear = move |_| {
        messages.set(vec![]);
        ctx.chat_history.set(vec![]);
        spawn_local(async move { let _ = api::chat_save_history(vec![]).await; });
        error.set(None);
        plan.set(vec![]);
        trace.set(vec![]);
        approvals.set(vec![]);
    };

    let cancel = move || {
        generating.set(false);
        if let Some(id) = current_agent.get_untracked() {
            let id_c = id.clone();
            spawn_local(async move { let _ = api::agent_cancel(id_c).await; });
        }
        let msgs = messages.get_untracked();
        ctx.chat_history.set(msgs.clone());
        spawn_local(async move { let _ = api::chat_save_history(msgs).await; });
    };

    let respond = move |agent_id: String, call_id: String, approved: bool| {
        approvals.update(|a| a.retain(|(_, r)| r.call_id != call_id));
        spawn_local(async move {
            let _ = api::agent_approve(ApprovalDecision { agent_id, call_id, approved }).await;
        });
    };

    // ── View ──────────────────────────────────────────────────────────────────
    view! {
        <div class="chat" style="position: relative;">
            // Polished chat header
            <div class="chat-header glass" style="display: flex; align-items: center; gap: 10px; padding: 8px 14px; min-height: 52px;">
                // Left: status dot + connection label
                <div style="display: flex; align-items: center; gap: 8px; flex-shrink: 0;">
                    <div style=move || format!(
                        "width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; background: {}; box-shadow: 0 0 0 2px {};",
                        if online.get() { "#10b981" } else { "#ef4444" },
                        if online.get() { "rgba(16,185,129,0.2)" } else { "rgba(239,68,68,0.2)" }
                    )></div>
                    <span style="font-size: 12.5px; font-weight: 600; color: var(--ink); white-space: nowrap; max-width: 160px; overflow: hidden; text-overflow: ellipsis;">
                        {move || match ctx.routed_instance.get() {
                            Some((h, p)) => format!("{}:{}", h, p),
                            None => "Local Server".to_string()
                        }}
                    </span>
                </div>

                // Divider
                <div style="width: 1px; height: 20px; background: var(--hairline); flex-shrink: 0;"></div>

                // Center: model selector (takes most space)
                <select
                    class="model-select"
                    style="flex: 1; min-width: 0; background-color: var(--canvas); border-color: var(--hairline); color: var(--ink); border-radius: var(--r-sm); height: 30px; font-size: 13px; font-weight: 500;"
                    prop:value=move || { let _ = models.get(); selected_model.get() }
                    on:change=move |e| on_model_change(event_target_value(&e))
                >
                    {move || models.get().into_iter().map(|m| {
                        let label = m.clone();
                        view! { <option value=m>{label}</option> }
                    }).collect_view()}
                </select>

                // Right: mode toggles + action buttons
                <div style="display: flex; align-items: center; gap: 6px; flex-shrink: 0;">
                    // Agent Mode toggle
                    <button
                        style=move || if use_agent.get() {
                            "height: 28px; padding: 0 10px; border-radius: 14px; border: var(--border-width) solid transparent; font-size: 12px; font-weight: 600; cursor: pointer; display: inline-flex; align-items: center; gap: 4px; background: var(--primary); color: var(--on-primary);"
                        } else {
                            "height: 28px; padding: 0 10px; border-radius: 14px; border: var(--border-width) solid var(--hairline); font-size: 12px; font-weight: 500; cursor: pointer; display: inline-flex; align-items: center; gap: 4px; background: transparent; color: var(--body);"
                        }
                        on:click=move |_| use_agent.set(!use_agent.get_untracked())
                        title="Toggle Agent Mode"
                    >
                        "🤖 Agent"
                    </button>
                    // Immersive toggle
                    <button
                        style=move || if ctx.immersive.get() {
                            "height: 28px; padding: 0 10px; border-radius: 14px; border: var(--border-width) solid transparent; font-size: 12px; font-weight: 600; cursor: pointer; display: inline-flex; align-items: center; gap: 4px; background: var(--primary); color: var(--on-primary);"
                        } else {
                            "height: 28px; padding: 0 10px; border-radius: 14px; border: var(--border-width) solid var(--hairline); font-size: 12px; font-weight: 500; cursor: pointer; display: inline-flex; align-items: center; gap: 4px; background: transparent; color: var(--body);"
                        }
                        on:click=move |_| ctx.immersive.set(!ctx.immersive.get_untracked())
                        title="Toggle Immersive Mode"
                    >
                        "✨ Focus"
                    </button>
                    // Context overview toggle
                    <button
                        style=move || if ctx.chat_show_context.get() {
                            "height: 28px; padding: 0 10px; border-radius: 14px; border: var(--border-width) solid transparent; font-size: 12px; font-weight: 600; cursor: pointer; display: inline-flex; align-items: center; gap: 4px; background: var(--primary); color: var(--on-primary);"
                        } else {
                            "height: 28px; padding: 0 10px; border-radius: 14px; border: var(--border-width) solid var(--hairline); font-size: 12px; font-weight: 500; cursor: pointer; display: inline-flex; align-items: center; gap: 4px; background: transparent; color: var(--body);"
                        }
                        on:click=move |_| ctx.chat_show_context.set(!ctx.chat_show_context.get_untracked())
                        title="Show/hide context overview"
                    >
                        "Ctx"
                    </button>
                    // Divider
                    <div style="width: 1px; height: 20px; background: var(--hairline);"></div>
                    // Clear button
                    <button
                        style="height: 28px; padding: 0 10px; border-radius: 14px; border: var(--border-width) solid var(--hairline); font-size: 12px; font-weight: 500; cursor: pointer; display: inline-flex; align-items: center; gap: 4px; background: transparent; color: var(--muted);"
                        on:click=clear
                        title="Clear chat"
                    >
                        "✕ Clear"
                    </button>
                </div>
            </div>

            // Context overview panel (collapsible)
            {move || ctx.chat_show_context.get().then(|| {
                let msgs = ctx.chat_messages.get();
                let total_tokens: usize = msgs.iter().map(|m| m.content.len().saturating_div(4)).sum();
                let ctx_size = ctx.config.get().ctx_size.max(1) as usize;
                let percent = ((total_tokens as f64 / ctx_size as f64) * 100.0).min(100.0) as usize;
                let bar_color = if percent > 80 { "#ef4444" } else if percent > 60 { "#f59e0b" } else { "#10b981" };
                view! {
                    <div style="padding: 10px 14px; border-bottom: 1px solid var(--hairline); background: var(--canvas); max-height: 200px; overflow-y: auto; flex-shrink: 0;">
                        <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;">
                            <span style="font-size: 12px; font-weight: 700; color: var(--ink);">"Context Window"</span>
                            <span style="font-size: 11px; color: var(--muted);">{format!("~{total_tokens} / {ctx_size} tokens ({percent}%)")}</span>
                        </div>
                        <div style="height: 4px; background: var(--hairline); border-radius: 2px; margin-bottom: 8px; overflow: hidden;">
                            <div style=format!("height: 100%; width: {}%; background: {}; border-radius: 2px; transition: width 0.3s;", percent, bar_color)></div>
                        </div>
                        {msgs.into_iter().map(|m| {
                            let toks = m.content.len().saturating_div(4);
                            let role_color = match m.role.as_str() {
                                "user"      => "var(--primary)",
                                "assistant" => "var(--accent)",
                                _           => "var(--muted)",
                            };
                            let preview = if m.content.len() > 72 {
                                format!("{}…", &m.content[..72])
                            } else { m.content.clone() };
                            view! {
                                <div style="display: flex; align-items: center; gap: 8px; padding: 2px 0; font-size: 11px;">
                                    <span style=format!("flex-shrink: 0; font-weight: 700; color: {}; min-width: 56px;", role_color)>{m.role.clone()}</span>
                                    <span style="flex: 1; color: var(--body); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{preview}</span>
                                    <span style="flex-shrink: 0; color: var(--muted);">{format!("~{toks}t")}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }
            })}

            {move || (!online.get()).then(|| {
                if ctx.server_running.get() {
                    view! { <div class="banner info">"⏳ Model server is initializing or loading the model. Please wait…"</div> }
                } else {
                    view! { <div class="banner warn">"⚠️ Model server offline. Start a server to chat."</div> }
                }
            })}
            {move || error.get().map(|e| view! { <div class="toast error">"❌ "{e}</div> })}

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
                        msgs.into_iter().map(|m| {
                            let cls = if m.role == "user" { "msg user" } else if m.role == "system" { "msg system" } else { "msg assistant" };
                            view! {
                                <div class=cls>
                                    <div class="bubble">{m.content}</div>
                                </div>
                            }
                        }).collect_view().into_any()
                    }
                }}
                {move || generating.get().then(|| view! {
                    <div class="thinking"><Spinner/>"Working…"</div>
                })}
            </div>

            // Agent plan
            {move || (!plan.get().is_empty()).then(|| view! {
                <div class="panel glass" style="margin-bottom: var(--s-sm);">
                    <div class="panel-title">"📋 Plan"</div>
                    <ul class="plan">
                        {move || plan.get().into_iter().map(|s| {
                            let icon = match s.status {
                                PlanStatus::Done    => "✅",
                                PlanStatus::Running => "⏳",
                                PlanStatus::Failed  => "❌",
                                PlanStatus::Pending => "•",
                            };
                            view! { <li><span class="plan-icon">{icon}</span>{s.description}</li> }
                        }).collect_view()}
                    </ul>
                </div>
            })}

            // Approval prompts
            {move || (!approvals.get().is_empty()).then(|| view! {
                <div class="panel approval glass" style="margin-bottom: var(--s-sm);">
                    <div class="panel-title">"⚠️ Approval required"</div>
                    {move || approvals.get().into_iter().map(|(agent_id, req)| {
                        let (a1, a2) = (agent_id.clone(), agent_id);
                        let (c1, c2) = (req.call_id.clone(), req.call_id.clone());
                        view! {
                            <div class="approval-item">
                                <div class="approval-tool">
                                    <strong>{req.tool.clone()}</strong>
                                    <code>{req.args.to_string()}</code>
                                </div>
                                <div class="approval-actions">
                                    <button class="btn allow" on:click=move |_| respond(a1.clone(), c1.clone(), true)>"Allow"</button>
                                    <button class="btn deny"  on:click=move |_| respond(a2.clone(), c2.clone(), false)>"Deny"</button>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            })}

            // Agent trace
            {move || (!trace.get().is_empty()).then(|| view! {
                <details class="panel trace glass" open=true style="margin-bottom: var(--s-sm);">
                    <summary>{move || format!("🧭 Agent trace ({} events)", trace.get().len())}</summary>
                    <div class="trace-lines">
                        {move || trace.get().into_iter().map(|l| view! { <div class="trace-line">{l}</div> }).collect_view()}
                    </div>
                </details>
            })}

            // Slash-command popup
            {move || {
                if popup_open.get() && !filtered_options.get().is_empty() {
                    let opts = filtered_options.get();
                    let idx = active_index.get();
                    Some(view! {
                        <div class="composer-popup" style="position: absolute; bottom: 65px; left: 16px; right: 16px; background: var(--surface-card); border: var(--border-width, 1px) solid var(--hairline); border-radius: var(--r-md); box-shadow: 0 -4px 20px rgba(0,0,0,0.3); z-index: 10; display: flex; flex-direction: column; max-height: 280px; backdrop-filter: blur(10px);">
                            <div style="overflow-y: auto; flex-grow: 1; padding: 4px 0;">
                                {opts.into_iter().enumerate().map(|(i, opt)| {
                                    let active = i == idx;
                                    let (opt_c, opt_c2) = (opt.cmd.clone(), opt.cmd.clone());
                                    let on_click = move |_| {
                                        let suffix = match opt_c.as_str() {
                                            "/clear" | "/help" | "/mcp" | "/agents" => "",
                                            _ => " ",
                                        };
                                        input.set(format!("{}{}", opt_c, suffix));
                                        popup_open.set(false);
                                        if opt_c == "/clear" || opt_c == "/help" || opt_c == "/mcp" || opt_c == "/agents" || opt_c.starts_with("/agent-skills:") {
                                            send();
                                        }
                                    };
                                    view! {
                                        <div
                                            style=move || if active {
                                                "display: flex; justify-content: space-between; padding: 8px 12px; cursor: pointer; background: var(--primary); color: var(--on-primary); font-size: 13px; align-items: center;"
                                            } else {
                                                "display: flex; justify-content: space-between; padding: 8px 12px; cursor: pointer; color: var(--ink); font-size: 13px; align-items: center;"
                                            }
                                            on:click=on_click
                                        >
                                            <span style="font-family: var(--font-mono); font-weight: 600; flex-shrink: 0; margin-right: 16px;">{opt_c2}</span>
                                            <span style=move || if active {
                                                "font-size: 12px; opacity: 0.9; text-align: right; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 65%;"
                                            } else {
                                                "font-size: 12px; color: var(--muted); text-align: right; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 65%;"
                                            }>{opt.desc.clone()}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                            <div style="padding: 6px 12px; border-top: var(--border-width, 1px) solid var(--hairline); font-size: 11px; color: var(--muted); display: flex; justify-content: space-between; background: rgba(0,0,0,0.15); border-bottom-left-radius: var(--r-md); border-bottom-right-radius: var(--r-md);">
                                <div>
                                    <span style="font-weight: bold; color: var(--ink);">"↑/↓"</span>" Navigate  •  "
                                    <span style="font-weight: bold; color: var(--ink);">"enter"</span>" Select  •  "
                                    <span style="font-weight: bold; color: var(--ink);">"tab"</span>" Complete"
                                </div>
                                <div><span style="font-weight: bold; color: var(--ink);">"esc"</span>" to cancel"</div>
                            </div>
                        </div>
                    }.into_any())
                } else { None }
            }}

            <div class="modern-composer glass">
                <input
                    class="composer-input"
                    type="text"
                    placeholder=move || {
                        if online.get() { "Type a message, reference notes/planner/calendar or click suggestion…" }
                        else { "Start the server first…" }
                    }
                    prop:value=move || input.get()
                    prop:disabled=move || !online.get() || generating.get()
                    on:input=move |e| {
                        let val = event_target_value(&e);
                        input.set(val.clone());
                        if val.starts_with('/') && !val.contains(' ') { popup_open.set(true); }
                        else { popup_open.set(false); }
                    }
                    on:keydown=move |e: KeyboardEvent| {
                        if popup_open.get_untracked() && !filtered_options.get_untracked().is_empty() {
                            let opts = filtered_options.get_untracked();
                            let idx  = active_index.get_untracked();
                            match e.key().as_str() {
                                "ArrowDown" => {
                                    e.prevent_default();
                                    active_index.set(if idx + 1 < opts.len() { idx + 1 } else { 0 });
                                }
                                "ArrowUp" => {
                                    e.prevent_default();
                                    active_index.set(if idx > 0 { idx - 1 } else { opts.len() - 1 });
                                }
                                "Tab" => {
                                    e.prevent_default();
                                    let completed = &opts[idx].cmd;
                                    let suffix = match completed.as_str() { "/clear"|"/help"|"/mcp"|"/agents" => "", _ => " " };
                                    input.set(format!("{}{}", completed, suffix));
                                    popup_open.set(false);
                                }
                                "Enter" => {
                                    e.prevent_default();
                                    let completed = opts[idx].cmd.clone();
                                    let suffix = match completed.as_str() { "/clear"|"/help"|"/mcp"|"/agents" => "", _ => " " };
                                    input.set(format!("{}{}", completed, suffix));
                                    popup_open.set(false);
                                    if completed == "/clear" || completed == "/help" || completed == "/mcp" || completed == "/agents" || completed.starts_with("/agent-skills:") {
                                        send();
                                    }
                                }
                                "Escape" => { e.prevent_default(); popup_open.set(false); }
                                _ => {}
                            }
                        } else if e.key() == "Enter" {
                            send();
                        }
                    }
                />
                // Send / Stop button — when generating, becomes a stop button
                <button
                    class="composer-send-btn"
                    class:stop=move || generating.get()
                    prop:disabled=move || if generating.get() { false } else { !online.get() }
                    on:click=move |_| {
                        if generating.get_untracked() { cancel(); } else { send(); }
                    }
                    title=move || if generating.get() { "Stop generation" } else { "Send message" }
                >
                    {move || if generating.get() {
                        view! { <span style="font-size: 16px; font-weight: 900; line-height: 1; color: #ef4444;">"\u{25A0}"</span> }.into_any()
                    } else {
                        view! { <span style="font-size: 16px; font-weight: bold;">"↑"</span> }.into_any()
                    }}
                </button>
            </div>
        </div>
    }
}
