//! Observability tab — live dashboard of all I/O streams between agents and
//! models. Shows the obs_events ring buffer from AppCtx in a scrollable feed
//! with per-kind filtering and basic stats.

use leptos::prelude::*;
use crate::state::AppCtx;

#[component]
pub fn ObservabilityTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    let filter = RwSignal::new("all".to_string());

    let stats = Memo::new(move |_| {
        let events = ctx.obs_events.get();
        let now = js_sys::Date::now();
        let total = events.len();
        let recent = events.iter().filter(|e| now - e.ts < 5000.0).count();
        let chat_tokens: usize = events.iter().filter(|e| e.kind == "chat:token").count();
        let agent_tokens: usize = events.iter().filter(|e| e.kind == "agent:token").count();
        let tool_calls: usize = events.iter().filter(|e| e.kind == "agent:tool_call").count();
        (total, recent, chat_tokens, agent_tokens, tool_calls)
    });

    let filtered_events = Memo::new(move |_| {
        let events = ctx.obs_events.get();
        let f = filter.get();
        let filtered: Vec<_> = events.into_iter().rev()
            .filter(|ev| match f.as_str() {
                "all"     => true,
                "chat"    => ev.kind.starts_with("chat:"),
                "agent"   => ev.kind.starts_with("agent:") && !ev.kind.contains("token"),
                "tokens"  => ev.kind.ends_with(":token"),
                "tools"   => ev.kind.contains("tool"),
                "errors"  => ev.kind.contains("error") || ev.kind.contains("err"),
                _         => true,
            })
            .take(200)
            .collect();
        filtered
    });

    view! {
        <div class="page" style="padding: var(--s-md); gap: var(--s-md); height: 100%; overflow: hidden; display: flex; flex-direction: column;">
            // Header
            <div style="flex-shrink: 0;">
                <h2 style="font-size: 18px; font-weight: 700; color: var(--ink);">"Observability"</h2>
                <p style="font-size: 12px; color: var(--muted); margin-top: 2px;">"Live I/O streams between agents and models"</p>
            </div>

            // Stats strip
            <div style="display: flex; gap: var(--s-sm); flex-wrap: wrap; flex-shrink: 0;">
                {move || {
                    let (total, recent, chat_tok, agent_tok, tools) = stats.get();
                    view! {
                        <div class="card" style="padding: 10px 16px; min-width: 110px;">
                            <div style="font-size: 22px; font-weight: 800; color: var(--ink);">{total}</div>
                            <div style="font-size: 11px; color: var(--muted);">"Total Events"</div>
                        </div>
                        <div class="card" style="padding: 10px 16px; min-width: 110px;">
                            <div style=format!("font-size: 22px; font-weight: 800; color: {};", if recent > 0 { "#10b981" } else { "var(--muted)" })>{recent}</div>
                            <div style="font-size: 11px; color: var(--muted);">"Active (5s)"</div>
                        </div>
                        <div class="card" style="padding: 10px 16px; min-width: 120px;">
                            <div style="font-size: 22px; font-weight: 800; color: #3b82f6;">{chat_tok}</div>
                            <div style="font-size: 11px; color: var(--muted);">"Chat Tokens"</div>
                        </div>
                        <div class="card" style="padding: 10px 16px; min-width: 120px;">
                            <div style="font-size: 22px; font-weight: 800; color: #8b5cf6;">{agent_tok}</div>
                            <div style="font-size: 11px; color: var(--muted);">"Agent Tokens"</div>
                        </div>
                        <div class="card" style="padding: 10px 16px; min-width: 110px;">
                            <div style="font-size: 22px; font-weight: 800; color: #f59e0b;">{tools}</div>
                            <div style="font-size: 11px; color: var(--muted);">"Tool Calls"</div>
                        </div>
                    }
                }}
            </div>

            // Filter bar + clear
            <div style="display: flex; align-items: center; gap: 8px; flex-shrink: 0;">
                {["all", "chat", "agent", "tokens", "tools", "errors"].iter().map(|&f_val| {
                    let fv = f_val.to_string();
                    view! {
                        <button
                            style=move || {
                                let active = filter.get() == fv;
                                if active {
                                    "padding: 4px 12px; border-radius: var(--r-pill); border: 1px solid var(--primary); background: var(--primary); color: var(--on-primary); font-size: 12px; font-weight: 600; cursor: pointer;"
                                } else {
                                    "padding: 4px 12px; border-radius: var(--r-pill); border: 1px solid var(--hairline); background: transparent; color: var(--body); font-size: 12px; cursor: pointer;"
                                }
                            }
                            on:click={
                                let fv2 = f_val.to_string();
                                move |_| filter.set(fv2.clone())
                            }
                        >
                            {f_val}
                        </button>
                    }
                }).collect_view()}
                <div style="flex: 1;"></div>
                <button
                    style="padding: 4px 12px; border-radius: var(--r-pill); border: 1px solid var(--hairline); background: transparent; color: var(--muted); font-size: 12px; cursor: pointer;"
                    on:click=move |_| ctx.obs_events.set(vec![])
                >
                    "Clear"
                </button>
            </div>

            // Event feed
            <div class="card" style="flex: 1; min-height: 0; overflow-y: auto; padding: 0;">
                // Column headers
                <div style="display: grid; grid-template-columns: 80px 140px 140px 1fr; gap: 0; padding: 6px 12px; border-bottom: 1px solid var(--hairline); font-size: 11px; font-weight: 700; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; position: sticky; top: 0; background: var(--surface-card); z-index: 1;">
                    <div>"Time"</div>
                    <div>"Kind"</div>
                    <div>"ID"</div>
                    <div>"Content"</div>
                </div>
                {move || {
                    let events = filtered_events.get();
                    if events.is_empty() {
                        view! {
                            <div style="padding: 40px; text-align: center; color: var(--muted); font-size: 13px;">
                                "No events yet. Start a chat or run an agent."
                            </div>
                        }.into_any()
                    } else {
                        events.into_iter().map(|ev| {
                            let (badge_bg, badge_color) = match ev.kind.as_str() {
                                k if k == "chat:token" || k == "agent:token" =>
                                    ("rgba(99,102,241,0.12)", "#6366f1"),
                                k if k.starts_with("chat:") =>
                                    ("rgba(59,130,246,0.12)", "#3b82f6"),
                                k if k.contains("tool_call") =>
                                    ("rgba(245,158,11,0.12)", "#f59e0b"),
                                k if k.contains("tool_ok") =>
                                    ("rgba(16,185,129,0.12)", "#10b981"),
                                k if k.contains("error") || k.contains("err") =>
                                    ("rgba(239,68,68,0.12)", "#ef4444"),
                                k if k.starts_with("agent:") =>
                                    ("rgba(139,92,246,0.12)", "#8b5cf6"),
                                _ => ("var(--surface-strong)", "var(--muted)"),
                            };

                            // Format timestamp as HH:MM:SS.mmm
                            let ts_s = {
                                let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ev.ts));
                                format!("{:02}:{:02}:{:02}.{:03}",
                                    d.get_hours() as u32,
                                    d.get_minutes() as u32,
                                    d.get_seconds() as u32,
                                    d.get_milliseconds() as u32,
                                )
                            };

                            let preview = if ev.content.len() > 120 {
                                format!("{}…", &ev.content[..120])
                            } else {
                                ev.content.clone()
                            };

                            view! {
                                <div style="display: grid; grid-template-columns: 80px 140px 140px 1fr; gap: 0; padding: 5px 12px; border-bottom: 1px solid color-mix(in srgb, var(--hairline) 50%, transparent); font-size: 12px; align-items: start;">
                                    <div style="color: var(--muted); font-family: var(--font-mono, monospace); font-size: 10px; padding-top: 2px;">{ts_s}</div>
                                    <div>
                                        <span style=format!("padding: 1px 6px; border-radius: 4px; font-size: 10px; font-weight: 700; background: {}; color: {};", badge_bg, badge_color)>
                                            {ev.kind.clone()}
                                        </span>
                                    </div>
                                    <div style="color: var(--muted); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; padding-top: 2px;">{ev.id.clone()}</div>
                                    <div style="color: var(--body); font-size: 12px; word-break: break-word;">{preview}</div>
                                </div>
                            }
                        }).collect_view().into_any()
                    }
                }}
            </div>
        </div>
    }
}
