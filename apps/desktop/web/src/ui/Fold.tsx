import { ChevronRight } from "lucide-react";
import type { ReactNode } from "react";
import { Button as AriaButton, Disclosure, DisclosurePanel, Heading } from "react-aria-components";

/** 默认收起的一段：「详细说明」「技术信息」「高级」。点标题展开，不带动画。 */
export function Fold({ title, children, className = "" }: { title: string; children: ReactNode; className?: string }) {
  return (
    <Disclosure className={`group min-w-0 ${className}`}>
      <Heading className="m-0 text-note font-normal">
        <AriaButton
          slot="trigger"
          className="press -ml-1 inline-flex cursor-pointer items-center gap-1 rounded-sm border-0 bg-transparent px-1 py-0.5 font-sans text-note text-sub outline-none data-[hovered]:text-text data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
        >
          <ChevronRight size={14} aria-hidden="true" className="shrink-0 group-data-[expanded]:rotate-90" />
          {title}
        </AriaButton>
      </Heading>
      <DisclosurePanel className="min-w-0">
        <div className="pt-2">{children}</div>
      </DisclosurePanel>
    </Disclosure>
  );
}
