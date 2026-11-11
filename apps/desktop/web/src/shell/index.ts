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

/**
 * 开一个站点的登录窗口（src-tauri/src/login.rs）。只交站点名：登录页、要等的 cookie 由壳自己问内核，
 * cookie 由壳直接交给内核，前端从内核的 `plugins-updated` 事件知道登录好了。
 * 不在桌面壳里时回 false（浏览器标签页里开不了这种窗口）。壳回的错是给人看的一句话。
 */
export async function openLogin(site: string): Promise<boolean> {
  if (!isDesktopShell()) return false;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("open_login", { site });
  return true;
}

/**
 * 开取访客身份的窗口：不登录。只交站点名，打开哪一页、等哪些 cookie 由壳自己问内核；
 * 页面自己生成的访客 cookie 由壳直接交给内核，和同一主机的账号分开存。
 * `replace`：换一份。壳先清掉这个窗口那份网页数据，页面才会给一份新的；第一次取不用。
 */
export async function openVisitor(site: string, replace: boolean): Promise<boolean> {
  if (!isDesktopShell()) return false;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("open_visitor", { site, replace });
  return true;
}

/** 去掉访客身份时清掉那个窗口的网页数据（cookie、缓存）；再取就从一个干净的窗口开始。 */
export async function clearVisitorData(site: string): Promise<void> {
  if (!isDesktopShell()) return;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("clear_visitor", { site });
}

/** 退出登录以后清掉这个站点那份网页数据（登录窗口的 cookie、缓存），下次登录从头来。 */
export async function clearLoginData(site: string): Promise<void> {
  if (!isDesktopShell()) return;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("clear_login", { site });
}

// 以后放在这里：托盘、全局快捷键。
