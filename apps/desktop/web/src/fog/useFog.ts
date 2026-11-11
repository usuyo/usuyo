import { useEffect, useLayoutEffect, useRef, useState, type RefObject } from "react";
import { FogField, type FogEvents, type FogScene } from "./field";

/** 在一个 canvas 上跑雾；scene 变了就交给它，事件用最新的回调。 */
export function useFog(canvas: RefObject<HTMLCanvasElement | null>, scene: FogScene | null, events: FogEvents) {
  const field = useRef<FogField | null>(null);
  const latest = useRef(events);
  useLayoutEffect(() => {
    latest.current = events;
  });
  useEffect(() => {
    const el = canvas.current;
    if (!el) return undefined;
    const f = new FogField(el, {
      hover: (id) => latest.current.hover?.(id),
      hoverTopic: (id) => latest.current.hoverTopic?.(id),
      lampCount: (n) => latest.current.lampCount?.(n),
    });
    field.current = f;
    return () => {
      f.destroy();
      field.current = null;
    };
  }, [canvas]);
  useEffect(() => {
    if (scene) field.current?.setScene(scene);
  }, [scene]);
  return field;
}

/** 一个元素现在的大小；jsdom 里没有 ResizeObserver，就只量一次。 */
export function useSize(ref: RefObject<HTMLElement | null>): { width: number; height: number } {
  const [size, setSize] = useState({ width: 0, height: 0 });
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return undefined;
    const measure = () => setSize({ width: el.clientWidth, height: el.clientHeight });
    measure();
    if (typeof ResizeObserver === "undefined") return undefined;
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, [ref]);
  return size;
}
