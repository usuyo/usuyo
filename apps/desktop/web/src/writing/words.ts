// 引用和原话的几句话：在哪一层的哪一处、怎么跳回去、找回得怎样。主题页和成稿共用。
import type { Citation, FacetLayer, PageBlock, PagePatch, Position } from "../api/endpoints";
import { t } from "../i18n";
import { formatDay } from "../i18n/format";
import { LAYER_NAME, positionText } from "../item/format";
import type { ItemSearch } from "../router";

/** 「口播 02:30」「图里的字 · 图 2」「作者写的」：哪一层，有位置的带上位置。 */
export function whereText(layer: FacetLayer, position?: Position): string {
  const name = LAYER_NAME[layer];
  if (!position) return name;
  return position.kind === "time" ? `${name} ${positionText(position)}` : `${name} · ${positionText(position)}`;
}

/** 跳回原样那一处时条目页要带的：那一秒，或那一张图。 */
export function jumpOf(position?: Position): ItemSearch {
  if (!position) return {};
  return position.kind === "time" ? { t: position.sec } : { img: position.imageKey };
}

/** 找回得不一样时写的那一句；照原样找回了不写。 */
export function foundText(c: Citation): string | null {
  switch (c.found) {
    case "changed":
      return c.now ? t("writing.found.changedTo", { now: c.now }) : t("writing.found.changed");
    case "missing":
      return t("writing.found.missing");
    case "gone":
      return t("writing.found.gone");
    default:
      return null;
  }
}

/** 引用下面那一行出处：标题、作者、日期，日期按语言写（「8月14日」「Aug 14」），不是今年的带上年份。 */
export function citeSource(c: Pick<Citation, "title" | "author" | "date">, now = new Date()): string {
  const [y = 0, m = 1, d = 1] = c.date.split("-").map(Number);
  // 只有日期：按本地的那一天写，不经时区换算。
  const day = formatDay(new Date(y, m - 1, d).toISOString(), now);
  return [c.title, c.author, day].filter((x) => x !== undefined && x !== "").join(" · ");
}

/** 页上的一处改动：改一段的字（`id` 是 null 时在页尾写新的一段），或从一段里拿掉一处引用。 */
export type PageEdit = { kind: "text"; id: string | null; text: string } | { kind: "uncite"; id: string; label: string };

/**
 * 改完以后整页交回去的样子。改成空、又不带引用的段去掉；新写的段是空的就不加。
 * 别的段原样交回，顺序不变。
 */
export function pageAfter(blocks: PageBlock[], edit: PageEdit): PagePatch {
  const keep = (b: Omit<PageBlock, "id"> & { id?: string }) => b.text.trim() !== "" || b.cites.length > 0;
  if (edit.kind === "uncite") {
    return { blocks: blocks.map((b) => (b.id === edit.id ? { ...b, cites: b.cites.filter((c) => c !== edit.label) } : b)).filter(keep) };
  }
  const text = edit.text.trim();
  if (edit.id === null) return { blocks: text === "" ? blocks : [...blocks, { kind: "text", text, cites: [] }] };
  return { blocks: blocks.map((b) => (b.id === edit.id ? { ...b, text } : b)).filter(keep) };
}
