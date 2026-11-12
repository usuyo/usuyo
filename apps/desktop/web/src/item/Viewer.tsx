import { ChevronLeft, ChevronRight, Pause, Play, RefreshCw } from "lucide-react";
import type { KeyboardEvent, PointerEvent } from "react";
import { Button as AriaButton } from "react-aria-components";
import type { Facet, Item } from "../api/endpoints";
import { useReducedMotion } from "../motion/clock";
import { Button } from "../ui/Button";
import { MonoLabel } from "../ui/Chip";
import { FlashLine, useFlash } from "../ui/Flash";
import { clock, facetOf, imageName, keepText, type MediaKeep } from "./format";
import type { Player } from "./usePlayer";

type ImageFacet = Extract<Facet, { layer: "image" }>;

type Box = [number, number, number, number];

/**
 * 画面：视频、封面和关键帧，或图文的几张图。它和旁边几面一样是一个侧面，有自己的位置。
 * `speech`、`screen` 是时间轴上标出的口播和画面上的字开始的那几秒。
 */
export function Viewer({
  item,
  player,
  keep,
  speech,
  screen,
  onRetry,
}: {
  item: Item;
  player: Player;
  keep: MediaKeep;
  speech: number[];
  screen: number[];
  onRetry: () => void;
}) {
  const images = facetOf(item, "image");
  const flash = useFlash(1400);
  const reduced = useReducedMotion();
  const { video, attachVideo } = player;

  // 视频的封面：这一条的第一张图，没有图就不放。
  const poster = video ? images?.images[0] : undefined;
  const keys = images?.images.map((x) => x.key) ?? [];
  const picked = video && player.image !== null ? images?.images.find((x) => x.key === player.image) : undefined;
  let src: string | undefined;
  let alt = "";
  let size: { width: number; height: number } | undefined;
  if (video && player.still) {
    src = player.still;
    alt = `${clock(player.t)} 的画面`;
  } else if (picked) {
    src = picked.url;
    alt = imageName(picked.key, keys);
    size = picked;
  } else if (video) {
    if (poster && !player.playing && player.t === 0) {
      src = poster.url;
      alt = "封面";
      size = poster;
    } else {
      const frame = [...video.frames].reverse().find((f) => f.sec <= player.t + 0.01) ?? video.frames[0];
      src = frame?.url;
      alt = `${clock(player.t)} 的画面`;
    }
  } else if (images) {
    const im = images.images.find((x) => x.key === player.image) ?? images.images[0];
    src = im?.url;
    alt = im ? imageName(im.key, keys) : "";
    size = im;
  }
  // 框：点过的那一行，或指着的那一行（只在它就在这张图、这一帧上时）。
  const shownKey = video ? player.image : (player.image ?? images?.images[0]?.key ?? null);
  const peek = player.peek && (player.peek.imageKey === null ? player.still !== null : player.peek.imageKey === shownKey) ? player.peek.xywh : null;
  const box = peek ?? player.region;
  const textOnly = item.stored === "text-only";
  const remark = video?.remark ?? images?.remark;

  return (
    <div>
      <section
        aria-label="画面"
        className="relative aspect-video w-full overflow-hidden rounded-card border border-border bg-card2 [container-type:size]"
      >
        {video?.url && player.canPlay && !player.still && !picked ? (
          <video
            ref={attachVideo}
            src={video.url}
            poster={poster?.url}
            className="size-full object-contain"
            onTimeUpdate={(e) => player.onTime(e.currentTarget.currentTime)}
            onEnded={player.onEnd}
          />
        ) : src ? (
          <Picture src={src} alt={alt} size={size} box={box} dim={textOnly} />
        ) : null}

        {video || images ? (
          <div className="pointer-events-none absolute top-3 right-3 flex h-7 items-center gap-2 rounded-full border border-border bg-hud px-3">
            <MonoLabel>{video ? `画面 · ${clock(video.durationSec)}` : `画面 · ${images?.images.length ?? 0} 张`}</MonoLabel>
            {remark ? <span className="text-small text-label">{remark}</span> : null}
          </div>
        ) : null}

        {player.canPlay ? (
          <AriaButton
            aria-label="播放"
            onPress={() => {
              player.toggle();
              flash.show(`从 ${clock(player.t >= (video?.durationSec ?? 0) - 0.1 ? 0 : player.t)} 播`);
            }}
            className={`absolute top-1/2 left-1/2 -mt-7 -ml-7 flex size-14 cursor-pointer items-center justify-center rounded-full border border-border2 bg-hud pl-[3px] text-text outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent ${player.playing ? "pointer-events-none scale-[0.92] opacity-0 duration-[140ms]" : "scale-100 opacity-100 duration-200"} transition-[opacity,scale] ease-out ${reduced ? "scale-100" : ""}`}
          >
            <Play size={22} aria-hidden="true" />
          </AriaButton>
        ) : null}

        {textOnly ? (
          <div className="absolute inset-x-0 bottom-0 animate-rise border-t border-border bg-hud px-5 py-4">
            <div className="text-body font-medium text-text">{keep.gone ? "视频和封面已经从回收站清掉了" : "视频和封面在回收站里"}</div>
            <div className="mt-0.5 text-small text-sub">
              {keep.gone ? "口播文字、图里的字和画面缩略图都还在" : `${keepText(keep.days)}；口播文字、图里的字和画面缩略图都还在`}
            </div>
          </div>
        ) : null}

        {item.stored === "link" ? (
          <div className="absolute inset-0 flex animate-rise flex-col items-center justify-center text-center">
            {item.missing ? (
              <>
                <div className="text-note font-medium text-warn">{item.missing}</div>
                <div className="mt-1 text-small text-sub">原帖还在的话，可以再试一次</div>
                <Button variant="soft" size="md" className="mt-3.5 px-4" onPress={onRetry}>
                  <RefreshCw size={14} aria-hidden="true" />
                  <span>重试</span>
                </Button>
              </>
            ) : (
              <>
                <div className="text-body font-medium text-text">正在取内容</div>
                <div className="mt-1 text-small text-sub">取到以后这里是视频或图，口播会转成文字</div>
              </>
            )}
          </div>
        ) : null}

        <FlashLine
          flash={flash}
          className="absolute top-3 left-3 flex h-7 items-center rounded-full border border-border bg-hud px-3 text-small font-medium whitespace-nowrap text-text"
        />
      </section>

      {video ? <Timeline player={player} marks={speech} screen={screen} gone={keep.gone} onJump={flash.show} /> : null}
      {images ? <Strip images={images} player={player} /> : null}
    </div>
  );
}

/** 一张图按它自己的比例放进画面里；框照图的宽高算，所以落在字上，不落在留白上。 */
function Picture({ src, alt, size, box, dim }: { src: string; alt: string; size?: { width: number; height: number }; box: Box | null; dim: boolean }) {
  const ratio = size && size.width > 0 && size.height > 0 ? size.width / size.height : 16 / 9;
  return (
    <div className="absolute inset-0 flex items-center justify-center">
      <div className="relative" style={{ width: `min(100cqw, calc(100cqh * ${ratio}))`, aspectRatio: `${ratio}` }}>
        <img src={src} alt={alt} className={`size-full object-contain transition-opacity duration-200 ${dim ? "opacity-35" : ""}`} />
        {box ? (
          <span
            aria-hidden="true"
            className="pointer-events-none absolute animate-fade rounded-[3px] outline-2 outline-offset-2 outline-text [box-shadow:0_0_0_100vmax_color-mix(in_srgb,var(--bg)_55%,transparent)]"
            style={{ left: `${box[0] * 100}%`, top: `${box[1] * 100}%`, width: `${box[2] * 100}%`, height: `${box[3] * 100}%` }}
          />
        ) : null}
      </div>
    </div>
  );
}

function Timeline({
  player,
  marks,
  screen,
  gone,
  onJump,
}: {
  player: Player;
  marks: number[];
  screen: number[];
  gone: boolean;
  onJump: (text: string) => void;
}) {
  const dur = player.video?.durationSec ?? 1;
  const p = Math.min(1, player.t / dur);
  const current = marks.reduce((k, sec, i) => (sec <= player.t + 0.01 ? i : k), -1);
  const started = player.playing || player.t > 0;
  const jump = (sec: number) => {
    player.seek(sec);
    onJump(player.canPlay ? `从 ${clock(sec)} 播` : gone ? `视频已经清掉了，停在 ${clock(sec)}` : `视频在回收站里，恢复以后从 ${clock(sec)} 播`);
  };
  return (
    <div className="mt-2 flex h-ctl items-center gap-3">
      <Button variant="outline" size="md" square aria-label={!player.canPlay ? "视频不在，不能播" : player.playing ? "暂停" : "播放"} onPress={player.toggle} isDisabled={!player.canPlay}>
        {player.playing ? <Pause size={16} aria-hidden="true" /> : <Play size={16} aria-hidden="true" />}
      </Button>
      <div
        role="slider"
        tabIndex={0}
        aria-label="时间轴：点一下跳到那一秒，左右方向键前后 5 秒"
        aria-valuemin={0}
        aria-valuemax={Math.round(dur)}
        aria-valuenow={Math.round(player.t)}
        aria-valuetext={clock(player.t)}
        className="relative h-ctl-sm min-w-0 flex-1 cursor-pointer rounded-chip outline-none focus-visible:outline-2 focus-visible:outline-accent"
        onPointerDown={(e: PointerEvent<HTMLDivElement>) => {
          const rc = e.currentTarget.getBoundingClientRect();
          jump(((e.clientX - rc.left) / (rc.width || 1)) * dur);
        }}
        onKeyDown={(e: KeyboardEvent<HTMLDivElement>) => {
          if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
          e.preventDefault();
          jump(player.t + (e.key === "ArrowRight" ? 5 : -5));
        }}
      >
        <span className="absolute inset-x-0 top-4 h-1 rounded-[2px] bg-chip" />
        <span className="absolute inset-x-0 top-4 h-1 origin-left rounded-[2px] bg-sub" style={{ transform: `scaleX(${p})` }} />
        {screen.map((sec) => (
          <span
            key={`s${sec}`}
            className="pointer-events-none absolute top-[22px] -ml-[3px] size-1.5 rounded-[1px] border border-border2"
            style={{ left: `${(sec / dur) * 100}%` }}
          />
        ))}
        {marks.map((sec, i) => (
          <span
            key={sec}
            className={`pointer-events-none absolute top-3 -ml-px h-3 w-0.5 rounded-[1px] transition-colors duration-150 ${started && i === current ? "bg-label" : "bg-border2"}`}
            style={{ left: `${(sec / dur) * 100}%` }}
          />
        ))}
        <span className="pointer-events-none absolute inset-0" style={{ transform: `translateX(${p * 100}%)` }}>
          <span className="absolute top-3 -left-1.5 size-3 rounded-full bg-text" />
        </span>
      </div>
      <span className="mono w-24 shrink-0 text-right text-tag text-sub">
        {clock(player.t)} / {clock(dur)}
      </span>
    </div>
  );
}

function Strip({ images, player }: { images: ImageFacet; player: Player }) {
  const list = images.images;
  const index = Math.max(0, list.findIndex((x) => x.key === player.image));
  const go = (i: number) => {
    const im = list[(i + list.length) % list.length];
    if (im) player.showImage(im.key);
  };
  return (
    <div className="mt-2 flex h-ctl items-center gap-2">
      <Button variant="outline" size="md" square aria-label="上一张" onPress={() => go(index - 1)} isDisabled={list.length < 2}>
        <ChevronLeft size={16} aria-hidden="true" />
      </Button>
      <div role="group" aria-label="几张图" className="flex min-w-0 flex-1 gap-1.5 overflow-x-auto">
        {list.map((im, i) => (
          <AriaButton
            key={im.key}
            aria-label={imageName(im.key, list.map((x) => x.key))}
            aria-current={i === index}
            onPress={() => player.showImage(im.key)}
            className={`press h-ctl w-[64px] shrink-0 cursor-pointer overflow-hidden rounded-sm border bg-card2 p-0 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent ${i === index ? "border-label" : "border-border"}`}
          >
            <img src={im.url} alt="" className="size-full object-cover" />
          </AriaButton>
        ))}
      </div>
      <span className="mono w-14 shrink-0 text-right text-tag text-sub">
        {index + 1} / {list.length}
      </span>
      <Button variant="outline" size="md" square aria-label="下一张" onPress={() => go(index + 1)} isDisabled={list.length < 2}>
        <ChevronRight size={16} aria-hidden="true" />
      </Button>
    </div>
  );
}
