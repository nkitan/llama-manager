//! System Overview tab — SVG node+connection wiring diagram of all system
//! components. Nodes light up when recent obs_events reference them.

use leptos::prelude::*;
use crate::state::AppCtx;

#[derive(Clone)]
struct NodeDef {
    id: &'static str,
    label: &'static str,
    icon: &'static str,
    x: f64,
    y: f64,
    color: &'static str,
}

#[derive(Clone, Copy)]
struct EdgeDef {
    from: &'static str,
    to: &'static str,
}

fn nodes() -> &'static [NodeDef] {
    static NODES: std::sync::OnceLock<Vec<NodeDef>> = std::sync::OnceLock::new();
    NODES.get_or_init(|| vec![
        NodeDef { id: "llama-server",  label: "llama-server",  icon: "🦙", x: 440.0, y:  60.0, color: "#6366f1" },
        NodeDef { id: "chat",          label: "Chat",           icon: "💬", x: 170.0, y: 170.0, color: "#3b82f6" },
        NodeDef { id: "agent-engine",  label: "Agent Engine",   icon: "🧠", x: 710.0, y: 170.0, color: "#8b5cf6" },
        NodeDef { id: "mcp-tools",     label: "MCP Tools",      icon: "🔌", x: 540.0, y: 290.0, color: "#14b8a6" },
        NodeDef { id: "web-search",    label: "Web Search",     icon: "🌐", x: 670.0, y: 290.0, color: "#0ea5e9" },
        NodeDef { id: "file-tools",    label: "File Tools",     icon: "📄", x: 800.0, y: 290.0, color: "#f59e0b" },
        NodeDef { id: "shell",         label: "Shell",          icon: "⚡", x: 410.0, y: 290.0, color: "#ef4444" },
        NodeDef { id: "memory",        label: "Memory",         icon: "💾", x: 100.0, y: 410.0, color: "#10b981" },
        NodeDef { id: "notes",         label: "Quick Notes",    icon: "📝", x: 240.0, y: 410.0, color: "#84cc16" },
        NodeDef { id: "todos",         label: "Todos",          icon: "✓",  x: 370.0, y: 410.0, color: "#22c55e" },
        NodeDef { id: "calendar",      label: "Calendar",       icon: "📅", x: 500.0, y: 410.0, color: "#f97316" },
        NodeDef { id: "planner",       label: "Task Planner",   icon: "🗂", x: 630.0, y: 410.0, color: "#ec4899" },
        NodeDef { id: "deep-research", label: "Deep Research",  icon: "🔬", x: 760.0, y: 500.0, color: "#a855f7" },
    ])
}

fn edges() -> &'static [EdgeDef] {
    &[
        EdgeDef { from: "llama-server",  to: "chat" },
        EdgeDef { from: "llama-server",  to: "agent-engine" },
        EdgeDef { from: "chat",          to: "agent-engine" },
        EdgeDef { from: "agent-engine",  to: "mcp-tools" },
        EdgeDef { from: "agent-engine",  to: "web-search" },
        EdgeDef { from: "agent-engine",  to: "file-tools" },
        EdgeDef { from: "agent-engine",  to: "shell" },
        EdgeDef { from: "agent-engine",  to: "memory" },
        EdgeDef { from: "agent-engine",  to: "notes" },
        EdgeDef { from: "agent-engine",  to: "todos" },
        EdgeDef { from: "agent-engine",  to: "calendar" },
        EdgeDef { from: "agent-engine",  to: "planner" },
        EdgeDef { from: "agent-engine",  to: "deep-research" },
        EdgeDef { from: "memory",        to: "chat" },
    ]
}

fn classify_event_nodes(kind: &str, id: &str) -> Vec<&'static str> {
    let mut result = vec![];
    if kind.starts_with("chat:") {
        result.push("chat");
        result.push("llama-server");
    }
    if kind.starts_with("agent:") {
        result.push("agent-engine");
    }
    if kind == "agent:tool_call" || kind == "agent:tool_ok" || kind == "agent:tool_err" {
        let id_l = id.to_lowercase();
        if id_l.contains("web") || id_l.contains("search") || id_l.contains("searx") { result.push("web-search"); }
        if id_l.contains("file") || id_l.contains("read") || id_l.contains("write") { result.push("file-tools"); }
        if id_l.contains("mcp")      { result.push("mcp-tools"); }
        if id_l.contains("note")     { result.push("notes"); }
        if id_l.contains("todo")     { result.push("todos"); }
        if id_l.contains("calendar") { result.push("calendar"); }
        if id_l.contains("planner") || id_l.contains("task") { result.push("planner"); }
        if id_l.contains("memory") || id_l.contains("obsidian") { result.push("memory"); }
        if id_l.contains("run_command") || id_l.contains("shell") || id_l.contains("bash") { result.push("shell"); }
        if id_l.contains("research") { result.push("deep-research"); }
    }
    result
}

#[component]
pub fn OverviewTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    // Derive active node set from recent obs_events (last 4 seconds).
    let active_set = Memo::new(move |_| {
        let events = ctx.obs_events.get();
        let now = js_sys::Date::now();
        let mut active: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ev in events.iter().rev().take(100) {
            if now - ev.ts < 4000.0 {
                for node_id in classify_event_nodes(&ev.kind, &ev.id) {
                    active.insert(node_id.to_string());
                }
            } else {
                break;
            }
        }
        active
    });

    // Build position lookup for edges (using 'static str keys, safe in static context)
    let pos: std::collections::HashMap<&'static str, (f64, f64)> = nodes().iter()
        .map(|n| (n.id, (n.x, n.y)))
        .collect();

    let recent_events = Memo::new(move |_| {
        let events = ctx.obs_events.get();
        let now = js_sys::Date::now();
        events.into_iter().rev()
            .filter(|e| now - e.ts < 30000.0)
            .take(12)
            .collect::<Vec<_>>()
    });

    view! {
        <div class="page" style="padding: var(--s-md); gap: var(--s-md); height: 100%; overflow: hidden; display: flex; flex-direction: column;">
            // Header strip
            <div style="display: flex; align-items: center; justify-content: space-between; flex-shrink: 0;">
                <div>
                    <h2 style="font-size: 18px; font-weight: 700; color: var(--ink);">"System Overview"</h2>
                    <p style="font-size: 12px; color: var(--muted); margin-top: 2px;">"Live wiring diagram — nodes glow when active"</p>
                </div>
                <div style="display: flex; align-items: center; gap: 8px;">
                    <div style=move || format!(
                        "width: 8px; height: 8px; border-radius: 50%; background: {}; box-shadow: 0 0 0 2px {};",
                        if !active_set.get().is_empty() { "#10b981" } else { "var(--muted)" },
                        if !active_set.get().is_empty() { "rgba(16,185,129,0.3)" } else { "transparent" }
                    )></div>
                    <span style="font-size: 12px; color: var(--muted);">
                        {move || if !active_set.get().is_empty() {
                            format!("{} active nodes", active_set.get().len())
                        } else { "Idle".to_string() }}
                    </span>
                </div>
            </div>

            // Main layout: SVG diagram + recent events sidebar
            <div style="display: flex; flex: 1; min-height: 0; gap: var(--s-md);">
                // SVG diagram
                <div class="card" style="flex: 1; min-width: 0; display: flex; align-items: center; justify-content: center; overflow: hidden; padding: var(--s-md);">
                    <svg
                        viewBox="0 0 920 580"
                        width="100%"
                        height="100%"
                        style="max-height: calc(100vh - 260px);"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <defs>
                            <marker id="arrow" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
                                <path d="M0,0 L0,6 L8,3 z" fill="var(--hairline)" />
                            </marker>
                            <marker id="arrow-active" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
                                <path d="M0,0 L0,6 L8,3 z" fill="#10b981" />
                            </marker>
                        </defs>

                        // Edges — from/to are &'static str, safe to capture in 'static closures
                        {edges().iter().map(|edge| {
                            let from = edge.from;
                            let to   = edge.to;
                            let (x1, y1) = pos.get(from).copied().unwrap_or((0.0, 0.0));
                            let (x2, y2) = pos.get(to).copied().unwrap_or((0.0, 0.0));
                            view! {
                                <line
                                    x1=x1 y1=y1 x2=x2 y2=y2
                                    stroke=move || {
                                        if active_set.get().contains(from) && active_set.get().contains(to) { "#10b981" }
                                        else { "var(--hairline)" }
                                    }
                                    stroke-width=move || {
                                        if active_set.get().contains(from) && active_set.get().contains(to) { "2.5" } else { "1.5" }
                                    }
                                    stroke-opacity="0.7"
                                    marker-end=move || {
                                        if active_set.get().contains(from) && active_set.get().contains(to) { "url(#arrow-active)" }
                                        else { "url(#arrow)" }
                                    }
                                />
                            }
                        }).collect_view()}

                        // Nodes — clone per-node data for each reactive closure
                        {nodes().iter().map(|node| {
                            let id     = node.id;      // &'static str
                            let color  = node.color;   // &'static str
                            let label  = node.label.to_string();
                            let icon   = node.icon.to_string();
                            let (nx, ny) = (node.x, node.y);
                            view! {
                                <g transform=format!("translate({nx},{ny})")>
                                    // Glow ring when active
                                    {move || {
                                        if active_set.get().contains(id) {
                                            view! {
                                                <ellipse cx="0" cy="0" rx="46" ry="28"
                                                    fill="none"
                                                    stroke=color
                                                    stroke-width="3"
                                                    stroke-opacity="0.5"
                                                >
                                                    <animate attributeName="stroke-opacity" values="0.5;0.9;0.5" dur="1.2s" repeatCount="indefinite"/>
                                                </ellipse>
                                            }.into_any()
                                        } else { view! {}.into_any() }
                                    }}
                                    // Node body
                                    <rect
                                        x="-42" y="-20" width="84" height="40" rx="10"
                                        fill=move || {
                                            if active_set.get().contains(id) {
                                                format!("color-mix(in srgb, {} 25%, var(--surface-card))", color)
                                            } else { "var(--surface-card)".to_string() }
                                        }
                                        stroke=color
                                        stroke-width=move || {
                                            if active_set.get().contains(id) { "2" } else { "1" }
                                        }
                                        stroke-opacity="0.8"
                                    />
                                    // Icon
                                    <text x="-24" y="5" font-size="14" text-anchor="middle" dominant-baseline="middle">
                                        {icon}
                                    </text>
                                    // Label
                                    <text x="10" y="0" font-size="10" fill="var(--ink)" text-anchor="middle" dominant-baseline="middle" font-weight="600">
                                        {label}
                                    </text>
                                </g>
                            }
                        }).collect_view()}
                    </svg>
                </div>

                // Recent events feed (narrow sidebar)
                <div class="card" style="width: 260px; flex-shrink: 0; display: flex; flex-direction: column; overflow: hidden;">
                    <div style="padding: 10px 12px; border-bottom: 1px solid var(--hairline); font-size: 12px; font-weight: 700; color: var(--ink);">"Recent Activity"</div>
                    <div style="flex: 1; overflow-y: auto; padding: 6px 8px; display: flex; flex-direction: column; gap: 4px;">
                        {move || {
                            let evts = recent_events.get();
                            if evts.is_empty() {
                                view! {
                                    <div style="color: var(--muted); font-size: 12px; text-align: center; margin-top: 20px;">"No recent events"</div>
                                }.into_any()
                            } else {
                                evts.into_iter().map(|ev| {
                                    let (badge_bg, badge_color) = match ev.kind.as_str() {
                                        k if k.starts_with("chat:")        => ("rgba(59,130,246,0.15)", "#3b82f6"),
                                        k if k.starts_with("agent:tool")   => ("rgba(245,158,11,0.15)", "#f59e0b"),
                                        k if k.starts_with("agent:")       => ("rgba(139,92,246,0.15)", "#8b5cf6"),
                                        _                                  => ("var(--surface-strong)", "var(--muted)"),
                                    };
                                    let preview = if ev.content.len() > 40 {
                                        format!("{}…", &ev.content[..40])
                                    } else { ev.content.clone() };
                                    view! {
                                        <div style="display: flex; flex-direction: column; gap: 2px; padding: 5px 6px; border-radius: var(--r-sm); background: var(--surface-card);">
                                            <div style="display: flex; align-items: center; gap: 6px;">
                                                <span style=format!("font-size: 10px; padding: 1px 5px; border-radius: 4px; font-weight: 600; background: {}; color: {};", badge_bg, badge_color)>
                                                    {ev.kind.clone()}
                                                </span>
                                                {(!ev.id.is_empty()).then(|| view! {
                                                    <span style="font-size: 10px; color: var(--muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100px;">{ev.id.clone()}</span>
                                                })}
                                            </div>
                                            {(!preview.is_empty()).then(|| view! {
                                                <div style="font-size: 11px; color: var(--body); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{preview}</div>
                                            })}
                                        </div>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
