import { afterEach, describe, expect, it, vi } from "vitest";
import { createMockPlatforms } from "./platforms";

afterEach(() => {
  vi.useRealTimers();
});

describe("mock platforms", () => {
  it("lists one plugin per platform with its methods, and a paused method with its reason", async () => {
    const { plugins } = await createMockPlatforms().plugins();
    expect(plugins.map((p) => p.name)).toEqual(["X", "小红书", "微信视频号", "B站"]);
    const x = plugins.find((p) => p.id === "usuyo-fetch/x");
    expect(x?.methods).toHaveLength(9);
    expect(x?.methods.filter((m) => m.built).map((m) => m.name)).toEqual(["访客接口", "嵌入接口", "浏览器脚本", "官方归档"]);
    // 没做好的、要用私有接口或改客户端的都关着；要在 usuyo 里登录、还没登录的也关着。
    expect(plugins.flatMap((p) => p.methods).filter((m) => m.access === "private-api" || m.access === "client-hook").every((m) => !m.enabled)).toBe(true);
    expect(plugins.find((p) => p.id === "usuyo-fetch/wechat-channels")?.methods.find((m) => m.id === "yuanbao")?.enabled).toBe(false);
    const lists = plugins.find((p) => p.id === "usuyo-fetch/bilibili")?.methods.find((m) => m.id === "favorites");
    expect(lists?.pauses[0]).toMatchObject({ code: "rate-limited", byYou: false });
    expect(lists?.pauses[0]?.until).toBeDefined();
  });

  it("switches a built method, and refuses one that is not built", async () => {
    const m = createMockPlatforms();
    const x = await m.switchMethod("usuyo-fetch/x", "embed", false);
    expect(x.methods.find((y) => y.id === "embed")?.enabled).toBe(false);
    expect(x.methods.filter((y) => !y.built).map((y) => y.name)).toEqual(["内容投递", "会话接口", "签名接口", "官方 API", "手机镜像"]);
    await expect(m.switchMethod("usuyo-fetch/x", "signed", true)).rejects.toThrow();
  });

  it("holds and tries a plugin again", async () => {
    const m = createMockPlatforms();
    expect((await m.hold("usuyo-fetch/bilibili")).pauses).toEqual([{ scope: "*", byYou: true }]);
    expect((await m.tryNow("usuyo-fetch/bilibili")).pauses).toEqual([]);
  });

  it("starts empty when asked, with nothing installed", async () => {
    const m = createMockPlatforms(true);
    expect((await m.plugins()).plugins).toEqual([]);
    expect(await m.sites()).toEqual([]);
    expect(await m.imports()).toEqual([]);
  });

  it("lists page by page, previews, then takes at the plugin's pace after you confirm", async () => {
    vi.useFakeTimers({ toFake: ["Date"] });
    const m = createMockPlatforms();
    const started = await m.startImport({ pluginId: "usuyo-fetch/bilibili", source: "bilibili:toview" });
    expect(started).toMatchObject({ state: "listing", title: "稍后再看", pages: 0 });
    vi.advanceTimersByTime(1300);
    expect(await m.import(started.id)).toMatchObject({ state: "listing", pages: 1 });
    vi.advanceTimersByTime(4000);
    const ready = await m.import(started.id);
    expect(ready).toMatchObject({ state: "ready", preview: { new: 15, stored: 2, trashed: 1 } });
    expect(await m.importEntries(started.id, "new", 0, 100)).toHaveLength(15);
    expect(await m.importEntries(started.id, "new", 10, 100)).toHaveLength(5);
    const [first] = await m.importEntries(started.id, "new", 0, 1);
    expect((await m.confirmImport(started.id, { new: true, trashed: false, except: [first?.seq ?? -1] })).state).toBe("taking");
    vi.advanceTimersByTime(5100);
    expect((await m.import(started.id)).taken).toBe(1);
    expect((await m.importEntries(started.id, "new", 0, 2)).map((e) => e.taken)).toEqual([false, true]);
    const held = await m.holdImport(started.id);
    expect(held.state).toBe("held");
    expect(held.waiting).toBeUndefined();
  });

  it("shows a new deliver token once and lists it without the secret", async () => {
    const m = createMockPlatforms();
    const t = await m.createDeliverToken({ name: "手机", kind: "mobile" });
    expect(t.token.startsWith(`${t.id}.`)).toBe(true);
    const list = await m.deliverTokens();
    expect(list.some((x) => x.id === t.id && !("token" in x))).toBe(true);
  });
});
