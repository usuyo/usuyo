import { describe, expect, it } from "vitest";
import { SseParser } from "./sse";

describe("SseParser", () => {
  it("reads events split across chunks", () => {
    const p = new SseParser();
    expect(p.push('event: item-added\ndata: {"type":"item-')).toEqual([]);
    expect(p.push('added","id":"web-a"}\n')).toEqual([]);
    expect(p.push("\nevent: item-updated\r\ndata: {}\r\n\r\n")).toEqual([
      { event: "item-added", data: '{"type":"item-added","id":"web-a"}' },
      { event: "item-updated", data: "{}" },
    ]);
  });

  it("skips keep-alive comments and joins multi-line data", () => {
    const p = new SseParser();
    expect(p.push(":\n\n: ping\n\ndata: a\ndata: b\n\n")).toEqual([{ event: "message", data: "a\nb" }]);
  });

  it("drops an event that has no data", () => {
    expect(new SseParser().push("event: item-added\n\n")).toEqual([]);
  });
});
