import { expect, it } from "vitest";
import type { ExportRun } from "../api/endpoints";
import { setLanguageChoice } from "../i18n";
import { runText, willText } from "./words";

it("says what an export will write, and that private items stay out", () => {
  expect(willText({ items: 39, topics: 7, drafts: 1, privateLeft: 1 })).toBe("导出时会写出 39 个条目、7 个主题页和 1 篇文稿。1 个私密条目不会导出。");
  expect(willText({ items: 0, topics: 0, drafts: 0, privateLeft: 0 })).toBe("目前没有可导出的内容。");
});

it("sums up one export by what it did, and only what it did", () => {
  const run: ExportRun = { id: "e", at: "", state: "done", written: 46, readBack: [{ file: "a", title: "a", what: "记一句" }], both: [], left: [{ file: "b", reason: "读不了" }] };
  expect(runText(run)).toBe("写出 46 篇笔记；读回 1 处更改；跳过 1 篇");
  expect(runText({ ...run, state: "failed", hint: "文件夹不在了" })).toBe("未能导出：文件夹不在了");
});

it("counts in English with singular and plural", () => {
  setLanguageChoice("en");
  expect(willText({ items: 1, topics: 2, drafts: 0, privateLeft: 3 })).toBe("Exporting writes 1 item and 2 topic pages. 3 private items aren't exported.");
  const run: ExportRun = { id: "e", at: "", state: "done", written: 1, readBack: [], both: [], left: [{ file: "b", reason: "x" }, { file: "c", reason: "y" }] };
  expect(runText(run)).toBe("Wrote 1 note; skipped 2 notes");
});
