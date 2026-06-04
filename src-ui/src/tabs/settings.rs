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
    let new_theme_name = RwSignal::new(String::new());

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
            c.ui_radius_sm = p.radius_sm.clone();
            c.ui_radius_md = p.radius_md.clone();
            c.ui_radius_lg = p.radius_lg.clone();
            c.ui_radius_xl = p.radius_xl.clone();
            c.ui_border_width = p.border_width.clone();
            c.ui_size_xs = p.size_xs.clone();
            c.ui_size_sm = p.size_sm.clone();
            c.ui_size_md = p.size_md.clone();
            c.ui_size_lg = p.size_lg.clone();
            c.ui_size_xl = p.size_xl.clone();
            c.ui_button_bg = p.button_bg.clone();
            c.ui_button_text = p.button_text.clone();
            c.ui_card_text = p.card_text.clone();
        });
    };

    // Color editor: live-preview on input, persist (as Custom) on change.
    let set_color_live = move |field: fn(&mut shared::ServerConfig, String), v: String| {
        ctx.config.update(|c| {
            field(c, v);
            c.theme_name = "Custom".to_string();
        });
    };

    let field_theme_text = {
        let ctx = ctx.clone();
        move |label: &'static str, get_val: fn(&shared::ServerConfig) -> String, set_val: fn(&mut shared::ServerConfig, String)| {
            let val = Signal::derive(move || get_val(&ctx.config.get()));
            view! {
                <div class="field" style="margin-bottom: 12px; flex: 1 1 140px; min-width: 120px;">
                    <label class="field-label" style="font-size: 11.5px; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px;">{label}</label>
                    <input
                        class="input"
                        style="height: 32px; font-size: 13px; padding: 0 8px; border-radius: var(--r-sm); border: var(--border-width) solid var(--hairline);"
                        type="text"
                        prop:value=move || val.get()
                        on:input=move |e| {
                            let v = event_target_value(&e);
                            ctx.config.update(|c| {
                                set_val(c, v);
                                c.theme_name = "Custom".to_string();
                            });
                        }
                        on:change=move |_| ctx.save()
                    />
                </div>
            }
        }
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
                <div style="font-size: 12px; color: var(--muted); margin-bottom: 16px;">
                    "Fully customize the interface colors, font, corners, borders, and spacing tokens."
                </div>
                
                <h4 style="font-size: 13px; font-weight: 600; color: var(--ink); margin-bottom: 8px; border-bottom: var(--border-width) solid var(--hairline); padding-bottom: 6px;">"Dark Mode Colors"</h4>
                <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 16px; margin-bottom: 24px;">
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
                    <ColorInput
                        label="Card Background"
                        value=Signal::derive(move || ctx.config.get().ui_card_bg.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_card_bg = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Sidebar Background"
                        value=Signal::derive(move || ctx.config.get().ui_sidebar_bg.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_sidebar_bg = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Border Color"
                        value=Signal::derive(move || ctx.config.get().ui_border_color.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_border_color = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Button Background"
                        value=Signal::derive(move || ctx.config.get().ui_button_bg.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_button_bg = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Button Text"
                        value=Signal::derive(move || ctx.config.get().ui_button_text.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_button_text = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Card Text"
                        value=Signal::derive(move || ctx.config.get().ui_card_text.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_card_text = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                </div>

                <h4 style="font-size: 13px; font-weight: 600; color: var(--ink); margin-bottom: 8px; border-bottom: var(--border-width) solid var(--hairline); padding-bottom: 6px;">"Light Mode Colors"</h4>
                <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 16px; margin-bottom: 24px;">
                    <ColorInput
                        label="Background"
                        value=Signal::derive(move || ctx.config.get().ui_light_background_color.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_light_background_color = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Text"
                        value=Signal::derive(move || ctx.config.get().ui_light_text_color.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_light_text_color = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Accent"
                        value=Signal::derive(move || ctx.config.get().ui_light_accent_color.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_light_accent_color = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Card Background"
                        value=Signal::derive(move || ctx.config.get().ui_light_card_bg.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_light_card_bg = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Sidebar Background"
                        value=Signal::derive(move || ctx.config.get().ui_light_sidebar_bg.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_light_sidebar_bg = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Border Color"
                        value=Signal::derive(move || ctx.config.get().ui_light_border_color.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_light_border_color = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Button Background"
                        value=Signal::derive(move || ctx.config.get().ui_light_button_bg.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_light_button_bg = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Button Text"
                        value=Signal::derive(move || ctx.config.get().ui_light_button_text.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_light_button_text = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                    <ColorInput
                        label="Card Text"
                        value=Signal::derive(move || ctx.config.get().ui_light_card_text.clone())
                        on_input=Callback::new(move |v| set_color_live(|c, v| c.ui_light_card_text = v, v))
                        on_commit=Callback::new(move |_| ctx.save())
                    />
                </div>

                <h4 style="font-size: 13px; font-weight: 600; color: var(--ink); margin-bottom: 12px; border-bottom: var(--border-width) solid var(--hairline); padding-bottom: 6px;">"Typography & Borders"</h4>
                <div style="display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 24px; align-items: flex-end;">
                    // Font Family dropdown
                    <div class="field" style="margin-bottom: 0; flex: 1 1 180px; min-width: 160px;">
                        <label class="field-label" style="font-size: 11.5px; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px;">"Font Family"</label>
                        <select
                            class="input"
                            style="height: 32px; font-size: 13px; border-radius: var(--r-sm); border: var(--border-width) solid var(--hairline);"
                            prop:value=move || ctx.config.get().ui_font_family.clone()
                            on:change=move |e| {
                                let v = event_target_value(&e);
                                ctx.config.update(|c| {
                                    c.ui_font_family = v;
                                    c.theme_name = "Custom".to_string();
                                });
                                ctx.save();
                            }
                        >
                            <option value="Inter">"Inter"</option>
                            <option value="Geist">"Geist"</option>
                            <option value="Roboto">"Roboto"</option>
                            <option value="Source Sans 3">"Source Sans 3"</option>
                            <option value="Nunito">"Nunito"</option>
                            <option value="IBM Plex Sans">"IBM Plex Sans"</option>
                            <option value="DM Sans">"DM Sans"</option>
                            <option value="Outfit">"Outfit"</option>
                            <option value="system-ui">"System UI"</option>
                            <option value="JetBrains Mono">"JetBrains Mono (mono)"</option>
                        </select>
                    </div>
                    // Border Width dropdown
                    <div class="field" style="margin-bottom: 0; flex: 1 1 140px; min-width: 120px;">
                        <label class="field-label" style="font-size: 11.5px; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px;">"Border Width"</label>
                        <select
                            class="input"
                            style="height: 32px; font-size: 13px; border-radius: var(--r-sm); border: var(--border-width) solid var(--hairline);"
                            prop:value=move || ctx.config.get().ui_border_width.clone()
                            on:change=move |e| {
                                let v = event_target_value(&e);
                                ctx.config.update(|c| {
                                    c.ui_border_width = v;
                                    c.theme_name = "Custom".to_string();
                                });
                                ctx.save();
                            }
                        >
                            <option value="0px">"None (0px)"</option>
                            <option value="0.5px">"Hairline (0.5px)"</option>
                            <option value="1px">"Normal (1px)"</option>
                            <option value="1.5px">"Medium (1.5px)"</option>
                            <option value="2px">"Bold (2px)"</option>
                        </select>
                    </div>
                </div>

                <h4 style="font-size: 13px; font-weight: 600; color: var(--ink); margin-bottom: 12px; border-bottom: var(--border-width) solid var(--hairline); padding-bottom: 6px;">"Corner Style (Radius)"</h4>
                <div style="margin-bottom: 16px;">
                    <div style="display: flex; gap: 10px; flex-wrap: wrap; margin-bottom: 16px;">
                        {[("Sharp", "2px","4px","6px","8px"), ("Rounded", "4px","8px","12px","16px"), ("Soft", "6px","10px","16px","20px"), ("Pill", "8px","16px","24px","9999px")].into_iter().map(|(label, sm, md, lg, xl)| {
                            let active = move || {
                                let c = ctx.config.get();
                                c.ui_radius_sm == sm && c.ui_radius_md == md
                            };
                            view! {
                                <button
                                    style=move || if active() {
                                        "padding: 6px 14px; border: 2px solid var(--primary); border-radius: var(--r-md); background: color-mix(in srgb, var(--primary) 15%, transparent); color: var(--primary); font-size: 13px; font-weight: 600; cursor: pointer;"
                                    } else {
                                        "padding: 6px 14px; border: var(--border-width) solid var(--hairline); border-radius: var(--r-md); background: var(--canvas); color: var(--body); font-size: 13px; cursor: pointer;"
                                    }
                                    on:click=move |_| {
                                        ctx.config.update(|c| {
                                            c.ui_radius_sm = sm.to_string();
                                            c.ui_radius_md = md.to_string();
                                            c.ui_radius_lg = lg.to_string();
                                            c.ui_radius_xl = xl.to_string();
                                            c.theme_name = "Custom".to_string();
                                        });
                                        ctx.save();
                                    }
                                >
                                    {label}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                    <div style="display: flex; flex-wrap: wrap; gap: 16px;">
                        {field_theme_text("SM", |c| c.ui_radius_sm.clone(), |c, v| c.ui_radius_sm = v)}
                        {field_theme_text("MD", |c| c.ui_radius_md.clone(), |c, v| c.ui_radius_md = v)}
                        {field_theme_text("LG", |c| c.ui_radius_lg.clone(), |c, v| c.ui_radius_lg = v)}
                        {field_theme_text("XL", |c| c.ui_radius_xl.clone(), |c, v| c.ui_radius_xl = v)}
                    </div>
                </div>

                <h4 style="font-size: 13px; font-weight: 600; color: var(--ink); margin-bottom: 12px; border-bottom: var(--border-width) solid var(--hairline); padding-bottom: 6px;">"Spacings & Sizes"</h4>
                <div style="display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 24px;">
                    {field_theme_text("Extra Small (XS)", |c| c.ui_size_xs.clone(), |c, v| c.ui_size_xs = v)}
                    {field_theme_text("Small (SM)", |c| c.ui_size_sm.clone(), |c, v| c.ui_size_sm = v)}
                    {field_theme_text("Medium (MD)", |c| c.ui_size_md.clone(), |c, v| c.ui_size_md = v)}
                    {field_theme_text("Large (LG)", |c| c.ui_size_lg.clone(), |c, v| c.ui_size_lg = v)}
                    {field_theme_text("Extra Large (XL)", |c| c.ui_size_xl.clone(), |c, v| c.ui_size_xl = v)}
                </div>

                <h4 style="font-size: 13px; font-weight: 600; color: var(--ink); margin-bottom: 12px; border-bottom: var(--border-width) solid var(--hairline); padding-bottom: 6px;">"Saved Custom Themes"</h4>
                <div style="display: flex; gap: 8px; align-items: center; margin-bottom: 16px;">
                    <input class="input" style="height: 32px; font-size: 13px; flex: 1;"
                        placeholder="Theme name..."
                        prop:value=move || new_theme_name.get()
                        on:input=move |e| new_theme_name.set(event_target_value(&e))
                    />
                    <button class="btn primary sm" on:click=move |_| {
                        let name = new_theme_name.get_untracked().trim().to_string();
                        if name.is_empty() { return; }
                        ctx.update_cfg(move |c| {
                            let theme = shared::CustomTheme {
                                name: name.clone(),
                                background_color: c.ui_background_color.clone(),
                                text_color: c.ui_text_color.clone(),
                                accent_color: c.ui_accent_color.clone(),
                                card_bg: c.ui_card_bg.clone(),
                                sidebar_bg: c.ui_sidebar_bg.clone(),
                                border_color: c.ui_border_color.clone(),
                                font_family: c.ui_font_family.clone(),
                                transparency: c.ui_transparency,
                                blur: c.ui_blur,
                                blur_intensity: c.ui_blur_intensity,
                                radius_sm: c.ui_radius_sm.clone(),
                                radius_md: c.ui_radius_md.clone(),
                                radius_lg: c.ui_radius_lg.clone(),
                                radius_xl: c.ui_radius_xl.clone(),
                                border_width: c.ui_border_width.clone(),
                                size_xs: c.ui_size_xs.clone(),
                                size_sm: c.ui_size_sm.clone(),
                                size_md: c.ui_size_md.clone(),
                                size_lg: c.ui_size_lg.clone(),
                                size_xl: c.ui_size_xl.clone(),
                                button_bg: c.ui_button_bg.clone(),
                                button_text: c.ui_button_text.clone(),
                                card_text: c.ui_card_text.clone(),
                            };
                            if let Some(pos) = c.custom_themes.iter().position(|t| t.name == name) {
                                c.custom_themes[pos] = theme;
                            } else {
                                c.custom_themes.push(theme);
                            }
                        });
                        new_theme_name.set(String::new());
                    }>"Save Current"</button>
                </div>
                {move || {
                    let themes = ctx.config.get().custom_themes;
                    if themes.is_empty() {
                        view! { <div style="font-size: 12px; color: var(--muted); font-style: italic;">"No saved themes yet."</div> }.into_any()
                    } else {
                        themes.into_iter().map(|ct| {
                            let ct_name = ct.name.clone();
                            let ct_name_del = ct.name.clone();
                            let ct_for_apply = ct.clone();
                            view! {
                                <div style="display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: var(--surface-soft); border: var(--border-width) solid var(--hairline); border-radius: var(--r-md); margin-bottom: 6px;">
                                    <span style="font-size: 13px; font-weight: 500; color: var(--ink);">{ct_name.clone()}</span>
                                    <div style="display: flex; gap: 6px;">
                                        <button class="btn secondary sm" style="height: 26px; font-size: 11px;" on:click=move |_| {
                                            let t = ct_for_apply.clone();
                                            ctx.update_cfg(move |c| {
                                                c.theme_name = t.name.clone();
                                                c.ui_background_color = t.background_color.clone();
                                                c.ui_text_color = t.text_color.clone();
                                                c.ui_accent_color = t.accent_color.clone();
                                                c.ui_card_bg = t.card_bg.clone();
                                                c.ui_sidebar_bg = t.sidebar_bg.clone();
                                                c.ui_border_color = t.border_color.clone();
                                                c.ui_font_family = t.font_family.clone();
                                                c.ui_radius_sm = t.radius_sm.clone();
                                                c.ui_radius_md = t.radius_md.clone();
                                                c.ui_radius_lg = t.radius_lg.clone();
                                                c.ui_radius_xl = t.radius_xl.clone();
                                                c.ui_border_width = t.border_width.clone();
                                                c.ui_button_bg = t.button_bg.clone();
                                                c.ui_button_text = t.button_text.clone();
                                                c.ui_card_text = t.card_text.clone();
                                            });
                                        }>"Apply"</button>
                                        <button class="btn ghost sm" style="height: 26px; font-size: 11px; color: #ef4444;" on:click=move |_| {
                                            let n = ct_name_del.clone();
                                            ctx.update_cfg(move |c| {
                                                c.custom_themes.retain(|t| t.name != n);
                                            });
                                        }>"✕"</button>
                                    </div>
                                </div>
                            }
                        }).collect_view().into_any()
                    }
                }}
            </Card>

            <Card title="Icon Pack">
                <div style="font-size: 12px; color: var(--muted); margin-bottom: 16px;">"Controls whether sidebar navigation items show colored emoji icons, monochrome symbols, or no icons."</div>
                <div style="display: flex; gap: 12px; flex-wrap: wrap;">
                    {[("colored", "🎨 Colored", "Full color emoji icons"), ("mono", "⬛ Monochrome", "Plain symbol, no color"), ("none", "— None", "Text-only, no icons")].into_iter().map(|(val, label, desc)| {
                        let active = move || ctx.config.get().icon_pack == val;
                        view! {
                            <button
                                style=move || if active() {
                                    "display: flex; flex-direction: column; gap: 4px; padding: 12px 16px; border: 2px solid var(--primary); border-radius: var(--r-lg); background: color-mix(in srgb, var(--primary) 10%, var(--canvas)); cursor: pointer; text-align: left; min-width: 130px;"
                                } else {
                                    "display: flex; flex-direction: column; gap: 4px; padding: 12px 16px; border: var(--border-width) solid var(--hairline); border-radius: var(--r-lg); background: var(--canvas); cursor: pointer; text-align: left; min-width: 130px;"
                                }
                                on:click=move |_| {
                                    ctx.update_cfg(move |c| c.icon_pack = val.to_string());
                                }
                            >
                                <span style="font-size: 13px; font-weight: 600; color: var(--ink);">{label}</span>
                                <span style="font-size: 11.5px; color: var(--muted);">{desc}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
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
                            <div style="display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: var(--color-card-bg); border: var(--border-width) solid var(--color-border); border-radius: var(--r-md);">
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
                                    <div style="display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: var(--color-card-bg); border: var(--border-width) solid var(--color-border); border-radius: var(--r-md); font-size: 13px;">
                                        <span style="font-family: monospace; word-break: break-all; margin-right: 8px;">
                                            {dir_c}
                                        </span>
                                        <button
                                            class="btn"
                                            style="padding: 4px 8px; font-size: 12px; border-radius: var(--r-sm); border: var(--border-width) solid var(--color-border); background: transparent; cursor: pointer; color: var(--color-accent);"
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
                    style="padding: 8px 16px; border-radius: var(--r-md); cursor: pointer; border: var(--border-width) solid var(--color-border); background: var(--color-accent); color: var(--color-accent-on); font-weight: 500;"
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
        <div class="color-input" style="display: flex; flex-direction: column; gap: 4px; flex: 1 1 140px; min-width: 130px;">
            <label class="field-label" style="font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 2px;">{label}</label>
            <div style="display: flex; gap: 6px; align-items: center;">
                <input
                    type="color"
                    style="height: 32px; width: 36px; flex-shrink: 0; border-radius: var(--r-sm); border: var(--border-width) solid var(--hairline); padding: 1px; cursor: pointer; background: var(--canvas);"
                    prop:value=move || normalize_hex(value.get())
                    on:input=move |e| on_input.run(event_target_value(&e))
                    on:change=move |_| on_commit.run(())
                />
                <input
                    type="text"
                    class="input"
                    style="height: 32px; font-size: 12px; padding: 0 8px; border-radius: var(--r-sm); border: var(--border-width) solid var(--hairline); flex-grow: 1; background: var(--canvas); color: var(--ink);"
                    prop:value=move || value.get()
                    on:input=move |e| on_input.run(event_target_value(&e))
                    on:change=move |_| on_commit.run(())
                />
            </div>
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
