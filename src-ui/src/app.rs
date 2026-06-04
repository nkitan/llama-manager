//! App root: provides context, hydrates + syncs config, applies the theme, and
//! lays out the custom titlebar, topbar, grouped sidebar, and active tab.

use gloo_timers::callback::Interval;
use leptos::prelude::*;
use shared::ServerConfig;
use shared::ipc::CONFIG_CHANGED_EVENT;
use wasm_bindgen_futures::spawn_local;

use crate::state::{AppCtx, Tab};
use crate::tabs;
use crate::{api, ipc, theme};

#[component]
pub fn App() -> impl IntoView {
    let ctx = AppCtx {
        config: RwSignal::new(ServerConfig::default()),
        active_tab: RwSignal::new(Tab::Chat),
        routed_instance: RwSignal::new(None),
        routed_model: RwSignal::new(None),
        chat_history: RwSignal::new(vec![]),
        immersive: RwSignal::new(false),
        search: RwSignal::new(String::new()),
        server_running: RwSignal::new(false),
        tools_collapsed: RwSignal::new(false),
    };
    provide_context(ctx);

    // Hydrate config + keep in sync with the backend.
    let config = ctx.config;
    spawn_local(async move {
        match api::get_config().await {
            Ok(cfg) => config.set(cfg),
            Err(e) => tracing::error!("get_config failed: {e}"),
        }
    });

    // Hydrate chat history
    let chat_hist = ctx.chat_history;
    spawn_local(async move {
        match api::chat_load_history().await {
            Ok(hist) => chat_hist.set(hist),
            Err(e) => tracing::error!("chat_load_history failed: {e}"),
        }
    });
    ipc::listen::<ServerConfig, _>(CONFIG_CHANGED_EVENT, move |cfg| config.set(cfg));

    // Poll server process status for the topbar pill.
    let running = ctx.server_running;
    let poll = move || {
        spawn_local(async move {
            if let Ok(s) = api::server_status().await {
                running.set(s);
            }
        });
    };
    poll();
    Interval::new(2000, poll).forget();

    view! {
        <div class="app-root" style=move || theme::css_vars(&ctx.config.get())>
            <TitleBar/>
            <div class="app-body">
                {move || (!ctx.immersive.get()).then(|| view! { <Sidebar/> })}
                <div class="main-col">
                    {move || (!ctx.immersive.get()).then(|| view! { <TopBar/> })}
                    <main class="content">{move || render_tab(ctx.active_tab.get())}</main>
                </div>
            </div>
        </div>
    }
}

fn render_tab(tab: Tab) -> AnyView {
    use tabs::*;
    match tab {
        Tab::Chat => view! { <chat::ChatTab/> }.into_any(),
        Tab::Server
        | Tab::Model
        | Tab::Context
        | Tab::Gpu
        | Tab::Performance
        | Tab::Sampling
        | Tab::Advanced
        | Tab::Api => view! { <server::ServerTab/> }.into_any(),
        Tab::Settings => view! { <settings::SettingsTab/> }.into_any(),
        Tab::Todos => view! { <productivity::TodosTab/> }.into_any(),
        Tab::QuickNotes => view! { <productivity::NotesTab/> }.into_any(),
        
        // Remaining tabs
        Tab::Agents => view! { <remaining::AgentsTab/> }.into_any(),
        Tab::Mcp => view! { <remaining::McpTab/> }.into_any(),
        Tab::Planner => view! { <remaining::PlannerTab/> }.into_any(),
        Tab::Calendar => view! { <remaining::CalendarTab/> }.into_any(),
        Tab::Monitor => view! { <remaining::MonitorTab/> }.into_any(),
        Tab::Library => view! { <remaining::LibraryTab/> }.into_any(),
        Tab::Download => view! { <remaining::DownloadTab/> }.into_any(),
        Tab::Compare => view! { <remaining::CompareTab/> }.into_any(),
        Tab::DeepResearch => view! { <remaining::DeepResearchTab/> }.into_any(),
        Tab::Instances => view! { <remaining::InstancesTab/> }.into_any(),

    }
}


/// Custom titlebar (native decorations are off). Draggable; carries the window
/// controls. Themed like the rest of the app.
#[component]
fn TitleBar() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let is_dark_mode = move || ctx.config.get().dark_mode;
    let toggle_dark_mode = move |_| {
        ctx.config.update(|c| {
            c.dark_mode = !c.dark_mode;
        });
        let cfg = ctx.config.get_untracked();
        spawn_local(async move {
            let _ = api::update_config(cfg).await;
        });
    };

    view! {
        <div class="titlebar" data-tauri-drag-region=true>
            <div class="titlebar-brand" data-tauri-drag-region=true>
                <span class="titlebar-logo">"🦙"</span>
                <span class="titlebar-name">"Llama Manager"</span>
            </div>
            <div style="display:flex;align-items:center;gap:6px;">
                <button
                    class="theme-toggle-btn"
                    title=move || if is_dark_mode() { "Switch to Light Mode" } else { "Switch to Dark Mode" }
                    on:click=toggle_dark_mode
                >
                    {move || if is_dark_mode() { "☀️" } else { "🌙" }}
                </button>
                <div class="window-controls">
                    <button
                        class="win-btn min"
                        title="Minimize"
                        on:click=move |_| spawn_local(api::win_minimize())
                    >
                        "—"
                    </button>
                    <button
                        class="win-btn max"
                        title="Maximize"
                        on:click=move |_| spawn_local(api::win_toggle_maximize())
                    >
                        "▢"
                    </button>
                    <button
                        class="win-btn close"
                        title="Close"
                        on:click=move |_| spawn_local(api::win_close())
                    >
                        "✕"
                    </button>
                </div>
            </div>
        </div>
    }
}


#[derive(Clone)]
struct ConfigSearchEntry {
    label: &'static str,
    section: &'static str,
    tab: Tab,
    target_id: &'static str,
}

fn config_search_entries() -> &'static [ConfigSearchEntry] {
    &[
        // Server tab
        ConfigSearchEntry { label: "Host", section: "Server", tab: Tab::Server, target_id: "form-host" },
        ConfigSearchEntry { label: "Port", section: "Server", tab: Tab::Server, target_id: "form-port" },
        ConfigSearchEntry { label: "Timeout (s)", section: "Server", tab: Tab::Server, target_id: "form-timeout" },
        ConfigSearchEntry { label: "HTTP Threads", section: "Server", tab: Tab::Server, target_id: "form-threads_http" },

        // Model tab
        ConfigSearchEntry { label: "llama-server Executable", section: "Model", tab: Tab::Model, target_id: "form-exe_path" },
        ConfigSearchEntry { label: "Model Path (GGUF)", section: "Model", tab: Tab::Model, target_id: "form-model_path" },
        ConfigSearchEntry { label: "Models Directory", section: "Model", tab: Tab::Model, target_id: "form-model_dir" },
        ConfigSearchEntry { label: "Model Alias", section: "Model", tab: Tab::Model, target_id: "form-model_alias" },
        ConfigSearchEntry { label: "HF Repo (HuggingFace)", section: "Model", tab: Tab::Model, target_id: "form-hf_repo" },
        ConfigSearchEntry { label: "HF File (HuggingFace)", section: "Model", tab: Tab::Model, target_id: "form-hf_file" },
        ConfigSearchEntry { label: "HF Token (HuggingFace)", section: "Model", tab: Tab::Model, target_id: "form-hf_token" },
        ConfigSearchEntry { label: "Model URL", section: "Model", tab: Tab::Model, target_id: "form-model_url" },
        ConfigSearchEntry { label: "Chat Template", section: "Model", tab: Tab::Model, target_id: "form-chat_template" },
        ConfigSearchEntry { label: "System Prompt", section: "Model", tab: Tab::Model, target_id: "form-system_prompt" },

        // Context tab
        ConfigSearchEntry { label: "Context Size", section: "Context", tab: Tab::Context, target_id: "form-ctx_size" },
        ConfigSearchEntry { label: "Max Predict Tokens", section: "Context", tab: Tab::Context, target_id: "form-predict" },
        ConfigSearchEntry { label: "Batch Size", section: "Context", tab: Tab::Context, target_id: "form-batch_size" },
        ConfigSearchEntry { label: "Micro-batch Size", section: "Context", tab: Tab::Context, target_id: "form-ubatch_size" },
        ConfigSearchEntry { label: "Parallel Sequences", section: "Context", tab: Tab::Context, target_id: "form-parallel" },
        ConfigSearchEntry { label: "Continuous Batching", section: "Context", tab: Tab::Context, target_id: "form-cont_batching" },

        // GPU tab
        ConfigSearchEntry { label: "GPU Layers Offload", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-gpu_layers" },
        ConfigSearchEntry { label: "Split Mode", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-split_mode" },
        ConfigSearchEntry { label: "Tensor Split", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-tensor_split" },
        ConfigSearchEntry { label: "Main GPU ID", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-main_gpu" },
        ConfigSearchEntry { label: "Auto-fit VRAM", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-fit" },
        ConfigSearchEntry { label: "mlock RAM lock", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-mlock" },
        ConfigSearchEntry { label: "No mmap mapping", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-no_mmap" },
        ConfigSearchEntry { label: "No KV Offload", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-no_kv_offload" },
        ConfigSearchEntry { label: "KV Cache Type (K)", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-cache_type_k" },
        ConfigSearchEntry { label: "KV Cache Type (V)", section: "GPU & Memory", tab: Tab::Gpu, target_id: "form-cache_type_v" },

        // Performance tab
        ConfigSearchEntry { label: "Threads", section: "Performance", tab: Tab::Performance, target_id: "form-threads" },
        ConfigSearchEntry { label: "Batch Threads", section: "Performance", tab: Tab::Performance, target_id: "form-threads_batch" },
        ConfigSearchEntry { label: "Flash Attention", section: "Performance", tab: Tab::Performance, target_id: "form-flash_attn" },
        ConfigSearchEntry { label: "No Warmup", section: "Performance", tab: Tab::Performance, target_id: "form-no_warmup" },
        ConfigSearchEntry { label: "Check Tensors", section: "Performance", tab: Tab::Performance, target_id: "form-check_tensors" },

        // Sampling tab
        ConfigSearchEntry { label: "Temperature", section: "Sampling", tab: Tab::Sampling, target_id: "form-temp" },
        ConfigSearchEntry { label: "Top-K", section: "Sampling", tab: Tab::Sampling, target_id: "form-top_k" },
        ConfigSearchEntry { label: "Top-P", section: "Sampling", tab: Tab::Sampling, target_id: "form-top_p" },
        ConfigSearchEntry { label: "Min-P", section: "Sampling", tab: Tab::Sampling, target_id: "form-min_p" },
        ConfigSearchEntry { label: "Seed", section: "Sampling", tab: Tab::Sampling, target_id: "form-seed" },
        ConfigSearchEntry { label: "Repeat Penalty", section: "Sampling", tab: Tab::Sampling, target_id: "form-repeat_penalty" },
        ConfigSearchEntry { label: "Presence Penalty", section: "Sampling", tab: Tab::Sampling, target_id: "form-presence_penalty" },
        ConfigSearchEntry { label: "Frequency Penalty", section: "Sampling", tab: Tab::Sampling, target_id: "form-frequency_penalty" },
        ConfigSearchEntry { label: "Grammar GBNF", section: "Sampling", tab: Tab::Sampling, target_id: "form-grammar" },
        ConfigSearchEntry { label: "Grammar File", section: "Sampling", tab: Tab::Sampling, target_id: "form-grammar_file" },

        // Advanced tab
        ConfigSearchEntry { label: "RoPE Scaling", section: "Advanced", tab: Tab::Advanced, target_id: "form-rope_scaling" },
        ConfigSearchEntry { label: "RoPE Freq Base", section: "Advanced", tab: Tab::Advanced, target_id: "form-rope_freq_base" },
        ConfigSearchEntry { label: "RoPE Freq Scale", section: "Advanced", tab: Tab::Advanced, target_id: "form-rope_freq_scale" },
        ConfigSearchEntry { label: "YaRN Orig Context", section: "Advanced", tab: Tab::Advanced, target_id: "form-yarn_orig_ctx" },
        ConfigSearchEntry { label: "YaRN Ext Factor", section: "Advanced", tab: Tab::Advanced, target_id: "form-yarn_ext_factor" },
        ConfigSearchEntry { label: "YaRN Attn Factor", section: "Advanced", tab: Tab::Advanced, target_id: "form-yarn_attn_factor" },
        ConfigSearchEntry { label: "Draft Model Path", section: "Advanced", tab: Tab::Advanced, target_id: "form-draft_model" },
        ConfigSearchEntry { label: "Draft GPU Layers", section: "Advanced", tab: Tab::Advanced, target_id: "form-draft_gpu_layers" },
        ConfigSearchEntry { label: "Draft Tokens", section: "Advanced", tab: Tab::Advanced, target_id: "form-draft_tokens" },

        // API tab
        ConfigSearchEntry { label: "API Key", section: "API & Output", tab: Tab::Api, target_id: "form-api_key" },
        ConfigSearchEntry { label: "API Key File", section: "API & Output", tab: Tab::Api, target_id: "form-api_key_file" },
        ConfigSearchEntry { label: "Metrics Endpoint", section: "API & Output", tab: Tab::Api, target_id: "form-metrics" },
        ConfigSearchEntry { label: "Slots Endpoint", section: "API & Output", tab: Tab::Api, target_id: "form-slots" },
        ConfigSearchEntry { label: "Slot Save Path", section: "API & Output", tab: Tab::Api, target_id: "form-slot_save_path" },
        ConfigSearchEntry { label: "Embedding Mode", section: "API & Output", tab: Tab::Api, target_id: "form-embedding" },
        ConfigSearchEntry { label: "Pooling Type", section: "API & Output", tab: Tab::Api, target_id: "form-pooling" },
        ConfigSearchEntry { label: "Log Format", section: "API & Output", tab: Tab::Api, target_id: "form-log_format" },
        ConfigSearchEntry { label: "Verbose Output", section: "API & Output", tab: Tab::Api, target_id: "form-verbose" },

        // Settings tab
        ConfigSearchEntry { label: "SearXNG URL", section: "Settings", tab: Tab::Settings, target_id: "form-searxng_url" },
        ConfigSearchEntry { label: "Log Level", section: "Settings", tab: Tab::Settings, target_id: "form-app_log_level" },
        ConfigSearchEntry { label: "Default Server Port", section: "Settings", tab: Tab::Settings, target_id: "form-port" },
    ]
}

/// Topbar: active-tab title, global search, and a server status pill.
#[component]
fn TopBar() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    view! {
        <header class="topbar">
            <div class="topbar-title">
                <span class="topbar-icon">{move || ctx.active_tab.get().icon()}</span>
                <span>{move || ctx.active_tab.get().label()}</span>
            </div>
            <div style="position: relative; margin-left: auto; max-width: 280px; width: 100%;">
                <input
                    class="topbar-search input"
                    type="text"
                    placeholder="Search settings…"
                    prop:value=move || ctx.search.get()
                    on:input=move |e| ctx.search.set(event_target_value(&e))
                />
                {move || {
                    let q = ctx.search.get().trim().to_lowercase();
                    if q.is_empty() {
                        view! {}.into_any()
                    } else {
                        let filtered: Vec<_> = config_search_entries().iter()
                            .filter(|e| e.label.to_lowercase().contains(&q) || e.section.to_lowercase().contains(&q))
                            .cloned()
                            .take(8)
                            .collect();
                        if filtered.is_empty() {
                            view! {}.into_any()
                        } else {
                            view! {
                                <div class="search-dropdown-overlay" style="position: absolute; top: 44px; right: 0; width: 280px; background: var(--surface-card); border: 1px solid var(--hairline); border-radius: var(--r-md); box-shadow: 0 10px 25px -5px rgba(0,0,0,0.3); z-index: 1000; padding: 6px; display: flex; flex-direction: column; gap: 2px; -webkit-backdrop-filter: blur(var(--blur, 0px)); backdrop-filter: blur(var(--blur, 0px));">
                                    {filtered.into_iter().map(|entry| {
                                        let entry_c = entry.clone();
                                        view! {
                                            <button 
                                                style="display: flex; flex-direction: column; align-items: flex-start; width: 100%; border: none; background: transparent; padding: 6px 10px; border-radius: var(--r-sm); cursor: pointer; text-align: left; transition: background 0.15s;"
                                                class="search-result-btn"
                                                on:click=move |_| {
                                                    let tgt = entry_c.target_id;
                                                    ctx.active_tab.set(entry_c.tab);
                                                    ctx.search.set(String::new());
                                                    if !tgt.is_empty() {
                                                        let js = format!(
                                                            "setTimeout(() => {{ \
                                                                const el = document.getElementById('{}'); \
                                                                if (el) {{ \
                                                                    el.focus(); \
                                                                    el.scrollIntoView({{ behavior: 'smooth', block: 'center' }}); \
                                                                    el.style.outline = '3px solid var(--accent)'; \
                                                                    el.style.outlineOffset = '2px'; \
                                                                    setTimeout(() => el.style.outline = '', 2000); \
                                                                }} \
                                                            }}, 200);",
                                                            tgt
                                                        );
                                                        let _ = js_sys::eval(&js);
                                                    }
                                                }
                                            >
                                                <span style="font-size: 13px; font-weight: 600; color: var(--ink);">{entry.label}</span>
                                                <span style="font-size: 11px; color: var(--muted); margin-top: 2px;">{entry.section}</span>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </div>
            <button
                class="status-pill"
                class:online=move || ctx.server_running.get()
                on:click=move |_| ctx.active_tab.set(Tab::Server)
            >
                <span class="status-dot"></span>
                {move || if ctx.server_running.get() { "Server running" } else { "Server stopped" }}
            </button>
        </header>
    }
}

/// Grouped, searchable, and collapsible sidebar navigation with favorites.
#[component]
fn Sidebar() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    let favorites_expanded = RwSignal::new(true);
    let general_expanded = RwSignal::new(true);
    let models_expanded = RwSignal::new(true);
    let server_expanded = RwSignal::new(true);
    let productivity_expanded = RwSignal::new(true);

    let render_sidebar_item = move |tab: Tab, is_sub: bool| {
        let is_fav = move || {
            let t_str = tab.as_str().to_string();
            ctx.config.get().sidebar_favorites.contains(&t_str)
        };
        let toggle_fav = move |e: leptos::ev::MouseEvent| {
            e.stop_propagation();
            ctx.update_cfg(move |c| {
                let t_str = tab.as_str().to_string();
                if let Some(pos) = c.sidebar_favorites.iter().position(|x| x == &t_str) {
                    c.sidebar_favorites.remove(pos);
                } else {
                    c.sidebar_favorites.push(t_str);
                }
            });
        };
        let active = move || {
            let current = ctx.active_tab.get();
            current == tab
        };

        view! {
            <button
                class="nav-item"
                class:sidebar-sub-item=is_sub
                class:active=active
                on:click=move |_| ctx.active_tab.set(tab)
            >
                <span class="nav-icon">{tab.icon()}</span>
                <span>{tab.label()}</span>
                <span style="flex: 1;"></span>
                <button
                    class="sidebar-fav-btn"
                    class:sidebar-fav-btn-active=is_fav
                    on:click=toggle_fav
                >
                    {move || if is_fav() { "★" } else { "☆" }}
                </button>
            </button>
        }
    };

    view! {
        <aside class="sidebar">
            <nav class="nav">
                {move || {
                    let filter_tabs = move |tabs: &[Tab]| -> Vec<Tab> {
                        tabs.to_vec()
                    };

                    view! {
                        <div>
                            // ── 1. Favorites ───────────────────────────────────────────
                            {move || {
                                let fav_strs = ctx.config.get().sidebar_favorites.clone();
                                let fav_tabs: Vec<Tab> = fav_strs.iter()
                                    .filter_map(|s| Tab::from_str(s))
                                    .collect();
                                let filtered = filter_tabs(&fav_tabs);

                                (!filtered.is_empty()).then(|| {
                                    let arrow = move || if favorites_expanded.get() { "▼" } else { "▶" };
                                    let toggle = move |_| favorites_expanded.set(!favorites_expanded.get());
                                    view! {
                                        <div class="nav-group">
                                            <div class="nav-group-title interactive" on:click=toggle>
                                                <span class="section-arrow">{arrow}</span>
                                                <span>"★ Favorites"</span>
                                            </div>
                                            {move || (favorites_expanded.get()).then(|| {
                                                filtered.clone().into_iter()
                                                    .map(|tab| render_sidebar_item(tab, false))
                                                    .collect_view()
                                            })}
                                        </div>
                                    }
                                })
                            }}

                            // ── 2. General ──────────────────────────────────────────────
                            {move || {
                                let tabs = &[Tab::Chat, Tab::Planner, Tab::Monitor, Tab::Mcp, Tab::Agents];
                                let filtered = filter_tabs(tabs);
                                (!filtered.is_empty()).then(|| {
                                    let arrow = move || if general_expanded.get() { "▼" } else { "▶" };
                                    let toggle = move |_| general_expanded.set(!general_expanded.get());
                                    view! {
                                        <div class="nav-group">
                                            <div class="nav-group-title interactive" on:click=toggle>
                                                <span class="section-arrow">{arrow}</span>
                                                <span>"General"</span>
                                            </div>
                                            {move || (general_expanded.get()).then(|| {
                                                filtered.clone().into_iter()
                                                    .map(|tab| render_sidebar_item(tab, false))
                                                    .collect_view()
                                            })}
                                        </div>
                                    }
                                })
                            }}

                            // ── 3. Productivity & Research ─────────────────────────────
                            {move || {
                                let tabs = &[Tab::Calendar, Tab::Todos, Tab::QuickNotes, Tab::Compare, Tab::DeepResearch];
                                let filtered = filter_tabs(tabs);
                                (!filtered.is_empty()).then(|| {
                                    let arrow = move || if productivity_expanded.get() { "▼" } else { "▶" };
                                    let toggle = move |_| productivity_expanded.set(!productivity_expanded.get());
                                    view! {
                                        <div class="nav-group">
                                            <div class="nav-group-title interactive" on:click=toggle>
                                                <span class="section-arrow">{arrow}</span>
                                                <span>"Productivity & Research"</span>
                                            </div>
                                            {move || (productivity_expanded.get()).then(|| {
                                                filtered.clone().into_iter()
                                                    .map(|tab| render_sidebar_item(tab, false))
                                                    .collect_view()
                                            })}
                                        </div>
                                    }
                                })
                            }}

                            // ── 4. Models Library ──────────────────────────────────
                            {move || {
                                let tabs = &[Tab::Library, Tab::Download];
                                let filtered = filter_tabs(tabs);
                                (!filtered.is_empty()).then(|| {
                                    let arrow = move || if models_expanded.get() { "▼" } else { "▶" };
                                    let toggle = move |_| models_expanded.set(!models_expanded.get());
                                    view! {
                                        <div class="nav-group">
                                            <div class="nav-group-title interactive" on:click=toggle>
                                                <span class="section-arrow">{arrow}</span>
                                                <span>"Models Library"</span>
                                            </div>
                                            {move || (models_expanded.get()).then(|| {
                                                filtered.clone().into_iter()
                                                    .map(|tab| render_sidebar_item(tab, false))
                                                    .collect_view()
                                            })}
                                        </div>
                                    }
                                })
                            }}

                            // ── 5. Llama-Manager ────────────────────────────────────────
                            {move || {
                                let tabs = &[
                                    (Tab::Server, false),
                                    (Tab::Instances, false),
                                    (Tab::Model, true),
                                    (Tab::Context, true),
                                    (Tab::Gpu, true),
                                    (Tab::Performance, true),
                                    (Tab::Sampling, true),
                                    (Tab::Advanced, true),
                                    (Tab::Api, true),
                                ];
                                let filtered: Vec<(Tab, bool)> = tabs.iter()
                                    .filter(|(tab, _)| filter_tabs(&[*tab]).len() > 0)
                                    .copied()
                                    .collect();
                                (!filtered.is_empty()).then(|| {
                                    let arrow = move || if server_expanded.get() { "▼" } else { "▶" };
                                    let toggle = move |_| server_expanded.set(!server_expanded.get());
                                    view! {
                                        <div class="nav-group">
                                            <div class="nav-group-title interactive" on:click=toggle>
                                                <span class="section-arrow">{arrow}</span>
                                                <span>"Llama-Manager"</span>
                                            </div>
                                            {move || (server_expanded.get()).then(|| {
                                                filtered.clone().into_iter()
                                                    .map(|(tab, is_sub)| render_sidebar_item(tab, is_sub))
                                                    .collect_view()
                                            })}
                                        </div>
                                    }
                                })
                            }}
                        </div>
                    }
                }}
            </nav>

            // Settings button at the bottom of the sidebar.
            {move || {
                let active = move || ctx.active_tab.get() == Tab::Settings;
                view! {
                    <div style="padding: var(--s-sm) var(--s-md); border-top: 1px solid var(--hairline);">
                        <button
                            class="nav-item"
                            class:active=active
                            on:click=move |_| ctx.active_tab.set(Tab::Settings)
                        >
                            <span class="nav-icon">{Tab::Settings.icon()}</span>
                            <span>{Tab::Settings.label()}</span>
                        </button>
                    </div>
                }
            }}

            <div class="sidebar-foot">"Tauri v2 · Leptos"</div>
        </aside>
    }
}
