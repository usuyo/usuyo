import { ToggleButton, ToggleButtonGroup, type Key } from "react-aria-components";
import { t, type MessageKey } from "../i18n";
import { ACCENTS, useUiStore, type Accent, type ThemePreference } from "../store/ui";
import { Choice } from "../ui/Choice";
import { Row } from "./Row";

// 和 Apple 一样：浅色、深色、自动。
const THEMES = [
  { id: "light", text: "settings.appearance.light" },
  { id: "dark", text: "settings.appearance.dark" },
  { id: "system", text: "settings.appearance.auto" },
] as const satisfies { id: ThemePreference; text: MessageKey }[];

const ACCENT_NAME = {
  "#F2B866": "settings.appearance.amber",
  "#7FD1BE": "settings.appearance.teal",
  "#F08C6C": "settings.appearance.coral",
  "#A9B4FF": "settings.appearance.periwinkle",
} as const satisfies Record<Accent, MessageKey>;

/** 外观：深浅、强调色。强调色只给你自己的东西：关注的主题、备注、刚添加的那一圈。 */
export function Appearance() {
  const theme = useUiStore((s) => s.theme);
  const setTheme = useUiStore((s) => s.setTheme);
  const accent = useUiStore((s) => s.accent);
  const setAccent = useUiStore((s) => s.setAccent);
  return (
    <div className="flex flex-col gap-5">
      <Row label={t("settings.appearance.theme")}>
        <Choice label={t("settings.appearance.theme")} options={THEMES.map((o) => ({ id: o.id, text: t(o.text) }))} value={theme} onChange={setTheme} />
      </Row>
      <Row label={t("settings.appearance.accent")} sub={t("settings.appearance.accentSub")}>
        <ToggleButtonGroup
          aria-label={t("settings.appearance.accent")}
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
              aria-label={t(ACCENT_NAME[a])}
              className="press flex h-ctl-sm cursor-pointer items-center gap-2 rounded-btn border border-border2 bg-transparent px-3 font-sans text-note text-label outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60 data-[selected]:border-label data-[selected]:bg-chip data-[selected]:text-text"
            >
              <span className="size-3 rounded-full" style={{ background: a }} aria-hidden="true" />
              <span>{t(ACCENT_NAME[a])}</span>
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </Row>
    </div>
  );
}
