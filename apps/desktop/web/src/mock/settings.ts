// 样例库的设置：和内核同一套接口，在内存里。回收站的天数和样例库的回收站共用一个值。
import type { SettingsView } from "../api/endpoints";
import type { Settings } from "../api/source";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export function createMockSettings(trash: { days(): number | null; setDays(days: number | null): void }, empty: boolean): Settings {
  const view: Omit<SettingsView, "trashDays"> = {
    library: "~/Documents/usuyo",
    // 样例里放一次「时钟跳了」：好看到那一句和它唯一的那个按钮。
    ...(empty ? {} : { trashHeld: { count: 3, since: "2026-08-02T12:00:00.000Z" } }),
    network: { mode: "system", fakeIp: true },
    transforms: { auto: true },
    window: { systemTitleBar: false },
    language: "system",
    logs: "~/Library/Application Support/usuyo/logs/daemon.log",
  };
  const snapshot = (): SettingsView => ({ ...view, network: { ...view.network }, transforms: { ...view.transforms }, window: { ...view.window }, trashDays: trash.days() });
  return {
    async settings() {
      await wait(40);
      return snapshot();
    },
    async changeSettings(patch) {
      await wait(60);
      if (patch.trashDays !== undefined) trash.setDays(patch.trashDays);
      if (patch.network) {
        if (patch.network.mode === "custom" && !/^(https?|socks5):\/\/\S+$/.test(patch.network.proxy ?? "")) throw new Error("invalid-args: proxy address");
        view.network = { ...patch.network, fakeIp: view.network.fakeIp };
      }
      if (patch.transforms) view.transforms = { ...patch.transforms };
      if (patch.window) view.window = { ...patch.window };
      if (patch.language) view.language = patch.language;
      return snapshot();
    },
    async releaseHeld() {
      await wait(80);
      const removed = view.trashHeld?.count ?? 0;
      delete view.trashHeld;
      return { removed };
    },
  };
}
