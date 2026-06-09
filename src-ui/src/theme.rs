//! Theming. A [`Palette`] is the full set of design tokens (DESIGN.md). Built-in
//! presets carry complete palettes; a user "Custom" theme derives a full palette
//! from the custom `ui_*` config fields. The active theme is selected by
//! `cfg.theme_name`. Rendering reads only the resolved palette → CSS custom
//! properties on the app root (working-copy model).

use shared::ServerConfig;

/// Complete token set emitted as CSS variables.
pub struct Palette {
    pub canvas: String,
    pub surface_soft: String,
    pub surface_card: String,
    pub surface_strong: String,
    pub ink: String,
    pub body: String,
    pub muted: String,
    pub hairline: String,
    pub hairline_soft: String,
    pub primary: String,
    pub primary_active: String,
    pub on_primary: String,
    pub accent: String,
    pub sidebar_bg: String,
    pub app_bg: String,
    pub font: String,
    pub blur: u32,
    pub opacity: f32,
    pub radius_sm: String,
    pub radius_md: String,
    pub radius_lg: String,
    pub radius_xl: String,
    pub border_width: String,
    pub size_xs: String,
    pub size_sm: String,
    pub size_md: String,
    pub size_lg: String,
    pub size_xl: String,
    pub button_bg: String,
    pub button_text: String,
    pub card_text: String,
}

/// Names of the built-in presets, shown in Settings.
pub const PRESETS: [&str; 8] = ["Cal Light", "Midnight Glass", "Slate Dark", "Dracula", "Ocean Blue", "Forest Green", "Rose Gold", "Amber"];

fn cal_light() -> Palette {
    Palette {
        canvas: "#ffffff".into(),
        surface_soft: "#f8f9fa".into(),
        surface_card: "#f5f5f5".into(),
        surface_strong: "#e5e7eb".into(),
        ink: "#111111".into(),
        body: "#374151".into(),
        muted: "#6b7280".into(),
        hairline: "#e5e7eb".into(),
        hairline_soft: "#f3f4f6".into(),
        primary: "#111111".into(),
        primary_active: "#242424".into(),
        on_primary: "#ffffff".into(),
        accent: "#3b82f6".into(),
        sidebar_bg: "#ffffff".into(),
        app_bg: "#ffffff".into(),
        font: "Inter".into(),
        blur: 0,
        opacity: 1.0,
        radius_sm: "2px".into(),
        radius_md: "4px".into(),
        radius_lg: "6px".into(),
        radius_xl: "8px".into(),
        border_width: "1px".into(),
        size_xs: "8px".into(),
        size_sm: "12px".into(),
        size_md: "16px".into(),
        size_lg: "24px".into(),
        size_xl: "32px".into(),
        button_bg: "#111111".into(),
        button_text: "#ffffff".into(),
        card_text: "#111111".into(),
    }
}

fn midnight_glass() -> Palette {
    // Revamped to solid premium look to honor "disable blur/transparency for now"
    Palette {
        canvas: "#1e1b4b".into(),
        surface_soft: "#312e81".into(),
        surface_card: "#1e1b4b".into(),
        surface_strong: "#4338ca".into(),
        ink: "#f8fafc".into(),
        body: "#cbd5e1".into(),
        muted: "#818cf8".into(),
        hairline: "#312e81".into(),
        hairline_soft: "#1e1b4b".into(),
        primary: "#6366f1".into(),
        primary_active: "#4f46e5".into(),
        on_primary: "#ffffff".into(),
        accent: "#818cf8".into(),
        sidebar_bg: "#111827".into(),
        app_bg: "#111827".into(),
        font: "Inter".into(),
        blur: 0,
        opacity: 1.0,
        radius_sm: "2px".into(),
        radius_md: "4px".into(),
        radius_lg: "6px".into(),
        radius_xl: "8px".into(),
        border_width: "1px".into(),
        size_xs: "8px".into(),
        size_sm: "12px".into(),
        size_md: "16px".into(),
        size_lg: "24px".into(),
        size_xl: "32px".into(),
        button_bg: "#6366f1".into(),
        button_text: "#ffffff".into(),
        card_text: "#f8fafc".into(),
    }
}

fn slate_dark() -> Palette {
    Palette {
        canvas: "#1e293b".into(),
        surface_soft: "#0f172a".into(),
        surface_card: "#1e293b".into(),
        surface_strong: "#334155".into(),
        ink: "#f1f5f9".into(),
        body: "#cbd5e1".into(),
        muted: "#94a3b8".into(),
        hairline: "#334155".into(),
        hairline_soft: "#1e293b".into(),
        primary: "#6366f1".into(),
        primary_active: "#4f46e5".into(),
        on_primary: "#ffffff".into(),
        accent: "#818cf8".into(),
        sidebar_bg: "#0b1220".into(),
        app_bg: "#0b1220".into(),
        font: "Inter".into(),
        blur: 0,
        opacity: 1.0,
        radius_sm: "2px".into(),
        radius_md: "4px".into(),
        radius_lg: "6px".into(),
        radius_xl: "8px".into(),
        border_width: "1px".into(),
        size_xs: "8px".into(),
        size_sm: "12px".into(),
        size_md: "16px".into(),
        size_lg: "24px".into(),
        size_xl: "32px".into(),
        button_bg: "#6366f1".into(),
        button_text: "#ffffff".into(),
        card_text: "#f1f5f9".into(),
    }
}

fn dracula() -> Palette {
    Palette {
        canvas: "#282a36".into(), surface_soft: "#21222c".into(), surface_card: "#282a36".into(),
        surface_strong: "#44475a".into(), ink: "#f8f8f2".into(), body: "#cdd6f4".into(),
        muted: "#6272a4".into(), hairline: "#44475a".into(), hairline_soft: "#282a36".into(),
        primary: "#bd93f9".into(), primary_active: "#a580f5".into(), on_primary: "#282a36".into(),
        accent: "#ff79c6".into(), sidebar_bg: "#21222c".into(), app_bg: "#1e1f29".into(),
        font: "Inter".into(), blur: 0, opacity: 1.0,
        radius_sm: "2px".into(), radius_md: "4px".into(), radius_lg: "6px".into(), radius_xl: "8px".into(),
        border_width: "1px".into(), size_xs: "8px".into(), size_sm: "12px".into(),
        size_md: "16px".into(), size_lg: "24px".into(), size_xl: "32px".into(),
        button_bg: "#bd93f9".into(), button_text: "#282a36".into(), card_text: "#f8f8f2".into(),
    }
}

fn ocean_blue() -> Palette {
    Palette {
        canvas: "#0d1b2a".into(), surface_soft: "#1b2838".into(), surface_card: "#1b2838".into(),
        surface_strong: "#2a4a6b".into(), ink: "#e8f4fd".into(), body: "#b8d4e8".into(),
        muted: "#7bafc8".into(), hairline: "#2a4a6b".into(), hairline_soft: "#1b2838".into(),
        primary: "#38bdf8".into(), primary_active: "#0ea5e9".into(), on_primary: "#0d1b2a".into(),
        accent: "#06b6d4".into(), sidebar_bg: "#0a1520".into(), app_bg: "#0a1520".into(),
        font: "Inter".into(), blur: 0, opacity: 1.0,
        radius_sm: "3px".into(), radius_md: "6px".into(), radius_lg: "10px".into(), radius_xl: "14px".into(),
        border_width: "1px".into(), size_xs: "8px".into(), size_sm: "12px".into(),
        size_md: "16px".into(), size_lg: "24px".into(), size_xl: "32px".into(),
        button_bg: "#38bdf8".into(), button_text: "#0d1b2a".into(), card_text: "#e8f4fd".into(),
    }
}

fn forest_green() -> Palette {
    Palette {
        canvas: "#0f1a14".into(), surface_soft: "#1a2d1f".into(), surface_card: "#1a2d1f".into(),
        surface_strong: "#2d4a35".into(), ink: "#ecfdf5".into(), body: "#a7f3d0".into(),
        muted: "#6ee7b7".into(), hairline: "#2d4a35".into(), hairline_soft: "#1a2d1f".into(),
        primary: "#10b981".into(), primary_active: "#059669".into(), on_primary: "#ecfdf5".into(),
        accent: "#34d399".into(), sidebar_bg: "#0a1410".into(), app_bg: "#0a1410".into(),
        font: "Inter".into(), blur: 0, opacity: 1.0,
        radius_sm: "3px".into(), radius_md: "6px".into(), radius_lg: "10px".into(), radius_xl: "14px".into(),
        border_width: "1px".into(), size_xs: "8px".into(), size_sm: "12px".into(),
        size_md: "16px".into(), size_lg: "24px".into(), size_xl: "32px".into(),
        button_bg: "#10b981".into(), button_text: "#ecfdf5".into(), card_text: "#ecfdf5".into(),
    }
}

fn rose_gold() -> Palette {
    Palette {
        canvas: "#fff5f7".into(), surface_soft: "#fce7eb".into(), surface_card: "#fce7eb".into(),
        surface_strong: "#f9cdd4".into(), ink: "#1a0a0d".into(), body: "#5c2130".into(),
        muted: "#b07080".into(), hairline: "#f3b8c3".into(), hairline_soft: "#fce7eb".into(),
        primary: "#e11d48".into(), primary_active: "#be123c".into(), on_primary: "#ffffff".into(),
        accent: "#f43f5e".into(), sidebar_bg: "#fff0f3".into(), app_bg: "#fff5f7".into(),
        font: "Inter".into(), blur: 0, opacity: 1.0,
        radius_sm: "4px".into(), radius_md: "8px".into(), radius_lg: "12px".into(), radius_xl: "20px".into(),
        border_width: "1px".into(), size_xs: "8px".into(), size_sm: "12px".into(),
        size_md: "16px".into(), size_lg: "24px".into(), size_xl: "32px".into(),
        button_bg: "#e11d48".into(), button_text: "#ffffff".into(), card_text: "#1a0a0d".into(),
    }
}

fn amber() -> Palette {
    Palette {
        canvas: "#fffbeb".into(), surface_soft: "#fef3c7".into(), surface_card: "#fef3c7".into(),
        surface_strong: "#fde68a".into(), ink: "#1c1508".into(), body: "#4a3500".into(),
        muted: "#92620d".into(), hairline: "#fcd34d".into(), hairline_soft: "#fef3c7".into(),
        primary: "#d97706".into(), primary_active: "#b45309".into(), on_primary: "#ffffff".into(),
        accent: "#f59e0b".into(), sidebar_bg: "#fffdf5".into(), app_bg: "#fffbeb".into(),
        font: "Inter".into(), blur: 0, opacity: 1.0,
        radius_sm: "3px".into(), radius_md: "6px".into(), radius_lg: "10px".into(), radius_xl: "14px".into(),
        border_width: "1px".into(), size_xs: "8px".into(), size_sm: "12px".into(),
        size_md: "16px".into(), size_lg: "24px".into(), size_xl: "32px".into(),
        button_bg: "#d97706".into(), button_text: "#ffffff".into(), card_text: "#1c1508".into(),
    }
}

fn preset(name: &str) -> Option<Palette> {
    match name {
        "Cal Light" => Some(cal_light()),
        "Midnight Glass" => Some(midnight_glass()),
        "Slate Dark" => Some(slate_dark()),
        "Dracula" => Some(dracula()),
        "Ocean Blue" => Some(ocean_blue()),
        "Forest Green" => Some(forest_green()),
        "Rose Gold" => Some(rose_gold()),
        "Amber" => Some(amber()),
        _ => None,
    }
}

/// Resolve the active palette for a config: a named preset, else a palette
/// derived from the custom `ui_*` fields.
pub fn resolve(cfg: &ServerConfig) -> Palette {
    // 1. Determine resolved theme name based on dark_mode toggle
    let resolved_theme = if cfg.dark_mode {
        // Wants dark mode
        if cfg.theme_name == "Cal Light" {
            "Slate Dark".to_string()
        } else {
            cfg.theme_name.clone() // Already dark presets: "Slate Dark", "Midnight Glass"
        }
    } else {
        // Wants light mode
        if cfg.theme_name == "Slate Dark" || cfg.theme_name == "Midnight Glass" {
            "Cal Light".to_string()
        } else {
            cfg.theme_name.clone() // Already light: "Cal Light"
        }
    };

    // 2. Resolve to the preset, or fallback to custom from_ui
    let mut p = preset(&resolved_theme).unwrap_or_else(|| from_ui(cfg));

    // 3. For custom themes, if the dark_mode flag doesn't match the luminance of the custom colors,
    // we dynamically swap/adapt them!
    if resolved_theme == "default" || resolved_theme == "Custom" {
        let is_custom_dark = is_dark(&cfg.ui_background_color);
        if cfg.dark_mode && !is_custom_dark {
            // Custom theme is currently light, but dark_mode is requested!
            // Invert/swap background and text colors to make a dark version
            p.app_bg = "#0f172a".to_string();
            p.canvas = "#1e293b".to_string();
            p.surface_soft = "#0f172a".to_string();
            p.surface_card = "#1e293b".to_string();
            p.surface_strong = "#334155".to_string();
            p.hairline = "#334155".to_string();
            p.hairline_soft = "#1e293b".to_string();
            p.ink = "#f1f5f9".to_string();
            p.body = "#cbd5e1".to_string();
            p.muted = "rgba(255,255,255,0.6)".to_string();
            p.sidebar_bg = "#0b1220".to_string();
        } else if !cfg.dark_mode && is_custom_dark {
            // Custom theme is currently dark, but light_mode is requested!
            // Swap background and text colors to make a light version
            p.app_bg = "#ffffff".to_string();
            p.canvas = "#ffffff".to_string();
            p.surface_soft = "#f8f9fa".to_string();
            p.surface_card = "#f5f5f5".to_string();
            p.surface_strong = "#e5e7eb".to_string();
            p.hairline = "#e5e7eb".to_string();
            p.hairline_soft = "#f3f4f6".to_string();
            p.ink = "#111111".to_string();
            p.body = "#374151".to_string();
            p.muted = "rgba(0,0,0,0.55)".to_string();
            p.sidebar_bg = "#ffffff".to_string();
        }
    }

    p
}

/// Derive a full palette from the custom `ui_*` fields, choosing
/// readable on-colors and surfaces from luminance.
/// When dark_mode is false, uses the `ui_light_*` fields instead.
fn from_ui(cfg: &ServerConfig) -> Palette {
    if !cfg.dark_mode {
        let dark = is_dark(&cfg.ui_light_background_color);
        let primary_active = format!("color-mix(in srgb, {} 85%, black)", cfg.ui_light_button_bg);
        Palette {
            canvas: cfg.ui_light_card_bg.clone(),
            surface_soft: cfg.ui_light_card_bg.clone(),
            surface_card: cfg.ui_light_card_bg.clone(),
            surface_strong: cfg.ui_light_border_color.clone(),
            ink: cfg.ui_light_text_color.clone(),
            body: cfg.ui_light_text_color.clone(),
            muted: if dark { "rgba(255,255,255,0.6)".into() } else { "rgba(0,0,0,0.55)".into() },
            hairline: cfg.ui_light_border_color.clone(),
            hairline_soft: cfg.ui_light_border_color.clone(),
            primary: cfg.ui_light_button_bg.clone(),
            primary_active,
            on_primary: cfg.ui_light_button_text.clone(),
            accent: cfg.ui_light_accent_color.clone(),
            sidebar_bg: cfg.ui_light_sidebar_bg.clone(),
            app_bg: cfg.ui_light_background_color.clone(),
            font: cfg.ui_font_family.clone(),
            blur: 0,
            opacity: 1.0,
            radius_sm: cfg.ui_radius_sm.clone(),
            radius_md: cfg.ui_radius_md.clone(),
            radius_lg: cfg.ui_radius_lg.clone(),
            radius_xl: cfg.ui_radius_xl.clone(),
            border_width: cfg.ui_border_width.clone(),
            size_xs: cfg.ui_size_xs.clone(),
            size_sm: cfg.ui_size_sm.clone(),
            size_md: cfg.ui_size_md.clone(),
            size_lg: cfg.ui_size_lg.clone(),
            size_xl: cfg.ui_size_xl.clone(),
            button_bg: cfg.ui_light_button_bg.clone(),
            button_text: cfg.ui_light_button_text.clone(),
            card_text: cfg.ui_light_card_text.clone(),
        }
    } else {
        let dark = is_dark(&cfg.ui_background_color);
        let primary_active = format!("color-mix(in srgb, {} 85%, black)", cfg.ui_button_bg);
        Palette {
            canvas: cfg.ui_card_bg.clone(),
            surface_soft: cfg.ui_card_bg.clone(),
            surface_card: cfg.ui_card_bg.clone(),
            surface_strong: cfg.ui_border_color.clone(),
            ink: cfg.ui_text_color.clone(),
            body: cfg.ui_text_color.clone(),
            muted: if dark {
                "rgba(255,255,255,0.6)".into()
            } else {
                "rgba(0,0,0,0.55)".into()
            },
            hairline: cfg.ui_border_color.clone(),
            hairline_soft: cfg.ui_border_color.clone(),
            primary: cfg.ui_button_bg.clone(),
            primary_active,
            on_primary: cfg.ui_button_text.clone(),
            accent: cfg.ui_accent_color.clone(),
            sidebar_bg: cfg.ui_sidebar_bg.clone(),
            app_bg: cfg.ui_background_color.clone(),
            font: cfg.ui_font_family.clone(),
            blur: 0,
            opacity: 1.0,
            radius_sm: cfg.ui_radius_sm.clone(),
            radius_md: cfg.ui_radius_md.clone(),
            radius_lg: cfg.ui_radius_lg.clone(),
            radius_xl: cfg.ui_radius_xl.clone(),
            border_width: cfg.ui_border_width.clone(),
            size_xs: cfg.ui_size_xs.clone(),
            size_sm: cfg.ui_size_sm.clone(),
            size_md: cfg.ui_size_md.clone(),
            size_lg: cfg.ui_size_lg.clone(),
            size_xl: cfg.ui_size_xl.clone(),
            button_bg: cfg.ui_button_bg.clone(),
            button_text: cfg.ui_button_text.clone(),
            card_text: cfg.ui_card_text.clone(),
        }
    }
}

/// Rough luminance test for `#rrggbb`. Non-hex inputs default to "dark".
fn is_dark(color: &str) -> bool {
    let hex = color.trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
            return lum < 140.0;
        }
    }
    true
}

/// Emit the CSS custom properties for the app root from the active theme.
pub fn css_vars(cfg: &ServerConfig) -> String {
    let p = resolve(cfg);
    format!(
        "--canvas:{canvas}; --surface-soft:{soft}; --surface-card:{card}; \
         --surface-strong:{strong}; --ink:{ink}; --body:{body}; --muted:{muted}; \
         --hairline:{hair}; --hairline-soft:{hairs}; --primary:{primary}; \
         --primary-active:{primary_active}; --on-primary:{on_primary}; --accent:{accent}; \
         --sidebar-bg:{sidebar}; --app-bg:{app_bg}; --blur:{blur}px; --app-opacity:{op}; \
         --font-display:'{font}', 'Inter', system-ui, sans-serif; \
         --font-ui:'{font}', 'Inter', system-ui, sans-serif; \
         --font-mono:'JetBrains Mono', ui-monospace, monospace; \
         --r-sm:{radius_sm}; --r-md:{radius_md}; --r-lg:{radius_lg}; --r-xl:{radius_xl}; \
         --border-width:{border_width}; \
         --s-xs:{size_xs}; --s-sm:{size_sm}; --s-md:{size_md}; --s-lg:{size_lg}; --s-xl:{size_xl}; \
         --card-text:{card_text};",
        canvas = p.canvas,
        soft = p.surface_soft,
        card = p.surface_card,
        strong = p.surface_strong,
        ink = p.ink,
        body = p.body,
        muted = p.muted,
        hair = p.hairline,
        hairs = p.hairline_soft,
        primary = p.primary,
        primary_active = p.primary_active,
        on_primary = p.on_primary,
        accent = p.accent,
        sidebar = p.sidebar_bg,
        app_bg = p.app_bg,
        blur = p.blur,
        op = p.opacity,
        font = p.font,
        radius_sm = p.radius_sm,
        radius_md = p.radius_md,
        radius_lg = p.radius_lg,
        radius_xl = p.radius_xl,
        border_width = p.border_width,
        size_xs = p.size_xs,
        size_sm = p.size_sm,
        size_md = p.size_md,
        size_lg = p.size_lg,
        size_xl = p.size_xl,
        card_text = p.card_text,
    )
}
