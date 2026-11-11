// 一次导入走到哪了，用平常的话说。只说事实：收下了几条、为什么在等、下一条大概什么时候；不说还剩多少。
import type { ImportView, ImportWaiting } from "../api/endpoints";
import { timeText } from "../plugins/words";

/** 记录栏里那一行。 */
export function stateLine(v: ImportView): string {
  switch (v.state) {
    case "listing":
      return `在列，列出 ${v.preview.new + v.preview.stored + v.preview.trashed} 条`;
    case "ready":
      return `列完了，等你看一眼 · 新的 ${v.preview.new} 条`;
    case "taking":
      return `收下了 ${v.taken} 条 · 在收`;
    case "held":
      return `收下了 ${v.taken} 条 · 停着`;
    default:
      return `收下了 ${v.taken} 条${v.notStored > 0 ? `，${v.notStored} 条内容没取到` : ""}`;
  }
}

/** 为什么在等。`platform` 是平台的显示名，有就写进去。 */
export function waitingText(w: ImportWaiting, platform: string | undefined, now = new Date()): string {
  const at = w.until ? timeText(w.until, now) : null;
  const on = platform ? `${platform}的` : "平台的";
  switch (w.code) {
    case "pacing":
      return at ? `按${on}节奏，下一条大概 ${at}` : `按${on}节奏，隔一会儿收下一条`;
    case "daily-limit":
      return at ? `今天批量的次数用完了，${at} 接着收` : "今天批量的次数用完了，明天接着收";
    case "held":
      return "你让这个插件先别试；在「获取插件」里点「现在再试」就接着收";
    case "batch-off":
      return "这个平台不做批量导入";
    case "login-required":
      return `要重新登录${platform ?? ""}：登录好了点「接着导入」`;
    default: {
      const why = w.hint ?? w.code;
      return at ? `${why} · ${at} 自己再试` : `${why} · 等你点「接着导入」`;
    }
  }
}
