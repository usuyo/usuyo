// 一次导入的状态。只说事实：添加了几条、为什么在等、下一条大概什么时候；不说还剩多少。
import type { ImportView, ImportWaiting, VisitorIdentity } from "../api/endpoints";
import { t } from "../i18n";
import { timeText, visitorText } from "../plugins/words";

/** 记录栏里那一行。 */
export function stateLine(v: ImportView): string {
  switch (v.state) {
    case "listing": {
      const count = v.preview.new + v.preview.stored + v.preview.trashed;
      return v.kind === "file" ? t("imports.state.listingFile", { count }) : t("imports.state.listingSource", { count });
    }
    case "ready":
      return v.kind === "file" ? t("imports.state.readyFile", { count: v.preview.new }) : t("imports.state.readySource", { count: v.preview.new });
    case "taking":
      return t("imports.state.taking", { count: v.taken });
    case "held":
      return t("imports.state.held", { count: v.taken });
    default:
      return v.notStored > 0 ? t("imports.state.doneMissing", { count: v.taken, missing: v.notStored }) : t("imports.state.done", { count: v.taken });
  }
}

/** 为什么在等。`platform` 是平台的显示名，有就写进去；`visitor` 是这个插件的访客身份存着的状态，页面有就给。 */
export function waitingText(
  w: ImportWaiting,
  platform: string | undefined,
  visitor?: VisitorIdentity["state"],
  now = new Date(),
): string {
  const time = w.until ? timeText(w.until, now) : null;
  switch (w.code) {
    case "pacing":
      if (time) return platform ? t("imports.waiting.pacingAt", { platform, time }) : t("imports.waiting.pacingAtAny", { time });
      return platform ? t("imports.waiting.pacing", { platform }) : t("imports.waiting.pacingAny");
    case "daily-limit":
      return time ? t("imports.waiting.dailyAt", { time }) : t("imports.waiting.daily");
    case "held":
      return t("imports.waiting.held");
    case "batch-off":
      return t("imports.waiting.batchOff");
    case "login-required":
      return platform ? t("imports.waiting.login", { platform }) : t("imports.waiting.loginAny");
    case "visitor-required":
      return t("imports.waiting.visitor", { visitor: visitorText(visitor) });
    default: {
      const why = w.hint ?? w.code;
      return time ? t("imports.waiting.otherAt", { why, time }) : t("imports.waiting.other", { why });
    }
  }
}

/** 挑文件时过滤用的后缀：「某站-*.zip」只认得出 `.zip`，文件名的样子由插件再看。 */
export function acceptOf(patterns: string[]): string[] {
  return [
    ...new Set(
      patterns.flatMap((p) => {
        const ext = /\.[A-Za-z0-9]+$/.exec(p)?.[0];
        return ext ? [ext.toLowerCase()] : [];
      }),
    ),
  ];
}
