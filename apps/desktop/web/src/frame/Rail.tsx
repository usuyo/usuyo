import { Link } from "@tanstack/react-router";
import { FileText, FolderInput, FolderOutput, History, Monitor, Moon, Plug, Rss, Settings, Smartphone, Sun, Trash2 } from "lucide-react";
import type { ReactNode } from "react";
import { t, type MessageKey } from "../i18n";
import { useUiStore, type ThemePreference } from "../store/ui";
import { Button } from "../ui/Button";
import { TakeField } from "./TakeField";

/** 底下去别的页。 */
type NavKey = Extract<MessageKey, `frame.nav.${string}`>;

const PAGES: { label: NavKey; icon: ReactNode; to: "/drafts" | "/follow" | "/changes" | "/export" | "/import" | "/plugins" | "/phone" | "/trash" | "/settings" }[] = [
  { to: "/drafts", label: "frame.nav.drafts", icon: <FileText size={16} aria-hidden="true" /> },
  { to: "/follow", label: "frame.nav.follow", icon: <Rss size={16} aria-hidden="true" /> },
  { to: "/import", label: "frame.nav.import", icon: <FolderInput size={16} aria-hidden="true" /> },
  { to: "/export", label: "frame.nav.export", icon: <FolderOutput size={16} aria-hidden="true" /> },
  { to: "/changes", label: "frame.nav.changes", icon: <History size={16} aria-hidden="true" /> },
  { to: "/trash", label: "frame.nav.trash", icon: <Trash2 size={16} aria-hidden="true" /> },
  { to: "/plugins", label: "frame.nav.plugins", icon: <Plug size={16} aria-hidden="true" /> },
  { to: "/phone", label: "frame.nav.phone", icon: <Smartphone size={16} aria-hidden="true" /> },
  { to: "/settings", label: "frame.nav.settings", icon: <Settings size={16} aria-hidden="true" /> },
];

// 路由给当前页的链接标 `data-status="active"`。
const PAGE_LINK =
  "press flex h-ctl-sm shrink-0 items-center gap-2.5 rounded-btn px-2.5 text-note text-sub no-underline hover:bg-chip/60 hover:text-label data-[status=active]:bg-chip data-[status=active]:font-semibold data-[status=active]:text-text";

const NEXT_THEME: Record<ThemePreference, ThemePreference> = { system: "dark", dark: "light", light: "system" };
const THEME_LABEL = { system: "frame.theme.system", dark: "frame.theme.dark", light: "frame.theme.light" } as const satisfies Record<ThemePreference, MessageKey>;

function ThemeSwitch() {
  const theme = useUiStore((s) => s.theme);
  const setTheme = useUiStore((s) => s.setTheme);
  const Icon = theme === "system" ? Monitor : theme === "dark" ? Moon : Sun;
  return (
    <Button
      variant="ghost"
      size="xs"
      square
      aria-label={t("frame.theme.switch", { current: t(THEME_LABEL[theme]), next: t(THEME_LABEL[NEXT_THEME[theme]]) })}
      onPress={() => setTheme(NEXT_THEME[theme])}
    >
      <Icon size={14} aria-hidden="true" />
    </Button>
  );
}

/** 左栏：名字、收下、中间一段各页不同、底下去别的页。 */
export function Rail({ label, children }: { label: string; children?: ReactNode }) {
  return (
    <nav
      aria-label={label}
      // 矮的窗口里放不下时整栏能上下滚，底下几页不被挤没。
      className="absolute top-0 left-0 z-20 flex h-full w-rail flex-col overflow-y-auto border-r border-border bg-rail px-4 pt-5 pb-4"
    >
      <div className="flex h-7 items-center justify-between px-1">
        <span className="text-name font-semibold tracking-[0.005em] text-text">usuyo</span>
        <ThemeSwitch />
      </div>
      <TakeField />
      {children}
      <div className="flex-1" />
      {PAGES.map((l) => (
        <Link key={l.to} to={l.to} className={PAGE_LINK}>
          {l.icon}
          <span>{t(l.label)}</span>
        </Link>
      ))}
    </nav>
  );
}

/** 左栏里一个关注的主题：圆点是强调色。 */
export function RailTopic({
  name,
  active,
  onHoverChange,
  onPress,
}: {
  name: string;
  active: boolean;
  onHoverChange?: (on: boolean) => void;
  onPress?: () => void;
}) {
  return (
    <Button
      variant="bare"
      size="none"
      onHoverChange={onHoverChange}
      onPress={onPress}
      className={`h-ctl w-full gap-2.5 rounded-btn px-2.5 text-left text-body ${active ? "bg-chip text-text" : "bg-transparent text-label"}`}
    >
      <span className="size-[7px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
      <span>{name}</span>
    </Button>
  );
}
