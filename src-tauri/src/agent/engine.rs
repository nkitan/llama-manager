//! The ReAct execution loop (L1), integrating planning (L2), sub-agents (L3), and
//! approval/timeout/retry/cancellation (L4).
//!
//! Flow per run: emit `Started` → build a plan → loop up to [`MAX_STEPS`]:
//! call the model with the tool schemas; if it returns tool calls, execute each
//! (gating sensitive ones behind approval, bounding every call by a timeout +
//! retries) and feed results back; otherwise treat the content as the final
//! answer, persist a lesson, and finish.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use shared::ipc::{AGENT_EVENT, AgentEvent, PlanStatus, ToolCall, ToolResult};
use tauri::{AppHandle, Emitter};

use super::tools::{SPAWN_SUBAGENT, ToolContext, ToolRegistry};
use super::{AgentContext, MAX_STEPS, approval, llm, memory::MemoryManager, planner, supervisor};
use crate::state::AgentHandle;
use crate::util::new_id;
use crate::commands::remaining::log_monitor_event_internal;

/// Per-tool execution budget (the sensitive `run_command` also self-limits).
const TOOL_TIMEOUT: Duration = Duration::from_secs(30);
const TOOL_RETRIES: u32 = 2;

fn load_all_agent_rules(config_dir: &std::path::Path) -> String {
    let mut rules = String::new();
    
    let search_dirs = vec![
        config_dir.to_path_buf(),
        std::path::PathBuf::from("/home/notroot/Work/llama-manager"),
    ];

    let standalone_files = vec![
        ("AGENTS.md", "AGENTS.md"),
        ("agents.md", "agents.md"),
        ("CLAUDE.md", "CLAUDE.md"),
        ("claude.md", "claude.md"),
        (".cursorrules", ".cursorrules"),
        (".copilotrules", ".copilotrules"),
        (".windsurfrules", ".windsurfrules"),
        (".ai-instructions", ".ai-instructions"),
        (".agentrules", ".agentrules"),
        (".github/copilot-instructions.md", ".github/copilot-instructions.md"),
        (".clauderc", ".clauderc"),
        (".codexrules", ".codexrules"),
        ("codex.json", "codex.json"),
        (".codex", ".codex"),
        (".antigravityrules", ".antigravityrules"),
        (".antigravity", ".antigravity"),
        (".hermesrules", ".hermesrules"),
        ("hermes.json", "hermes.json"),
        (".openclawrules", ".openclawrules"),
        ("openclaw.json", "openclaw.json"),
    ];

    // Maintain a set of loaded paths to avoid duplication if dirs overlap
    let mut loaded_paths = std::collections::HashSet::new();

    for dir in &search_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }
        for (filename, label) in &standalone_files {
            let path = dir.join(filename);
            if path.is_file() {
                if let Ok(abs_path) = path.canonicalize() {
                    if !loaded_paths.insert(abs_path) {
                        continue;
                    }
                }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if !content.trim().is_empty() {
                        rules.push_str(&format!(
                            "\n\n=== AGENT RULES ({}) ===\n{}\n",
                            label,
                            content.trim()
                        ));
                    }
                }
            }
        }

        let cursor_rules_dir = dir.join(".cursor").join("rules");
        if cursor_rules_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(cursor_rules_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "mdc") {
                        if let Ok(abs_path) = path.canonicalize() {
                            if !loaded_paths.insert(abs_path) {
                                continue;
                            }
                        }
                        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let mut display_content = content.clone();
                            let mut globs_info = String::new();
                            if content.starts_with("---") {
                                let parts: Vec<&str> = content.split("---").collect();
                                if parts.len() >= 3 {
                                    let yaml = parts[1];
                                    for line in yaml.lines() {
                                        if line.trim().starts_with("globs:") {
                                            globs_info = format!(" (Applies to: {})", line.replace("globs:", "").trim());
                                        }
                                    }
                                    display_content = parts[2..].join("---");
                                }
                            }
                            
                            rules.push_str(&format!(
                                "\n\n=== AGENT RULE ({}{}) ===\n{}\n",
                                filename,
                                globs_info,
                                display_content.trim()
                            ));
                        }
                    }
                }
            }
        }
    }

    rules
}

pub async fn run(
    ctx: AgentContext,
    handle: Arc<AgentHandle>,
    parent: Option<String>,
    role: String,
    task: String,
    depth: u8,
) -> String {
    let app = ctx.app.clone();
    emit(
        &app,
        AgentEvent::Started {
            agent_id: handle.id.clone(),
            parent,
            role: role.clone(),
            task: task.clone(),
        },
    );

    log_monitor_event_internal(
        &handle.id,
        "Active",
        &format!("Starting task: {}", truncate(&task, 50)),
        &format!("Agent spawned with role '{}' and task '{}'", role, task),
    );

    let registry = ToolRegistry::builtin();
    let tool_ctx = ToolContext {
        searxng_url: ctx.searxng_url.clone(),
        config_dir: ctx.config_dir.clone(),
    };
    let memory = MemoryManager::new(&ctx.config_dir);

    // ── L2: plan ──────────────────────────────────────────────────────────
    let plan = planner::make_plan(&ctx, &task).await;
    emit(
        &app,
        AgentEvent::Plan {
            agent_id: handle.id.clone(),
            steps: plan.clone(),
        },
    );

    // ── Load MCP context ─────────────────────────────────────────────────
    let mcp_context = match std::fs::read_to_string(ctx.config_dir.join("mcp_registry.json")) {
        Ok(content) => {
            if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&content) {
                let servers = map.get("mcpServers").or_else(|| map.get("mcp_servers")).and_then(|v| v.as_object());
                if let Some(servers) = servers {
                    if !servers.is_empty() {
                        let mut prompt_part = "\n\nAvailable Model Context Protocol (MCP) servers and tools (you can run them using the `call_mcp_tool` tool):\n".to_string();
                        for (name, server_val) in servers {
                            prompt_part.push_str(&format!("- Server '{}':\n", name));
                            if let Some(tools_arr) = server_val.get("tools").and_then(|v| v.as_array()) {
                                for t in tools_arr {
                                    if let Some(t_str) = t.as_str() {
                                        prompt_part.push_str(&format!("  - Tool '{}'\n", t_str));
                                    }
                                }
                            } else {
                                prompt_part.push_str("  - (no pre-listed tools, inspect server or invoke its tools)\n");
                            }
                        }
                        prompt_part
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        Err(_) => String::new(),
    };

    // Load all agent rules natively if present (AGENTS.md, CLAUDE.md, .cursorrules, etc.)
    let agents_md_context = load_all_agent_rules(&ctx.config_dir);

    // ── Conversation seed ─────────────────────────────────────────────────
    let system = format!(
        "You are a {role} agent. You have autonomous capabilities to complete the user's task using the provided tools. \
         {}\
         \n\n=== AUTONOMOUS DATA WRITE GUIDELINES ===\n\
         If you determine that writing or updating user data is necessary to complete the task or improve future runs, you have full authority to execute write operations autonomously:\n\
         - Use `write_note` to record documentation, analysis, or persistent knowledge.\n\
         - Use `add_todo` / `complete_todo` to log and track checklist tasks.\n\
         - Use `add_calendar_event` / `delete_calendar_event` to schedule, reschedule, or clear calendar events.\n\
         - Use `add_planner_task` / `update_planner_task` / `delete_planner_task` to create, transition, or resolve tasks on the Kanban board.\n\
         \n=== SELF-IMPROVEMENT & MEMORY LOOP ===\n\
         Read the 'Lessons learned from previous tasks' and other database records carefully. You are a self-learning agent: use this context to refine your plans, adopt user preferences, correct past errors, and avoid duplicate actions. Ensure your decisions build on past learnings.\n\n\
         Think step-by-step, call tools when useful, and stop when done by replying with the final answer (no tool call).{}{}",
        agents_md_context,
        memory.context(),
        mcp_context
    );
    let mut messages = vec![
        json!({ "role": "system", "content": system }),
        json!({ "role": "user", "content": task }),
    ];

    let specs = registry.specs();
    let mut final_text = String::from("No answer produced.");

    for step in 0..MAX_STEPS {
        if handle.cancel.is_cancelled() {
            emit(&app, err(&handle.id, "cancelled by user"));
            return "Cancelled.".to_string();
        }

        // Surface plan progress loosely against step index.
        if let Some(s) = plan.get(step as usize) {
            emit(
                &app,
                AgentEvent::PlanUpdate {
                    agent_id: handle.id.clone(),
                    step_id: s.id.clone(),
                    status: PlanStatus::Running,
                },
            );
        }

        let reply = match llm::call(&ctx, &messages, Some(&specs), 0.2).await {
            Ok(r) => r,
            Err(e) => {
                emit(&app, err(&handle.id, &e));
                return format!("Error: {e}");
            }
        };

        // No tool calls → the model is answering. We're done.
        if reply.tool_calls.is_empty() {
            final_text = reply.content.clone();
            emit(
                &app,
                AgentEvent::Token {
                    agent_id: handle.id.clone(),
                    delta: final_text.clone(),
                },
            );
            mark_done(&app, &handle.id, &plan);
            break;
        }

        // Emit the model's reasoning preface (if any) as a step thought.
        if !reply.content.trim().is_empty() {
            emit(
                &app,
                AgentEvent::Step {
                    agent_id: handle.id.clone(),
                    index: step,
                    thought: reply.content.clone(),
                },
            );
        }

        // Reconstruct the assistant tool-call message so the model keeps context
        // for the tool responses we append below. Generate stable call ids.
        let mut tool_call_json = Vec::new();
        let mut calls = Vec::new();
        for tc in &reply.tool_calls {
            let call_id = if tc.id.is_empty() {
                new_id("call")
            } else {
                tc.id.clone()
            };
            let args: Value = serde_json::from_str(&tc.arguments).unwrap_or_else(|_| json!({}));
            tool_call_json.push(json!({
                "id": call_id,
                "type": "function",
                "function": { "name": tc.name, "arguments": tc.arguments }
            }));
            calls.push(ToolCall {
                call_id,
                tool: tc.name.clone(),
                args,
                sensitive: registry.is_sensitive(&tc.name),
            });
        }
        messages.push(json!({
            "role": "assistant",
            "content": reply.content,
            "tool_calls": tool_call_json,
        }));

        // Execute each call, appending a tool response message for each.
        for call in &calls {
            emit(
                &app,
                AgentEvent::ToolCall {
                    agent_id: handle.id.clone(),
                    call: call.clone(),
                },
            );

            log_monitor_event_internal(
                &handle.id,
                "Active",
                &format!("Running tool '{}'", call.tool),
                &format!("Invoked tool '{}' with arguments: {}", call.tool, call.args),
            );

            let result = execute_call(&ctx, &handle, &registry, &tool_ctx, call, depth).await;

            log_monitor_event_internal(
                &handle.id,
                "Active",
                &format!("Finished tool '{}'", call.tool),
                &format!("Tool '{}' returned: {}", call.tool, truncate(&result.output, 100)),
            );

            emit(
                &app,
                AgentEvent::ToolResult {
                    agent_id: handle.id.clone(),
                    result: result.clone(),
                },
            );
            messages.push(json!({
                "role": "tool",
                "tool_call_id": call.call_id,
                "content": result.output,
            }));
        }
    }

    // ── Reflection: keep a one-line lesson for future runs (best effort) ────
    if !final_text.trim().is_empty() && final_text != "No answer produced." {
        let reflect_system = "You are an agent memory consolidation system. Review the task the user requested, and the final answer produced by the assistant. Extract a single concise lesson, fact, rule, or user preference that should be stored in memory to help future runs of the agent. Respond with ONLY that single key takeaway (max 100 characters, no prefix like 'Lesson:', no quotes, no conversational filler).";
        let reflect_user = format!("Task: {}\n\nFinal Answer: {}", task, final_text);
        let reflect_messages = vec![
            json!({ "role": "system", "content": reflect_system }),
            json!({ "role": "user", "content": reflect_user }),
        ];
        
        let mut lesson_added = false;
        if let Ok(reply) = llm::call(&ctx, &reflect_messages, None, 0.1).await {
            let lesson = reply.content.trim().trim_matches('"').trim().to_string();
            if !lesson.is_empty() {
                memory.add_lesson(&lesson);
                lesson_added = true;
            }
        }
        if !lesson_added {
            memory.add_lesson(&format!("Task '{}' → {}", truncate(&task, 80), truncate(&final_text, 160)));
        }
    }

    emit(
        &app,
        AgentEvent::Done {
            agent_id: handle.id.clone(),
            final_text: final_text.clone(),
        },
    );

    log_monitor_event_internal(
        &handle.id,
        "Idle",
        "Idle",
        &format!("Completed task. Final output: {}", truncate(&final_text, 120)),
    );

    final_text
}

/// Run one tool call: route `spawn_subagent` to the supervisor, gate sensitive
/// tools behind approval, and bound everything by a timeout + retries.
async fn execute_call(
    ctx: &AgentContext,
    handle: &Arc<AgentHandle>,
    registry: &ToolRegistry,
    tool_ctx: &ToolContext,
    call: &ToolCall,
    depth: u8,
) -> ToolResult {
    // L3: sub-agent delegation.
    if call.tool == SPAWN_SUBAGENT {
        let output = supervisor::spawn(ctx, handle, call, depth).await;
        return ToolResult {
            call_id: call.call_id.clone(),
            ok: true,
            output,
        };
    }

    // L4: human approval for side-effecting tools.
    if call.sensitive {
        let approved = approval::gate(
            &ctx.app,
            handle,
            call,
            &format!("Tool '{}' performs a side effect.", call.tool),
        )
        .await;
        if !approved {
            return ToolResult {
                call_id: call.call_id.clone(),
                ok: false,
                output: "Denied by user (or timed out).".to_string(),
            };
        }
    }

    let Some(tool) = registry.get(&call.tool) else {
        return ToolResult {
            call_id: call.call_id.clone(),
            ok: false,
            output: format!("Unknown tool '{}'.", call.tool),
        };
    };

    // Bounded retries with linear backoff; timeout each attempt (L4 sandboxing).
    let mut last_err = String::new();
    for attempt in 0..=TOOL_RETRIES {
        if handle.cancel.is_cancelled() {
            last_err = "cancelled".into();
            break;
        }
        match tokio::time::timeout(TOOL_TIMEOUT, tool.run(&call.args, tool_ctx)).await {
            Ok(Ok(output)) => {
                return ToolResult {
                    call_id: call.call_id.clone(),
                    ok: true,
                    output,
                };
            }
            Ok(Err(e)) => last_err = e,
            Err(_) => last_err = format!("timed out after {}s", TOOL_TIMEOUT.as_secs()),
        }
        if attempt < TOOL_RETRIES {
            tokio::time::sleep(Duration::from_millis(300 * (attempt as u64 + 1))).await;
        }
    }
    // Structured error fed back to the model so it can adapt (recoverable).
    ToolResult {
        call_id: call.call_id.clone(),
        ok: false,
        output: format!("Tool '{}' failed: {}", call.tool, last_err),
    }
}

fn mark_done(app: &AppHandle, agent_id: &str, plan: &[shared::ipc::PlanStep]) {
    for s in plan {
        emit(
            app,
            AgentEvent::PlanUpdate {
                agent_id: agent_id.to_string(),
                step_id: s.id.clone(),
                status: PlanStatus::Done,
            },
        );
    }
}

fn emit(app: &AppHandle, event: AgentEvent) {
    if let Err(e) = app.emit(AGENT_EVENT, event) {
        tracing::warn!(%e, "failed to emit agent event");
    }
}

fn err(agent_id: &str, message: &str) -> AgentEvent {
    AgentEvent::Error {
        agent_id: agent_id.to_string(),
        message: message.to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
