// 获取插件的标签和停着的原因，换成平常的话。只是给你挑的信息，不拦什么。
import type { Access, FetchMethod, LoginNeed, MethodAccount, MethodCall, OutputKind, PluginPause, TransformJob, VisitorIdentity } from "../api/endpoints";

/** 转换插件做的几样，写在插件卡片上，条目页的「再转」里也用。 */
export const JOB_TEXT: Record<TransformJob, string> = {
  play: "合成能播的视频",
  keyframes: "取关键帧",
  asr: "口播转成文字",
  ocr: "认图里的字",
  frames: "认画面上的字",
};

export const ACCESS_TEXT: Record<Access, string> = {
  "official-export": "读平台给你的官方导出",
  "public-web": "不登录，读公开页面",
  "vision-ui": "看屏幕，替你点",
  "browser-session": "在登录着的网页里读",
  "session-api": "用你的登录调平台的接口",
  "private-api": "调平台没公开的接口",
  "client-hook": "读取或改动平台客户端本身",
};

/** 「会用到你的 X 账号」：站点名由内核给，插件声明的站点不在 `/sites` 里时写主机名。 */
export function loginText(need: LoginNeed, siteNames: string[]): string {
  const names = siteNames.join("、");
  switch (need) {
    case "none":
      return "不用账号";
    case "optional":
      return names ? `登录了${names}能多取一些` : "登录了能多取一些";
    default:
      return names ? `要登录你的${names}账号` : "要登录";
  }
}

/** 今天的只写时分，别的日子带上日期；十分钟以内的带上秒（照节奏几秒一条时有用）。 */
export function timeText(iso: string, now = new Date()): string {
  const d = new Date(iso);
  const two = (n: number) => n.toString().padStart(2, "0");
  const soon = Math.abs(d.getTime() - now.getTime()) < 10 * 60_000;
  const hm = `${two(d.getHours())}:${two(d.getMinutes())}${soon ? `:${two(d.getSeconds())}` : ""}`;
  const sameDay = d.toDateString() === now.toDateString();
  return sameDay ? hm : `${d.getMonth() + 1} 月 ${d.getDate()} 日 ${hm}`;
}

/**
 * 页面要访客身份时怎么说。`state` 是内核存着的那份的状态：没有就写怎么取，存着的就说换一份；
 * 页面手上没有这个状态时写 `要访客身份`。
 */
export function visitorText(state?: VisitorIdentity["state"]): string {
  if (state === undefined) return "要访客身份";
  return state === "missing" ? "要先取访客身份：在「获取插件」页的「访客身份」里取一份" : "这份访客身份不管用了，换一份再试";
}

const PAUSE_CODE: Record<string, string> = {
  "rate-limited": "平台说请求太频繁",
  blocked: "平台挡住了这个插件",
  "login-required": "要重新登录",
  "visitor-required": visitorText(),
  "upstream-changed": "平台改版了，插件读不到",
};

/** 停着的那一行：为什么、到什么时候。平台挡住的停一小时就自己再试。`platforms` 用来把范围写成平台名。 */
export function pauseText(p: PluginPause, platforms: { id: string; name: string }[], now = new Date()): string {
  if (p.byYou) return "你让它先别试：到你点「现在再试」为止，哪儿都不调它";
  const why = p.hint ?? (p.code ? (PAUSE_CODE[p.code] ?? p.code) : "停一会儿");
  // 只有一个平台的插件不用再说是哪个。
  const where = p.scope === "*" || platforms.length <= 1 ? "" : `（${platforms.find((x) => x.id === p.scope)?.name ?? p.scope}）`;
  return p.until ? `${why}${where} · ${timeText(p.until, now)} 自己再试` : `${why}${where} · 等你动手`;
}

/** 一种取法的账号用在哪，一句话。名字用内核给的 `where`（取自清单），没有时用 `names`（站点给人看的名字，没有站点时是平台名）。 */
export function accountText(account: MethodAccount, names: string[]): string {
  const who = account.where ?? names.join("、");
  const text: Record<MethodAccount["kind"], string> = {
    none: "不用账号",
    visitor: `不用账号，用${who || "站点"}给访客的身份`,
    "own-browser": `账号留在你自己的浏览器里：脚本在登录着的${who}网页里读，把链接交给 usuyo`,
    "own-phone": `账号留在你手机的${who} App 里`,
    "own-app": `账号留在这台电脑上的${who}客户端里`,
    developer: `用你自己申请的${who}开发者凭据，不用你的账号登录`,
    "in-usuyo": `要在 usuyo 里登录你的${who}账号`,
  };
  return text[account.kind];
}

/** 取一条时拿得到的几样，换成平常的话。 */
export const OUTPUT_TEXT: Record<OutputKind, string> = {
  text: "正文",
  html: "网页原样",
  images: "图",
  "media:video": "视频",
  "media:audio": "音频",
  subtitle: "字幕",
  meta: "标题和作者",
  comments: "评论",
};

/** 取得到什么：「取一条：正文、图、视频」「列出：书签、喜欢」「从一条链接列出」「链接交进来」。 */
export function getsText(gets: FetchMethod["gets"]): string[] {
  const list = gets.list ?? [];
  return [
    ...(gets.fetch && gets.fetch.length > 0 ? [`取一条：${gets.fetch.map((k) => OUTPUT_TEXT[k]).join("、")}`] : []),
    ...(list.length > 0 ? [`列出：${list.join("、")}`] : gets.link !== undefined ? [] : gets.list ? ["列出你存过的清单"] : []),
    ...(gets.link !== undefined ? [`从一条链接列出：${gets.link}`] : []),
    ...(gets.delivers === "links" ? ["链接交进来"] : []),
  ];
}

/** 能列出一批：有 `list`（写死的来源可以是空的），或从一条链接开始。 */
export function canList(m: FetchMethod): boolean {
  return m.gets.list !== undefined || m.gets.link !== undefined;
}

/**
 * 这种取法现在怎样，一句话：用不上的照内核的原因一句句说；用得上又没停着的说用得上。
 * 停着的另由 `pauseText` 说（内核的 `usable` 不看暂停），这里回 null。没做好的行上有标，也回 null。
 */
export function methodStatus(m: FetchMethod): string | null {
  if (!m.built) return null;
  if (!m.usable) return m.blockers.map((b) => b.hint).join("；");
  // 「先别试」整个插件的那句写在插件上，这里只短短说一句。
  if (m.pauses.some((p) => p.byYou)) return "你让它先别试";
  return m.pauses.length > 0 ? null : "现在能用";
}

/** 上一次调用：什么时候，没出错，或那一句错。 */
export function lastCallText(c: MethodCall, now = new Date()): string {
  if (c.ok) return `上次 ${timeText(c.at, now)} 用过，没出错`;
  const why = c.hint ?? (c.code ? (PAUSE_CODE[c.code] ?? c.code) : "没成");
  return `上次 ${timeText(c.at, now)} 没成：${why}`;
}
