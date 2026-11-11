import { Link } from "@tanstack/react-router";
import { FolderInput, FolderOutput, History, Monitor, Moon, Plug, Rss, Settings, Sun, Trash2 } from "lucide-react";
import type { ReactNode } from "react";
import { useUiStore, type ThemePreference } from "../store/ui";
import { Button } from "../ui/Button";
import { TakeField } from "./TakeField";

const LATER: { page: string; label: string; icon: ReactNode }[] = [
  { page: "follow", label: "跟着收", icon: <Rss size={16} aria-hidden="true" /> },
  { page: "import", label: "导入", icon: <FolderInput size={16} aria-hidden="true" /> },
  { page: "export", label: "导出", icon: <FolderOutput size={16} aria-hidden="true" /> },
  { page: "changes", label: "改动记录", icon: <History size={16} aria-hidden="true" /> },
  { page: "trash", label: "回收站", icon: <Trash2 size={16} aria-hidden="true" /> },
  { page: "plugins", label: "获取插件", icon: <Plug size={16} aria-hidden="true" /> },
  { page: "settings", label: "设置", icon: <Settings size={16} aria-hidden="true" /> },
];

const NEXT_THEME: Record<ThemePreference, ThemePreference> = { system: "dark", dark: "light", light: "system" };
const THEME_LABEL: Record<ThemePreference, string> = { system: "外观跟系统", dark: "深色", light: "浅色" };

function ThemeSwitch() {
  const theme = useUiStore((s) => s.theme);
  const setTheme = useUiStore((s) => s.setTheme);
  const Icon = theme === "system" ? Monitor : theme === "dark" ? Moon : Sun;
  return (
    <Button
      variant="ghost"
      size="xs"
      square
      aria-label={`${THEME_LABEL[theme]}，点一下换成${THEME_LABEL[NEXT_THEME[theme]]}`}
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
      className="absolute top-0 left-0 z-20 flex h-full w-rail flex-col border-r border-border bg-rail px-4 pt-5 pb-4"
    >
      <div className="flex h-7 items-center justify-between px-1">
        <span className="text-name font-semibold tracking-[0.005em] text-text">usuyo</span>
        <ThemeSwitch />
      </div>
      <TakeField />
      {children}
      <div className="flex-1" />
      {LATER.map((l) => (
        <Link
          key={l.page}
          to="/later/$page"
          params={{ page: l.page }}
          className="press flex h-ctl-sm items-center gap-2.5 rounded-btn px-2.5 text-note text-sub no-underline hover:bg-chip/60 hover:text-label"
        >
          {l.icon}
          <span>{l.label}</span>
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
