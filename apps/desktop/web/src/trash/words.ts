// 回收站页上的话：说清楚是什么、什么时候清除，不催。
import type { TrashEntry } from "../api/endpoints";
import { dayText } from "../item/format";

/** 页面标题下那一句。 */
export function introText(trashDays: number | null): string {
  return trashDays === null
    ? "删掉的条目和只留文字时移出来的图和视频放在这里，现在设成一直留着，不会自己清除。"
    : `删掉的条目和只留文字时移出来的图和视频，先在这里放 ${trashDays} 天，到期就彻底清除，不留记录。`;
}

/** 标题下那一行：哪个平台、谁的，这一项是什么。 */
export function whatText(e: TrashEntry): string {
  const from = e.author ? `${e.platform.name} · ${e.author}` : e.platform.name;
  const what =
    e.reason === "keep-text"
      ? "只留文字时移出来的图和视频，条目和文字还在"
      : e.reason === "undo"
        ? "撤销收下时移进来的整条"
        : "删掉的整条";
  return `${from}　·　${what}`;
}

/** 什么时候进来的。 */
export function trashedText(e: TrashEntry): string {
  return `${dayText(e.trashedAt)}${e.reason === "delete" ? "删掉" : "移进来"}`;
}

/** 什么时候清除。到期检查几个小时一次，过了时间还在就说下一次检查时清除。 */
export function clearsText(e: TrashEntry, now = new Date()): string {
  if (e.expiresAt === undefined) return "一直留着";
  if (new Date(e.expiresAt).getTime() <= now.getTime()) return "到期了，下一次检查时清除";
  return `${dayText(e.expiresAt)}清除`;
}

/** 点了「立即清除」以后，确认前那一句。 */
export function askText(e: TrashEntry): string {
  return e.reason === "keep-text" ? "清除以后找不回来；条目和文字不受影响" : "清除以后找不回来，引用它的地方只留原话";
}

/** 做完以后页面上那一句。 */
export function doneText(e: TrashEntry, did: "restore" | "purge"): string {
  if (did === "purge") return `清除了「${e.title}」`;
  return e.reason === "keep-text" ? `图和视频放回了「${e.title}」` : `「${e.title}」拿回来了，回到了地图上`;
}
