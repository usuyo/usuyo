import { useEffect, useRef, useState } from "react";
import type { Facet, Position } from "../api/endpoints";
import { subscribe } from "../motion/clock";

type VideoFacet = Extract<Facet, { layer: "video" }>;

export interface Player {
  /** 现在在第几秒。 */
  t: number;
  playing: boolean;
  /** 画面里现在显示哪张图：图文的图片 key，或视频的 `cover`；视频播放时为 null。 */
  image: string | null;
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
 * 条目页左边画面的状态，右边各个侧面点位置时改它。
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

  const seek = (sec: number) => {
    const s = Math.max(0, Math.min(dur, sec));
    setT(s);
    setImage(null);
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
      setPlaying(true);
      void el?.play();
    },
    go: (p) => {
      if (p.kind === "time") seek(p.sec);
      else {
        setPlaying(false);
        videoRef.current?.pause();
        setImage(p.imageKey);
      }
    },
    seek,
    showImage: (key) => {
      setPlaying(false);
      videoRef.current?.pause();
      setImage(key);
    },
    onTime: setT,
  };
}
