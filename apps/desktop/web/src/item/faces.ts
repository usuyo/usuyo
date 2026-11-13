// 条目页上几面的话和摆法：每一份从哪来、还没转出来的那一句、按时间对齐几面。
import { t } from "../i18n";
import { formatBytes, formatClockTime } from "../i18n/format";
import type { Facet, FacetSource, Pending, Segment, TransformLayer } from "../api/endpoints";
import { dayText } from "./format";

export type CopyFacet = Extract<Facet, { layer: TransformLayer }>;

/** 这一面的名字（标题、标签里用）；按当前语言现取。 */
export const FACE_NAME: Record<Pending["layer"], string> = {
  get asr() {
    return t("item.layer.asr");
  },
  get ocr() {
    return t("item.layer.ocr");
  },
  get frames() {
    return t("item.layer.frames");
  },
  get video() {
    return t("item.layer.video");
  },
};

/** 一份的来源，写在那一份头上：「平台提供的字幕」「自动转写 · SenseVoice · 11月10日」。 */
export function sourceText(layer: TransformLayer, source: FacetSource): string {
  if (source.by === "platform") return layer === "asr" ? t("item.source.platformSubtitles") : t("item.source.platform");
  const verb = layer === "asr" ? t("item.source.autoTranscribed") : t("item.source.autoRecognized");
  return [verb, source.tool, source.at ? dayText(source.at) : ""].filter(Boolean).join(" · ");
}

/** 切换几份时按钮上的短名：平台的写「平台字幕」，机器的写工具名；同一个工具转过两次的带上日子，同一天的再带上几点几分。 */
export function copyLabel(copy: CopyFacet, all: CopyFacet[]): string {
  if (copy.source.by === "platform") return copy.layer === "asr" ? t("item.copy.platformSubtitles") : t("item.copy.platform");
  const tool = copy.source.tool ?? t("item.copy.automatic");
  const at = copy.source.at;
  const twins = all.filter((c) => c.source.by === "machine" && c.source.tool === copy.source.tool);
  if (twins.length < 2 || !at) return tool;
  const day = dayText(at);
  const sameDay = twins.some((c) => c !== copy && c.source.at !== undefined && dayText(c.source.at) === day);
  return sameDay ? `${tool} · ${day} ${formatClockTime(at)}` : `${tool} · ${day}`;
}

/** 字节写成「163 MB」「1.2 GB」（`i18n/format.ts`）。 */
export function sizeText(bytes: number): string {
  return formatBytes(bytes);
}

/**
 * 还没转出来的那一面只写一句实话（feature 003 §4.4 的表）：不写百分比，不写「未转」「待处理」。
 * 模型照它的 `name` 写；`downloading` 是那个模型正在下。
 */
export function pendingText(p: Pending, downloading = false): string {
  const m = p.model;
  switch (p.state) {
    case "later":
      return t(`item.pending.${p.layer}.later`);
    case "on-power":
      return t(`item.pending.${p.layer}.onPower`);
    case "now":
      return t(`item.pending.${p.layer}.now`);
    case "needs-model":
      if (!m) return t("item.pending.needsSomeModel");
      return downloading ? t("item.pending.downloading", { model: m.name, size: sizeText(m.size) }) : t(`item.pending.${p.layer}.needsModel`, { model: m.name, size: sizeText(m.size) });
    default:
      return t(`item.pending.${p.layer}.failed`, { reason: p.hint ?? t("common.reasonUnknown") });
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

/** 播到第 `sec` 秒时，亮哪一行：开始时间不晚于 `sec` 的最后一行。 */
export function currentRow(rows: AlignedRow[], sec: number): number {
  let k = -1;
  rows.forEach((r, i) => {
    if (r.sec <= sec + 0.01) k = i;
  });
  return k;
}
