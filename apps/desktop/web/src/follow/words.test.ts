import { expect, it } from "vitest";
import { hourChoices, rhythmText, takeText, trialText } from "./words";

const name = (id: string) => (id === "film" ? "胶片摄影" : id);

it("says what a follow takes", () => {
  expect(takeText({ all: true }, name)).toBe("新出现的都收");
  expect(takeText({ all: false, topics: ["film"], words: ["C41"] }, name)).toBe("只收命中「胶片摄影」的关键词、「C41」的");
});

it("says how often it looks, and why it waits without blaming the others", () => {
  const now = new Date("2026-11-12T08:00:00");
  expect(rhythmText({ on: true, everyHours: 6, nextAt: "2026-11-12T12:40:00" }, now)).toBe("每 6 小时看一次，下次 12:40");
  expect(rhythmText({ on: false, everyHours: 6 }, now)).toBe("你停下了，不再去看");
  expect(rhythmText({ on: true, everyHours: 6, waiting: { code: "rate-limited", hint: "平台说请求太频繁", until: "2026-11-12T13:44:00" } }, now)).toBe("平台说请求太频繁，只停这一个，13:44 自己再试");
  expect(rhythmText({ on: true, everyHours: 6, waiting: { code: "login-required" } }, now)).toBe("login-required，只停这一个，等你动手");
});

it("words a trial run by what happens on the first real run", () => {
  expect(trialText({ title: "t", firstRun: "record", listed: 5, would: [], left: { stored: 0, trashed: 0, unmatched: 3 } })).toBe(
    "这一次列到 5 条，其中没命中 3 条。第一次只记下现在有的，不收；以后新出现的，照下面这样收。",
  );
  expect(trialText({ title: "t", firstRun: "take", listed: 24, would: [], left: { stored: 21, trashed: 1, unmatched: 0 } })).toBe(
    "这一次列到 24 条，其中已经存着 21 条、在回收站里 1 条。新加进去的直接收下，不逐条看；下面是这一次会收下的。",
  );
});

it("offers rhythms from the plugin's fastest up", () => {
  expect(hourChoices(12).map((c) => c.id)).toEqual(["12", "24", "72"]);
});
