import { describe, expect, it } from "vitest";
import type { TrashEntry } from "../api/endpoints";
import { setLanguageChoice } from "../i18n";
import { clearsText, doneText, introText, trashedText, whatText } from "./words";

const base: TrashEntry = {
  id: "0193f2b0-0000-7000-8000-000000000001",
  reason: "delete",
  itemId: "k2",
  title: "泡菜坛子怎么养",
  platform: { id: "bilibili", name: "B站" },
  author: "厨房",
  trashedAt: new Date(2026, 10, 3, 12).toISOString(),
  expiresAt: new Date(2026, 11, 3, 12).toISOString(),
};

describe("trash words", () => {
  it("says plainly when an entry is cleared, without counts", () => {
    const now = new Date(2026, 10, 11, 9);
    expect(clearsText(base, now)).toBe("将于12月3日删除");
    expect(clearsText({ ...base, expiresAt: new Date(2026, 10, 10).toISOString() }, now)).toBe("已到期，将在下次检查时删除");
    const { expiresAt: _e, ...forever } = base;
    expect(clearsText(forever, now)).toBe("永久保留");
    expect(introText(30)).toContain("保留 30 天");
    expect(introText(null)).toContain("永久保留");
  });

  it("tells a whole item from the media moved out by keep-text", () => {
    expect(trashedText(base)).toBe("11月3日删除");
    expect(whatText(base)).toBe("B站 · 厨房 · 删除的条目");
    const media = { ...base, reason: "keep-text" as const };
    expect(trashedText(media)).toBe("11月3日移入");
    expect(whatText(media)).toContain("条目和文字仍在");
    expect(doneText(media, "restore")).toBe("图片和视频已放回「泡菜坛子怎么养」");
    expect(doneText(base, "purge")).toBe("已删除「泡菜坛子怎么养」");
  });

  it("says the same in English, with singular and plural days", () => {
    setLanguageChoice("en");
    const now = new Date(2026, 10, 11, 9);
    expect(clearsText(base, now)).toBe("Will be deleted Dec 3");
    expect(trashedText(base)).toBe("Deleted Nov 3");
    expect(introText(1)).toContain("stay here for 1 day,");
    expect(introText(30)).toContain("stay here for 30 days,");
    expect(doneText(base, "purge")).toBe("Deleted “泡菜坛子怎么养”");
  });
});
