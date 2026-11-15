import { describe, expect, it } from "vitest";
import type { FetchPlugin } from "../api/endpoints";
import { method } from "../mock/platforms";
import { platformPages, roleOf, savedKind, scriptPages } from "./platforms";

const plugin = (id: string, methods: FetchPlugin["methods"], platforms = [{ id: "bilibili", name: "B站" }]): FetchPlugin => ({
  id,
  name: id,
  version: "0.1.0",
  platforms,
  access: "public-web",
  login: "none",
  canList: false,
  sources: [],
  sites: [],
  visitors: [],
  settings: [],
  pacing: [],
  pauses: [],
  methods,
});

const guest = method({ id: "guest", name: "访客接口", access: "public-web", account: { kind: "none" }, gets: { fetch: ["text", "meta"] } });
const video = method({ id: "video", name: "高清视频", access: "session-api", account: { kind: "in-usuyo", sites: ["www.bilibili.com"] }, gets: { fetch: ["media:video"] } });
const later = method({ id: "later", name: "以后", access: "public-web", account: { kind: "none" }, gets: {}, built: false, enabled: false });
const favorites = method({ id: "favorites", name: "收藏夹", access: "session-api", account: { kind: "in-usuyo", sites: ["www.bilibili.com"] }, gets: { list: ["稍后再看"] } });
const script = method({ id: "userscript", name: "浏览器脚本", access: "browser-session", account: { kind: "own-browser" }, gets: { delivers: "links" } });

function page(methods: FetchPlugin["methods"], extra: FetchPlugin[] = []) {
  const p = platformPages({ plugins: [plugin("usuyo-fetch/bilibili", methods), ...extra], notLoaded: [], failureLog: "" })[0];
  if (!p) throw new Error("no page");
  return p;
}

describe("where a method goes on a platform page", () => {
  it("puts fetchers under links, part-only fetchers under parts, lists and scripts under saved", () => {
    expect([guest, video, later, favorites, script].map(roleOf)).toEqual(["link", "part", "link", "saved", "saved"]);
    const telegram = method({ id: "export", name: "导出", access: "official-export", account: { kind: "none" }, gets: { fetch: ["text", "meta"], list: [] } });
    // 能列又按 ref 取一条的（读文件的）不拿来收链接
    expect(roleOf(telegram)).toBe("saved");
  });

  it("gives each platform of a multi-platform plugin its own page with that plugin on it", () => {
    const multi = plugin("example/multi", [method({ id: "clip", name: "多平台", access: "public-web", account: { kind: "none" }, gets: { fetch: ["meta", "media:video"] } })], [
      { id: "bilibili", name: "B站" },
      { id: "youtube", name: "YouTube" },
    ]);
    const pages = platformPages({ plugins: [plugin("usuyo-fetch/bilibili", [guest]), multi], notLoaded: [], failureLog: "" });
    expect(pages.map((p) => [p.id, p.plugins.map((x) => x.id), p.link.map((u) => u.method.id)])).toEqual([
      ["bilibili", ["usuyo-fetch/bilibili", "example/multi"], ["guest", "clip"]],
      ["youtube", ["example/multi"], ["clip"]],
    ]);
  });
});

describe("saved methods and their scripts", () => {
  it("tells file readers, scripts, phones, link lists and plain lists apart", () => {
    const p = page([
      favorites,
      script,
      method({ id: "archive", name: "官方归档", access: "official-export", account: { kind: "none" }, gets: { list: [] } }),
      method({ id: "public", name: "公开收藏夹", access: "public-web", account: { kind: "none" }, gets: { list: [], link: "收藏夹的链接" } }),
      method({ id: "iphone", name: "iPhone", access: "vision-ui", account: { kind: "own-phone" }, gets: { list: ["收藏"] } }),
    ]);
    const readers = new Set(["usuyo-fetch/bilibili#archive"]);
    expect(p.saved.map((u) => savedKind(u, readers))).toEqual(["list", "script", "file", "link-list", "phone"]);
  });

  it("reads the page a script runs on from its own header", () => {
    const text = "// ==UserScript==\n// @description  在你自己的「书签」「喜欢」页上，只看页面自己请求回来的。\n// @match        https://x.com/i/bookmarks\n// @match        https://x.com/*/likes\n";
    expect(scriptPages(text)).toEqual({ pages: "「书签」「喜欢」", matches: ["https://x.com/i/bookmarks", "https://x.com/*/likes"] });
    expect(scriptPages("// @match https://a.test/*")).toEqual({ pages: null, matches: ["https://a.test/*"] });
  });
});
