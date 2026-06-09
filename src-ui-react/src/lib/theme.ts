import { hexToHsl, lightenHsl, darkenHsl } from "./utils";
import type { ServerConfig } from "./ipc";

/**
 * Apply ServerConfig theme values as CSS custom properties on :root.
 * Called on initial load and whenever config changes.
 *
 * Light vs dark is selected by `cfg.dark_mode`. Each mode has a complete
 * palette (background, text, accent, card, sidebar, border, button bg/text,
 * card text) — no fallbacks to other-mode values.
 */
export function applyTheme(cfg: ServerConfig) {
  const root = document.documentElement;
  const dark = cfg.dark_mode;

  // ── Pick dark vs light palette ──────────────────────────────────────────
  const bg = dark ? cfg.ui_background_color : cfg.ui_light_background_color;
  const fg = dark ? cfg.ui_text_color : cfg.ui_light_text_color;
  const accent = dark ? cfg.ui_accent_color : cfg.ui_light_accent_color;
  const cardBg = dark ? cfg.ui_card_bg : cfg.ui_light_card_bg;
  const sidebarBg = dark ? cfg.ui_sidebar_bg : cfg.ui_light_sidebar_bg;
  const cardFg = dark ? cfg.ui_card_text : cfg.ui_light_card_text;
  const border = dark ? cfg.ui_border_color : cfg.ui_light_border_color;
  const buttonBg = dark ? cfg.ui_button_bg : cfg.ui_light_button_bg;
  const buttonFg = dark ? cfg.ui_button_text : cfg.ui_light_button_text;

  // ── Convert to HSL (shadcn format: "H S% L%") ─────────────────────────
  const bgHsl = hexToHsl(bg || "#0f172a");
  const fgHsl = hexToHsl(fg || "#f8fafc");
  const accentHsl = hexToHsl(accent || "#6366f1");
  const cardHsl = hexToHsl(cardBg || bg);
  const sidebarHsl = hexToHsl(sidebarBg || cardBg || bg);
  const borderHsl = hexToHsl(border || "#1e293b");
  const cardFgHsl = hexToHsl(cardFg || fg);
  const buttonBgHsl = hexToHsl(buttonBg || accent);
  const buttonFgHsl = hexToHsl(buttonFg || "#ffffff");

  // ── Derived tokens (muted backgrounds for subtle surfaces) ───
  // --muted is an HSL triple (no alpha) — alpha is applied at usage sites.
  const mutedHsl = dark
    ? lightenHsl(bgHsl, 4)
    : darkenHsl(bgHsl, 4);
  const mutedFgHsl = dark
    ? lightenHsl(fgHsl, -25)
    : darkenHsl(fgHsl, 30);
  const popoverHsl = dark ? darkenHsl(bgHsl, 3) : lightenHsl(bgHsl, 2);

  // ── Set shadcn CSS vars ──────────────────────────────────────────────────
  root.style.setProperty("--background", bgHsl);
  root.style.setProperty("--foreground", fgHsl);
  root.style.setProperty("--card", cardHsl);
  root.style.setProperty("--card-foreground", cardFgHsl);
  root.style.setProperty("--popover", popoverHsl);
  root.style.setProperty("--popover-foreground", cardFgHsl);
  root.style.setProperty("--primary", buttonBgHsl);
  root.style.setProperty("--primary-foreground", buttonFgHsl);
  root.style.setProperty("--secondary", mutedHsl);
  root.style.setProperty("--secondary-foreground", fgHsl);
  root.style.setProperty("--muted", mutedHsl);
  root.style.setProperty("--muted-foreground", mutedFgHsl);
  root.style.setProperty("--accent", accentHsl);
  root.style.setProperty("--accent-foreground", cardFgHsl);
  root.style.setProperty("--destructive", "0 63% 55%");
  root.style.setProperty("--destructive-foreground", "0 0% 100%");
  root.style.setProperty("--border", borderHsl);
  root.style.setProperty("--input", borderHsl);
  root.style.setProperty("--ring", buttonBgHsl);
  root.style.setProperty("--sidebar", sidebarHsl);
  root.style.setProperty("--sidebar-foreground", cardFgHsl);

  // ── Border radius ────────────────────────────────────────────────────────
  root.style.setProperty("--radius-sm", cfg.ui_radius_sm || "4px");
  root.style.setProperty("--radius-md", cfg.ui_radius_md || "6px");
  root.style.setProperty("--radius-lg", cfg.ui_radius_lg || "8px");
  root.style.setProperty("--radius-xl", cfg.ui_radius_xl || "12px");

  // ── Transparency / blur ──────────────────────────────────────────────────
  const opacity = Math.max(0.1, 1.0 - (cfg.ui_transparency ?? 0.1));
  root.style.setProperty("--window-bg-opacity", String(opacity.toFixed(2)));
  root.style.setProperty("--card-bg-opacity", String(Math.max(0.3, opacity - 0.2).toFixed(2)));
  root.style.setProperty("--sidebar-bg-opacity", String(Math.max(0.2, opacity - 0.3).toFixed(2)));

  const blur = cfg.ui_blur ? `${cfg.ui_blur_intensity ?? 28}px` : "0px";
  root.style.setProperty("--blur-radius", blur);

  // ── Font family (applied to <body> so the whole UI inherits it) ──────────
  const font = (cfg.ui_font_family || "Inter").replace(/^['"]+|['"]+$/g, "");
  document.body.style.fontFamily = `"${font}", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`;

  // ── Dark mode class ──────────────────────────────────────────────────────
  if (dark) {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }
}
