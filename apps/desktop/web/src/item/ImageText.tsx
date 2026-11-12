import { Button as AriaButton } from "react-aria-components";
import type { Segment } from "../api/endpoints";
import type { CopyFacet } from "./faces";
import { imageName } from "./format";
import type { Player } from "./usePlayer";

/**
 * 图里的字，按图分组：每行指着时先在画面里框出来，点一下换到那张图、框住那一行。
 * 看得见是图上哪一块读出来的，认错了的字也能对着原图看。
 */
export function ImageText({ copy, player, keys }: { copy: CopyFacet; player: Player; keys: string[] }) {
  const groups = new Map<string, Segment[]>();
  for (const s of copy.segments) {
    const key = s.position?.kind === "image" ? s.position.imageKey : "";
    groups.set(key, [...(groups.get(key) ?? []), s]);
  }
  return (
    <div className="flex flex-col gap-2">
      {[...groups.entries()].map(([key, lines]) => (
        <div key={key} className="flex gap-3">
          {key ? (
            <AriaButton
              onPress={() => player.showImage(key)}
              aria-label={`看${imageName(key, keys)}`}
              aria-current={player.image === key}
              className={`mono h-ctl-tight w-14 shrink-0 cursor-pointer rounded-sm border-0 bg-transparent px-2 text-left text-tag outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text ${player.image === key ? "text-text" : "text-faint"}`}
            >
              {imageName(key, keys)}
            </AriaButton>
          ) : (
            <span className="w-14 shrink-0" />
          )}
          <div className="flex min-w-0 flex-1 flex-col">
            {lines.map((s) => {
              const pos = s.position;
              const on = pos?.kind === "image" && player.image === pos.imageKey && player.region === (pos.xywh ?? null);
              if (pos?.kind !== "image" || !pos.xywh) {
                return (
                  <p key={s.text} className="m-0 px-1 py-[5px] text-body text-label select-text">
                    {s.text}
                  </p>
                );
              }
              return (
                <AriaButton
                  key={s.text}
                  onPress={() => player.go(pos)}
                  onHoverChange={(h) => player.setPeek(h ? pos : null)}
                  onFocusChange={(f) => player.setPeek(f ? pos : null)}
                  aria-label={`在${imageName(pos.imageKey, keys)} 上框出「${s.text}」`}
                  className={`w-full cursor-pointer rounded-sm border-0 px-1 py-[5px] text-left font-sans text-body outline-none select-text data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60 ${on ? "bg-card2 text-text" : "bg-transparent text-label"}`}
                >
                  {s.text}
                </AriaButton>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}
