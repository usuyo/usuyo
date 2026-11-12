import { Switch } from "react-aria-components";

/** 开或关的一个开关，旁边一句说现在是哪样。 */
export function Toggle({
  isSelected,
  isDisabled,
  onChange,
  children,
  label,
}: {
  isSelected: boolean;
  isDisabled?: boolean;
  onChange: (on: boolean) => void;
  /** 旁边那一句。 */
  children: string;
  /** 读屏念的名字；不给就念旁边那一句。 */
  label?: string;
}) {
  return (
    <Switch
      isSelected={isSelected}
      isDisabled={isDisabled}
      onChange={onChange}
      {...(label ? { "aria-label": label } : {})}
      className="group flex h-ctl-sm w-fit cursor-pointer items-center gap-3 font-sans text-note text-label outline-none data-[disabled]:cursor-default data-[disabled]:opacity-60"
    >
      <span className="flex h-6 w-10 shrink-0 items-center rounded-full border border-border2 bg-chip px-0.5 transition-colors duration-150 group-data-[focus-visible]:outline-2 group-data-[focus-visible]:outline-accent group-data-[selected]:border-label group-data-[selected]:bg-label">
        <span className="size-[18px] rounded-full bg-text transition-transform duration-150 ease-out group-data-[selected]:translate-x-4 group-data-[selected]:bg-bg motion-reduce:transition-none" />
      </span>
      <span>{children}</span>
    </Switch>
  );
}
