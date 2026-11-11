// 所有会动的东西共用一个时钟：一个 requestAnimationFrame 循环，标签页藏起来就停。
// 每个订阅者每帧返回还要不要下一帧；都不要了循环就停，有新的事再叫醒。
import { useSyncExternalStore } from "react";

/** dt 是秒，最多 0.05；返回 true 表示还要下一帧。 */
export type Tick = (dt: number) => boolean;

const ticks = new Set<Tick>();
let raf = 0;
let last = 0;

function loop(ts: number): void {
  raf = 0;
  const dt = last ? Math.min(0.05, (ts - last) / 1000) : 1 / 60;
  last = ts;
  let more = false;
  for (const tick of ticks) if (tick(dt)) more = true;
  if (more && !document.hidden) raf = requestAnimationFrame(loop);
  else last = 0;
}

export function wake(): void {
  if (raf || typeof requestAnimationFrame !== "function" || document.hidden) return;
  raf = requestAnimationFrame(loop);
}

export function subscribe(tick: Tick): () => void {
  ticks.add(tick);
  wake();
  return () => {
    ticks.delete(tick);
  };
}

if (typeof document !== "undefined") {
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      if (raf) cancelAnimationFrame(raf);
      raf = 0;
      last = 0;
    } else wake();
  });
}

// 减少动效：跟系统设置。
const reducedQuery =
  typeof window !== "undefined" && typeof window.matchMedia === "function"
    ? window.matchMedia("(prefers-reduced-motion: reduce)")
    : null;

export function reducedMotion(): boolean {
  return reducedQuery?.matches ?? false;
}

function subscribeReduced(onChange: () => void): () => void {
  if (!reducedQuery) return () => undefined;
  const handler = () => {
    onChange();
    wake();
  };
  reducedQuery.addEventListener("change", handler);
  return () => reducedQuery.removeEventListener("change", handler);
}

export function useReducedMotion(): boolean {
  return useSyncExternalStore(subscribeReduced, reducedMotion, () => false);
}
