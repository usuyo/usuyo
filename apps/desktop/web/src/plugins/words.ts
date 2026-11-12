// 获取插件的标签和停着的原因，换成平常的话。只是给你挑的信息，不拦什么。
import type { Access, FetchMethod, LoginNeed, MethodAccount, PluginPause, SiteLogin, VisitorIdentity } from "../api/endpoints";

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

/** 用到的账号现在怎样：几个站点里有过期的先说过期。 */
export function loginStateText(states: SiteLogin["login"][]): string {
  if (states.includes("expired")) return "登录过期了";
  if (states.length > 0 && states.every((s) => s === "present")) return "现在登录着";
  return "还没登录";
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

/** 一种取法的账号用在哪，一句话。`names` 是站点给人看的名字（`/sites`、`/visitors` 里的）。 */
export function accountText(account: MethodAccount, names: string[]): string {
  const who = account.where ?? names.join("、");
  switch (account.kind) {
    case "none":
      return "不用账号";
    case "visitor":
      return `不用账号，用${names.join("、") || "站点"}给访客的身份`;
    case "own-browser":
      return `账号留在你自己的浏览器里：脚本在登录着的${who}网页里读，把链接交给 usuyo`;
    case "own-phone":
      return `账号留在你手机的${who} App 里`;
    case "developer":
      return `用你自己申请的${who}开发者凭据，不用你的账号登录`;
    default:
      return `要在 usuyo 里登录你的${who}账号`;
  }
}

/** 取得到什么：「正文、图、视频」「列出：书签、点过赞的」「由别处把链接交进来」。 */
export function getsText(gets: FetchMethod["gets"]): string[] {
  return [
    ...(gets.fetch && gets.fetch.length > 0 ? [`取一条：${gets.fetch.join("、")}`] : []),
    ...(gets.list && gets.list.length > 0 ? [`列出：${gets.list.join("、")}`] : []),
    ...(gets.delivers === "links" ? ["链接交进来"] : []),
  ];
}
