import type { Facet, FacetLayer, Item, Position, SearchHit, Stored } from "../api/endpoints";

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
  text: "作者写的",
  image: "画面",
  video: "画面",
  comments: "评论",
  note: "记一句",
  title: "标题",
  author: "作者",
};

export function dayText(iso: string): string {
  const d = new Date(iso);
  return `${d.getMonth() + 1} 月 ${d.getDate()} 日`;
}

export const STORED_TEXT: Record<Stored, string> = {
  complete: "都存好了",
  link: "只有链接",
  "text-only": "只留文字",
};

/** 「存下了」那一行：有哪些就写哪些。 */
export function savedText(item: Item): string {
  if (item.stored === "link") return "只有链接。";
  const parts: string[] = [];
  for (const f of item.facets) {
    if (f.layer === "video") {
      if (item.stored !== "text-only") parts.push(`视频${f.quality ? ` ${f.quality}` : ""}`);
      if (f.cover && item.stored !== "text-only") parts.push("封面");
      if (item.stored === "text-only") parts.push("画面缩略图");
    } else if (f.layer === "image") parts.push(`${f.images.length} 张图`);
    else if (f.layer === "asr") parts.push(f.by === "platform" ? "字幕" : "口播文字");
    else if (f.layer === "ocr") parts.push("图里的字");
    else if (f.layer === "text") parts.push("作者写的字");
    else parts.push(`${f.comments.length} 条评论`);
  }
  const list = parts.join("、");
  return item.stored === "text-only" ? `${list}。视频和封面在回收站，30 天内能恢复` : `${list} — 都存好了`;
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
