//! Theming. A [`Palette`] is the full set of design tokens (DESIGN.md). Built-in
//! presets carry complete palettes; a user "Custom" theme derives a full palette
//! from the limited `ui_*` config fields (with sensible contrast). The active
//! theme is selected by `cfg.theme_name`. Rendering reads only the resolved
//! palette → CSS custom properties on the app root (working-copy model).

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
}

/// Names of the built-in presets, shown in Settings.
pub const PRESETS: [&str; 3] = ["Cal Light", "Midnight Glass", "Slate Dark"];

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
    }
}

fn midnight_glass() -> Palette {
    // The prior frosted dark look — translucent surfaces + real blur.
    Palette {
        canvas: "rgba(255,255,255,0.06)".into(),
        surface_soft: "rgba(255,255,255,0.04)".into(),
        surface_card: "rgba(255,255,255,0.06)".into(),
        surface_strong: "rgba(255,255,255,0.14)".into(),
        ink: "#f8fafc".into(),
        body: "#cbd5e1".into(),
        muted: "rgba(248,250,252,0.6)".into(),
        hairline: "rgba(255,255,255,0.12)".into(),
        hairline_soft: "rgba(255,255,255,0.07)".into(),
        primary: "#6366f1".into(),
        primary_active: "#4f46e5".into(),
        on_primary: "#ffffff".into(),
        accent: "#818cf8".into(),
        sidebar_bg: "linear-gradient(135deg, rgba(255,255,255,0.08), rgba(255,255,255,0.02))".into(),
        app_bg: "rgba(15,23,42,0.72)".into(),
        font: "Inter".into(),
        blur: 30,
        opacity: 1.0,
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
    }
}

fn preset(name: &str) -> Option<Palette> {
    match name {
        "Cal Light" => Some(cal_light()),
        "Midnight Glass" => Some(midnight_glass()),
        "Slate Dark" => Some(slate_dark()),
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


/// Derive a full palette from the limited custom `ui_*` fields, choosing
/// readable on-colors and surfaces from luminance.
fn from_ui(cfg: &ServerConfig) -> Palette {
    let dark = is_dark(&cfg.ui_background_color);
    let on_primary = if is_dark(&cfg.ui_accent_color) {
        "#ffffff"
    } else {
        "#111111"
    };
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
        primary: cfg.ui_accent_color.clone(),
        primary_active: cfg.ui_accent_color.clone(),
        on_primary: on_primary.into(),
        accent: cfg.ui_accent_color.clone(),
        sidebar_bg: cfg.ui_sidebar_bg.clone(),
        app_bg: cfg.ui_background_color.clone(),
        font: cfg.ui_font_family.clone(),
        blur: if cfg.ui_blur { cfg.ui_blur_intensity } else { 0 },
        opacity: cfg.ui_transparency,
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
         --font-mono:'JetBrains Mono', ui-monospace, monospace;",
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
    )
}
