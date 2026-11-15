// 获取策略（协议 §2.3.2）在页面上的话和改法：功能、预设、一段去向叫什么，换一项时交回去的整份策略。
// 每项功能会用哪几种方式由内核算（`GET /strategy` 的 `routes`），这里不再排一遍。
import type { FetchFunction, FetchPlugin, MethodAccount, PlatformStrategy, Strategy, StrategyPreset, StrategyRule, StrategyStep, StrategyView, WayRef } from "../api/endpoints";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import type { Use } from "./platforms";
import { ACCESS_TEXT } from "./words";

export const FUNCTIONS: FetchFunction[] = ["text", "images", "video", "audio", "comments", "list:favorite", "list:like", "list:share", "list:view", "list"];

const FUNCTION_NAMES: Record<FetchFunction, () => string> = {
  text: () => t("plugins.strategy.fn.text"),
  images: () => t("plugins.strategy.fn.images"),
  video: () => t("plugins.strategy.fn.video"),
  audio: () => t("plugins.strategy.fn.audio"),
  comments: () => t("plugins.strategy.fn.comments"),
  "list:favorite": () => t("plugins.strategy.fn.listFavorite"),
  "list:like": () => t("plugins.strategy.fn.listLike"),
  "list:share": () => t("plugins.strategy.fn.listShare"),
  "list:view": () => t("plugins.strategy.fn.listView"),
  list: () => t("plugins.strategy.fn.list"),
};

export function functionName(f: FetchFunction): string {
  return FUNCTION_NAMES[f]();
}

export const PRESETS: StrategyPreset[] = ["no-account-only", "no-account-first", "account-first", "custom"];

const PRESET_TEXT: Record<StrategyPreset, { name: () => string; sub: () => string }> = {
  "no-account-only": { name: () => t("plugins.strategy.preset.noAccountOnly"), sub: () => t("plugins.strategy.preset.noAccountOnlySub") },
  "no-account-first": { name: () => t("plugins.strategy.preset.noAccountFirst"), sub: () => t("plugins.strategy.preset.noAccountFirstSub") },
  "account-first": { name: () => t("plugins.strategy.preset.accountFirst"), sub: () => t("plugins.strategy.preset.accountFirstSub") },
  custom: { name: () => t("plugins.strategy.preset.custom"), sub: () => t("plugins.strategy.preset.customSub") },
};

export function presetName(p: StrategyPreset): string {
  return PRESET_TEXT[p].name();
}

export function presetSub(p: StrategyPreset): string {
  return PRESET_TEXT[p].sub();
}

/** `<插件 id>#<方法 id>`：指定一种方式、规则里的一段都这样写。 */
export function wayKey(w: WayRef): string {
  return `${w.plugin}#${w.method}`;
}

export function wayUse(plugins: FetchPlugin[], w: WayRef): Use | undefined {
  const plugin = plugins.find((p) => p.id === w.plugin);
  const method = plugin?.methods.find((m) => m.id === w.method);
  return plugin && method ? { plugin, method } : undefined;
}

/** 带不带账号，几个字。 */
export function accountShort(kind: MethodAccount["kind"]): string {
  if (kind === "none") return t("plugins.strategy.account.none");
  if (kind === "visitor") return t("plugins.strategy.account.visitor");
  return t("plugins.strategy.account.with");
}

/** 「访客接口（不使用账号）」。 */
export function wayName(u: Use): string {
  return t("plugins.strategy.wayName", { name: u.method.name, account: accountShort(u.method.account.kind) });
}

/** 只留存着的那几项：`GET /strategy` 多给的预设和求值结果不交回去。 */
export function strategyOf(v: Strategy): Strategy {
  return { preset: v.preset, ...(v.rules && v.rules.length > 0 ? { rules: v.rules } : {}), ...(v.platforms ? { platforms: v.platforms } : {}) };
}

/** 改一个平台自己的那一项；改空了就去掉。 */
export function withPlatform(v: Strategy, platform: string, change: (p: PlatformStrategy) => PlatformStrategy): Strategy {
  const base = strategyOf(v);
  const next = change(base.platforms?.[platform] ?? {});
  const platforms = { ...base.platforms };
  const empty = next.preset === undefined && Object.keys(next.functions ?? {}).length === 0;
  if (empty) delete platforms[platform];
  else platforms[platform] = next;
  const out: Strategy = { ...base, platforms };
  if (Object.keys(platforms).length === 0) delete out.platforms;
  return out;
}

/** 这个平台另选一个预设，`null` 是跟随总策略。 */
export function withPlatformPreset(v: Strategy, platform: string, preset: PlatformStrategy["preset"] | null): Strategy {
  return withPlatform(v, platform, (p) => {
    const next: PlatformStrategy = { ...p };
    if (preset === null) delete next.preset;
    else next.preset = preset;
    return next;
  });
}

/** 某项功能上指定一种方式（`<插件 id>#<方法 id>`）或 `skip`；`null` 是跟随策略。 */
export function withPick(v: Strategy, platform: string, f: FetchFunction, pick: string | null): Strategy {
  return withPlatform(v, platform, (p) => {
    const functions = { ...p.functions };
    if (pick === null) delete functions[f];
    else functions[f] = pick;
    const next: PlatformStrategy = { ...p, functions };
    if (Object.keys(functions).length === 0) delete next.functions;
    return next;
  });
}

/** 换一个总策略。换到「自定义」时没有规则就从现在那个预设的规则抄起；预设之间换，自定义的规则留着。 */
export function withPreset(v: StrategyView, preset: StrategyPreset): Strategy {
  const base = strategyOf(v);
  if (preset !== "custom" || (base.rules?.length ?? 0) > 0) return { ...base, preset };
  const from = v.presets.find((p) => p.preset === v.preset)?.rules ?? [{ use: [{}] }];
  return { ...base, preset, rules: from.map((r) => ({ ...r, use: r.use.map((s) => ({ ...s })) })) };
}

export function withRules(v: Strategy, rules: StrategyRule[]): Strategy {
  return { ...strategyOf(v), rules };
}

/** 收下以后照策略取的那几项（不含列清单）各自先用哪种：「正文、图片：访客接口（不使用账号）」。 */
export function fetchSummary(v: StrategyView, platform: string, plugins: FetchPlugin[]): string[] {
  const route = v.routes.find((r) => r.platform.id === platform);
  const byWay = new Map<string, FetchFunction[]>();
  for (const f of route?.functions ?? []) {
    if (f.function.startsWith("list")) continue;
    const first = f.ways[0];
    const u = first ? wayUse(plugins, first) : undefined;
    const way = u ? wayName(u) : t("plugins.strategy.skip");
    byWay.set(way, [...(byWay.get(way) ?? []), f.function]);
  }
  return [...byWay].map(([way, fs]) => t("plugins.strategy.summary", { what: formatList(fs.map(functionName)), way }));
}

// —— 规则表里的一段 ——

const NO_ACCOUNT: MethodAccount["kind"][] = ["none", "visitor"];
const WITH_ACCOUNT: MethodAccount["kind"][] = ["own-browser", "own-phone", "own-app", "in-usuyo", "developer"];

/** 规则表里能加的几组：和内核的预设用的是同样的圈法。 */
export const GROUPS: { id: string; step: StrategyStep; name: () => string }[] = [
  { id: "no-account", step: { accounts: NO_ACCOUNT }, name: () => t("settings.strategy.group.noAccount") },
  { id: "with-account", step: { accounts: WITH_ACCOUNT }, name: () => t("settings.strategy.group.withAccount") },
  { id: "all", step: {}, name: () => t("settings.strategy.group.all") },
  { id: "private-api", step: { access: ["private-api"] }, name: () => t("settings.strategy.group.privateApi") },
  { id: "client-hook", step: { access: ["client-hook"] }, name: () => t("settings.strategy.group.clientHook") },
];

/** 一段的键：同一段只出现一次。 */
export function stepKey(s: StrategyStep): string {
  if (s.method) return `m:${s.method}`;
  return `g:${[...(s.accounts ?? [])].sort().join(",")}|${[...(s.access ?? [])].sort().join(",")}`;
}

function kindName(k: MethodAccount["kind"]): string {
  const names: Record<MethodAccount["kind"], () => string> = {
    none: () => t("settings.strategy.kind.none"),
    visitor: () => t("settings.strategy.kind.visitor"),
    "own-browser": () => t("settings.strategy.kind.ownBrowser"),
    "own-phone": () => t("settings.strategy.kind.ownPhone"),
    "own-app": () => t("settings.strategy.kind.ownApp"),
    "in-usuyo": () => t("settings.strategy.kind.inUsuyo"),
    developer: () => t("settings.strategy.kind.developer"),
  };
  return names[k]();
}

/** 一段叫什么：一种方式写它的名字和带不带账号，认得的组写组名，别的照账号类型和层级写。 */
export function stepName(s: StrategyStep, plugins: FetchPlugin[]): string {
  if (s.method) {
    const [plugin = "", method = ""] = s.method.split("#");
    const u = wayUse(plugins, { plugin, method });
    return u ? wayName(u) : s.method;
  }
  const known = GROUPS.find((g) => stepKey(g.step) === stepKey(s));
  if (known) return known.name();
  const accounts = s.accounts && s.accounts.length > 0 ? formatList(s.accounts.map(kindName)) : t("settings.strategy.group.anyAccount");
  const tiers = s.access && s.access.length > 0 ? formatList(s.access.map((a) => ACCESS_TEXT[a])) : t("settings.strategy.group.anyTier");
  return t("settings.strategy.group.other", { accounts, tiers });
}
