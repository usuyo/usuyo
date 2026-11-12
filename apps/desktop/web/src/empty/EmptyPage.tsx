import { Link2 } from "lucide-react";
import { useEffect, useMemo, useRef, useState, type DragEvent } from "react";
import { Input, Label, TextField } from "react-aria-components";
import { useHealth, useTake } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import type { EmptyScene } from "../fog/field";
import { clamp, strHash } from "../fog/noise";
import { useFog, useSize } from "../fog/useFog";
import { useReducedMotion } from "../motion/clock";
import { Button } from "../ui/Button";
import { Ways } from "./Ways";

interface Drop {
  id: string;
  x: number;
  y: number;
  label: string;
  sub: string;
}

/**
 * 空资料库，也是第一次打开：只有雾和名字，什么都不用设就能收下一条，看它落进雾里。
 * 收下的落进雾里，同一个站的落在一起；旁边轻轻放着「存过的一批也能拿回来」的几种路子，不催。内核还不收文件，这里只收链接。
 */
export function EmptyPage({ onLanded, onLeave }: { onLanded: () => void; onLeave: () => void }) {
  const root = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const size = useSize(root);
  const [drops, setDrops] = useState<Drop[]>([]);
  const [value, setValue] = useState("");
  const [toast, setToast] = useState(0);
  const [showWays, setShowWays] = useState(false);
  // 收起那一层以后，焦点回到打开它的那个按钮上。
  const waysTrigger = useRef<HTMLDivElement>(null);
  const closeWays = () => {
    setShowWays(false);
    requestAnimationFrame(() => waysTrigger.current?.querySelector<HTMLButtonElement>("button[aria-expanded]")?.focus());
  };
  const pointer = useRef<{ x: number; y: number } | null>(null);
  const take = useTake();
  const library = useHealth().data?.library;
  const reduced = useReducedMotion();

  // 中间那块字的位置：画布上是 1440×900 里的 (720, 445)。
  const cx = size.width / 2;
  const cy = size.height / 2 - 5;
  const scene = useMemo<EmptyScene | null>(
    () =>
      size.width === 0
        ? null
        : {
            kind: "empty",
            frame: { width: size.width, height: size.height, mapX: 0 },
            clearing: { cx, cy, rx: 330, ry: 185 },
            drops,
          },
    [size.width, size.height, cx, cy, drops],
  );
  const field = useFog(canvas, scene, {});

  useEffect(() => {
    if (toast === 0) return undefined;
    const id = setTimeout(() => setToast(0), 1120);
    return () => clearTimeout(id);
  }, [toast]);

  function land(raw: string) {
    const input = raw.trim();
    if (input === "") return;
    const site = /https?:\/\/[^\s/]+/i.exec(input)?.[0].replace(/^https?:\/\//i, "").replace(/^www\./i, "");
    const label = site ?? (input.length > 18 ? `${input.slice(0, 18)}…` : input);
    const hs = strHash(site ?? input);
    const hj = strHash(`${input}:${drops.length}`);
    const a = ((hs % 3600) / 3600) * Math.PI * 2;
    let x = cx + Math.cos(a) * 470;
    let y = cy + Math.sin(a) * 300;
    x += ((hj & 255) / 255 - 0.5) * 90;
    y += (((hj >>> 8) & 255) / 255 - 0.5) * 70;
    x = clamp(x, 90, size.width - 170);
    y = clamp(y, 90, size.height - 100);
    // 不落在中间那块字上。
    if (Math.abs(x - cx) < 290 && Math.abs(y - cy) < 165) y = y < cy ? cy - 185 : cy + 185;
    take.mutate(
      { input, kind: "link" },
      {
        onSuccess: (res) => {
          setDrops((d) => [...d, { id: res.id, x, y, label, sub: "只有链接" }]);
          setToast((n) => n + 1);
          onLanded();
        },
      },
    );
  }

  function locate(e: { clientX: number; clientY: number }) {
    const rc = root.current?.getBoundingClientRect();
    if (!rc) return null;
    return { x: e.clientX - rc.left, y: e.clientY - rc.top };
  }

  return (
    <div
      ref={root}
      className="relative h-full overflow-hidden bg-bg"
      onPointerMove={(e) => {
        pointer.current = locate(e);
        field.current?.pointer(pointer.current);
      }}
      onPointerLeave={() => {
        pointer.current = null;
        field.current?.pointer(null);
      }}
      onDragOver={(e) => e.preventDefault()}
      onDrop={(e: DragEvent) => {
        e.preventDefault();
        land(e.dataTransfer.getData("text/uri-list") || e.dataTransfer.getData("text"));
      }}
    >
      <canvas ref={canvas} aria-hidden="true" className="absolute inset-0 size-full" />
      {drops.map((d) => (
        <div
          key={d.id}
          className="pointer-events-none absolute z-[4] flex items-baseline gap-2 whitespace-nowrap opacity-0"
          style={{
            left: d.x + 12,
            top: d.y - 8,
            animation: `${reduced ? "fade" : "rise"} 280ms ease-out ${reduced ? 0 : 1000}ms both`,
          }}
        >
          <span className="text-tag font-medium text-label">{d.label}</span>
          <span className="text-tiny text-faint">{d.sub}</span>
        </div>
      ))}
      <div
        inert={showWays}
        className={`absolute z-[5] flex w-[520px] flex-col items-center text-center transition-opacity duration-200 ${showWays ? "opacity-0" : "opacity-100"}`}
        style={{ left: cx - 260, top: cy - 127 }}
      >
        <h1 className="m-0 text-display font-semibold tracking-[-0.01em] text-text">usuyo</h1>
        <div className="relative mt-[18px] h-[26px] w-full text-lede text-label">
          <p
            className={`absolute inset-x-0 top-0 m-0 transition-opacity duration-[180ms] ease-out ${drops.length ? "opacity-0" : "opacity-100"}`}
          >
            还没有收下任何东西。
          </p>
          <p
            className={`absolute inset-x-0 top-0 m-0 transition-opacity delay-[120ms] duration-[320ms] ease-out ${drops.length ? "opacity-100" : "opacity-0"}`}
          >
            收下的会落进雾里，以后从哪一片进去都行。
          </p>
        </div>
        <p className="m-0 mt-1 text-body text-sub">复制一条分享链接，粘贴到下面，或者把链接拖进来。</p>
        <TextField value={value} onChange={setValue} aria-label="收下一条链接" className="mt-[22px] w-[420px]">
          <Label className="flex h-field items-center gap-2 rounded-input border border-border2 bg-hud pr-1.5 pl-3.5 text-sub">
            <Link2 size={16} aria-hidden="true" className="shrink-0" />
            <span className="sr-only-text">收下一条链接</span>
            <Input
              placeholder="粘贴一条分享链接"
              className="h-ctl min-w-0 flex-1 border-0 bg-transparent p-0 text-body text-text outline-none"
              onPaste={(e) => {
                const text = e.clipboardData.getData("text");
                if (!text) return;
                e.preventDefault();
                land(text);
                setValue("");
              }}
              onKeyDown={(e) => {
                if (e.key !== "Enter") return;
                land(value);
                setValue("");
              }}
            />
            <Button
              variant="soft"
              size="tight"
              onPress={() => {
                land(value);
                setValue("");
              }}
            >
              收下
            </Button>
          </Label>
        </TextField>
        <div
          role="status"
          className={`mt-2.5 h-5 text-note font-medium text-text transition-[opacity,translate] motion-reduce:translate-y-0 ${toast ? "translate-y-0 opacity-100 duration-[120ms]" : "translate-y-[3px] opacity-0 duration-[180ms]"}`}
        >
          {toast ? "收下了" : ""}
        </div>
        {take.error ? <ErrorLine error={take.error} /> : null}
        <div ref={waysTrigger} className="mt-1 flex items-center gap-2">
          {drops.length > 0 ? (
            <Button variant="soft" size="tight" onPress={onLeave}>
              去地图看看
            </Button>
          ) : null}
          <Button variant="quiet" size="tight" aria-expanded={showWays} onPress={() => setShowWays(true)}>
            存过的一批，也能拿回来
          </Button>
        </div>
      </div>
      {showWays ? (
        <div className="absolute inset-0 z-[6] flex items-center justify-center px-6 pt-6 pb-12">
          <Ways onClose={closeWays} />
        </div>
      ) : null}
      <div className="absolute bottom-4 left-5 z-10 text-tag whitespace-pre text-sub">
        移动鼠标拨开雾　·　粘贴或把链接拖进来，就收下了
      </div>
      {library ? (
        <div className="absolute right-5 bottom-4 z-10 max-w-[60%] truncate text-tag text-sub select-text" title={library}>
          收下的都存在 {library}
        </div>
      ) : null}
    </div>
  );
}
