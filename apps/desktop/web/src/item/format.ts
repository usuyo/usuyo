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
export function imageName(key: string): string {
  if (key === "cover") return "封面";
  const n = /^p(\d+)$/.exec(key)?.[1];
  return n ? `图 ${n}` : key;
}

export function positionText(p: Position): string {
  if (p.kind === "time") return clock(p.sec);
  return imageName(p.imageKey);
}

export const LAYER_NAME: Record<FacetLayer | SearchHit["layer"], string> = {
  asr: "口播",
  ocr: "图里的字",
  frames: "画面上的字",
  text: "作者写的",
  image: "画面",
  video: "画面",
  comments: "评论",
  note: "记一句",
  title: "标题",
  author: "作者",
};

/** 回收站留几天：「30 天内能恢复」「一直能恢复」；还没取到回收站的设置时只说能恢复。 */
export function keepText(trashDays: number | null | undefined): string {
  if (trashDays === undefined) return "能恢复";
  return trashDays === null ? "一直能恢复" : `${trashDays} 天内能恢复`;
}

export function dayText(iso: string): string {
  const d = new Date(iso);
  return `${d.getMonth() + 1} 月 ${d.getDate()} 日`;
}

export const STORED_TEXT: Record<Stored, string> = {
  complete: "都存好了",
  partial: "存下了一部分",
  link: "只有链接",
  "text-only": "只留文字",
};

/** 「存下了」那一行：有哪些就写哪些。 */
export function savedText(item: Item, keep: MediaKeep): string {
  if (item.stored === "link") return "只有链接。";
  const parts: string[] = [];
  for (const f of item.facets) {
    if (f.layer === "video") {
      if (item.stored !== "text-only") parts.push(`视频${f.quality ? ` ${f.quality}` : ""}`);
      if (f.cover && item.stored !== "text-only") parts.push("封面");
      if (item.stored === "text-only") parts.push("画面缩略图");
    } else if (f.layer === "image") parts.push(`${f.images.length} 张图`);
    else if (f.layer === "asr") {
      // 空的一份（没听到人说话）不算存下了口播。
      if (f.segments.length > 0) parts.push(f.source.by === "platform" ? "字幕" : "口播文字");
    }
    else if (f.layer === "ocr") parts.push("图里的字");
    else if (f.layer === "frames") parts.push("画面上的字");
    else if (f.layer === "text") parts.push("作者写的字");
    else parts.push(`${f.comments.length} 条评论`);
  }
  // 同一层有几份（字幕和机器转的）只写一次。
  const list = [...new Set(parts)].join("、");
  if (item.stored === "text-only") {
    return keep.gone ? `${list}。视频和封面已经从回收站清掉了` : `${list}。视频和封面在回收站，${keepText(keep.days)}`;
  }
  // 缺了什么写在后面的 `missing` 里。
  if (item.stored === "partial") return `${list}。`;
  return `${list} — 都存好了`;
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
export const GONE_TEXT = "这一条已经不在资料库里了。";
