// 成稿上的几句话：是哪一种、什么时候改的；搜到的哪些能当原话引。
import type { DraftKind, SearchHit, SegmentRef } from "../api/endpoints";
import { t } from "../i18n";
import { dayText } from "../item/format";

const KINDS: DraftKind[] = ["article", "post", "script", "other"];

export function kindText(kind: DraftKind): string {
  switch (kind) {
    case "article":
      return t("drafts.kind.article");
    case "post":
      return t("drafts.kind.post");
    case "script":
      return t("drafts.kind.script");
    default:
      return t("drafts.kind.misc");
  }
}

/** 「类型」那一组的选项，按当前语言现写。 */
export function kindChoices(): { id: DraftKind; text: string }[] {
  return KINDS.map((id) => ({ id, text: kindText(id) }));
}

/** 「11月11日编辑」；今天的写「今天编辑」。 */
export function editedText(iso: string, now = new Date()): string {
  return new Date(iso).toDateString() === now.toDateString() ? t("drafts.editedToday") : t("drafts.editedOn", { day: dayText(iso) });
}

/** 搜到的里头能当一段原话引的：各层文字里的那一句；标题、作者、你记的一句不算。 */
export function quotableOf(hits: SearchHit[]): SegmentRef[] {
  return hits.flatMap((h) =>
    h.layer === "title" || h.layer === "author" || h.layer === "note" || h.layer === "image" || h.layer === "video"
      ? []
      : [{ itemId: h.itemId, title: h.title, platform: h.platform, layer: h.layer, text: h.snippet, ...(h.position ? { position: h.position } : {}) }],
  );
}
