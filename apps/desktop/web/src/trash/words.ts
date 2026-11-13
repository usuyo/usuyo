// 废纸篓页上的话：说清楚是什么、什么时候删除，不催。
import type { TrashEntry } from "../api/endpoints";
import { t } from "../i18n";
import { dayText } from "../item/format";

/** 页面标题下那一句。 */
export function introText(trashDays: number | null): string {
  return trashDays === null ? t("trash.introForever") : t("trash.introDays", { count: trashDays });
}

/** 标题下那一行：哪个平台、谁的，这一项是什么。 */
export function whatText(e: TrashEntry): string {
  const what = t(e.reason === "keep-text" ? "trash.keepText" : e.reason === "undo" ? "trash.undo" : "trash.deleted");
  return e.author ? t("trash.fromAuthor", { platform: e.platform.name, author: e.author, what }) : t("trash.from", { platform: e.platform.name, what });
}

/** 什么时候进来的。 */
export function trashedText(e: TrashEntry): string {
  return t(e.reason === "delete" ? "trash.deletedOn" : "trash.movedOn", { day: dayText(e.trashedAt) });
}

/** 什么时候删除。到期检查几个小时一次，过了时间还在就说下一次检查时删除。 */
export function clearsText(e: TrashEntry, now = new Date()): string {
  if (e.expiresAt === undefined) return t("trash.keptForever");
  if (new Date(e.expiresAt).getTime() <= now.getTime()) return t("trash.due");
  return t("trash.deletesOn", { day: dayText(e.expiresAt) });
}

/** 点了「立即删除」以后，确认前那一句。 */
export function askText(e: TrashEntry): string {
  return e.reason === "keep-text" ? t("trash.askKeepText") : t("trash.askItem");
}

/** 做完以后页面上那一句。 */
export function doneText(e: TrashEntry, did: "restore" | "purge"): string {
  if (did === "purge") return t("trash.purged", { title: e.title });
  return t(e.reason === "keep-text" ? "trash.restoredMedia" : "trash.restored", { title: e.title });
}
