import { ChevronDown } from "lucide-react";
import { Button as AriaButton, ListBox, ListBoxItem, Popover, Select as AriaSelect, SelectValue, Text, type Key } from "react-aria-components";

export interface SelectOption {
  id: string;
  text: string;
  /** 选项下面的一行小字。 */
  sub?: string;
  disabled?: boolean;
}

/**
 * 一个选择框：按下展开一列，每项一个名字、一行小字；关掉的项灰着选不了。`value` 是 null 时框里写 `placeholder`
 * （现在的样子不是哪一项）。
 */
export function Select({
  label,
  options,
  value,
  onChange,
  placeholder,
  isDisabled,
}: {
  label: string;
  options: SelectOption[];
  value: string | null;
  onChange: (id: string) => void;
  placeholder?: string;
  isDisabled?: boolean;
}) {
  return (
    <AriaSelect
      aria-label={label}
      selectedKey={value}
      placeholder={placeholder}
      isDisabled={isDisabled}
      disabledKeys={options.filter((o) => o.disabled).map((o) => o.id)}
      onSelectionChange={(k: Key | null) => {
        if (k !== null && k !== value) onChange(String(k));
      }}
      className="w-[300px] max-w-full"
    >
      <AriaButton className="press flex h-ctl w-full cursor-pointer items-center gap-2 rounded-input border border-border2 bg-transparent px-3 text-left font-sans text-body text-text outline-none data-[disabled]:cursor-default data-[disabled]:opacity-60 data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60">
        <SelectValue className="min-w-0 flex-1 truncate data-[placeholder]:text-sub [&_[slot=description]]:hidden" />
        <ChevronDown size={15} aria-hidden="true" className="shrink-0 text-sub" />
      </AriaButton>
      <Popover className="max-h-[60vh] w-(--trigger-width) min-w-[300px] overflow-y-auto rounded-btn border border-border bg-card p-1 shadow-panel">
        <ListBox className="outline-none">
          {options.map((o) => (
            <ListBoxItem
              key={o.id}
              id={o.id}
              textValue={o.text}
              className="flex cursor-pointer flex-col gap-0.5 rounded-sm px-2.5 py-1.5 outline-none data-[disabled]:cursor-default data-[disabled]:opacity-60 data-[focused]:bg-chip data-[selected]:bg-chip/70"
            >
              <Text slot="label" className="text-note font-medium text-text">
                {o.text}
              </Text>
              {o.sub ? (
                <Text slot="description" className="text-small text-sub">
                  {o.sub}
                </Text>
              ) : null}
            </ListBoxItem>
          ))}
        </ListBox>
      </Popover>
    </AriaSelect>
  );
}
