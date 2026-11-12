import type { ReactNode } from "react";

/** 设置里的一行：左边是名字，右边是控件和一句说明；窄的时候上下叠。 */
export function Row({ label, sub, children }: { label: string; sub?: ReactNode; children: ReactNode }) {
  return (
    <div className="grid gap-x-6 gap-y-2 @xl:grid-cols-[120px_minmax(0,1fr)]">
      <div className="pt-2 text-note text-label">{label}</div>
      <div className="min-w-0">
        {children}
        {sub ? <p className="m-0 mt-2 text-small text-sub">{sub}</p> : null}
      </div>
    </div>
  );
}
