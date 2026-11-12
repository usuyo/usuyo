import { ToggleButton, ToggleButtonGroup, type Key } from "react-aria-components";
import { ACCENTS, useUiStore, type Accent, type ThemePreference } from "../store/ui";
import { Choice } from "../ui/Choice";
import { Row } from "./Row";

const THEMES: { id: ThemePreference; text: string }[] = [
  { id: "system", text: "跟系统" },
  { id: "light", text: "浅色" },
  { id: "dark", text: "深色" },
];

const ACCENT_NAME: Record<Accent, string> = { "#F2B866": "琥珀", "#7FD1BE": "青绿", "#F08C6C": "珊瑚", "#A9B4FF": "淡蓝紫" };

/** 外观：深浅、强调色。强调色只给你自己的东西：关注的主题、记一句、刚收下的那一圈。 */
export function Appearance() {
  const theme = useUiStore((s) => s.theme);
  const setTheme = useUiStore((s) => s.setTheme);
  const accent = useUiStore((s) => s.accent);
  const setAccent = useUiStore((s) => s.setAccent);
  return (
    <div className="flex flex-col gap-5">
      <Row label="深浅">
        <Choice label="深浅" options={THEMES} value={theme} onChange={setTheme} />
      </Row>
      <Row label="强调色" sub="只用在你自己的东西上：关注的主题、记一句、刚收下的那一圈。">
        <ToggleButtonGroup
          aria-label="强调色"
          selectionMode="single"
          disallowEmptySelection
          selectedKeys={[accent]}
          onSelectionChange={(keys: Set<Key>) => {
            const picked = ACCENTS.find((a) => keys.has(a));
            if (picked) setAccent(picked);
          }}
          className="flex flex-wrap gap-2"
        >
          {ACCENTS.map((a) => (
            <ToggleButton
              key={a}
              id={a}
              aria-label={ACCENT_NAME[a]}
              className="press flex h-ctl-sm cursor-pointer items-center gap-2 rounded-btn border border-border2 bg-transparent px-3 font-sans text-note text-label outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60 data-[selected]:border-label data-[selected]:bg-chip data-[selected]:text-text"
            >
              <span className="size-3 rounded-full" style={{ background: a }} aria-hidden="true" />
              <span>{ACCENT_NAME[a]}</span>
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </Row>
    </div>
  );
}
