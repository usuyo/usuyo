import { useEffect } from "react";
import { useUiStore, type Accent, type ThemePreference } from "../store/ui";

export type Scheme = "dark" | "light";

/** 画布要的颜色，从 CSS 变量里读；主题一换就重读。 */
export interface Palette {
  scheme: Scheme;
  bg: string;
  fogRgb: string;
  itemRgb: string;
  accentRgb: string;
}

function rgbOf(hex: string): [number, number, number] {
  const x = Number.parseInt(hex.slice(1), 16);
  return [(x >> 16) & 255, (x >> 8) & 255, x & 255];
}

/** 浅色下把选的颜色和黑色各半混合，保证对比度。 */
export function accentFor(accent: Accent, scheme: Scheme): [number, number, number] {
  const rgb = rgbOf(accent);
  return scheme === "light" ? [Math.round(rgb[0] / 2), Math.round(rgb[1] / 2), Math.round(rgb[2] / 2)] : rgb;
}

function systemScheme(): Scheme {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") return "dark";
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

let palette: Palette = { scheme: "dark", bg: "#0B0D10", fogRgb: "150,160,174", itemRgb: "214,219,226", accentRgb: "242,184,102" };
const listeners = new Set<() => void>();

export function currentPalette(): Palette {
  return palette;
}

export function onPaletteChange(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function applyTheme(pref: ThemePreference, accent: Accent): void {
  const root = document.documentElement;
  const scheme = pref === "system" ? systemScheme() : pref;
  root.dataset.theme = scheme;
  const [r, g, b] = accentFor(accent, scheme);
  root.style.setProperty("--accent", `rgb(${r}, ${g}, ${b})`);
  root.style.setProperty("--accent-rgb", `${r}, ${g}, ${b}`);
  const css = getComputedStyle(root);
  const read = (name: string, fallback: string) => css.getPropertyValue(name).trim() || fallback;
  palette = {
    scheme,
    bg: read("--bg", palette.bg),
    fogRgb: read("--fog-rgb", palette.fogRgb).replaceAll(" ", ""),
    itemRgb: read("--item-rgb", palette.itemRgb).replaceAll(" ", ""),
    accentRgb: `${r},${g},${b}`,
  };
  for (const l of listeners) l();
}

/** 跟着用户的选择和系统设置更新主题；在根组件里用一次。 */
export function useThemeSync(): void {
  const pref = useUiStore((s) => s.theme);
  const accent = useUiStore((s) => s.accent);
  useEffect(() => {
    applyTheme(pref, accent);
    if (pref !== "system" || typeof window.matchMedia !== "function") return undefined;
    const mq = window.matchMedia("(prefers-color-scheme: light)");
    const onChange = () => applyTheme(pref, accent);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, [pref, accent]);
}
