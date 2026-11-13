import type { ReactNode } from "react";

/** 一张卡片：标题、一句说明，下面几行，行和行之间一条细线。 */
export function Card({ title, sub, label, children }: { title: string; sub?: ReactNode; label?: string; children: ReactNode }) {
  return (
    <section aria-label={label ?? title} className="min-w-0 rounded-card border border-border bg-card">
      <header className="px-4 pt-3.5 pb-1">
        <h2 className="m-0 text-section font-semibold text-text">{title}</h2>
        {sub ? <p className="m-0 mt-0.5 text-note text-sub">{sub}</p> : null}
      </header>
      <div className="flex flex-col">{children}</div>
    </section>
  );
}

/**
 * 卡片里的一行：左边名字和一句说明，右边一个控件；`children` 是这一行下面跟着的说明和步骤，占满整行。
 * 窄的时候控件排到名字下面。
 */
export function CardRow({ title, sub, control, children }: { title: ReactNode; sub?: ReactNode; control?: ReactNode; children?: ReactNode }) {
  return (
    <div className="border-t border-border px-4 py-3 first:border-t-0">
      <div className="flex flex-wrap items-start justify-between gap-x-4 gap-y-2">
        <div className="min-w-0 flex-[1_1_220px]">
          <div className="text-body font-medium text-text">{title}</div>
          {sub ? <div className="mt-0.5 text-small text-sub">{sub}</div> : null}
        </div>
        {control ? <div className="flex max-w-full min-w-0 flex-none items-center gap-1.5">{control}</div> : null}
      </div>
      {children ? <div className="mt-2.5 min-w-0">{children}</div> : null}
    </div>
  );
}
