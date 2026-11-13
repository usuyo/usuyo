import { describe, expect, it } from "vitest";
import { loginText, pauseText, timeText, visitorText } from "./words";

describe("plugin words", () => {
  const now = new Date(2026, 10, 11, 14, 0, 0);

  it("writes a platform pause with its reason and when it tries again", () => {
    const until = new Date(2026, 10, 11, 15, 0, 0).toISOString();
    const p = { method: "favorites", scope: "bilibili", byYou: false, until, code: "rate-limited" };
    expect(pauseText(p, [{ id: "bilibili", name: "B站" }], now)).toBe("平台提示请求过于频繁 · 15:00 自动重试");
    expect(pauseText({ ...p, hint: "B站说请求太频繁" }, [], now)).toBe("B站说请求太频繁 · 15:00 自动重试");
  });

  it("says a hold you asked for lasts until you try again", () => {
    expect(pauseText({ method: "*", scope: "*", byYou: true }, [], now)).toContain("点按「立即重试」前不会使用");
  });

  it("names the account a plugin needs in plain words", () => {
    expect(loginText("required", ["B站"])).toBe("要登录你的B站账号");
    expect(loginText("none", [])).toBe("不用账号");
  });

  it("tells a missing visitor identity from one that no longer works, and stays neutral without the state", () => {
    expect(visitorText("missing")).toBe("需要访客身份：在「获取插件」页面中获取访客身份");
    expect(visitorText("present")).toBe("访客身份已失效：更换后重试");
    expect(visitorText("expired")).toBe("访客身份已失效：更换后重试");
    expect(visitorText()).toBe("需要访客身份");
    expect(pauseText({ method: "page", scope: "x", byYou: false, code: "visitor-required" }, [], now)).toBe("需要访客身份 · 需要处理");
  });

  it("adds seconds only when the time is close, and the date when it is another day", () => {
    expect(timeText(new Date(2026, 10, 11, 14, 0, 15).toISOString(), now)).toBe("14:00:15");
    expect(timeText(new Date(2026, 10, 11, 18, 30).toISOString(), now)).toBe("18:30");
    expect(timeText(new Date(2026, 10, 12, 9, 5).toISOString(), now)).toBe("11 月 12 日 09:05");
  });
});
