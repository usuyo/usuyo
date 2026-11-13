import { describe, expect, it } from "vitest";
import type { FetchPlugin } from "../api/endpoints";
import { method } from "../mock/platforms";
import { autoOrder, linkChoice, linkSwitches, platformPages, roleOf, savedKind, scriptPages, type Use } from "./platforms";

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
const session = method({ id: "session", name: "会话接口", access: "session-api", account: { kind: "in-usuyo", sites: ["www.bilibili.com"] }, gets: { fetch: ["text"] } });
const signed = method({ id: "signed", name: "签名接口", access: "private-api", account: { kind: "none" }, gets: { fetch: ["text"] }, enabled: false });
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
    const ytdlp = plugin("usuyo-fetch/ytdlp", [method({ id: "ytdlp", name: "yt-dlp", access: "public-web", account: { kind: "none" }, gets: { fetch: ["meta", "media:video"] } })], [
      { id: "bilibili", name: "B站" },
      { id: "youtube", name: "YouTube" },
    ]);
    const pages = platformPages({ plugins: [plugin("usuyo-fetch/bilibili", [guest]), ytdlp], notLoaded: [], failureLog: "" });
    expect(pages.map((p) => [p.id, p.plugins.map((x) => x.id), p.link.map((u) => u.method.id)])).toEqual([
      ["bilibili", ["usuyo-fetch/bilibili", "usuyo-fetch/ytdlp"], ["guest", "ytdlp"]],
      ["youtube", ["usuyo-fetch/ytdlp"], ["ytdlp"]],
    ]);
  });
});

describe("the select's meaning", () => {
  it("reads 自动 when every link method sits at its tier default, one method when only it is on, and mixed otherwise", () => {
    expect(linkChoice(page([guest, session, signed, video, later]).link, "bilibili")).toEqual({ kind: "auto" });
    const only = page([{ ...guest, enabled: false }, { ...session, enabled: false }, { ...signed, enabled: true }]);
    expect(linkChoice(only.link, "bilibili")).toEqual({ kind: "only", key: "usuyo-fetch/bilibili#signed" });
    const mixed = page([guest, { ...session, enabled: false }, { ...signed, enabled: true }]);
    expect(linkChoice(mixed.link, "bilibili").kind).toBe("mixed");
  });

  it("switches the chosen method on first, the platform's other link methods off, and leaves parts, lists, scripts and unbuilt ones alone", () => {
    const p = page([guest, session, signed, video, later, favorites, script]);
    expect(linkSwitches(p.link, "bilibili", "usuyo-fetch/bilibili#signed")).toEqual([
      { pluginId: "usuyo-fetch/bilibili", methodId: "signed", enabled: true },
      { pluginId: "usuyo-fetch/bilibili", methodId: "guest", enabled: false },
      { pluginId: "usuyo-fetch/bilibili", methodId: "session", enabled: false },
    ]);
    // 已经是那样的不再拨
    expect(linkSwitches(p.link, "bilibili", "auto")).toEqual([]);
  });

  it("clears the switch of each link method that is off its tier default on 自动, and only those", () => {
    const p = page([{ ...guest, enabled: false }, session, { ...signed, enabled: true }, { ...video, enabled: false }]);
    expect(linkSwitches(p.link, "bilibili", "auto")).toEqual([
      { pluginId: "usuyo-fetch/bilibili", methodId: "guest", enabled: null },
      { pluginId: "usuyo-fetch/bilibili", methodId: "signed", enabled: null },
    ]);
  });

  it("clears a multi-platform method on 自动 even when it already matches here, because elsewhere it does not", () => {
    // 在 B站上照默认开着，在 YouTube 上那一层默认关着、却被拨成了开着：一个 PUT 回不去，去掉开关才各回各的
    const ytdlp = plugin(
      "usuyo-fetch/ytdlp",
      [
        method({
          id: "ytdlp",
          name: "yt-dlp",
          access: "public-web",
          account: { kind: "none" },
          gets: { fetch: ["text", "meta"] },
          perPlatform: [{ platform: "youtube", access: "private-api", login: "none", enabled: true }],
        }),
      ],
      [
        { id: "bilibili", name: "B站" },
        { id: "youtube", name: "YouTube" },
      ],
    );
    const p = page([guest], [ytdlp]);
    expect(linkChoice(p.link, "bilibili")).toEqual({ kind: "auto" });
    expect(linkSwitches(p.link, "bilibili", "auto")).toEqual([{ pluginId: "usuyo-fetch/ytdlp", methodId: "ytdlp", enabled: null }]);
  });

  it("takes the tier on this platform for a method another plugin raised here, across plugins", () => {
    const ytdlp = plugin(
      "usuyo-fetch/ytdlp",
      [
        method({
          id: "ytdlp",
          name: "yt-dlp",
          access: "public-web",
          account: { kind: "none" },
          gets: { fetch: ["text", "meta"] },
          perPlatform: [{ platform: "bilibili", access: "private-api", login: "none", enabled: false }],
        }),
      ],
      [
        { id: "bilibili", name: "B站" },
        { id: "youtube", name: "YouTube" },
      ],
    );
    const p = page([guest], [ytdlp]);
    // 在 B站上那一层是 private-api，默认关着：这就是「自动」
    expect(linkChoice(p.link, "bilibili")).toEqual({ kind: "auto" });
    expect(linkSwitches(p.link, "bilibili", "usuyo-fetch/ytdlp#ytdlp")).toEqual([
      { pluginId: "usuyo-fetch/ytdlp", methodId: "ytdlp", enabled: true },
      { pluginId: "usuyo-fetch/bilibili", methodId: "guest", enabled: false },
    ]);
    expect(autoOrder(p.link, "bilibili").map((u: Use) => u.method.id)).toEqual(["guest", "ytdlp"]);
  });

  it("orders 自动 by tier, then account-free first, platform plugins before the generic web plugin", () => {
    const web = plugin("builtin/web", [method({ id: "page", name: "网页", access: "public-web", account: { kind: "none" }, gets: { fetch: ["text"] } })], [{ id: "bilibili", name: "B站" }, { id: "web", name: "网页" }]);
    const visitor = method({ id: "visitor", name: "访客身份", access: "public-web", account: { kind: "visitor", sites: ["www.bilibili.com"] }, gets: { fetch: ["text"] } });
    const p = page([session, visitor, guest, later], [web]);
    expect(autoOrder(p.link, "bilibili").map((u) => u.method.id)).toEqual(["guest", "visitor", "session", "page"]);
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
