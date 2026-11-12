import { useState, type ReactNode } from "react";
import { ToggleButton } from "react-aria-components";
import type { Facet, Item, Pending, TransformLayer } from "../api/endpoints";
import { MonoLabel } from "../ui/Chip";
import { sourceText, type CopyFacet } from "./faces";
import { FaceHead, PendingLine } from "./FaceHead";
import type { MediaKeep } from "./format";
import { ImageText } from "./ImageText";
import { TimeFaces, type Lane } from "./TimeFaces";
import type { Player } from "./usePlayer";

// 放不下时下沿淡出，让人知道这一块里还能往下滚。
const FADE = "[mask-image:linear-gradient(to_bottom,black_calc(100%-20px),transparent)]";

function copiesOf(item: Item, layer: TransformLayer): CopyFacet[] {
  return item.facets.filter((f): f is CopyFacet => f.layer === layer);
}

/** 现在摆哪一份：点过的那份还在就是它，不然是第一份（平台的在前，机器的新的在前）。 */
function pick(copies: CopyFacet[], chosen: string | undefined): CopyFacet | undefined {
  return copies.find((c) => c.key === chosen) ?? copies[0];
}

/**
 * 这一条的几面：说的和画面上的按时间对齐摆在一起，图里的字按图分，作者写的、评论各一块。
 * 有哪个摆哪个，不藏也不挑；同一面有几份（平台字幕和机器转的）都在，点一下换，口播还能两份并排。
 */
export function FacetColumn({ item, player, keep }: { item: Item; player: Player; keep: MediaKeep }) {
  const [chosen, setChosen] = useState<Partial<Record<TransformLayer, string>>>({});
  const [together, setTogether] = useState(false);
  const asr = copiesOf(item, "asr");
  const frames = copiesOf(item, "frames");
  const ocr = copiesOf(item, "ocr");
  const pendingOf = (layer: Pending["layer"]) => item.pending.find((p) => p.layer === layer);
  const busy = (layer: TransformLayer) => pendingOf(layer)?.state === "now";
  const shownAsr = pick(asr, chosen.asr);
  const shownFrames = pick(frames, chosen.frames);
  const shownOcr = pick(ocr, chosen.ocr);
  const choose = (layer: TransformLayer) => (key: string) => setChosen((c) => ({ ...c, [layer]: key }));
  const isVideo = item.facets.some((f) => f.layer === "video");

  // 对照里的几栏：口播（并排时两份）、画面上的字；还没转出来的那一面也占一栏，写一句。
  const lanes: Lane[] = [];
  const asrPending = pendingOf("asr");
  if (together && asr.length > 1) for (const c of asr.slice(0, 2)) lanes.push({ id: c.key, layer: "asr", copy: c });
  else if (shownAsr) lanes.push({ id: shownAsr.key, layer: "asr", copy: shownAsr, ...(asrPending ? { pending: asrPending } : {}) });
  else if (asrPending) lanes.push({ id: "asr", layer: "asr", pending: asrPending });
  const framesPending = pendingOf("frames");
  if (shownFrames) lanes.push({ id: shownFrames.key, layer: "frames", copy: shownFrames, ...(framesPending ? { pending: framesPending } : {}) });
  else if (framesPending) lanes.push({ id: "frames", layer: "frames", pending: framesPending });

  // 对照每一栏的头。口播两份并排时，第一栏放切换，第二栏只写来源。
  const headFor = (l: Lane, i: number): ReactNode => {
    if (l.layer === "frames") {
      return <FaceHead layer="frames" copies={frames} shown={shownFrames} onShow={choose("frames")} itemId={item.id} busy={busy("frames")} />;
    }
    if (together && i > 0) return l.copy ? <p className="m-0 pt-[34px] text-small text-faint">{sourceText("asr", l.copy.source)}</p> : null;
    return (
      <FaceHead
        layer="asr"
        copies={asr}
        shown={together ? undefined : shownAsr}
        onShow={(k) => {
          setTogether(false);
          choose("asr")(k);
        }}
        itemId={item.id}
        busy={busy("asr")}
        extra={
          asr.length > 1 ? (
            <ToggleButton
              isSelected={together}
              onChange={setTogether}
              className="press h-6 cursor-pointer rounded-full border border-dashed border-border2 bg-transparent px-2.5 font-sans text-small text-sub outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text data-[selected]:border-solid data-[selected]:border-label data-[selected]:text-text"
            >
              两份并排
            </ToggleButton>
          ) : undefined
        }
      />
    );
  };

  const text = item.facets.find((f): f is Extract<Facet, { layer: "text" }> => f.layer === "text");
  const comments = item.facets.find((f): f is Extract<Facet, { layer: "comments" }> => f.layer === "comments");
  const ocrPending = pendingOf("ocr");
  const videoPending = pendingOf("video");
  const empty = lanes.length === 0 && ocr.length === 0 && !ocrPending && !text && !comments && !videoPending;

  if (empty) {
    return (
      <div className="px-1 py-2.5 text-body text-sub">
        {item.stored === "link" ? "取到内容以后，这里会有口播、图里的字和作者写的字。" : "这一条没有文字。"}
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-col gap-5">
      {videoPending ? <PendingLine pending={videoPending} itemId={item.id} /> : null}
      {lanes.length > 0 ? (
        <Block label="说的和画面上的" grow>
          <TimeFaces lanes={lanes} heads={lanes.map((l, i) => headFor(l, i))} player={player} itemId={item.id} />
          <p role="status" className="m-0 mt-1.5 px-1 text-small text-faint">
            {hintText(item, keep, isVideo)}
          </p>
        </Block>
      ) : null}

      {shownOcr || ocrPending ? (
        <Block label="图里的字">
          {shownOcr ? (
            <>
              <div className="mb-2">
                <FaceHead layer="ocr" copies={ocr} shown={shownOcr} onShow={choose("ocr")} itemId={item.id} busy={busy("ocr")} />
              </div>
              <ImageText copy={shownOcr} player={player} />
            </>
          ) : (
            <h3 className="m-0 text-note font-semibold text-text">图里的字</h3>
          )}
          {ocrPending ? <PendingLine pending={ocrPending} itemId={item.id} /> : null}
        </Block>
      ) : null}

      {text ? (
        <Block label="作者写的">
          <h3 className="m-0 mb-1.5 flex items-baseline gap-2 text-note font-semibold text-text">
            作者写的 <MonoLabel>{text.segments.length} 段</MonoLabel>
          </h3>
          <div className={`flex max-h-60 flex-col overflow-y-auto ${FADE}`}>
            {text.segments.map((s) => (
              <p key={s.text} className="m-0 px-1 py-[5px] text-body text-label select-text">
                {s.text}
              </p>
            ))}
          </div>
        </Block>
      ) : null}

      {comments ? (
        <Block label="评论">
          <h3 className="m-0 mb-1.5 flex items-baseline gap-2 text-note font-semibold text-text">
            评论 <MonoLabel>{comments.comments.length} 条</MonoLabel>
          </h3>
          <ul className={`m-0 flex max-h-60 list-none flex-col overflow-y-auto p-0 px-1 pb-2 ${FADE}`}>
            {comments.comments.map((c) => (
              <li key={`${c.author}${c.text}`} className="flex items-baseline gap-3 py-[5px] pr-2">
                <span className="w-[84px] shrink-0 truncate text-small text-faint">{c.author}</span>
                <span className="min-w-0 flex-1 text-body text-label select-text">{c.text}</span>
                {c.likes !== undefined ? <MonoLabel>{c.likes} 赞</MonoLabel> : null}
              </li>
            ))}
          </ul>
        </Block>
      ) : null}
    </div>
  );
}

function Block({ label, grow, children }: { label: string; grow?: boolean; children: ReactNode }) {
  return (
    <section aria-label={label} className={`@container flex min-h-0 flex-col border-t border-border pt-3 ${grow ? "flex-[1_1_auto]" : "shrink-0"}`}>
      {children}
    </section>
  );
}

/** 对照下面那一行：能点什么。视频不在了，时间点都还在。 */
function hintText(item: Item, keep: MediaKeep, isVideo: boolean): string {
  if (!isVideo) return "";
  if (item.stored === "text-only") return keep.gone ? "视频已经清掉了；时间点都还在" : "视频在回收站里；时间点都还在，恢复以后能从那一秒播";
  return "点时间从那一秒播；点画面上的字，停在那一帧、框出那一行";
}
