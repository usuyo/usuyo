import { expect, it } from "vitest";
import { setLanguageChoice, t } from "../i18n";
import { hourChoices, rhythmText, takeText, trialText } from "./words";

const name = (id: string) => (id === "film" ? "胶片摄影" : id);

it("says what a follow takes", () => {
  expect(takeText({ all: true }, name)).toBe("添加所有新内容");
  expect(takeText({ all: false, topics: ["film"], words: ["C41"] }, name)).toBe("仅添加匹配「胶片摄影」的关键词和「C41」的内容");
});

it("says how often it looks, and why it waits without blaming the others", () => {
  const now = new Date("2026-11-12T08:00:00");
  expect(rhythmText({ on: true, everyHours: 6, nextAt: "2026-11-12T12:40:00" }, now)).toBe("每 6 小时检查一次，下次在 12:40");
  expect(rhythmText({ on: false, everyHours: 6 }, now)).toBe("已暂停");
  expect(rhythmText({ on: true, everyHours: 6, waiting: { code: "rate-limited", hint: "平台说请求太频繁", until: "2026-11-12T13:44:00" } }, now)).toBe("平台说请求太频繁。此订阅已暂停，将于 13:44 重试。");
  expect(rhythmText({ on: true, everyHours: 6, waiting: { code: "login-required" } }, now)).toBe("login-required。此订阅已暂停。");
});

it("words a trial run by what happens on the first real run", () => {
  expect(trialText({ title: "t", firstRun: "record", listed: 5, would: [], left: { stored: 0, trashed: 0, unmatched: 3 } })).toBe(
    "这次列出 5 条，其中 3 条不匹配。首次运行只记录现有内容，不添加；之后新出现的内容按以下设置添加。",
  );
  expect(trialText({ title: "t", firstRun: "take", listed: 24, would: [], left: { stored: 21, trashed: 1, unmatched: 0 } })).toBe(
    `这次列出 24 条，其中 21 条已保存、1 条在${t("common.os.linux.trash")}中。新加入的内容会直接添加，无需逐条确认。以下是这次会添加的内容。`,
  );
});

it("offers rhythms from the plugin's fastest up", () => {
  expect(hourChoices(12).map((c) => c.id)).toEqual(["12", "24", "72"]);
});

it("says the rhythm and a trial in English, singular and plural", () => {
  setLanguageChoice("en");
  expect(rhythmText({ on: true, everyHours: 1 })).toBe("Checks every hour");
  expect(rhythmText({ on: true, everyHours: 6 })).toBe("Checks every 6 hours");
  expect(trialText({ title: "t", firstRun: "take", listed: 1, would: [], left: { stored: 0, trashed: 0, unmatched: 1 } })).toBe(
    "Found 1 post: 1 doesn't match. New posts are added directly, without review. These would be added this time.",
  );
  expect(hourChoices(6).map((c) => c.text)).toEqual(["Every 6 hours", "Every 12 hours", "Once a day", "Every 3 days"]);
});
