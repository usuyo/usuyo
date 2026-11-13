// 获取插件的标签和停着的原因，换成平常的话。只是给你挑的信息，不拦什么。
// 话都在 `i18n/<语言>/plugins.ts`；这里的表用 getter，换了语言再读就是新语言。
import type { Access, FetchMethod, LoginNeed, MethodAccount, MethodCall, OutputKind, PluginPause, TransformJob, VisitorIdentity } from "../api/endpoints";
import { t, type MessageKey } from "../i18n";
import { formatClockTime, formatDayTime, formatList } from "../i18n/format";

/** 不带变量的一句。 */
type PlainKey = { [K in MessageKey]: Parameters<typeof t<K>>["length"] extends 1 ? K : never }[MessageKey];

/** 一张按键取话的表：读一项时才取，取的是当前语言。 */
function table<K extends string>(keys: Record<K, PlainKey>): Record<K, string> {
  const out: Record<K, string> = { ...keys };
  for (const [k, key] of Object.entries<PlainKey>(keys)) {
    Object.defineProperty(out, k, { enumerable: true, get: () => t(key) });
  }
  return out;
}

/** 转换插件做的几样，写在插件卡片上，条目页的「再转」里也用。 */
export const JOB_TEXT: Record<TransformJob, string> = table({
  play: "plugins.jobs.play",
  keyframes: "plugins.jobs.keyframes",
  asr: "plugins.jobs.asr",
  ocr: "plugins.jobs.ocr",
  frames: "plugins.jobs.frames",
});

export const ACCESS_TEXT: Record<Access, string> = table({
  "official-export": "plugins.access.officialExport",
  "public-web": "plugins.access.publicWeb",
  "vision-ui": "plugins.access.visionUi",
  "browser-session": "plugins.access.browserSession",
  "session-api": "plugins.access.sessionApi",
  "private-api": "plugins.access.privateApi",
  "client-hook": "plugins.access.clientHook",
});

/** 每一层是什么意思，一句话。只是给你挑的信息。 */
export const TIER_TEXT: Record<Access, string> = table({
  "official-export": "plugins.tier.officialExport",
  "public-web": "plugins.tier.publicWeb",
  "vision-ui": "plugins.tier.visionUi",
  "browser-session": "plugins.tier.browserSession",
  "session-api": "plugins.tier.sessionApi",
  "private-api": "plugins.tier.privateApi",
  "client-hook": "plugins.tier.clientHook",
});

/** 「会用到你的 X 账号」：站点名由内核给，插件声明的站点不在 `/sites` 里时写主机名。 */
export function loginText(need: LoginNeed, siteNames: string[]): string {
  const names = formatList(siteNames);
  switch (need) {
    case "none":
      return t("plugins.login.none");
    case "optional":
      return names ? t("plugins.login.optional", { names }) : t("plugins.login.optionalAny");
    default:
      return names ? t("plugins.login.required", { names }) : t("plugins.login.requiredAny");
  }
}

/** 今天的只写时分，别的日子带上日期；十分钟以内的带上秒（照节奏几秒一条时有用）。按当前语言写（`i18n/format.ts`）。 */
export function timeText(iso: string, now = new Date()): string {
  const d = new Date(iso);
  const soon = Math.abs(d.getTime() - now.getTime()) < 10 * 60_000;
  return d.toDateString() === now.toDateString() ? formatClockTime(iso, soon) : formatDayTime(iso, now);
}

/**
 * 页面要访客身份时怎么说。`state` 是内核存着的那份的状态：没有就写怎么取，存着的就说换一份；
 * 页面手上没有这个状态时只说需要。
 */
export function visitorText(state?: VisitorIdentity["state"]): string {
  if (state === undefined) return t("plugins.visitor.needed");
  return state === "missing" ? t("plugins.visitor.missing") : t("plugins.visitor.expired");
}

/** 停着、上次没成的码换成一句；不认识的码照原样。 */
function reasonOf(code: string): string {
  switch (code) {
    case "rate-limited":
      return t("plugins.reason.rateLimited");
    case "blocked":
      return t("plugins.reason.blocked");
    case "login-required":
      return t("plugins.reason.loginRequired");
    case "visitor-required":
      return visitorText();
    case "upstream-changed":
      return t("plugins.reason.upstreamChanged");
    default:
      return code;
  }
}

/** 停着的那一行：为什么、到什么时候。平台挡住的停一小时就自己再试。`platforms` 用来把范围写成平台名。 */
export function pauseText(p: PluginPause, platforms: { id: string; name: string }[], now = new Date()): string {
  if (p.byYou) return t("plugins.pause.held");
  const why = p.hint ?? (p.code ? reasonOf(p.code) : t("plugins.reason.paused"));
  // 只有一个平台的插件不用再说是哪个。
  const reason = p.scope === "*" || platforms.length <= 1 ? why : t("plugins.pause.scoped", { reason: why, platform: platforms.find((x) => x.id === p.scope)?.name ?? p.scope });
  return p.until ? t("plugins.pause.until", { reason, time: timeText(p.until, now) }) : t("plugins.pause.manual", { reason });
}

/** 一种取法的账号用在哪，一句话。名字用内核给的 `where`（取自清单），没有时用 `names`（站点给人看的名字，没有站点时是平台名）。 */
export function accountText(account: MethodAccount, names: string[]): string {
  const site = account.where ?? formatList(names);
  const text: Record<MethodAccount["kind"], () => string> = {
    none: () => t("plugins.account.none"),
    visitor: () => (site ? t("plugins.account.visitor", { site }) : t("plugins.account.visitorAny")),
    "own-browser": () => t("plugins.account.ownBrowser", { site }),
    "own-phone": () => t("plugins.account.ownPhone", { site }),
    "own-app": () => t("plugins.account.ownApp", { site }),
    developer: () => t("plugins.account.developer", { site }),
    "in-usuyo": () => t("plugins.account.inUsuyo", { site }),
  };
  return text[account.kind]();
}

/** 取一条时拿得到的几样，换成平常的话。 */
export const OUTPUT_TEXT: Record<OutputKind, string> = table({
  text: "plugins.output.text",
  html: "plugins.output.html",
  images: "plugins.output.images",
  "media:video": "plugins.output.video",
  "media:audio": "plugins.output.audio",
  subtitle: "plugins.output.subtitle",
  meta: "plugins.output.meta",
  comments: "plugins.output.comments",
});

/** 取得到什么：「单条：正文、图片和视频」「列出：书签和喜欢」「从链接列出：…」「由浏览器发送链接」。 */
export function getsText(gets: FetchMethod["gets"]): string[] {
  const list = gets.list ?? [];
  return [
    ...(gets.fetch && gets.fetch.length > 0 ? [t("plugins.gets.fetch", { kinds: formatList(gets.fetch.map((k) => OUTPUT_TEXT[k])) })] : []),
    ...(list.length > 0 ? [t("plugins.gets.list", { lists: formatList(list) })] : gets.link !== undefined ? [] : gets.list ? [t("plugins.gets.listSaved")] : []),
    ...(gets.link !== undefined ? [t("plugins.gets.link", { link: gets.link })] : []),
    ...(gets.delivers === "links" ? [t("plugins.gets.delivers")] : []),
  ];
}

/** 几句并排写成一行（内核的几条原因、取得到的几样）：中文用「；」，英文用「; 」。 */
export function clauses(parts: string[]): string {
  return parts.join(t("common.clauseSeparator"));
}

/** 能列出一批：有 `list`（写死的来源可以是空的），或从一条链接开始。 */
export function canList(m: FetchMethod): boolean {
  return m.gets.list !== undefined || m.gets.link !== undefined;
}

/**
 * 这种取法现在怎样，一句话：用不上的照内核的原因一句句说（停着的也是一条原因，写到什么时候）；用得上的说用得上。
 * 用得上又有暂停的（通用网页按网站停的）另由 `pauseText` 说，这里回 null。没做好的行上有标，也回 null。
 */
export function methodStatus(m: FetchMethod): string | null {
  if (!m.built) return null;
  if (!m.usable) return clauses(m.blockers.map((b) => b.hint));
  return m.pauses.length > 0 ? null : t("plugins.usable");
}

/** 停着已经写成一条原因（`paused`）的，暂停不再另写一行。 */
export function pausedAsBlocker(m: FetchMethod): boolean {
  return m.blockers.some((b) => b.code === "paused");
}

/** 上一次调用：什么时候，没出错，或那一句错。 */
export function lastCallText(c: MethodCall, now = new Date()): string {
  const time = timeText(c.at, now);
  if (c.ok) return t("plugins.lastCall.ok", { time });
  const reason = c.hint ?? (c.code ? reasonOf(c.code) : t("plugins.reason.failed"));
  return t("plugins.lastCall.failed", { time, reason });
}

// —— 平台页（一个平台一页）上的话 ——

export const LOGIN_STATE: Record<"missing" | "present" | "expired", string> = table({
  missing: "plugins.loginState.missing",
  present: "plugins.loginState.present",
  expired: "plugins.loginState.expired",
});

export const VISITOR_STATE: Record<"missing" | "present" | "expired", string> = table({
  missing: "plugins.visitorState.missing",
  present: "plugins.visitorState.present",
  expired: "plugins.visitorState.expired",
});

/** 条目页上那一句（`item/Viewer.tsx`）。获取插件页自己的话直接用 `t("plugins.…")`。 */
export const PAGE = {
  itemPlatformLink: (platform: string) => t("plugins.itemPlatformLink", { platform }),
};
