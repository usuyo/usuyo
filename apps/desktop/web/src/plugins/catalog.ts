// 插件目录页的筛选、状态和显示方式。显示方式（卡片或列表）记在这台电脑的浏览器存储里；
// 搜索和筛选只记在这次打开里，从详情回来还在。
import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { CatalogPlugin } from "../api/endpoints";

export type ViewMode = "cards" | "list";
export type StateFilter = "all" | "installed" | "update" | "available";

interface BrowserState {
  view: ViewMode;
  query: string;
  /** 平台 id；空的是所有平台。 */
  platform: string;
  state: StateFilter;
  setView: (view: ViewMode) => void;
  setQuery: (query: string) => void;
  setPlatform: (platform: string) => void;
  setState: (state: StateFilter) => void;
}

export const useBrowserStore = create<BrowserState>()(
  persist(
    (set) => ({
      view: "cards",
      query: "",
      platform: "",
      state: "all",
      setView: (view) => set({ view }),
      setQuery: (query) => set({ query }),
      setPlatform: (platform) => set({ platform }),
      setState: (state) => set({ state }),
    }),
    // 只记显示方式；存不了（无痕窗口）时 zustand 照常用内存里的
    { name: "usuyo.plugins.browser", partialize: (s) => ({ view: s.view }) },
  ),
);

/** 一个插件现在属于哪一种：有更新、已安装、未安装。 */
export function stateOf(p: CatalogPlugin): Exclude<StateFilter, "all"> {
  if (p.installed && p.update) return "update";
  if (p.installed) return "installed";
  return "available";
}

/** 搜索：名字、id、说明、平台名里有这几个字（不分大小写，空白分开的每一段都要有）。 */
export function matches(p: CatalogPlugin, query: string): boolean {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return true;
  const hay = [p.name, p.id, p.description ?? "", ...p.platforms.map((x) => `${x.id} ${x.name}`)].join(" ").toLowerCase();
  return words.every((w) => hay.includes(w));
}

export function filterPlugins(plugins: CatalogPlugin[], f: { query: string; platform: string; state: StateFilter }): CatalogPlugin[] {
  return plugins.filter((p) => matches(p, f.query) && (f.platform === "" || p.platforms.some((x) => x.id === f.platform)) && (f.state === "all" || stateOf(p) === f.state));
}

/** 目录里出现的平台，照名字排；同一个 id 用第一次见到的名字。 */
export function platformsOf(plugins: CatalogPlugin[]): { id: string; name: string }[] {
  const seen = new Map<string, string>();
  for (const p of plugins) for (const x of p.platforms) if (!seen.has(x.id)) seen.set(x.id, x.name);
  return [...seen].map(([id, name]) => ({ id, name })).sort((a, b) => a.name.localeCompare(b.name));
}

/**
 * 同一个插件在目录里的卡片和详情页的标题用同一个名字（`view-transition-name`）：以后加从卡片到详情的过渡时，
 * 两边认得出是同一个。名字只许字母、数字、连字符。
 */
export function sharedName(id: string): string {
  return `plugin-${id.replace(/[^a-zA-Z0-9-]/g, "-")}`;
}
