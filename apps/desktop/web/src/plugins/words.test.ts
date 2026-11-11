import { describe, expect, it } from "vitest";
import { loginText, pauseText, timeText } from "./words";

describe("plugin words", () => {
  const now = new Date(2026, 10, 11, 14, 0, 0);

  it("writes a platform pause with its reason and when it tries again", () => {
    const until = new Date(2026, 10, 11, 15, 0, 0).toISOString();
    const p = { scope: "bilibili", byYou: false, until, code: "rate-limited" };
    expect(pauseText(p, [{ id: "bilibili", name: "B站" }], now)).toBe("平台说请求太频繁 · 15:00 自己再试");
    expect(pauseText({ ...p, hint: "B站说请求太频繁" }, [], now)).toBe("B站说请求太频繁 · 15:00 自己再试");
  });

  it("says a hold you asked for lasts until you try again", () => {
    expect(pauseText({ scope: "*", byYou: true }, [], now)).toContain("到你点「现在再试」为止");
  });

  it("names the account a plugin needs in plain words", () => {
    expect(loginText("required", ["B站"])).toBe("要登录你的B站账号");
    expect(loginText("none", [])).toBe("不用账号");
  });

  it("adds seconds only when the time is close, and the date when it is another day", () => {
    expect(timeText(new Date(2026, 10, 11, 14, 0, 15).toISOString(), now)).toBe("14:00:15");
    expect(timeText(new Date(2026, 10, 11, 18, 30).toISOString(), now)).toBe("18:30");
    expect(timeText(new Date(2026, 10, 12, 9, 5).toISOString(), now)).toBe("11 月 12 日 09:05");
  });
});
