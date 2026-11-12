import { expect, it } from "vitest";
import type { ChangeBatch, ChangeOp } from "../api/endpoints";
import { actorText, byWho, dayHeading, opText, stateText } from "./words";

const op = (o: Partial<ChangeOp> & Pick<ChangeOp, "op">): ChangeOp => ({ id: "o", target: { kind: "item", id: "c7", title: "这周的豆子" }, ...o });

it("says what each change did in plain words", () => {
  expect(opText(op({ op: "add-item" }))).toBe("收下「这周的豆子」");
  expect(opText(op({ op: "add-to-topic", after: "咖啡" }))).toBe("把「这周的豆子」放进「咖啡」");
  expect(opText(op({ op: "edit-page", target: { kind: "topic", id: "coffee", title: "咖啡" } }))).toBe("在「咖啡」里改一段");
  expect(opText(op({ op: "set-note" }))).toBe("给「这周的豆子」记一句");
});

it("names who did it and where a batch stands", () => {
  expect(actorText({ kind: "agent", name: "Codex", device: "m" })).toBe("Codex");
  expect(actorText({ kind: "readback", target: "Obsidian", device: "m" })).toBe("从 Obsidian 读回");
  expect(stateText({ state: "proposed" })).toBe("改动提议：你接受了才生效");
  expect(stateText({ state: "declined" })).toBe("都不要了，资料库没变");
});

it("heads each day as today, yesterday or a date", () => {
  const now = new Date("2026-11-12T10:00:00");
  expect(dayHeading("2026-11-12T01:00:00", now)).toBe("今天");
  expect(dayHeading("2026-11-11T23:00:00", now)).toBe("昨天");
  expect(dayHeading("2026-11-09T12:00:00", now)).toBe("11 月 9 日");
});

it("filters by who did it", () => {
  const b = (by: ChangeBatch["by"]): ChangeBatch => ({ id: "b", at: "", by, state: "applied", via: "", ops: [] });
  expect(byWho(b({ kind: "sync", device: "d" }), "other")).toBe(true);
  expect(byWho(b({ kind: "you", device: "d" }), "agent")).toBe(false);
});
