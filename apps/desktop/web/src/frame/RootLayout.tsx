import { useQueryClient } from "@tanstack/react-query";
import { Outlet, useNavigate } from "@tanstack/react-router";
import { Fragment, useEffect, useRef, useState } from "react";
import { startLiveUpdates } from "../api/events";
import { queryKeys, useSettings } from "../api/hooks";
import { setLanguageChoice, useLocale } from "../i18n";
import { useLocaleStore } from "../i18n/locale";
import { titleBarHidden } from "../shell";
import { useThemeSync } from "../theme/apply";

/**
 * 最外层：主题和语言跟着设置走（换了语言整页重新挂上，内核给的话重新取）；内核推事件就刷新；⌥Space 叫出收下浮层（桌面壳以后换成全局快捷键）。
 * 窗口开的时候去掉了系统标题栏（macOS），顶上留一条自己的：红绿灯浮在它上面，拖它移动窗口、双击放大，上面什么控件都不放。
 */
export function RootLayout() {
  useThemeSync();
  const queryClient = useQueryClient();
  useEffect(() => startLiveUpdates(queryClient), [queryClient]);
  const navigate = useNavigate();
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.altKey && e.code === "Space") {
        e.preventDefault();
        void navigate({ to: "/take" });
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [navigate]);
  const locale = useLanguageSync(queryClient);
  const [bare, setBare] = useState(false);
  useEffect(() => {
    titleBarHidden().then(setBare, (error: unknown) => console.warn("usuyo: could not tell whether the window has a system title bar", error));
  }, []);
  const page = (
    <Fragment key={locale}>
      <Outlet />
    </Fragment>
  );
  if (!bare) return page;
  return (
    <div className="relative h-full">
      <div data-tauri-drag-region className="absolute inset-x-0 top-0 flex h-titlebar">
        <div data-tauri-drag-region className="h-full w-rail shrink-0 border-r border-border bg-rail" />
        <div data-tauri-drag-region className="h-full min-w-0 flex-1 bg-bg" />
      </div>
      <div className="absolute inset-x-0 top-titlebar bottom-0">{page}</div>
    </div>
  );
}

/**
 * 语言：内核的设置到了就照它（这台电脑上次记下的只管第一眼）；「跟随系统」时系统换了语言也跟着换。
 * 换了以后让设置以外的查询重新取，内核按新语言给话。
 */
function useLanguageSync(queryClient: ReturnType<typeof useQueryClient>) {
  const locale = useLocale();
  const chosen = useSettings().data?.language;
  useEffect(() => {
    if (chosen !== undefined && chosen !== useLocaleStore.getState().choice) setLanguageChoice(chosen);
  }, [chosen]);
  useEffect(() => {
    const follow = () => {
      const { choice } = useLocaleStore.getState();
      if (choice === "system") setLanguageChoice("system");
    };
    window.addEventListener("languagechange", follow);
    return () => window.removeEventListener("languagechange", follow);
  }, []);
  const shown = useRef(locale);
  useEffect(() => {
    document.documentElement.lang = locale;
    if (shown.current === locale) return;
    shown.current = locale;
    // 设置本身不带内核写的话；不重取它，免得一个早发出的读回把刚改的语言又换回去。
    void queryClient.invalidateQueries({ predicate: (q) => q.queryKey[0] !== queryKeys.settings[0] });
  }, [locale, queryClient]);
  return locale;
}
