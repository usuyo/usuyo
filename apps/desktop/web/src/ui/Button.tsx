import type { ReactNode } from "react";
import { Button as AriaButton, type ButtonProps as AriaButtonProps } from "react-aria-components";

type Variant = "outline" | "soft" | "ghost" | "quiet" | "bare";
/** "none" 不带尺寸，尺寸全由 className 给。 */
type Size = "xs" | "tight" | "sm" | "md" | "none";

const variants: Record<Variant, string> = {
  // 边框按钮：打开原帖、改一下
  outline: "border border-border2 bg-transparent text-label data-[hovered]:bg-chip/60",
  // 底色按钮：记下、收下、恢复
  soft: "border border-border2 bg-chip text-text font-medium data-[hovered]:border-label/60",
  // 无边框：关闭、返回
  ghost: "border-0 bg-transparent text-sub data-[hovered]:bg-chip data-[hovered]:text-text",
  // 无边框也不变底色：行内的文字按钮
  quiet: "border-0 bg-transparent text-sub data-[hovered]:text-text",
  // 颜色全由 className 给
  bare: "border-0",
};

const sizes: Record<Size, string> = {
  xs: "h-ctl-xs rounded-sm px-2 text-small",
  tight: "h-ctl-tight rounded-sm px-3.5 text-note",
  sm: "h-ctl-sm rounded-btn px-3.5 text-note",
  md: "h-ctl rounded-btn px-3.5 text-note",
  none: "",
};

const squares: Record<Size, string> = {
  xs: "size-ctl-xs justify-center rounded-sm",
  tight: "size-ctl-tight justify-center rounded-sm",
  sm: "size-ctl-sm justify-center rounded-btn",
  md: "size-ctl justify-center rounded-btn",
  none: "justify-center",
};

export interface ButtonProps extends Omit<AriaButtonProps, "className" | "children"> {
  variant?: Variant;
  size?: Size;
  /** 方形，只放图标。 */
  square?: boolean;
  className?: string;
  children: ReactNode;
}

export function Button({ variant = "outline", size = "md", square, className = "", children, ...rest }: ButtonProps) {
  const shape = square ? squares[size] : sizes[size];
  return (
    <AriaButton
      {...rest}
      className={`press inline-flex shrink-0 cursor-pointer items-center gap-2 font-sans whitespace-nowrap outline-none data-[disabled]:cursor-default data-[disabled]:opacity-60 data-[focus-visible]:outline-2 data-[focus-visible]:outline-offset-2 data-[focus-visible]:outline-accent ${variants[variant]} ${shape} ${className}`}
    >
      {children}
    </AriaButton>
  );
}
