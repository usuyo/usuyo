import { describe, expect, it } from "vitest";
import { createMockLibrary } from "./library";

describe("mock library", () => {
  it("finds a word in the speech text with its second", async () => {
    const lib = createMockLibrary(false);
    const hits = await lib.search("手冲");
    const c1 = hits.find((h) => h.itemId === "c1");
    expect(c1?.layer).toBe("asr");
    expect(c1?.position).toMatchObject({ kind: "time", sec: 12 });
    expect(c1?.snippet.slice(c1.match[0], c1.match[1])).toBe("手冲");
  });

  it("keeps every facet an item has", async () => {
    const item = await createMockLibrary(false).item("c1");
    expect(item.facets.map((f) => f.layer)).toEqual(["video", "asr", "ocr", "text", "comments"]);
  });

  it("says a share it has seen before is seen, and lands a new one", async () => {
    const lib = createMockLibrary(false);
    const seen = await lib.take({ input: "【4:6 冲煮法复刻（一）：比例-哔哩哔哩】 https://b23.tv/x", kind: "link" });
    expect(seen).toMatchObject({ id: "c1", seen: true });
    const fresh = await lib.take({ input: "https://www.xiaohongshu.com/explore/abc", kind: "link" });
    expect(fresh.seen).toBe(false);
    expect(fresh.platform.name).toBe("小红书");
    expect((await lib.map()).points.some((p) => p.itemId === fresh.id)).toBe(true);
  });

  it("writes every time as RFC 3339 UTC with milliseconds", async () => {
    for (const p of (await createMockLibrary(false).map()).points) {
      expect(new Date(p.takenAt).toISOString()).toBe(p.takenAt);
    }
  });

  it("creates a topic by name once and puts an item in it", async () => {
    const lib = createMockLibrary(false);
    const a = await lib.createTopic(" 新主题 ");
    expect(a.name).toBe("新主题");
    expect((await lib.createTopic("新主题")).id).toBe(a.id);
    const item = await lib.setTopic("c1", a.id, true);
    expect(item.topicIds).toContain(a.id);
  });

  it("lists the trash newest first and clears or brings back an entry", async () => {
    const lib = createMockLibrary(false);
    const before = await lib.trashList();
    expect(before.entries.map((e) => [e.itemId, e.reason])).toEqual([
      ["k2", "delete"],
      ["f3", "keep-text"],
      ["e4", "delete"],
      ["w3", "keep-text"],
    ]);
    expect((await lib.map()).points.some((p) => p.itemId === "k2")).toBe(false);
    await lib.restore("k2");
    await lib.restoreMedia("f3");
    const [e4] = (await lib.trashList()).entries;
    await lib.purge(e4?.id ?? "");
    expect((await lib.trashList()).entries.map((e) => e.itemId)).toEqual(["w3"]);
    expect((await lib.map()).points.some((p) => p.itemId === "e4")).toBe(false);
  });

  it("starts empty when asked", async () => {
    expect((await createMockLibrary(true).map()).points).toHaveLength(0);
  });
});
