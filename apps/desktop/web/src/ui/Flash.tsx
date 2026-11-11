import { useEffect, useState } from "react";

/**
 * 用过一个控件以后，旁边出一行短字：140ms 进来，停一会儿，200ms 出去。
 * `token` 每次变都重放一次，同样的字也会再出来。
 */
export function useFlash(holdMs = 1400) {
  const [state, setState] = useState<{ text: string; token: number; on: boolean }>({ text: "", token: 0, on: false });
  useEffect(() => {
    if (!state.on) return undefined;
    const id = setTimeout(() => setState((s) => ({ ...s, on: false })), 140 + holdMs);
    return () => clearTimeout(id);
  }, [state, holdMs]);
  return {
    text: state.text,
    on: state.on,
    show: (text: string) => setState((s) => ({ text, token: s.token + 1, on: true })),
  };
}

export function FlashLine({ flash, className = "" }: { flash: { text: string; on: boolean }; className?: string }) {
  return (
    <span
      role="status"
      className={`pointer-events-none transition-[opacity,translate] ease-out motion-reduce:translate-y-0 ${flash.on ? "translate-y-0 opacity-100 duration-[140ms]" : "translate-y-[3px] opacity-0 duration-200"} ${className}`}
    >
      {flash.text}
    </span>
  );
}
