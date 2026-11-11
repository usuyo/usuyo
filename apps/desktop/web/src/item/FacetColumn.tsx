import { useEffect, useRef } from "react";
import { Button as AriaButton } from "react-aria-components";
import type { Facet, Item, Segment } from "../api/endpoints";
import { reducedMotion } from "../motion/clock";
import { MachineTag, MonoLabel } from "../ui/Chip";
import { LAYER_NAME, positionText } from "./format";
import type { Player } from "./usePlayer";

type TextFacet = Exclude<Facet, { layer: "video" } | { layer: "image" }>;

const TEXT_LAYERS = new Set(["asr", "ocr", "text", "comments"]);

function isTextFacet(f: Facet): f is TextFacet {
  return TEXT_LAYERS.has(f.layer);
}

// 放不下时下沿淡出，让人知道这一块里还能往下滚。
const FADE = "[mask-image:linear-gradient(to_bottom,black_calc(100%-20px),transparent)]";

function size(f: TextFacet): number {
  return f.layer === "comments" ? f.comments.length : f.segments.length;
}

/**
 * 右边一栏：口播、图里的字、作者写的、评论，有哪个摆哪个，并排各占一块，不藏也不挑。
 * 每块自己滚动，长的口播不会把别的挤出去。
 */
export function FacetColumn({ item, player }: { item: Item; player: Player }) {
  const facets = item.facets.filter(isTextFacet);
  if (facets.length === 0) {
    return (
      <div className="flex-1 px-1 py-2.5 text-body text-sub">
        {item.stored === "link" ? "取到内容以后，这里会有口播、图里的字和作者写的字。" : "这一条没有文字。"}
      </div>
    );
  }
  const hasTime = facets.some((f) => f.layer !== "comments" && f.segments.some((s) => s.position?.kind === "time"));
  const hasImage = facets.some((f) => f.layer !== "comments" && f.segments.some((s) => s.position?.kind === "image"));
  const hint = [
    hasTime ? (item.stored === "text-only" ? "视频在回收站里；时间点都还在，恢复以后能从那一秒播" : "点时间，从那一秒播") : "",
    hasImage ? "点「封面」「图 N」，左边换到那一张" : "",
  ]
    .filter(Boolean)
    .join("　·　");
  return (
    <>
      <div className="flex min-h-0 flex-col gap-3">
        {facets.map((f) => (
          <FacetSection key={f.layer} facet={f} player={player} />
        ))}
      </div>
      <div role="status" className="mb-auto flex min-h-7 shrink-0 items-center px-1 text-small text-sub">
        {hint}
      </div>
    </>
  );
}

function FacetSection({ facet, player }: { facet: TextFacet; player: Player }) {
  const n = size(facet);
  return (
    <section
      aria-label={LAYER_NAME[facet.layer]}
      className="flex min-h-24 flex-[0_1_auto] flex-col"
    >
      <header className="flex h-ctl shrink-0 items-center gap-2 border-b border-border px-2.5">
        <h2 className="m-0 text-note font-semibold text-text">{LAYER_NAME[facet.layer]}</h2>
        <MonoLabel>{facet.layer === "comments" ? `${n} 条` : `${n} 段`}</MonoLabel>
        {facet.layer === "asr" ? <MachineTag>{facet.by === "platform" ? "平台的字幕" : "机器转的"}</MachineTag> : null}
        {facet.layer === "ocr" ? <MachineTag>机器认的</MachineTag> : null}
        {facet.remark ? <span className="text-small text-sub">{facet.remark}</span> : null}
      </header>
      {facet.layer === "comments" ? (
        <ul className={`m-0 mt-1 min-h-0 flex-1 list-none overflow-y-auto p-0 px-1 pb-2 ${FADE}`}>
          {facet.comments.map((c) => (
            <li key={`${c.author}${c.text}`} className="flex items-baseline gap-3 py-[7px] pr-2">
              <span className="w-[84px] shrink-0 truncate text-small text-faint">{c.author}</span>
              <span className="min-w-0 flex-1 text-body text-label select-text">{c.text}</span>
              {c.likes !== undefined ? <MonoLabel>{c.likes} 赞</MonoLabel> : null}
            </li>
          ))}
        </ul>
      ) : (
        <Lines segments={facet.segments} player={player} follow={facet.layer === "asr"} />
      )}
    </section>
  );
}

function Lines({ segments, player, follow }: { segments: Segment[]; player: Player; follow: boolean }) {
  const box = useRef<HTMLDivElement>(null);
  const started = player.playing || player.t > 0;
  let current = -1;
  if (started && player.image === null) {
    segments.forEach((s, i) => {
      if (s.position?.kind === "time" && s.position.sec <= player.t + 0.01) current = i;
    });
  }
  if (player.image !== null) current = segments.findIndex((s) => s.position?.kind === "image" && s.position.imageKey === player.image);

  // 播放时把正在说的那一句留在这一块里看得见的地方。
  useEffect(() => {
    if (!follow || !player.playing || current < 0) return;
    const el = box.current;
    const row = el?.querySelector<HTMLElement>(`[data-row="${current}"]`);
    if (!el || !row) return;
    if (row.offsetTop >= el.scrollTop + 8 && row.offsetTop + row.offsetHeight <= el.scrollTop + el.clientHeight - 8) return;
    el.scrollTo({ top: Math.max(0, row.offsetTop - 48), behavior: reducedMotion() ? "auto" : "smooth" });
  }, [follow, player.playing, current]);

  return (
    <div ref={box} className={`relative mt-1 flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto pb-2 ${FADE}`}>
      {segments.map((s, i) => {
        const on = i === current;
        const pos = s.position;
        return (
          <div
            key={`${pos ? positionText(pos) : i}${s.text}`}
            data-row={i}
            className={`flex items-start gap-1 rounded-btn px-1 transition-colors duration-150 ${on ? "bg-card2" : "bg-transparent"}`}
          >
            {pos ? (
              <AriaButton
                onPress={() => player.go(pos)}
                aria-label={pos.kind === "time" ? `从 ${positionText(pos)} 播` : `看${positionText(pos)}`}
                className={`mono h-ctl-sm w-14 shrink-0 cursor-pointer rounded-sm border-0 bg-transparent px-2 text-left text-tag leading-[36px] outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text ${on ? "text-text" : "text-faint"}`}
              >
                {positionText(pos)}
              </AriaButton>
            ) : null}
            <span className={`min-w-0 flex-1 py-[7px] text-body select-text ${pos ? "" : "px-1.5"} ${on ? "text-text" : "text-label"}`}>
              {s.text}
            </span>
          </div>
        );
      })}
    </div>
  );
}
