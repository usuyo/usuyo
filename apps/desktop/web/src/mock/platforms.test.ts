import { describe, expect, it } from "vitest";
import { createMockPlatforms } from "./platforms";

describe("mock platforms", () => {
  it("lists the three plugins with labels and a pause with its reason", async () => {
    const { plugins } = await createMockPlatforms().plugins();
    expect(plugins.map((p) => p.name)).toEqual(["小红书", "微信视频号", "B站"]);
    const bili = plugins.find((p) => p.id === "usuyo-fetch/bilibili");
    expect(bili?.pauses[0]).toMatchObject({ code: "rate-limited", byYou: false });
    expect(bili?.pauses[0]?.until).toBeDefined();
  });

  it("holds and tries a plugin again", async () => {
    const m = createMockPlatforms();
    expect((await m.hold("usuyo-fetch/bilibili")).pauses).toEqual([{ scope: "*", byYou: true }]);
    expect((await m.tryNow("usuyo-fetch/bilibili")).pauses).toEqual([]);
  });

  it("marks a site logged in once its named cookies arrive, never echoing values", async () => {
    const m = createMockPlatforms();
    const s = await m.saveLogin("www.bilibili.com", { cookies: [{ name: "SESSDATA", value: "secret", domain: ".bilibili.com" }] });
    expect(s.login).toBe("present");
    expect(JSON.stringify(s)).not.toContain("secret");
  });

  it("previews an import, then takes it after you confirm", async () => {
    const m = createMockPlatforms();
    const started = await m.startImport({ pluginId: "usuyo-fetch/bilibili", source: "bilibili:toview" });
    expect(started).toMatchObject({ state: "ready", title: "稍后再看", preview: { new: 16, stored: 2, trashed: 0 } });
    expect(await m.importEntries(started.id, "new")).toHaveLength(16);
    const confirmed = await m.confirmImport(started.id, {});
    expect(confirmed.state).toBe("taking");
    expect((await m.holdImport(started.id)).state).toBe("held");
  });

  it("shows a new deliver token once and lists it without the secret", async () => {
    const m = createMockPlatforms();
    const t = await m.createDeliverToken({ name: "手机", kind: "mobile" });
    expect(t.token.startsWith(`${t.id}.`)).toBe(true);
    const list = await m.deliverTokens();
    expect(list.some((x) => x.id === t.id && !("token" in x))).toBe(true);
  });
});
