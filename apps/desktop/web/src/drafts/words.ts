// 成稿上的几句话：是哪一种、什么时候改的；搜到的哪些能当原话引。
import type { DraftKind, SearchHit, SegmentRef } from "../api/endpoints";
import { dayText } from "../item/format";

export const KIND_CHOICES: { id: DraftKind; text: string }[] = [
  { id: "article", text: "文章" },
  { id: "post", text: "帖子" },
  { id: "script", text: "脚本" },
  { id: "other", text: "别的" },
];

export function kindText(kind: DraftKind): string {
  return KIND_CHOICES.find((k) => k.id === kind)?.text ?? kind;
}

/** 「11 月 11 日改过」；今天的写「今天改过」。 */
export function editedText(iso: string, now = new Date()): string {
  return new Date(iso).toDateString() === now.toDateString() ? "今天改过" : `${dayText(iso)}改过`;
}

/** 搜到的里头能当一段原话引的：各层文字里的那一句；标题、作者、你记的一句不算。 */
export function quotableOf(hits: SearchHit[]): SegmentRef[] {
  return hits.flatMap((h) =>
    h.layer === "title" || h.layer === "author" || h.layer === "note" || h.layer === "image" || h.layer === "video"
      ? []
      : [{ itemId: h.itemId, title: h.title, platform: h.platform, layer: h.layer, text: h.snippet, ...(h.position ? { position: h.position } : {}) }],
  );
}
