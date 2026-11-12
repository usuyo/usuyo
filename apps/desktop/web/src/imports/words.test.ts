import { describe, expect, it } from "vitest";
import type { ImportView } from "../api/endpoints";
import { acceptOf, stateLine, waitingText } from "./words";

const base: ImportView = {
  id: "i",
  kind: "source",
  title: "稍后再看",
  createdAt: "2026-11-11T09:00:00.000Z",
  state: "taking",
  pages: 2,
  preview: { new: 20, stored: 3, trashed: 1 },
  taken: 7,
  notStored: 1,
  skipped: 0,
};

describe("import words", () => {
  it("states facts and never a remaining count or a percentage", () => {
    for (const state of ["listing", "ready", "taking", "held", "done"] as const) {
      const line = stateLine({ ...base, state });
      expect(line).not.toMatch(/%|还剩|剩下/);
    }
    expect(stateLine(base)).toBe("收下了 7 条 · 在收");
    expect(stateLine({ ...base, state: "done" })).toBe("收下了 7 条，1 条内容没取到");
  });

  it("explains why an import waits", () => {
    const now = new Date(2026, 10, 11, 14, 0, 0);
    const until = new Date(2026, 10, 11, 14, 0, 5).toISOString();
    expect(waitingText({ code: "pacing", until }, "B站", undefined, now)).toBe("按B站的节奏，下一条大概 14:00:05");
    expect(waitingText({ code: "login-required" }, "B站", undefined, now)).toBe("要重新登录B站：登录好了点「接着导入」");
    expect(waitingText({ code: "visitor-required" }, "小红书", undefined, now)).toBe("要访客身份；好了点「接着导入」");
    expect(waitingText({ code: "visitor-required" }, "小红书", "missing", now)).toBe(
      "要先取访客身份：在「获取插件」页的「访客身份」里取一份；好了点「接着导入」",
    );
    expect(waitingText({ code: "visitor-required" }, "小红书", "present", now)).toBe("这份访客身份不管用了，换一份再试；好了点「接着导入」");
    expect(waitingText({ code: "blocked", hint: "B站挡住了" }, "B站", undefined, now)).toBe("B站挡住了 · 等你点「接着导入」");
  });
});

it("filters the file picker by the extensions a reader's patterns name", () => {
  expect(acceptOf(["twitter-*.zip"])).toEqual([".zip"]);
  expect(acceptOf(["*.json", "*.CSV", "*.json"])).toEqual([".json", ".csv"]);
  expect(acceptOf(["archive"])).toEqual([]);
});
