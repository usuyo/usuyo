import type { ReactNode } from "react";

/** 平台名的小块：B站、小红书。 */
export function PlatformChip({ children }: { children: ReactNode }) {
  return (
    <span className="flex h-[22px] shrink-0 items-center rounded-chip bg-chip px-2 text-small leading-none font-medium text-label">
      {children}
    </span>
  );
}

/** 主题的圆标签；关注着的主题是你自己的东西，用强调色。 */
export function TopicPill({ followed, size = "sm", children }: { followed: boolean; size?: "sm" | "md"; children: ReactNode }) {
  return (
    <span
      className={`flex items-center rounded-full border text-small leading-none ${size === "md" ? "h-[26px] px-2.5" : "h-6 px-[9px]"} ${followed ? "border-accent text-accent" : "border-border2 text-label"}`}
    >
      {children}
    </span>
  );
}

/** 等宽的小标签：时间、位置、「口播」这类栏名。 */
export function MonoLabel({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <span className={`mono text-tag text-faint ${className}`}>{children}</span>;
}

/** 区块上面那行淡的小字：「关注的主题」「排列」。 */
export function SectionLabel({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <div className={`px-1 text-tag font-medium text-faint ${className}`}>{children}</div>;
}

/** 机器写的标记。 */
export function MachineTag({ children }: { children: ReactNode }) {
  return (
    <span className="flex h-5 items-center rounded-chip bg-chip px-[7px] text-small leading-none text-label">{children}</span>
  );
}
