import { Link } from "@tanstack/react-router";
import { ArrowRight, Check, History, Link2 } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import type { TakeResponse } from "../api/endpoints";
import { useTake } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { dayText } from "../item/format";
import { useReducedMotion } from "../motion/clock";
import { onPaletteChange } from "../theme/apply";
import { useUiStore } from "../store/ui";
import { Button } from "../ui/Button";
import { drawBackdrop } from "./backdrop";

type Phase = { kind: "idle" } | { kind: "taken"; res: TakeResponse } | { kind: "gone" };

function Kbd({ children }: { children: ReactNode }) {
  return (
    <kbd className="mono inline-flex h-5 min-w-5 items-center justify-center rounded-[5px] border border-b-2 border-border2 bg-card2 px-1.5 text-tiny leading-none text-label">
      {children}
    </kbd>
  );
}

function Sep() {
  return <span className="h-3 w-px bg-border2" aria-hidden="true" />;
}

/** 收下浮层：粘贴一条，回车收下，结果停一下就收起。桌面上由全局快捷键叫出来；浏览器里是 /take 这一页。 */
export function CapturePage() {
  const canvas = useRef<HTMLCanvasElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const [phase, setPhase] = useState<Phase>({ kind: "idle" });
  const [last, setLast] = useState<Phase>({ kind: "idle" });
  const [value, setValue] = useState("");
  const [hover, setHover] = useState(false);
  const take = useTake();
  const markLanding = useUiStore((s) => s.markLanding);
  const reduced = useReducedMotion();

  useEffect(() => {
    const el = canvas.current;
    if (!el) return undefined;
    const draw = () => drawBackdrop(el);
    draw();
    const off = onPaletteChange(draw);
    window.addEventListener("resize", draw);
    return () => {
      off();
      window.removeEventListener("resize", draw);
    };
  }, []);

  // 结果停一下再收起；「以前收下过」上有按钮，鼠标停在上面就一直留着。
  useEffect(() => {
    if (phase.kind !== "taken" || (phase.res.seen && hover)) return undefined;
    const id = setTimeout(() => setPhase({ kind: "gone" }), phase.res.seen ? 1800 : 1000);
    return () => clearTimeout(id);
  }, [phase, hover]);

  useEffect(() => {
    if (phase.kind === "idle") input.current?.focus({ preventScroll: true });
  }, [phase]);

  const open = phase.kind !== "gone";
  if (open && phase !== last) setLast(phase);
  // 窗口淡出时，上面停着它最后的样子。
  const shown = open ? phase : last;
  const result = shown.kind === "taken" ? shown.res : null;

  function submit(raw: string) {
    const text = raw.trim();
    if (text === "" || phase.kind !== "idle") return;
    take.mutate(
      { input: text, kind: "link" },
      {
        onSuccess: (res) => {
          if (!res.seen) markLanding(res.id);
          setPhase({ kind: "taken", res });
        },
      },
    );
  }

  const layer = (on: boolean) =>
    `absolute inset-0 flex items-center transition-[opacity,translate] ${on ? "visible translate-y-0 opacity-100 duration-200 ease-out" : `invisible opacity-0 duration-[120ms] ease-in ${reduced ? "" : "translate-y-1"}`}`;

  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <canvas ref={canvas} aria-hidden="true" className="absolute inset-0 size-full" />
      <div
        aria-hidden="true"
        className={`pointer-events-none absolute inset-0 bg-bg transition-opacity ${open ? "opacity-45 duration-[240ms] ease-out" : "opacity-0 duration-[160ms]"}`}
      />

      <div
        role="dialog"
        aria-label="收下一条"
        aria-hidden={!open}
        inert={!open}
        onPointerEnter={() => setHover(true)}
        onPointerLeave={() => setHover(false)}
        className={`absolute top-[216px] left-1/2 z-[5] -ml-80 w-160 overflow-hidden rounded-card border bg-card shadow-panel transition-[opacity,translate,scale,border-color] ${open ? "visible opacity-100 duration-[240ms] ease-out" : `invisible opacity-0 duration-[160ms] ${reduced ? "" : "-translate-y-2 scale-[0.98]"}`}`}
        style={{
          borderColor: open && phase.kind === "idle" ? "color-mix(in srgb, var(--border2) 40%, var(--accent))" : "var(--border2)",
        }}
      >
        <div className="relative h-16">
          <label className={`${layer(shown.kind === "idle")} gap-3 px-5`}>
            <Link2 size={18} aria-hidden="true" className="shrink-0 text-sub" />
            <span className="sr-only-text">分享链接</span>
            <input
              ref={input}
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder="粘贴一条分享链接，回车收下"
              autoComplete="off"
              spellCheck={false}
              className="m-0 h-[60px] min-w-0 flex-1 border-0 bg-transparent p-0 font-sans text-[17px] text-text caret-accent outline-none"
              onPaste={(e) => {
                const text = e.clipboardData.getData("text");
                if (!text) return;
                e.preventDefault();
                setValue(text);
                setTimeout(() => submit(text), 280);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  submit(value);
                } else if (e.key === "Escape") {
                  e.preventDefault();
                  setPhase({ kind: "gone" });
                }
              }}
            />
          </label>
          <div role="status" aria-live="polite" className={`${layer(shown.kind === "taken")} gap-3 px-5`}>
            {result?.seen ? (
              <History size={18} aria-hidden="true" className="shrink-0 text-sub" />
            ) : (
              <Check size={18} aria-hidden="true" className="shrink-0 text-accent" />
            )}
            <div className="min-w-0 flex-1">
              <div className="text-section font-semibold text-text">{result?.seen ? "这条以前收下过，补了一次快照" : "收下了"}</div>
              <div className="truncate text-small text-sub">
                {result ? `${result.platform.name} · ${result.title}${result.seen ? ` · ${dayText(result.takenAt)}收下的` : ""}` : ""}
              </div>
            </div>
          </div>
        </div>
        <div className="relative h-12 border-t border-border text-small text-sub">
          <div className={`${layer(!result?.seen)} gap-2 px-5`}>
            {take.error ? (
              <ErrorLine error={take.error} />
            ) : (
              <>
                <span className="flex items-center gap-1">
                  <Kbd>⌥</Kbd>
                  <Kbd>Space</Kbd>
                </span>
                <span>随时叫出这里</span>
                <Sep />
                <Kbd>回车</Kbd>
                <span>收下</span>
                <Sep />
                <Kbd>Esc</Kbd>
                <span>收起</span>
              </>
            )}
          </div>
          <div className={`${layer(result?.seen === true)} gap-2 pr-2 pl-5`}>
            <span className="min-w-0 flex-1">原来存的不动，新快照另存一份</span>
            {result ? (
              <Link
                to="/items/$id"
                params={{ id: result.id }}
                className="press flex h-ctl-sm items-center gap-1.5 rounded-btn px-3 text-note font-medium text-text no-underline hover:bg-chip"
              >
                <span>打开这条</span>
                <ArrowRight size={14} aria-hidden="true" />
              </Link>
            ) : null}
          </div>
        </div>
      </div>

      <div
        inert={open}
        className={`absolute inset-x-0 top-[230px] z-[4] flex flex-col items-center gap-2.5 transition-opacity ${open ? "invisible opacity-0 duration-100" : "visible opacity-100 delay-[160ms] duration-200"}`}
      >
        <Button
          variant="bare"
          size="none"
          className="h-field gap-2 rounded-input border border-border2 bg-card px-[18px] text-body font-medium text-text"
          onPress={() => {
            setValue("");
            take.reset();
            setPhase({ kind: "idle" });
          }}
        >
          <Link2 size={16} aria-hidden="true" />
          <span>再收下一条</span>
        </Button>
        <div className="flex items-center gap-1.5 text-small text-sub">
          <span>平时按</span>
          <Kbd>⌥</Kbd>
          <Kbd>Space</Kbd>
          <span>叫出来</span>
        </div>
        <Link to="/" className="mt-2 text-small text-sub underline decoration-border2 underline-offset-4 hover:text-text">
          回到地图
        </Link>
      </div>
    </div>
  );
}
