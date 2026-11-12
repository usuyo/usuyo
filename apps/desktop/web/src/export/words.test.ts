import { expect, it } from "vitest";
import type { ExportRun } from "../api/endpoints";
import { runText, willText } from "./words";

it("says what an export will write, and that private items stay out", () => {
  expect(willText({ items: 39, topics: 7, drafts: 1, privateLeft: 1 })).toBe("会写出 39 篇条目、7 篇主题页、1 篇成稿。私密的 1 条不导出。");
  expect(willText({ items: 0, topics: 0, drafts: 0, privateLeft: 0 })).toBe("现在没有要写出的。");
});

it("sums up one export by what it did, and only what it did", () => {
  const run: ExportRun = { id: "e", at: "", state: "done", written: 46, readBack: [{ file: "a", title: "a", what: "记一句" }], both: [], left: [{ file: "b", reason: "读不了" }] };
  expect(runText(run)).toBe("写出 46 篇，读回 1 处，没读也没写 1 篇");
  expect(runText({ ...run, state: "failed", hint: "文件夹不在了" })).toBe("没导出：文件夹不在了");
});
