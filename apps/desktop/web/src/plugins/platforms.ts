// 获取插件页按平台分：一个平台一页。一种方法放在哪一张卡片上、「用哪种方法」那个选择框现在是哪一项、
// 换一项要拨哪几个开关，都在这里算；页面只照着摆。
import type { Access, FetchMethod, FetchPlugin, MethodAccount, PluginsView } from "../api/endpoints";
import enPlugins from "../i18n/en/plugins";
import zhPlugins from "../i18n/zh-Hans/plugins";
import { canList } from "./words";

/** 一个平台上的一种方法：方法和它所在的插件。一个插件管几个平台，就在几个平台上各出现一次。 */
export interface Use {
  plugin: FetchPlugin;
  method: FetchMethod;
}

/** 一种方法在页面上的去处：收一条链接的、只补上一样的、拿回存过的。 */
export type Role = "link" | "part" | "saved";

export interface PlatformPage {
  id: string;
  name: string;
  /** 管这个平台的插件，照装着的顺序。 */
  plugins: FetchPlugin[];
  /** 收一条链接的方法，照清单的顺序，没做的也在。 */
  link: Use[];
  /** 只补上一样（不取正文、标题这一类）的方法：「自动」下列成「补上：…」。 */
  parts: Use[];
  /** 列出你存过的、由浏览器交链接进来的、读文件的方法。 */
  saved: Use[];
}

/** `owner/repo#方法 id`：选择框的值、导入页的 `pick`、读文件的方法的 id 都是它。 */
export function keyOf(u: Use): string {
  return `${u.plugin.id}#${u.method.id}`;
}

/**
 * 能列的、交链接进来的是拿回存过的；取一条却不取正文也不取标题和作者的只是补上一样；别的（取一条，或没做、还没写能力的）
 * 是收一条链接的。能列又能取一条的（读文件、按 `ref` 回去取的）算拿回存过的：它们不拿来收链接。
 */
export function roleOf(m: FetchMethod): Role {
  if (canList(m) || m.gets.delivers === "links") return "saved";
  const fetch = m.gets.fetch;
  if (fetch && fetch.length > 0 && !fetch.includes("text") && !fetch.includes("meta")) return "part";
  return "link";
}

/** 照装着的插件分平台，平台照第一次出现的顺序。 */
export function platformPages(view: PluginsView): PlatformPage[] {
  const out = new Map<string, PlatformPage>();
  for (const plugin of view.plugins) {
    for (const p of plugin.platforms) {
      const page = out.get(p.id) ?? { id: p.id, name: p.name, plugins: [], link: [], parts: [], saved: [] };
      page.plugins.push(plugin);
      for (const method of plugin.methods) {
        const role = roleOf(method);
        const u = { plugin, method };
        if (role === "link") page.link.push(u);
        else if (role === "part") page.parts.push(u);
        else page.saved.push(u);
      }
      out.set(p.id, page);
    }
  }
  return [...out.values()];
}

/** 这种方法在这个平台上的那一层：清单的 `perPlatform` 给这个平台抬过的照它。 */
export function accessOn(u: Use, platform: string): Access {
  return u.method.perPlatform?.find((t) => t.platform === platform)?.access ?? u.method.access;
}

/** 这种方法在这个平台上开着没有：没拨过开关时照这个平台上的那一层，内核在 `perPlatform` 里算好了。 */
export function enabledOn(u: Use, platform: string): boolean {
  return u.method.perPlatform?.find((t) => t.platform === platform)?.enabled ?? u.method.enabled;
}

/** 这一层默认开着没有：`private-api`、`client-hook` 关着，别的开着（协议 §2.3.1）。 */
export function onByDefault(access: Access): boolean {
  return access !== "private-api" && access !== "client-hook";
}

const ACCESS_RANK: Access[] = ["official-export", "public-web", "vision-ui", "browser-session", "session-api", "private-api", "client-hook"];
const ACCOUNT_RANK: MethodAccount["kind"][] = ["none", "visitor", "own-browser", "own-phone", "own-app", "in-usuyo", "developer"];

/**
 * 内核取一条链接时的先后：平台插件的在前、通用网页的在后，各自里先层级低的，同一层先不带账号的，再照插件和清单的顺序。
 * 只排做了的；开没开、用不用得上由调用处看。
 */
export function autoOrder(link: Use[], platform: string): Use[] {
  const generic = (u: Use) => (u.plugin.platforms.some((p) => p.id === "web") ? 1 : 0);
  return link
    .filter((u) => u.method.built)
    .map((u, i) => ({ u, i }))
    .sort(
      (a, b) =>
        generic(a.u) - generic(b.u) ||
        ACCESS_RANK.indexOf(accessOn(a.u, platform)) - ACCESS_RANK.indexOf(accessOn(b.u, platform)) ||
        ACCOUNT_RANK.indexOf(a.u.method.account.kind) - ACCOUNT_RANK.indexOf(b.u.method.account.kind) ||
        a.i - b.i,
    )
    .map((x) => x.u);
}

/**
 * 「用哪种方法」现在是哪一项，照开关读出来：每种做了的都是它那一层的默认是「自动」；只有一种开着是那一种；
 * 别的样子（在旧的页面上一个个拨过的）照实说是「照你拨过的开关」。
 */
export type LinkChoice = { kind: "auto" } | { kind: "only"; key: string } | { kind: "mixed"; on: Use[] };

export function linkChoice(link: Use[], platform: string): LinkChoice {
  const built = link.filter((u) => u.method.built);
  if (built.every((u) => enabledOn(u, platform) === onByDefault(accessOn(u, platform)))) return { kind: "auto" };
  const on = built.filter((u) => enabledOn(u, platform));
  const only = on[0];
  return on.length === 1 && only ? { kind: "only", key: keyOf(only) } : { kind: "mixed", on };
}

/** 一个开关：开、关，或 null 去掉拨过的（回到各平台各自那一层的默认）。 */
export interface Switch {
  pluginId: string;
  methodId: string;
  enabled: boolean | null;
}

/** 每个平台上都是它那一层的默认：没拨过，或拨的正好和默认一样。 */
function atDefaults(u: Use): boolean {
  const m = u.method;
  return m.enabled === onByDefault(m.access) && (m.perPlatform ?? []).every((t) => t.enabled === onByDefault(t.access));
}

/**
 * 选了一项要拨的开关，只拨和现在不一样的：「自动」把拨过的收链接的方法的开关去掉（null），回到各平台各自那一层的默认，
 * 一个开关管几个平台、各平台默认不同的（yt-dlp 一类）也就各回各的；选一种是开它、关掉这个平台上别的收链接的方法。
 * 要开的排在前面，中途没成也不会一种都不开。只补上一样的、拿回存过的不碰。
 */
export function linkSwitches(link: Use[], platform: string, choice: string): Switch[] {
  const built = link.filter((u) => u.method.built);
  const id = (u: Use) => ({ pluginId: u.plugin.id, methodId: u.method.id });
  if (choice === "auto") return built.filter((u) => !atDefaults(u)).map((u) => ({ ...id(u), enabled: null }));
  const wanted = built
    .map((u) => ({ u, enabled: keyOf(u) === choice }))
    .filter((x) => x.enabled !== enabledOn(x.u, platform))
    .map((x) => ({ ...id(x.u), enabled: x.enabled }));
  return [...wanted.filter((s) => s.enabled), ...wanted.filter((s) => !s.enabled)];
}

/** 开关只有一种方法一个：插件还管别的平台时，拨它那几个平台也跟着变。回那几个平台的名字。 */
export function alsoMoves(u: Use, platform: string): string[] {
  return u.plugin.platforms.filter((p) => p.id !== platform).map((p) => p.name);
}

/** 拿回存过的那几种，各自的步骤照哪一类走。 */
export type SavedKind = "file" | "script" | "phone" | "link-list" | "list";

export function savedKind(u: Use, readers: Set<string>): SavedKind {
  if (readers.has(keyOf(u))) return "file";
  if (u.method.gets.delivers === "links") return "script";
  if (u.method.account.kind === "own-phone") return "phone";
  if (u.method.gets.link !== undefined) return "link-list";
  return "list";
}

/** 脚本说明里写读哪一页的那种说法，每种语言一个（`plugins.script.pagesPattern`），`{pages}` 处是要取出来的一段。 */
const PAGES_PATTERNS = [zhPlugins.script.pagesPattern, enPlugins.script.pagesPattern].map((p) => {
  const [before = "", after = ""] = p.split("{pages}");
  const esc = (x: string) => x.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(`${esc(before)}(.+?)${esc(after)}`, "i");
});

/**
 * 脚本里写的那一页：说明里「在你自己的……页上」那一段（例如「书签」「喜欢」），和 `@match` 的地址。
 * 都取自插件带的脚本，应用里不写死哪个平台。脚本用哪种语言写说明和界面用哪种语言无关，两种说法都认。
 */
export function scriptPages(text: string): { pages: string | null; matches: string[] } {
  const description = /^\/\/\s*@description\s+(.+)$/m.exec(text)?.[1] ?? "";
  const pages = PAGES_PATTERNS.map((r) => r.exec(description)?.[1]?.trim()).find((x) => x !== undefined) ?? null;
  const matches = [...text.matchAll(/^\/\/\s*@match\s+(\S+)/gm)].map((m) => m[1] ?? "").filter((m) => m !== "");
  return { pages, matches };
}
