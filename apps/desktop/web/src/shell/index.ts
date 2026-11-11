// 唯一可以引入 Tauri 接口的模块。其余代码在普通浏览器标签页里也要能跑，
// 所以这里的能力在非桌面环境下都是空操作。换壳只改这一处。

export function isDesktopShell(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** 桌面壳给的内核地址和令牌；壳会等内核起来再回。不在桌面壳里时是 null。 */
export async function daemonConnection(): Promise<{ base: string; token: string } | null> {
  if (!isDesktopShell()) return null;
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<{ base: string; token: string }>("daemon_connection");
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

/** 只认 http、https 的网址；别的（`javascript:`、`file:`、写坏的）回 null。原帖地址来自资料库，是不可信的字。 */
export function webUrl(url: string): string | null {
  if (!URL.canParse(url)) return null;
  const u = new URL(url);
  return u.protocol === "http:" || u.protocol === "https:" ? u.href : null;
}

/**
 * 在系统浏览器里打开原帖，回打开了没有。桌面壳里交给 opener 插件（壳那边也只放行 http、https）；
 * 浏览器里开一个新标签页。不是网页地址就什么都不做。
 */
export async function openExternal(url: string): Promise<boolean> {
  const safe = webUrl(url);
  if (safe === null) return false;
  if (!isDesktopShell()) {
    window.open(safe, "_blank", "noopener,noreferrer");
    return true;
  }
  try {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(safe);
    return true;
  } catch (error) {
    console.warn("usuyo: 没能打开原帖", error);
    return false;
  }
}

// 以后放在这里：托盘、全局快捷键、登录窗口（见 src-tauri/src/main.rs）。
