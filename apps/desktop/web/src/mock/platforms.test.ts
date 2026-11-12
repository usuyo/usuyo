import { afterEach, describe, expect, it, vi } from "vitest";
import { createMockPlatforms } from "./platforms";

afterEach(() => {
  vi.useRealTimers();
});

describe("mock platforms", () => {
  it("lists one plugin per platform with every method, and a spread of states the daemon gives", async () => {
    const { plugins } = await createMockPlatforms().plugins();
    expect(plugins.map((p) => p.name)).toEqual(["X", "小红书", "微信视频号", "B站", "yt-dlp"]);
    const x = plugins.find((p) => p.id === "usuyo-fetch/x");
    expect(x?.methods).toHaveLength(9);
    expect(x?.methods.filter((m) => m.built).map((m) => m.name)).toEqual(["访客接口", "嵌入接口", "浏览器脚本", "官方归档"]);
    const all = plugins.flatMap((p) => p.methods);
    // 用得上的就是没有原因的。
    expect(all.every((m) => m.usable === (m.blockers.length === 0))).toBe(true);
    const codes = new Set(all.flatMap((m) => m.blockers.map((b) => b.code)));
    for (const code of ["off", "not-built", "login-missing", "visitor-missing", "check-failed", "paused"]) expect(codes.has(code)).toBe(true);
    expect(all.filter((m) => m.drives !== undefined).map((m) => m.drives)).toEqual(["iphone", "iphone", "desktop-app", "iphone"]);
    // 没做好的也说做好以后取得到什么。
    expect(all.filter((m) => !m.built).every((m) => Object.keys(m.gets).length > 0)).toBe(true);
    expect(all.some((m) => m.lastCall?.ok === false)).toBe(true);
    expect(all.some((m) => m.account.kind === "own-app")).toBe(true);
    const bili = plugins.find((p) => p.id === "usuyo-fetch/bilibili");
    expect(bili?.methods.find((m) => m.id === "favorites")?.pauses[0]).toMatchObject({ method: "favorites", code: "rate-limited", byYou: false });
    expect(bili?.pauses.map((p) => p.method)).toEqual(["favorites"]);
    expect(bili?.methods.find((m) => m.id === "favorites")).toMatchObject({ usable: false, blockers: [{ code: "paused" }] });
  });

  it("keeps one switch for a method with per-platform tiers: once switched, every platform follows", async () => {
    const m = createMockPlatforms();
    const before = (await m.plugins()).plugins.find((p) => p.id === "usuyo-fetch/ytdlp")?.methods[0];
    expect(before).toMatchObject({ enabled: true, perPlatform: [{ platform: "youtube", enabled: false }, { platform: "weibo", enabled: false }] });
    const after = (await m.switchMethod("usuyo-fetch/ytdlp", "ytdlp", true)).methods[0];
    expect(after?.perPlatform?.every((t) => t.enabled)).toBe(true);
  });

  it("switches a built method and its reasons follow, and refuses one that is not built", async () => {
    const m = createMockPlatforms();
    const x = await m.switchMethod("usuyo-fetch/x", "embed", true);
    expect(x.methods.find((y) => y.id === "embed")).toMatchObject({ enabled: true, usable: true, blockers: [] });
    const off = await m.switchMethod("usuyo-fetch/x", "guest", false);
    expect(off.methods.find((y) => y.id === "guest")).toMatchObject({ enabled: false, usable: false, blockers: [{ code: "off" }] });
    expect(x.methods.filter((y) => !y.built).map((y) => y.name)).toEqual(["内容投递", "会话接口", "签名接口", "官方 API", "手机镜像"]);
    await expect(m.switchMethod("usuyo-fetch/x", "signed", true)).rejects.toThrow();
  });

  it("lists sources per method, from a link for a method that starts from one", async () => {
    const m = createMockPlatforms();
    const fromLink = await m.sources("usuyo-fetch/bilibili", "favorites-public", "https://space.bilibili.com/1/favlist?fid=2");
    expect(fromLink.every((s) => s.method === "favorites-public")).toBe(true);
    await expect(m.sources("usuyo-fetch/bilibili", "favorites-public")).rejects.toThrow();
    await expect(m.sources("usuyo-fetch/bilibili", "view")).rejects.toThrow();
  });

  it("holds a whole plugin and tries it again", async () => {
    const m = createMockPlatforms();
    const held = await m.hold("usuyo-fetch/bilibili");
    expect(held.pauses).toContainEqual({ method: "*", scope: "*", byYou: true });
    expect(held.methods.every((x) => x.pauses.some((p) => p.byYou))).toBe(true);
    expect(held.methods.filter((x) => x.built).every((x) => x.blockers.some((b) => b.code === "paused"))).toBe(true);
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
    const started = await m.startImport({ pluginId: "usuyo-fetch/bilibili", method: "favorites", source: "bilibili:toview" });
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
