import { useEffect, useRef, type ReactNode } from "react";
import { Button as AriaButton } from "react-aria-components";
import type { Pending, Segment } from "../api/endpoints";
import { reducedMotion } from "../motion/clock";
import { alignRows, currentRow, FACE_NAME, type CopyFacet } from "./faces";
import { PendingLine } from "./FaceHead";
import { clock } from "./format";
import type { Player } from "./usePlayer";

/** 宽的时候几面各占一栏；最多三栏（两份口播加画面上的字）。 */
const LANE_COLS: Record<number, string> = { 1: "@md:grid-cols-1", 2: "@md:grid-cols-2", 3: "@md:grid-cols-3" };

/** 对照里的一栏：一份口播或画面上的字；还没有的那一面只写一句。 */
export interface Lane {
  id: string;
  layer: "asr" | "frames";
  copy?: CopyFacet;
  pending?: Pending;
}

/**
 * 说的和画面上的按时间对齐，一栏一面，同一时刻落在同一行：说的和屏幕上写的对不对得上，一眼就看得出。
 * 点时间从那一秒播；点画面上的一行，画面停在那一帧，框出那一行。容器窄时每一行里几面上下叠。
 */
export function TimeFaces({ lanes, heads, player, itemId }: { lanes: Lane[]; heads: ReactNode[]; player: Player; itemId: string }) {
  const box = useRef<HTMLDivElement>(null);
  const rows = alignRows(lanes.map((l) => l.copy?.segments ?? []), 5);
  const started = player.playing || player.t > 0 || player.still !== null;
  const current = started ? currentRow(rows, player.t) : -1;

  // 播放时把正在说的那一行留在看得见的地方。
  useEffect(() => {
    if (!player.playing || current < 0) return;
    const el = box.current;
    const row = el?.querySelector<HTMLElement>(`[data-row="${current}"]`);
    if (!el || !row) return;
    const top = row.offsetTop - el.offsetTop;
    if (top >= el.scrollTop + 40 && top + row.offsetHeight <= el.scrollTop + el.clientHeight - 8) return;
    el.scrollTo({ top: Math.max(0, top - 56), behavior: reducedMotion() ? "auto" : "smooth" });
  }, [player.playing, current]);

  return (
    <div className="@container flex min-h-0 flex-col">
      <div ref={box} role="table" aria-label="说的和画面上的，按时间对齐" className="relative min-h-0 overflow-y-auto">
        <div role="row" className="sticky top-0 z-[1] grid gap-x-3 gap-y-3 border-b border-border bg-bg pb-2.5 @md:grid-cols-[3.5rem_minmax(0,1fr)]">
          <span role="columnheader" className="hidden @md:block">
            <span className="sr-only-text">时间</span>
          </span>
          <div className={`grid min-w-0 gap-x-3 gap-y-3 ${LANE_COLS[lanes.length] ?? LANE_COLS[3]}`}>
            {lanes.map((l, i) => (
              <div role="columnheader" key={l.id} className="min-w-0">
                {heads[i]}
              </div>
            ))}
          </div>
        </div>
        {lanes.map((l) =>
          l.pending || (l.copy && l.copy.segments.length === 0 && !l.copy.remark) ? (
            <div key={l.id} className="border-b border-border px-1">
              {l.pending ? (
                <PendingLine pending={l.pending} itemId={itemId} />
              ) : (
                <p className="m-0 py-2 text-note text-sub">
                  {FACE_NAME[l.layer]}：这一份是空的
                </p>
              )}
            </div>
          ) : null,
        )}
        {rows.map((r, i) => {
          const on = i === current;
          return (
            <div
              role="row"
              key={r.sec}
              data-row={i}
              className={`grid gap-x-3 rounded-btn px-0 py-1 transition-colors duration-150 @md:py-0.5 ${on ? "bg-card2" : "bg-transparent"}`}
              style={{ gridTemplateColumns: "3.5rem minmax(0, 1fr)" }}
            >
              <span role="rowheader" className="contents">
                <AriaButton
                  onPress={() => player.seek(r.sec)}
                  aria-label={`从 ${clock(r.sec)} 播`}
                  className={`mono h-ctl-tight cursor-pointer self-start rounded-sm border-0 bg-transparent px-2 text-left text-tag outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text ${on ? "text-text" : "text-faint"}`}
                >
                  {clock(r.sec)}
                </AriaButton>
              </span>
              <div className={`grid min-w-0 gap-x-3 gap-y-0.5 ${LANE_COLS[lanes.length] ?? LANE_COLS[3]}`}>
                {lanes.map((l, k) => (
                  <Cell key={l.id} lane={l} segments={r.cells[k] ?? []} on={on} player={player} />
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function Cell({ lane, segments, on, player }: { lane: Lane; segments: Segment[]; on: boolean; player: Player }) {
  return (
    <div role="cell" className={`min-w-0 ${segments.length === 0 ? "hidden @md:block" : ""}`}>
      {segments.map((s) => {
        const pos = s.position;
        const text = (
          <>
            <span className="mr-1.5 text-tag text-faint @md:hidden">{FACE_NAME[lane.layer]}</span>
            {s.text}
          </>
        );
        // 画面上的字点了停在那一帧、框出那一行；口播的字只是文字，时间点在行头。
        if (lane.layer === "frames" && pos?.kind === "time") {
          return (
            <AriaButton
              key={`${pos.sec}${s.text}`}
              onPress={() => player.go(pos)}
              onHoverChange={(h) => player.setPeek(h ? pos : null)}
              onFocusChange={(f) => player.setPeek(f ? pos : null)}
              aria-label={`看 ${clock(pos.sec)} 画面上的「${s.text}」`}
              className={`block w-full cursor-pointer rounded-sm border-0 bg-transparent px-1 py-[5px] text-left font-sans text-body outline-none select-text data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60 ${on ? "text-text" : "text-label"}`}
            >
              <span className="mr-1 inline-block size-1.5 rounded-[1px] border border-border2 align-middle" aria-hidden="true" />
              {text}
            </AriaButton>
          );
        }
        return (
          <p key={`${pos?.kind === "time" ? pos.sec : ""}${s.text}`} className={`m-0 px-1 py-[5px] text-body select-text ${on ? "text-text" : "text-label"}`}>
            {text}
          </p>
        );
      })}
    </div>
  );
}
