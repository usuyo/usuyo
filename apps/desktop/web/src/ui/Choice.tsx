import { ToggleButton, ToggleButtonGroup, type Key } from "react-aria-components";

/** 几个里挑一个：一排圆角的小块，键盘左右键能换。 */
export function Choice<T extends string>({
  label,
  options,
  value,
  onChange,
  isDisabled,
}: {
  label: string;
  options: { id: T; text: string }[];
  value: T | undefined;
  onChange: (v: T) => void;
  isDisabled?: boolean;
}) {
  return (
    <ToggleButtonGroup
      aria-label={label}
      selectionMode="single"
      disallowEmptySelection
      isDisabled={isDisabled}
      selectedKeys={value === undefined ? [] : [value]}
      onSelectionChange={(keys: Set<Key>) => {
        const picked = options.find((o) => keys.has(o.id));
        if (picked) onChange(picked.id);
      }}
      className="flex flex-wrap gap-1.5"
    >
      {options.map((o) => (
        <ToggleButton
          key={o.id}
          id={o.id}
          className="press h-ctl-sm cursor-pointer rounded-btn border border-border2 bg-transparent px-3.5 font-sans text-note text-label outline-none data-[disabled]:cursor-default data-[disabled]:opacity-60 data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60 data-[selected]:border-label data-[selected]:bg-chip data-[selected]:font-medium data-[selected]:text-text"
        >
          {o.text}
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}
