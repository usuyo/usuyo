import { useQueryClient } from "@tanstack/react-query";
import { Outlet, useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { startLiveUpdates } from "../api/events";
import { titleBarHidden } from "../shell";
import { useThemeSync } from "../theme/apply";

/**
 * 最外层：主题跟着设置走；内核推事件就刷新；⌥Space 叫出收下浮层（桌面壳以后换成全局快捷键）。
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
  const [bare, setBare] = useState(false);
  useEffect(() => {
    titleBarHidden().then(setBare, (error: unknown) => console.warn("usuyo: 没能问到窗口有没有系统标题栏", error));
  }, []);
  if (!bare) return <Outlet />;
  return (
    <div className="relative h-full">
      <div data-tauri-drag-region className="absolute inset-x-0 top-0 flex h-titlebar">
        <div data-tauri-drag-region className="h-full w-rail shrink-0 border-r border-border bg-rail" />
        <div data-tauri-drag-region className="h-full min-w-0 flex-1 bg-bg" />
      </div>
      <div className="absolute inset-x-0 top-titlebar bottom-0">
        <Outlet />
      </div>
    </div>
  );
}
