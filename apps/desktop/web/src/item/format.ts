import { t } from "../i18n";
import { formatDay, formatList } from "../i18n/format";
import type { Facet, FacetLayer, Item, Position, SearchHit, Stored } from "../api/endpoints";

/** 只留文字的条目，移出去的视频和图现在在哪：还在回收站里（留几天），还是已经清掉了。 */
export interface MediaKeep {
  /** 回收站放多少天，`null` 是一直；还没取到时省略。 */
  days?: number | null;
  /** 回收站里已经没有这一条移出去的媒体了（到期或你清掉了）。 */
  gone: boolean;
}

/** 媒体位置向下取整：102.5 写 01:42，过一小时写 1:01:42。 */
export function clock(sec: number): string {
  const n = Math.max(0, Math.floor(sec));
  const h = Math.floor(n / 3600);
  const m = Math.floor((n % 3600) / 60);
  const s = n % 60;
  const mm = `${m < 10 ? "0" : ""}${m}`;
  const ss = `${s < 10 ? "0" : ""}${s}`;
  return h > 0 ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}

/** 图片 key 给人看的名字：p2 是「图 2」，cover 是「封面」。 */
export function imageName(key: string, keys: string[] = []): string {
  if (key === "cover") return t("item.image.cover");
  // 内核的图片 key 是内容的哈希，给人看时按它在这一条里的先后叫「图 N」。
  const at = keys.indexOf(key);
  if (at >= 0) return t("item.image.numbered", { n: at + 1 });
  const n = /^p(\d+)$/.exec(key)?.[1];
  return n ? t("item.image.numbered", { n }) : key;
}

export function positionText(p: Position): string {
  if (p.kind === "time") return clock(p.sec);
  return imageName(p.imageKey);
}

/** 每一层在界面上的名字；按当前语言现取（取值时才读语言文件）。 */
export const LAYER_NAME: Record<FacetLayer | SearchHit["layer"], string> = {
  get asr() {
    return t("item.layer.asr");
  },
  get ocr() {
    return t("item.layer.ocr");
  },
  get frames() {
    return t("item.layer.frames");
  },
  get text() {
    return t("item.layer.text");
  },
  get image() {
    return t("item.layer.image");
  },
  get video() {
    return t("item.layer.video");
  },
  get comments() {
    return t("item.layer.comments");
  },
  get note() {
    return t("item.layer.note");
  },
  get title() {
    return t("item.layer.title");
  },
  get author() {
    return t("item.layer.author");
  },
};

/** 回收站留几天，一整句：「30 天内可以恢复。」「可随时恢复。」；还没取到回收站的设置时只说可以恢复。 */
export function keepText(trashDays: number | null | undefined): string {
  if (trashDays === undefined) return t("item.keep.unknown");
  return trashDays === null ? t("item.keep.forever") : t("item.keep.days", { count: trashDays });
}

/** 两句接在一起：中文直接接，英文中间空一格。 */
export function sentences(first: string, second: string): string {
  return t("item.twoSentences", { first, second });
}

/** 「11月13日」「Nov 13」，不是今年的带年份（`i18n/format.ts`）。 */
export function dayText(iso: string): string {
  return formatDay(iso);
}

export const STORED_TEXT: Record<Stored, string> = {
  get complete() {
    return t("item.stored.complete");
  },
  get partial() {
    return t("item.stored.partial");
  },
  get link() {
    return t("item.stored.link");
  },
  get "text-only"() {
    return t("item.stored.textOnly");
  },
};

/** 「已保存」那一行：有哪些就写哪些。 */
export function savedText(item: Item, keep: MediaKeep): string {
  if (item.stored === "link") return t("item.saved.linkOnly");
  const parts: string[] = [];
  for (const f of item.facets) {
    if (f.layer === "video") {
      parts.push(item.stored === "text-only" ? t("item.saved.thumbnails") : t("item.saved.video"));
    } else if (f.layer === "image") parts.push(t("item.saved.images", { count: f.images.length }));
    else if (f.layer === "asr") {
      // 空的一份（没听到人说话）不算存下了口播。
      if (f.segments.length > 0) parts.push(f.source.by === "platform" ? t("item.saved.subtitles") : t("item.saved.transcript"));
    }
    else if (f.layer === "ocr") parts.push(t("item.saved.ocr"));
    else if (f.layer === "frames") parts.push(t("item.saved.frames"));
    else if (f.layer === "text") parts.push(t("item.saved.text"));
    else parts.push(t("item.saved.comments", { count: f.comments.length }));
  }
  // 同一层有几份（字幕和机器转的）只写一次。
  const list = formatList([...new Set(parts)]);
  if (item.stored === "text-only") {
    return keep.gone ? t("item.saved.textOnlyGone", { list }) : sentences(t("item.saved.textOnly", { list }), keepText(keep.days));
  }
  // 缺了什么写在后面的 `missing` 里。
  if (item.stored === "partial") return t("item.saved.partial", { list });
  return t("item.saved.complete", { list });
}

export function facetOf<L extends FacetLayer>(item: Item, layer: L): Extract<Facet, { layer: L }> | undefined {
  return item.facets.find((f): f is Extract<Facet, { layer: L }> => f.layer === layer);
}

/** 命中那句话，前面最多留 12 个字。 */
export function snippetParts(hit: SearchHit): { pre: string; mid: string; post: string } {
  const [at, end] = hit.match;
  const a0 = Math.max(0, at - 12);
  return {
    pre: (a0 > 0 ? "…" : "") + hit.snippet.slice(a0, at),
    mid: hit.snippet.slice(at, end),
    post: hit.snippet.slice(end),
  };
}

/** 开着的这一条从资料库里没了（回收站里到期、或清掉了）：条目页和地图旁的详情都说这句。 */
export function goneText(): string {
  return t("item.gone");
}
