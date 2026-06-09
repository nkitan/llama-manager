import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** Convert a hex color (e.g. "#6366f1") to the HSL string format shadcn uses ("239 84% 67%"). */
export function hexToHsl(hex: string): string {
  const clean = hex.replace("#", "");
  if (clean.length !== 6) return "0 0% 50%";

  const r = parseInt(clean.slice(0, 2), 16) / 255;
  const g = parseInt(clean.slice(2, 4), 16) / 255;
  const b = parseInt(clean.slice(4, 6), 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;

  if (max === min) return `0 0% ${Math.round(l * 100)}%`;

  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

  let h = 0;
  if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
  else if (max === g) h = ((b - r) / d + 2) / 6;
  else h = ((r - g) / d + 4) / 6;

  return `${Math.round(h * 360)} ${Math.round(s * 100)}% ${Math.round(l * 100)}%`;
}

/** Lighten an HSL string by a delta percentage. */
export function lightenHsl(hsl: string, delta: number): string {
  const parts = hsl.split(" ");
  if (parts.length !== 3) return hsl;
  const l = parseFloat(parts[2]);
  return `${parts[0]} ${parts[1]} ${Math.min(100, l + delta)}%`;
}

/** Darken an HSL string by a delta percentage. */
export function darkenHsl(hsl: string, delta: number): string {
  const parts = hsl.split(" ");
  if (parts.length !== 3) return hsl;
  const l = parseFloat(parts[2]);
  return `${parts[0]} ${parts[1]} ${Math.max(0, l - delta)}%`;
}

export function formatTime(ts: number): string {
  const d = new Date(ts);
  return d.toTimeString().slice(0, 8) + "." + String(d.getMilliseconds()).padStart(3, "0");
}

export function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4);
}
