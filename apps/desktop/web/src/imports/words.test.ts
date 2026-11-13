import { describe, expect, it } from "vitest";
import type { ImportView } from "../api/endpoints";
import { setLanguageChoice } from "../i18n";
import { visitorText } from "../plugins/words";
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
      expect(line).not.toMatch(/%|还剩|剩下|剩余/);
    }
    expect(stateLine(base)).toBe("已添加 7 条 · 正在添加");
    expect(stateLine({ ...base, state: "done" })).toBe("已添加 7 条，其中 1 条未获取到内容");
  });

  it("explains why an import waits", () => {
    const now = new Date(2026, 10, 11, 14, 0, 0);
    const until = new Date(2026, 10, 11, 14, 0, 5).toISOString();
    expect(waitingText({ code: "pacing", until }, "B站", undefined, now)).toBe("按B站的请求频率，下一条约在 14:00:05");
    expect(waitingText({ code: "login-required" }, "B站", undefined, now)).toBe("需要重新登录B站：登录后点按「继续导入」");
    expect(waitingText({ code: "visitor-required" }, "小红书", undefined, now)).toBe(`${visitorText()}；完成后点按「继续导入」`);
    expect(waitingText({ code: "visitor-required" }, "小红书", "missing", now)).toBe(`${visitorText("missing")}；完成后点按「继续导入」`);
    expect(waitingText({ code: "blocked", hint: "B站挡住了" }, "B站", undefined, now)).toBe("B站挡住了 · 处理后点按「继续导入」");
  });
});

it("states an import in English with the right plural", () => {
  setLanguageChoice("en");
  expect(stateLine({ ...base, state: "listing", kind: "file", preview: { new: 1, stored: 0, trashed: 0 } })).toBe("Reading · 1 item found");
  expect(stateLine({ ...base, state: "listing" })).toBe("Listing · 24 items found");
  expect(stateLine({ ...base, state: "done" })).toBe("7 added; content not fetched for 1");
});

it("filters the file picker by the extensions a reader's patterns name", () => {
  expect(acceptOf(["twitter-*.zip"])).toEqual([".zip"]);
  expect(acceptOf(["*.json", "*.CSV", "*.json"])).toEqual([".json", ".csv"]);
  expect(acceptOf(["archive"])).toEqual([]);
});
