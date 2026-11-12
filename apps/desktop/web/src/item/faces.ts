// 条目页上几面的话和摆法：每一份从哪来、还没转出来的那一句、按时间对齐几面。
import type { Facet, FacetSource, Pending, Segment, TransformLayer, TransformsView } from "../api/endpoints";
import { dayText } from "./format";

export type CopyFacet = Extract<Facet, { layer: TransformLayer }>;

/** 这一面在句子里叫什么。 */
export const FACE_NAME: Record<Pending["layer"], string> = { asr: "口播", ocr: "图里的字", frames: "画面上的字", video: "画面" };

/** 一份的来源，写在那一份头上：「平台给的字幕」「机器转的 · SenseVoice · 11 月 10 日」。 */
export function sourceText(layer: TransformLayer, source: FacetSource): string {
  if (source.by === "platform") return layer === "asr" ? "平台给的字幕" : "平台给的";
  const verb = layer === "asr" ? "机器转的" : "机器认的";
  return [verb, source.tool, source.at ? dayText(source.at) : ""].filter(Boolean).join(" · ");
}

/** 切换几份时按钮上的短名：平台的写「平台字幕」，机器的写工具名；同一个工具转过两次的带上日子。 */
export function copyLabel(copy: CopyFacet, all: CopyFacet[]): string {
  if (copy.source.by === "platform") return copy.layer === "asr" ? "平台字幕" : "平台给的";
  const tool = copy.source.tool ?? "机器";
  const twins = all.filter((c) => c.source.by === "machine" && c.source.tool === copy.source.tool).length;
  return twins > 1 && copy.source.at ? `${tool} · ${dayText(copy.source.at)}` : tool;
}

/** 字节写成「163 MB」「0.9 GB」。 */
export function sizeText(bytes: number): string {
  if (bytes >= 950_000_000) return `${(bytes / 1e9).toFixed(1)} GB`;
  if (bytes >= 1_000_000) return `${Math.round(bytes / 1e6)} MB`;
  return `${Math.max(1, Math.round(bytes / 1e3))} KB`;
}

/** 模型给人看的名字：`/transforms` 里有就用那里的，没有就用文件名。 */
export function modelLabel(name: string, transforms: TransformsView | undefined): string {
  for (const p of transforms?.plugins ?? []) {
    const m = p.models.find((x) => x.name === name);
    if (m) return m.label;
  }
  return name;
}

/**
 * 还没转出来的那一面只写一句实话（feature 003 §4.4 的表）：不写百分比，不写「未转」「待处理」。
 * `downloading` 是那个模型正在下。
 */
export function pendingText(p: Pending, label: string, downloading = false): string {
  const face = FACE_NAME[p.layer];
  switch (p.state) {
    case "later":
      return `${face}还没转出来，会慢慢转。`;
    case "on-power":
      return `${face}还没转出来，接上电源时会转。`;
    case "now":
      return `在转${face}。`;
    case "needs-model": {
      const size = p.model ? `，约 ${sizeText(p.model.size)}` : "";
      return downloading ? `在下载 ${label} 模型${size}。下好了这一条会接着转。` : `转${face}要先下载 ${label} 模型${size}，只下一次。`;
    }
    default:
      return `${face}没转出来：${p.hint ?? "原因没写"}`;
  }
}

/** 按时间对齐的一行：从 `sec` 开始，每一栏是这一段时间里开始的那几句（可以没有）。 */
export interface AlignedRow {
  sec: number;
  cells: Segment[][];
}

const startOf = (s: Segment) => (s.position?.kind === "time" ? s.position.sec : null);

/**
 * 几栏按时间对齐：每一句开头的时间各起一行，相隔不到 `mergeSec` 秒的并成一行，
 * 这样同一时刻说的和画面上写的落在同一行，一眼看得出对不对得上。没有时间的句子不进来。
 */
export function alignRows(lanes: Segment[][], mergeSec = 3): AlignedRow[] {
  const starts = [...new Set(lanes.flatMap((l) => l.map(startOf).filter((x): x is number => x !== null)))].sort((a, b) => a - b);
  const rowStarts: number[] = [];
  for (const sec of starts) {
    const last = rowStarts[rowStarts.length - 1];
    if (last === undefined || sec - last >= mergeSec) rowStarts.push(sec);
  }
  return rowStarts.map((sec, i) => {
    const next = rowStarts[i + 1] ?? Number.POSITIVE_INFINITY;
    return {
      sec,
      cells: lanes.map((l) =>
        l.filter((s) => {
          const at = startOf(s);
          return at !== null && at >= sec && at < next;
        }),
      ),
    };
  });
}

/** 播到第 `t` 秒时，亮哪一行：开始时间不晚于 `t` 的最后一行。 */
export function currentRow(rows: AlignedRow[], t: number): number {
  let k = -1;
  rows.forEach((r, i) => {
    if (r.sec <= t + 0.01) k = i;
  });
  return k;
}
