import { hexToHsl, lightenHsl, darkenHsl } from "./utils";
import type { ServerConfig } from "./ipc";

/**
 * Apply ServerConfig theme values as CSS custom properties on :root.
 * Called on initial load and whenever config changes.
 */
export function applyTheme(cfg: ServerConfig) {
  const root = document.documentElement;
  const dark = cfg.dark_mode;

  // ── Pick dark vs light palette ──────────────────────────────────────────
  const bg = dark ? cfg.ui_background_color : cfg.ui_light_bg;
  const fg = dark ? cfg.ui_text_color : cfg.ui_light_text;
  const accent = dark ? cfg.ui_accent_color : cfg.ui_light_accent;
  const cardBg = dark ? cfg.ui_background_color : cfg.ui_light_card_bg;
  const border = dark ? cfg.ui_border_color : cfg.ui_light_border;

  // ── Convert to HSL ──────────────────────────────────────────────────────
  const bgHsl = hexToHsl(bg);
  const fgHsl = hexToHsl(fg);
  const accentHsl = hexToHsl(accent);
  const cardHsl = hexToHsl(cardBg);
  const borderHsl = hexToHsl(border);

  const cardSlightly = dark ? darkenHsl(bgHsl, 3) : lightenHsl(bgHsl, 3);
  const sidebarHsl = dark ? darkenHsl(bgHsl, 5) : darkenHsl(bgHsl, 2);
  const mutedHsl = dark ? darkenHsl(bgHsl, 8) : darkenHsl(bgHsl, 4);
  const mutedFgHsl = dark ? lightenHsl(bgHsl, 40) : darkenHsl(bgHsl, 30);
  const popoverHsl = dark ? darkenHsl(bgHsl, 6) : lightenHsl(bgHsl, 1);

  // ── Set shadcn CSS vars ──────────────────────────────────────────────────
  root.style.setProperty("--background", bgHsl);
  root.style.setProperty("--foreground", fgHsl);
  root.style.setProperty("--card", cardHsl.length ? cardHsl : cardSlightly);
  root.style.setProperty("--card-foreground", fgHsl);
  root.style.setProperty("--popover", popoverHsl);
  root.style.setProperty("--popover-foreground", fgHsl);
  root.style.setProperty("--primary", accentHsl);
  root.style.setProperty("--primary-foreground", dark ? "0 0% 100%" : "0 0% 100%");
  root.style.setProperty("--secondary", mutedHsl);
  root.style.setProperty("--secondary-foreground", fgHsl);
  root.style.setProperty("--muted", mutedHsl);
  root.style.setProperty("--muted-foreground", mutedFgHsl);
  root.style.setProperty("--accent", mutedHsl);
  root.style.setProperty("--accent-foreground", fgHsl);
  root.style.setProperty("--destructive", "0 63% 55%");
  root.style.setProperty("--destructive-foreground", "0 0% 100%");
  root.style.setProperty("--border", borderHsl);
  root.style.setProperty("--input", borderHsl);
  root.style.setProperty("--ring", accentHsl);
  root.style.setProperty("--sidebar", sidebarHsl);
  root.style.setProperty("--sidebar-foreground", fgHsl);

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

  // ── Font family ──────────────────────────────────────────────────────────
  if (cfg.ui_font_family) {
    root.style.setProperty("--font-family", cfg.ui_font_family);
    document.body.style.fontFamily = `"${cfg.ui_font_family}", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`;
  }

  // ── Dark mode class ──────────────────────────────────────────────────────
  if (dark) {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }
}
