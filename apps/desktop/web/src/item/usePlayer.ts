import { useEffect, useRef, useState } from "react";
import type { Facet, Position } from "../api/endpoints";
import { subscribe } from "../motion/clock";

type VideoFacet = Extract<Facet, { layer: "video" }>;
type Box = [number, number, number, number];

export interface Player {
  /** 现在在第几秒。 */
  t: number;
  playing: boolean;
  /** 画面里现在显示哪张图：图文的图片 key，或视频的 `cover`；视频播放时为 null。 */
  image: string | null;
  /** 点了画面上的一行字：停在那一帧（认出这行字的关键帧），播放时为 null。 */
  still: string | null;
  /** 画面里框出来的那一块：点过的一行字认在哪儿（0–1 的 `[x, y, w, h]`）。 */
  region: Box | null;
  /** 指着（悬停、键盘停在）一行字时先框出来看看，不换图、不跳时间。 */
  peek: { imageKey: string | null; xywh: Box } | null;
  setPeek: (p: Position | null) => void;
  /** 有视频文件才能播；只留文字、只有链接时不能。 */
  canPlay: boolean;
  video: VideoFacet | undefined;
  /** 给 <video> 的 ref 回调。 */
  attachVideo: (el: HTMLVideoElement | null) => void;
  toggle: () => void;
  /** 跳到原样里的一处：视频跳到那一秒接着播，图片换到那一张。 */
  go: (p: Position) => void;
  seek: (sec: number) => void;
  showImage: (key: string) => void;
  onTime: (sec: number) => void;
}

/**
 * 条目页画面的状态，旁边各个侧面点位置时改它。
 * 有视频地址时跟着 <video> 走；样例库没有视频文件，就用共用的时钟走秒数，画面换成那一秒的缩略图。
 */
export function usePlayer(video: VideoFacet | undefined, firstImage: string | null, mediaHere: boolean): Player {
  const [t, setTState] = useState(0);
  const tRef = useRef(0);
  const setT = (sec: number) => {
    tRef.current = sec;
    setTState(sec);
  };
  const [playing, setPlaying] = useState(false);
  const [image, setImage] = useState<string | null>(video ? null : firstImage);
  const [still, setStill] = useState<string | null>(null);
  const [region, setRegion] = useState<Box | null>(null);
  const [peek, setPeekState] = useState<Player["peek"]>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const canPlay = video !== undefined && mediaHere;
  const dur = video?.durationSec ?? 0;
  const simulated = canPlay && video.url === undefined;

  useEffect(() => {
    if (!playing || !simulated) return undefined;
    let acc = 0;
    return subscribe((dt) => {
      acc += dt;
      if (acc < 0.1) return true;
      const next = Math.min(dur, tRef.current + acc);
      acc = 0;
      tRef.current = next;
      setTState(next);
      if (next < dur) return true;
      setPlaying(false);
      return false;
    });
  }, [playing, simulated, dur]);

  const pause = () => {
    setPlaying(false);
    videoRef.current?.pause();
  };

  const seek = (sec: number) => {
    const s = Math.max(0, Math.min(dur, sec));
    setT(s);
    setImage(null);
    setStill(null);
    setRegion(null);
    if (!canPlay) return;
    setPlaying(true);
    const el = videoRef.current;
    if (el) {
      el.currentTime = s;
      void el.play();
    }
  };

  return {
    t,
    playing,
    image,
    still,
    region,
    peek,
    setPeek: (p) => {
      if (p?.xywh === undefined) return setPeekState(null);
      setPeekState({ imageKey: p.kind === "image" ? p.imageKey : null, xywh: p.xywh });
    },
    canPlay,
    video,
    attachVideo: (el) => {
      videoRef.current = el;
    },
    toggle: () => {
      if (!canPlay) return;
      const el = videoRef.current;
      if (playing) {
        setPlaying(false);
        el?.pause();
        return;
      }
      if (t >= dur - 0.1) setT(0);
      setImage(null);
      setStill(null);
      setRegion(null);
      setPlaying(true);
      void el?.play();
    },
    go: (p) => {
      if (p.kind === "image") {
        pause();
        setImage(p.imageKey);
        setRegion(p.xywh ?? null);
        return;
      }
      // 画面上的字：停在认出它的那一帧，框出那一行；口播：从那一秒播。
      if (p.xywh && p.frameUrl) {
        pause();
        const s = Math.max(0, Math.min(dur, p.sec));
        setT(s);
        if (videoRef.current) videoRef.current.currentTime = s;
        setImage(null);
        setStill(p.frameUrl);
        setRegion(p.xywh);
        return;
      }
      seek(p.sec);
    },
    seek,
    showImage: (key) => {
      pause();
      setStill(null);
      setRegion(null);
      setImage(key);
    },
    onTime: setT,
  };
}
