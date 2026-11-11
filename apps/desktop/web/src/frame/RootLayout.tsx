import { Outlet, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { useThemeSync } from "../theme/apply";

/** 最外层：主题跟着设置走；⌥Space 叫出收下浮层（桌面壳以后换成全局快捷键）。 */
export function RootLayout() {
  useThemeSync();
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
  return <Outlet />;
}
