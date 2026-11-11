import { describe, expect, it } from "vitest";
import type { TrashEntry } from "../api/endpoints";
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
    expect(clearsText(base, now)).toBe("12 月 3 日清除");
    expect(clearsText({ ...base, expiresAt: new Date(2026, 10, 10).toISOString() }, now)).toBe("到期了，下一次检查时清除");
    const { expiresAt: _e, ...forever } = base;
    expect(clearsText(forever, now)).toBe("一直留着");
    expect(introText(30)).toContain("放 30 天");
    expect(introText(null)).toContain("一直留着");
  });

  it("tells a whole item from the media moved out by keep-text", () => {
    expect(trashedText(base)).toBe("11 月 3 日删掉");
    expect(whatText(base)).toBe("B站 · 厨房　·　删掉的整条");
    const media = { ...base, reason: "keep-text" as const };
    expect(trashedText(media)).toBe("11 月 3 日移进来");
    expect(whatText(media)).toContain("条目和文字还在");
    expect(doneText(media, "restore")).toBe("图和视频放回了「泡菜坛子怎么养」");
    expect(doneText(base, "purge")).toBe("清除了「泡菜坛子怎么养」");
  });
});
