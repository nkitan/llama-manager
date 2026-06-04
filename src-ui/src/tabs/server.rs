//! Server tab: network config, start/stop the llama-server process, the resolved
//! command line, and a live log console fed by `server://log` events.

use leptos::prelude::*;
use shared::ipc::{SERVER_LOG_EVENT, OptimizationSuggestion};
use wasm_bindgen_futures::spawn_local;

use crate::components::{Card, PageHeader};
use crate::state::{AppCtx, Tab};
use crate::tabs::config_tabs;
use crate::{api, field_num, field_text, ipc};

#[component]
pub fn ServerTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let logs = RwSignal::new(Vec::<String>::new());
    let error = RwSignal::new(None::<String>);
    let running = ctx.server_running;
    let suggestions = RwSignal::new(Vec::<OptimizationSuggestion>::new());
    let analyzed = RwSignal::new(false);

    // Stream llama-server output (bounded ring buffer).
    ipc::listen::<String, _>(SERVER_LOG_EVENT, move |line| {
        logs.update(|l| {
            l.push(line);
            if l.len() > 500 {
                let extra = l.len() - 500;
                l.drain(0..extra);
            }
        });
    });

    let start = move |_| {
        error.set(None);
        spawn_local(async move {
            if let Err(e) = api::server_start().await {
                error.set(Some(e));
            }
        });
    };
    let stop = move |_| {
        spawn_local(async move {
            let _ = api::server_stop().await;
        });
    };

    let analyze = move || {
        let cfg = ctx.config.get_untracked();
        spawn_local(async move {
            match api::server_suggest_optimizations(cfg).await {
                Ok(sugs) => {
                    suggestions.set(sugs);
                    analyzed.set(true);
                }
                Err(e) => {
                    tracing::error!("server_suggest_optimizations failed: {:?}", e);
                }
            }
        });
    };

    analyze();

    let apply_selected = move |_| {
        let selected_items = suggestions.get_untracked()
            .into_iter()
            .filter(|s| s.selected)
            .collect::<Vec<_>>();
        
        if !selected_items.is_empty() {
            ctx.update_cfg(|c| {
                for s in selected_items {
                    apply_optimization(c, &s.key, &s.recommended);
                }
            });
            // Re-run analysis
            analyze();
        }
    };

    let apply_all = move |_| {
        let all_items = suggestions.get_untracked();
        if !all_items.is_empty() {
            ctx.update_cfg(|c| {
                for s in all_items {
                    apply_optimization(c, &s.key, &s.recommended);
                }
            });
            // Re-run analysis
            analyze();
        }
    };

    let sub_tabs = vec![
        (Tab::Server, "Status & Control", "🖥"),
        (Tab::Model, "Model", "🦙"),
        (Tab::Context, "Context", "📐"),
        (Tab::Gpu, "GPU & Memory", "🎛"),
        (Tab::Performance, "Performance", "⚡"),
        (Tab::Sampling, "Sampling", "🎲"),
        (Tab::Advanced, "Advanced", "⚙"),
        (Tab::Api, "API & Output", "🔑"),
    ];

    view! {
        <div class="page">
            <PageHeader title="Server" desc="Configure launch parameters and manage the llama-server process."/>

            <div class="sub-tab-bar">
                {sub_tabs.into_iter().map(|(t, label, icon)| {
                    let active = move || ctx.active_tab.get() == t;
                    view! {
                        <button
                            class="sub-tab-btn"
                            class:active=active
                            on:click=move |_| ctx.active_tab.set(t)
                        >
                            <span>{icon}</span>
                            <span>{label}</span>
                        </button>
                    }
                }).collect_view()}
            </div>

            {move || match ctx.active_tab.get() {
                Tab::Server => view! {
                    <div style="display: flex; flex-direction: column; gap: 16px;">
                        <Card title="Network">
                            <div class="fields-grid">
                                {field_text!(ctx, host, "Host", "Use 0.0.0.0 to expose on the network")}
                                {field_num!(ctx, port, u16, "Port", "")}
                                {field_num!(ctx, timeout, u32, "Timeout (s)", "0 = no timeout")}
                                {field_num!(ctx, threads_http, u32, "HTTP Threads", "0 = auto")}
                            </div>
                        </Card>

                        <Card title="Launch Optimizer">
                            <div class="field-hint" style="margin-bottom: 12px;">
                                "Analyze your current llama-server launch settings and get reviewable optimization suggestions."
                            </div>
                            <div class="row-actions">
                                <button class="btn primary" on:click=move |_| analyze()>"Analyze Launch Parameters"</button>
                                <button class="btn secondary"
                                    prop:disabled=move || suggestions.get().iter().all(|s| !s.selected)
                                    on:click=apply_selected
                                >
                                    "Apply Selected"
                                </button>
                                <button class="btn secondary"
                                    prop:disabled=move || suggestions.get().is_empty()
                                    on:click=apply_all
                                >
                                    "Apply All"
                                </button>
                            </div>
                            {move || {
                                let list = suggestions.get();
                                let has_run = analyzed.get();
                                if list.is_empty() {
                                    if has_run {
                                        view! {
                                            <div class="toast success" style="margin-top: 12px; display: flex; align-items: center; gap: 8px;">
                                                <span>"✓ Your launch settings are fully optimized! No suggestions found."</span>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="field-hint" style="margin-top: 12px;">
                                                "Click Analyze to see suggested launch parameter optimizations."
                                            </div>
                                        }.into_any()
                                    }
                                } else {
                                    view! {
                                        <div style="display: grid; gap: 12px; margin-top: 16px;">
                                            {list.into_iter().enumerate().map(|(idx, s)| {
                                                let is_selected = s.selected;
                                                let label = s.label.clone();
                                                let reason = s.reason.clone();
                                                let current = s.current.clone();
                                                let recommended = s.recommended.clone();
                                                view! {
                                                    <div class="todo-item" style="flex-direction: column; align-items: stretch; gap: 8px; padding: 14px; background: var(--surface-soft);">
                                                        <div style="display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;">
                                                            <div>
                                                                <div style="font-weight: 600; color: var(--ink);">{label}</div>
                                                                <div class="field-hint" style="margin-top: 4px;">{reason}</div>
                                                            </div>
                                                            <button
                                                                class="btn sm"
                                                                class:primary=move || is_selected
                                                                class:secondary=move || !is_selected
                                                                on:click=move |_| {
                                                                    suggestions.update(|items| {
                                                                        if let Some(item) = items.get_mut(idx) {
                                                                            item.selected = !item.selected;
                                                                        }
                                                                    });
                                                                }
                                                            >
                                                                {if is_selected { "Selected" } else { "Select" }}
                                                            </button>
                                                        </div>
                                                        <div style="display: flex; gap: 12px; font-size: 12.5px; color: var(--muted); margin-top: 4px;">
                                                            <span>{format!("Current: {}", current)}</span>
                                                            <span>{format!("Recommended: {}", recommended)}</span>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </Card>

                        <Card title="Process">
                            <div class="row-actions">
                                <button class="btn primary" prop:disabled=move || running.get() on:click=start>
                                    "Start Server"
                                </button>
                                <button class="btn danger" prop:disabled=move || !running.get() on:click=stop>
                                    "Stop Server"
                                </button>
                                <span class="status-pill" class:online=move || running.get()>
                                    <span class="status-dot"></span>
                                    {move || if running.get() { "running" } else { "stopped" }}
                                </span>
                            </div>
                            {move || error.get().map(|e| view! { <div class="toast error">"❌ "{e}</div> })}
                            <div class="cmd-preview">
                                <code>{move || ctx.config.get().command_preview()}</code>
                            </div>
                        </Card>

                        <Card title="Logs">
                            <div class="log-console">
                                {move || {
                                    if logs.get().is_empty() {
                                        view! { <div class="log-empty">"No output yet. Start the server."</div> }
                                            .into_any()
                                    } else {
                                        logs.get()
                                            .into_iter()
                                            .map(|l| view! { <div class="log-line">{l}</div> })
                                            .collect_view()
                                            .into_any()
                                    }
                                }}
                            </div>
                        </Card>
                    </div>
                }.into_any(),
                Tab::Model => view! { <config_tabs::ModelTab/> }.into_any(),
                Tab::Context => view! { <config_tabs::ContextTab/> }.into_any(),
                Tab::Gpu => view! { <config_tabs::GpuTab/> }.into_any(),
                Tab::Performance => view! { <config_tabs::PerformanceTab/> }.into_any(),
                Tab::Sampling => view! { <config_tabs::SamplingTab/> }.into_any(),
                Tab::Advanced => view! { <config_tabs::AdvancedTab/> }.into_any(),
                Tab::Api => view! { <config_tabs::ApiTab/> }.into_any(),
                _ => view! {}.into_any()
            }}
        </div>
    }
}

fn apply_optimization(cfg: &mut shared::ServerConfig, key: &str, value: &str) {
    match key {
        "threads" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.threads = v;
            }
        }
        "threads_batch" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.threads_batch = v;
            }
        }
        "threads_http" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.threads_http = v;
            }
        }
        "cont_batching" => {
            cfg.cont_batching = value.eq_ignore_ascii_case("on")
                || value.eq_ignore_ascii_case("enabled")
                || value.eq_ignore_ascii_case("true");
        }
        "fit" => {
            cfg.fit = value.eq_ignore_ascii_case("on")
                || value.eq_ignore_ascii_case("enabled")
                || value.eq_ignore_ascii_case("true");
        }
        "flash_attn" => {
            cfg.flash_attn = value.eq_ignore_ascii_case("on")
                || value.eq_ignore_ascii_case("enabled")
                || value.eq_ignore_ascii_case("true");
        }
        "parallel" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.parallel = v;
            }
        }
        "ubatch_size" => {
            if let Ok(v) = value.parse::<u32>() {
                cfg.ubatch_size = v;
            }
        }
        "kv_cache_quant" => {
            cfg.cache_type_k = shared::config::CacheType::from_str(value);
            cfg.cache_type_v = shared::config::CacheType::from_str(value);
        }
        "draft_gpu_layers" => {
            if let Ok(v) = value.parse::<i32>() {
                cfg.draft_gpu_layers = v;
            }
        }
        "no_warmup" => {
            cfg.no_warmup = value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false");
        }
        "no_mmap" => {
            cfg.no_mmap = value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false");
        }
        _ => {}
    }
}

