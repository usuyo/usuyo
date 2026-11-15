import { describe, expect, it } from "vitest";
import type { FetchPlugin, StrategyView } from "../api/endpoints";
import { method } from "../mock/platforms";
import { strategyView } from "../mock/strategy";
import { fetchSummary, stepName, withPick, withPlatformPreset, withPreset } from "./strategy";

const plugin: FetchPlugin = {
  id: "usuyo-fetch/bilibili",
  name: "B站",
  version: "0.1.0",
  platforms: [{ id: "bilibili", name: "B站" }],
  access: "session-api",
  login: "required",
  canList: true,
  sources: [{ source: "bilibili:toview", title: "稍后再看", action: "favorite", method: "favorites" }],
  sites: [],
  visitors: [],
  settings: [],
  pacing: [],
  pauses: [],
  methods: [
    method({ id: "view", name: "公开接口", access: "public-web", account: { kind: "none" }, gets: { fetch: ["meta", "images", "media:video"] } }),
    method({ id: "session", name: "会话接口", access: "session-api", account: { kind: "in-usuyo", sites: ["www.bilibili.com"] }, gets: { fetch: ["media:video"] } }),
    method({ id: "signed", name: "签名接口", access: "private-api", account: { kind: "none" }, gets: { fetch: ["text"] }, enabled: false }),
    method({ id: "favorites", name: "收藏夹", access: "session-api", account: { kind: "in-usuyo", sites: ["www.bilibili.com"] }, gets: { list: ["稍后再看"] } }),
  ],
};

const ways = (v: StrategyView, f: string) => v.routes[0]?.functions.find((x) => x.function === f)?.ways.map((w) => w.method);

describe("the sample strategy follows the daemon's rules", () => {
  it("orders each preset as the daemon does and leaves the private API tier out", () => {
    const first = strategyView({ preset: "no-account-first" }, [plugin]);
    expect(ways(first, "video")).toEqual(["view", "session"]);
    expect(ways(first, "text")).toEqual(["view"]);
    expect(ways(first, "list:favorite")).toEqual(["favorites"]);
    const only = strategyView({ preset: "no-account-only" }, [plugin]);
    expect(ways(only, "video")).toEqual(["view"]);
    expect(ways(only, "list:favorite")).toEqual([]);
    expect(ways(strategyView({ preset: "account-first" }, [plugin]), "video")).toEqual(["session", "view"]);
  });

  it("a pick on a function beats the platform's preset, which beats the strategy", () => {
    const v = strategyView({ preset: "no-account-only", platforms: { bilibili: { preset: "account-first", functions: { text: "usuyo-fetch/bilibili#signed", images: "skip" } } } }, [plugin]);
    const fn = (f: string) => v.routes[0]?.functions.find((x) => x.function === f);
    expect([fn("text")?.from, ways(v, "text")]).toEqual(["function", ["signed"]]);
    expect([fn("images")?.from, fn("images")?.skip]).toEqual(["function", true]);
    expect([fn("video")?.from, ways(v, "video")]).toEqual(["platform", ["session", "view"]]);
  });
});

describe("what a change sends back", () => {
  const view = strategyView({ preset: "no-account-first" }, [plugin]);

  it("sets and clears a pick and a platform preset, dropping what becomes empty", () => {
    const picked = withPick(view, "bilibili", "video", "skip");
    expect(picked).toEqual({ preset: "no-account-first", platforms: { bilibili: { functions: { video: "skip" } } } });
    const both = withPlatformPreset(picked, "bilibili", "account-first");
    expect(both.platforms?.bilibili).toEqual({ functions: { video: "skip" }, preset: "account-first" });
    expect(withPick(withPlatformPreset(both, "bilibili", null), "bilibili", "video", null)).toEqual({ preset: "no-account-first" });
  });

  it("starts a custom strategy from the rules of the preset in use, and keeps custom rules across presets", () => {
    const custom = withPreset(view, "custom");
    expect(custom).toEqual({ preset: "custom", rules: [{ use: [{}] }] });
    const back = withPreset({ ...view, ...custom, rules: [{ function: "video", use: [] }, { use: [{}] }] }, "no-account-only");
    expect(back.rules).toHaveLength(2);
  });

  it("names steps and sums up what each function is fetched with", () => {
    expect(stepName({ accounts: ["none", "visitor"] }, [plugin])).toBe("不使用账号的方式");
    expect(stepName({ method: "usuyo-fetch/bilibili#session" }, [plugin])).toBe("会话接口（使用账号）");
    expect(stepName({ accounts: ["in-usuyo"], access: ["private-api"] }, [plugin])).toBe("账号：在 usuyo 中登录；层级：调用平台未公开的接口");
    expect(fetchSummary(view, "bilibili", [plugin])).toEqual(["正文、图片和视频：公开接口（不使用账号）"]);
  });
});
