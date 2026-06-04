//! Settings: theme presets + appearance + app preferences. All write the shared
//! config (and thus persist + re-theme live). Editing an individual color flips
//! the active theme to a "Custom" palette derived from the `ui_*` fields.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::{Card, PageHeader};
use crate::state::{AppCtx, Tab};
use crate::theme::{self, PRESETS};
use crate::{field_text, field_num};

#[component]
pub fn SettingsTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    // Apply a named preset: set the theme name and seed the ui_* fields from its
    // palette so later custom tweaks start from the preset's look.
    let apply_preset = move |name: &'static str| {
        ctx.update_cfg(move |c| {
            c.theme_name = name.to_string();
            let p = theme::resolve(c);
            c.ui_background_color = p.app_bg.clone();
            c.ui_text_color = p.ink.clone();
            c.ui_accent_color = p.accent.clone();
            c.ui_card_bg = p.canvas.clone();
            c.ui_sidebar_bg = p.sidebar_bg.clone();
            c.ui_border_color = p.hairline.clone();
            c.ui_blur = p.blur > 0;
            c.ui_blur_intensity = if p.blur > 0 { p.blur } else { 30 };
            c.ui_transparency = p.opacity;
        });
    };

    // Color editor: live-preview on input, persist (as Custom) on change.
    let set_color_live = move |field: fn(&mut shared::ServerConfig, String), v: String| {
        ctx.config.update(|c| {
            field(c, v);
            c.theme_name = "Custom".to_string();
        });
    };

    view! {
        <div class="page">
            <PageHeader title="Settings" desc="Theme, appearance, and application preferences."/>

            <Card title="Theme Presets">
                <div class="preset-grid">
                    {PRESETS
                        .iter()
                        .map(|&name| {
                            let active = move || ctx.config.get().theme_name == name;
                            view! {
                                <button
                                    class="preset-card"
                                    class:active=active
                                    on:click=move |_| apply_preset(name)
                                >
                                    <span class=format!("preset-swatch sw-{}", slug(name))></span>
                                    <span class="preset-name">{name}</span>
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </Card>

            <Card title="Appearance (Custom)">
                <div class="color-row">
                    <ColorInput
                        label="Background"
                        value=Signal::derive(move || ctx.config.get().ui_background_color.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_background_color = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Text"
                        value=Signal::derive(move || ctx.config.get().ui_text_color.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_text_color = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Accent"
                        value=Signal::derive(move || ctx.config.get().ui_accent_color.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_accent_color = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                </div>
                <label class="field-label">"Window opacity"</label>
                <input
                    class="slider"
                    type="range"
                    min="0.4"
                    max="1"
                    step="0.02"
                    prop:value=move || ctx.config.get().ui_transparency.to_string()
                    on:input=move |e| {
                        if let Ok(v) = event_target_value(&e).parse::<f32>() {
                            ctx.config.update(|c| c.ui_transparency = v);
                        }
                    }
                    on:change=move |_| ctx.save()
                />
                <label class="field-label">"Blur intensity"</label>
                <input
                    class="slider"
                    type="range"
                    min="0"
                    max="60"
                    step="1"
                    prop:value=move || ctx.config.get().ui_blur_intensity.to_string()
                    on:input=move |e| {
                        if let Ok(v) = event_target_value(&e).parse::<u32>() {
                            ctx.config.update(|c| {
                                c.ui_blur_intensity = v;
                                c.ui_blur = v > 0;
                            });
                        }
                    }
                    on:change=move |_| ctx.save()
                />
            </Card>

            <Card title="Application">
                {field_text!(ctx, searxng_url, "SearXNG URL", "Used by web search / research")}
                {field_text!(ctx, app_log_level, "Log Level", "INFO · DEBUG · WARN · ERROR")}
                {field_num!(ctx, port, u16, "Default Server Port", "")}
            </Card>

            <Card title="Sidebar Navigation & Favorites">
                <div class="form-hint" style="margin-bottom: 12px; font-size: 0.85em; opacity: 0.8;">
                    "Choose which sections appear in the 'Favorites' group at the top of your sidebar for quick access."
                </div>
                <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 10px;">
                    {vec![
                        Tab::Chat, Tab::Planner, Tab::Monitor, Tab::Mcp, Tab::Agents,
                        Tab::Model, Tab::Library, Tab::Download, Tab::Instances,
                        Tab::Server, Tab::Context, Tab::Gpu, Tab::Performance, Tab::Sampling, Tab::Advanced, Tab::Api,
                        Tab::Calendar, Tab::Todos, Tab::QuickNotes, Tab::Compare, Tab::DeepResearch
                    ]
                    .into_iter()
                    .map(|tab| {
                        let is_fav = move || {
                            let t_str = tab.as_str().to_string();
                            ctx.config.get().sidebar_favorites.contains(&t_str)
                        };
                        let toggle_fav = move |_| {
                            ctx.update_cfg(move |c| {
                                let t_str = tab.as_str().to_string();
                                if let Some(pos) = c.sidebar_favorites.iter().position(|x| x == &t_str) {
                                    c.sidebar_favorites.remove(pos);
                                } else {
                                    c.sidebar_favorites.push(t_str);
                                }
                            });
                        };
                        view! {
                            <div style="display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: var(--color-card-bg); border: 1px solid var(--color-border); border-radius: 8px;">
                                <div style="display: flex; align-items: center; gap: 8px;">
                                    <span style="font-size: 14px;">{tab.icon()}</span>
                                    <span style="font-size: 12.5px; font-weight: 500;">{tab.label()}</span>
                                </div>
                                <button
                                    style="background: none; border: none; cursor: pointer; font-size: 16px; color: var(--color-accent);"
                                    on:click=toggle_fav
                                >
                                    {move || if is_fav() { "★" } else { "☆" }}
                                </button>
                            </div>
                        }
                    })
                    .collect_view()}
                </div>
            </Card>

            <Card title="Scan Directories">
                <div class="form-hint" style="margin-bottom: 12px; font-size: 0.85em; opacity: 0.8;">
                    "Specify folders on your computer where llama-manager should look for GGUF model files."
                </div>
                
                <div style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px;">
                    {move || {
                        let dirs = ctx.config.get().model_scan_dirs.clone();
                        if dirs.is_empty() {
                            view! {
                                <div style="font-size: 13px; font-style: italic; opacity: 0.6; padding: 8px;">
                                    "No scan directories configured. Add one below."
                                </div>
                            }.into_any()
                        } else {
                            dirs.into_iter().enumerate().map(|(idx, dir)| {
                                let dir_c = dir.clone();
                                let remove_dir = move |_| {
                                    ctx.update_cfg(move |c| {
                                        if idx < c.model_scan_dirs.len() {
                                            c.model_scan_dirs.remove(idx);
                                        }
                                    });
                                };
                                view! {
                                    <div style="display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: var(--color-card-bg); border: 1px solid var(--color-border); border-radius: 8px; font-size: 13px;">
                                        <span style="font-family: monospace; word-break: break-all; margin-right: 8px;">
                                            {dir_c}
                                        </span>
                                        <button
                                            class="btn"
                                            style="padding: 4px 8px; font-size: 12px; border-radius: 4px; border: 1px solid var(--color-border); background: transparent; cursor: pointer; color: var(--color-accent);"
                                            on:click=remove_dir
                                        >
                                            "Remove"
                                        </button>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>

                <button
                    class="btn"
                    style="padding: 8px 16px; border-radius: 6px; cursor: pointer; border: 1px solid var(--color-border); background: var(--color-accent); color: var(--color-accent-on); font-weight: 500;"
                    on:click=move |_| {
                        spawn_local(async move {
                            match api::pick_path(true).await {
                                Ok(Some(path)) => {
                                    ctx.update_cfg(move |c| {
                                        if !c.model_scan_dirs.contains(&path) {
                                            c.model_scan_dirs.push(path);
                                        }
                                    });
                                }
                                Ok(None) => {}
                                Err(e) => tracing::error!("pick_path failed: {e}"),
                            }
                        });
                    }
                >
                    "Add Directory"
                </button>
            </Card>
        </div>
    }
}

#[component]
fn ColorInput(
    label: &'static str,
    value: Signal<String>,
    on_input: Callback<String>,
    on_commit: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="color-input">
            <label class="field-label">{label}</label>
            <input
                type="color"
                prop:value=move || normalize_hex(value.get())
                on:input=move |e| on_input.run(event_target_value(&e))
                on:change=move |_| on_commit.run(())
            />
        </div>
    }
}

/// `<input type=color>` only accepts `#rrggbb`; fall back to black for
/// rgba()/gradient custom values so the picker still renders.
fn normalize_hex(v: String) -> String {
    if v.starts_with('#') && v.len() == 7 {
        v
    } else {
        "#000000".to_string()
    }
}

fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}
