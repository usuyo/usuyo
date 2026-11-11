// 唯一可以引入 Tauri 接口的模块。其余代码在普通浏览器标签页里也要能跑，
// 所以这里的能力在非桌面环境下都是空操作。换壳只改这一处。

export function isDesktopShell(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function currentWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

export async function minimizeWindow(): Promise<void> {
  if (isDesktopShell()) await (await currentWindow()).minimize();
}

export async function toggleMaximizeWindow(): Promise<void> {
  if (isDesktopShell()) await (await currentWindow()).toggleMaximize();
}

export async function closeWindow(): Promise<void> {
  if (isDesktopShell()) await (await currentWindow()).close();
}

// 以后放在这里：托盘、全局快捷键、登录窗口（见 src-tauri/src/main.rs）。
