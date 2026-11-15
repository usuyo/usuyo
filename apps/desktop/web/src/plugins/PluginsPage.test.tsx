import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { CatalogView, FetchMethod, FetchPlugin, FileReader, PluginsView, SiteLogin, VisitorIdentity } from "../api/endpoints";
import { setLanguageChoice, t } from "../i18n";
import { method } from "../mock/platforms";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./PluginsPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const SCRIPT = "// ==UserScript==\n// @description  在你自己的「收藏夹」「稍后再看」页上，只看页面自己请求回来的列表。\n// @match        https://space.bilibili.com/*/favlist*\n";

interface Served {
  switched: { path: string; method?: string; body: unknown }[];
  revealed: string[];
  created: unknown[];
}

function serve(view: Omit<PluginsView, "failureLog">, sites: SiteLogin[], visitors: VisitorIdentity[] = [], at = "/plugins", readers: FileReader[] = []): Served {
  const served: Served = { switched: [], revealed: [], created: [] };
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL, init?: RequestInit) => {
      const path = new URL(input).pathname;
      const body: unknown = typeof init?.body === "string" ? JSON.parse(init.body) : null;
      if (path.endsWith("/script/reveal")) {
        served.revealed.push(path);
        return Promise.resolve(new Response(null, { status: 204 }));
      }
      if (path.endsWith("/script")) return Promise.resolve(Response.json({ text: SCRIPT }));
      if (path.includes("/methods/")) {
        served.switched.push(init?.method === "DELETE" ? { path, method: "DELETE", body } : { path, body });
        return Promise.resolve(Response.json(view.plugins[0]));
      }
      if (path.endsWith("/deliver-tokens") && init?.method === "POST") {
        served.created.push(body);
        return Promise.resolve(Response.json({ id: "t1", name: "B站的脚本", kind: "extension", createdAt: "2026-11-12T08:00:00.000Z", token: "t1.secret" }));
      }
      if (path.endsWith("/deliver-tokens")) return Promise.resolve(Response.json([]));
      if (path.endsWith("/plugins")) return Promise.resolve(Response.json({ ...view, failureLog: "/m/logs/fetch-failures.jsonl" }));
      if (path.endsWith("/catalog")) return Promise.resolve(Response.json(catalogOf(view.plugins)));
      if (path.endsWith("/sites")) return Promise.resolve(Response.json(sites));
      if (path.endsWith("/visitors")) return Promise.resolve(Response.json(visitors));
      if (path.endsWith("/file-readers")) return Promise.resolve(Response.json(readers));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(new Response("nope", { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: [at] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return served;
}

/** 装着的获取插件在目录里的样子：开发时装的，不在订阅源里。 */
function catalogOf(plugins: FetchPlugin[]): CatalogView {
  return {
    target: "macos-arm64",
    sources: [],
    plugins: plugins.map((p) => ({
      id: p.id,
      kind: "fetch",
      name: p.name,
      platforms: p.platforms,
      methods: [],
      installed: { version: p.version, origin: "development", dir: `/m/plugins/${p.id}/${p.version}` },
      update: false,
      blocked: { code: "development", hint: "这个插件是用 just dev-plugins 安装的开发版本，不从订阅源更新或删除。" },
    })),
  };
}

const SITE: SiteLogin = { site: "www.bilibili.com", name: "B站", loginUrl: "https://passport.bilibili.com/login", cookies: ["SESSDATA"], domains: [], login: "missing", plugins: ["usuyo-fetch/bilibili"], credential: "cookies" };

const bili = (methods: FetchMethod[], extra: Partial<FetchPlugin> = {}): FetchPlugin => ({
  id: "usuyo-fetch/bilibili",
  name: "B站",
  version: "0.1.0",
  platforms: [{ id: "bilibili", name: "B站" }],
  access: "session-api",
  login: "required",
  canList: true,
  sources: [],
  sites: ["www.bilibili.com"],
  visitors: [],
  settings: [],
  pacing: [{ platform: "bilibili", batch: true, intervalSec: 5, daily: 500 }],
  pauses: [],
  methods,
  ...extra,
});

const guest = method({ id: "guest", name: "访客接口", access: "public-web", account: { kind: "none" }, gets: { fetch: ["text", "meta"] } });
const session = method({
  id: "session",
  name: "会话接口",
  access: "session-api",
  account: { kind: "in-usuyo", sites: ["www.bilibili.com"], where: "B站" },
  gets: { fetch: ["text", "images"] },
  blockers: [{ code: "login-missing", hint: "要先在 usuyo 里登录B站" }],
});
const signed = method({ id: "signed", name: "签名接口", access: "private-api", account: { kind: "none" }, gets: { fetch: ["text"] }, enabled: false });
const video = method({ id: "video", name: "视频文件", access: "session-api", account: { kind: "in-usuyo", sites: ["www.bilibili.com"], where: "B站" }, gets: { fetch: ["media:video"] } });
const unbuilt = method({ id: "app", name: "客户端", access: "client-hook", account: { kind: "own-app" }, gets: {}, built: false, enabled: false });

async function choose(label: string, option: string) {
  fireEvent.click(await screen.findByRole("button", { name: new RegExp(label) }));
  fireEvent.click(await screen.findByRole("option", { name: new RegExp(`^${option}`) }));
}

it("says so when no source lists a plugin and none are installed", async () => {
  serve({ plugins: [], notLoaded: [] }, []);
  expect(await screen.findByText("订阅源中还没有插件，这台电脑上也没有安装插件。")).toBeTruthy();
});

it("lists installed plugins as cards that open the plugin's detail, and the old platform address lands there", async () => {
  serve({ plugins: [bili([guest, session, signed])], notLoaded: [] }, [SITE]);
  const card = await screen.findByRole("link", { name: "B站" });
  expect(card.getAttribute("href")).toBe("/plugins/usuyo-fetch/bilibili");
  expect(within(card).getByText("开发版本 0.1.0")).toBeTruthy();
  cleanup();
  serve({ plugins: [bili([guest, session, signed])], notLoaded: [] }, [SITE], [], "/plugins/bilibili");
  expect(await screen.findByRole("region", { name: "B站" })).toBeTruthy();
  expect(screen.getAllByRole("link", { name: "获取插件" }).every((a) => a.getAttribute("href") === "/plugins")).toBe(true);
});

it("writes the platform page in English when English is chosen, with plurals and the daemon's numbers", async () => {
  setLanguageChoice("en");
  const visitor = method({ id: "visitor", name: "visitor-api", access: "public-web", account: { kind: "visitor", sites: ["www.bilibili.com"], where: "Bilibili" }, gets: { fetch: ["text"] } });
  serve(
    { plugins: [bili([guest, session, visitor], { upstream: [{ name: "parse-video", license: "MIT" }] })], notLoaded: [] },
    [SITE],
    [{ site: "www.bilibili.com", name: "Bilibili", url: "https://www.bilibili.com/", cookies: [], domains: [], state: "missing", plugins: ["usuyo-fetch/bilibili"] }],
    "/plugins/bilibili",
  );
  expect(await screen.findByText(/^Fetched by the B站 plugin\./)).toBeTruthy();
  expect(screen.getAllByRole("link", { name: "Fetch Plugins" }).length).toBeGreaterThan(0);
  expect(screen.getByText("Development 0.1.0")).toBeTruthy();
  expect(screen.getByRole("region", { name: "Add Link" })).toBeTruthy();
  const pacing = screen.getByRole("region", { name: "Request Rate" });
  expect(within(pacing).getByText("At least 5 seconds between requests; daily import limit: 500.")).toBeTruthy();
  const about = screen.getByRole("region", { name: "About" });
  expect(within(about).getByText("Uses: parse-video (MIT)")).toBeTruthy();
  const accounts = screen.getByRole("region", { name: "Accounts & Visitor Identities" });
  expect(within(accounts).getByRole("button", { name: "Get Visitor Identity" })).toBeTruthy();
});

it("explains 自动 in the daemon's order, with the unbuilt one disabled and parts as 补上 lines", async () => {
  serve({ plugins: [bili([guest, session, signed, video, unbuilt])], notLoaded: [] }, [SITE], [], "/plugins/bilibili");
  const card = await screen.findByRole("region", { name: "添加链接" });
  expect(within(card).getByText("可用")).toBeTruthy();
  expect(within(card).getAllByText("在 usuyo 中登录B站账号：未登录")[0]).toBeTruthy();
  expect(within(card).getByText(/已关闭：签名接口（此层级默认关闭）/)).toBeTruthy();
  expect(within(card).getByText("补充：视频")).toBeTruthy();
  expect(within(card).getByRole("switch", { name: "补充：视频文件" })).toBeTruthy();
  fireEvent.click(within(card).getByRole("button", { name: /获取方式/ }));
  const option = await screen.findByRole("option", { name: /^客户端/ });
  expect(option.getAttribute("aria-disabled")).toBe("true");
});

it("choosing one method switches it on and the platform's other link methods off; 自动 puts each back to its default", async () => {
  const served = serve({ plugins: [bili([guest, session, signed, video])], notLoaded: [] }, [SITE], [], "/plugins/bilibili");
  await choose("获取方式", "签名接口");
  await waitFor(() =>
    expect(served.switched).toEqual([
      { path: "/api/plugins/usuyo-fetch/bilibili/methods/signed", body: { enabled: true } },
      { path: "/api/plugins/usuyo-fetch/bilibili/methods/guest", body: { enabled: false } },
      { path: "/api/plugins/usuyo-fetch/bilibili/methods/session", body: { enabled: false } },
    ]),
  );
  cleanup();

  const back = serve({ plugins: [bili([{ ...guest, enabled: false }, { ...session, enabled: false }, { ...signed, enabled: true }, video])], notLoaded: [] }, [SITE], [], "/plugins/bilibili");
  // 现在只有签名接口开着：选择框里是它，说明是它的
  expect(await screen.findByText("模拟平台客户端调用未公开的接口。平台通常不允许这种方式，因此默认关闭。")).toBeTruthy();
  await choose("获取方式", "自动");
  // 「自动」去掉拨过的开关（DELETE，不带身子），回到各自那一层的默认
  await waitFor(() =>
    expect(back.switched).toEqual([
      { path: "/api/plugins/usuyo-fetch/bilibili/methods/guest", method: "DELETE", body: null },
      { path: "/api/plugins/usuyo-fetch/bilibili/methods/session", method: "DELETE", body: null },
      { path: "/api/plugins/usuyo-fetch/bilibili/methods/signed", method: "DELETE", body: null },
    ]),
  );
});

it("a specific method's explanation has its needs with one action each, risk and last call", async () => {
  serve(
    {
      plugins: [
        bili([
          { ...guest, enabled: false },
          {
            ...session,
            lastCall: { at: "2026-11-12T08:00:00.000Z", ok: false, code: "login-required", hint: "登录过期了" },
            riskNote: { text: "只读，不改。", source: { label: "某个 issue", url: "https://example.com/1" } },
            blockers: [
              { code: "login-missing", hint: "要先在 usuyo 里登录B站" },
              { code: "check-failed", hint: "没找到 ffmpeg：brew install ffmpeg" },
            ],
          },
        ]),
      ],
      notLoaded: [],
    },
    [SITE],
    [],
    "/plugins/bilibili",
  );
  const needs = await screen.findByRole("list", { name: "会话接口需要的项目" });
  expect(within(needs).getByText("在 usuyo 中登录B站账号：未登录")).toBeTruthy();
  expect(within(needs).getByRole("button", { name: "登录" })).toBeTruthy();
  expect(within(needs).getByText("没找到 ffmpeg：brew install ffmpeg")).toBeTruthy();
  expect(within(needs).getByRole("button", { name: "重新检查" })).toBeTruthy();
  expect(screen.getByText(/只读，不改。/)).toBeTruthy();
  expect(screen.getByText(/上次使用：.*，失败：登录过期了/)).toBeTruthy();
});

it("steps for a list that needs a login: log in, then 去导入 with the method chosen", async () => {
  const favorites = method({
    id: "favorites",
    name: "我的收藏夹",
    access: "session-api",
    account: { kind: "in-usuyo", sites: ["www.bilibili.com"], where: "B站" },
    gets: { list: ["稍后再看"] },
    blockers: [{ code: "login-missing", hint: "要先在 usuyo 里登录B站" }],
  });
  serve({ plugins: [bili([guest, favorites])], notLoaded: [] }, [SITE], [], "/plugins/bilibili");
  const steps = await screen.findByRole("list", { name: "我的收藏夹的步骤" });
  expect(within(steps).getByRole("button", { name: "登录" })).toBeTruthy();
  expect(within(steps).getByRole("link", { name: "前往导入" }).getAttribute("href")).toBe("/import?pick=usuyo-fetch%2Fbilibili%23favorites");
});

it("steps for a list that starts from a link: the link field right here", async () => {
  const pub = method({ id: "public", name: "公开收藏夹", access: "public-web", account: { kind: "none" }, gets: { list: [], link: "收藏夹或个人空间的链接" } });
  serve({ plugins: [bili([guest, pub])], notLoaded: [] }, [SITE], [], "/plugins/bilibili");
  const steps = await screen.findByRole("list", { name: "公开收藏夹的步骤" });
  expect(within(steps).getByText("收藏夹或个人空间的链接")).toBeTruthy();
  expect(within(steps).getByRole("button", { name: t("imports.picker.find") })).toBeTruthy();
});

it("steps for a browser script: where the file is, reveal and copy, an inline token, the page from the script, 交给 usuyo", async () => {
  const script = method({
    id: "userscript",
    name: "浏览器脚本",
    access: "browser-session",
    account: { kind: "own-browser" },
    gets: { delivers: "links" },
    script: { path: "/m/plugins/usuyo-fetch/bilibili/0.1.0/userscript/a.user.js", present: true },
  });
  const served = serve({ plugins: [bili([guest, script])], notLoaded: [] }, [SITE], [], "/plugins/bilibili");
  const steps = await screen.findByRole("list", { name: "浏览器脚本的步骤" });
  expect(within(steps).getByText("/m/plugins/usuyo-fetch/bilibili/0.1.0/userscript/a.user.js")).toBeTruthy();
  expect(await within(steps).findByText(/打开你的「收藏夹」「稍后再看」页面/)).toBeTruthy();
  expect(within(steps).getByText("https://space.bilibili.com/*/favlist*")).toBeTruthy();
  fireEvent.click(within(steps).getByRole("button", { name: t("plugins.script.reveal") }));
  await waitFor(() => expect(served.revealed).toEqual(["/api/plugins/usuyo-fetch/bilibili/methods/userscript/script/reveal"]));
  fireEvent.click(within(steps).getByRole("button", { name: "生成令牌" }));
  expect(await within(steps).findByText("t1.secret")).toBeTruthy();
  expect(served.created).toEqual([{ name: "B站脚本", kind: "extension" }]);
  expect(within(steps).getByText(/在脚本面板中点按「交给 usuyo」/)).toBeTruthy();
  expect(within(steps).getByText(/「复制诊断」/)).toBeTruthy();
});

it("steps for a window method: what it does, the plugin script named before first use, sign in here, confirm on the Import page", async () => {
  const site: SiteLogin = { site: "www.example.test", name: "例子", loginUrl: "https://www.example.test/login", cookies: [], domains: [], login: "missing", plugins: ["usuyo-fetch/bilibili"], credential: "browser-profile" };
  const reading = method({
    id: "in-usuyo",
    name: "在 usuyo 中登录并读取",
    access: "browser-session",
    login: "required",
    account: { kind: "in-usuyo", sites: ["www.example.test"], where: "例子" },
    gets: { delivers: "links" },
    window: true,
    guide: "登录以后打开你的个人页，先读「收藏」",
    script: { path: "/m/plugins/usuyo-fetch/bilibili/0.1.0/userscript/window.js", present: true },
    riskNote: { text: "自动往下翻是用你的账号做的自动操作", source: { label: "某处 #1", url: "https://example.test/1" } },
  });
  serve({ plugins: [bili([guest, reading], { sites: ["www.example.test"] })], notLoaded: [] }, [site], [], "/plugins/bilibili");
  const steps = await screen.findByRole("list", { name: "在 usuyo 中登录并读取的步骤" });
  expect(within(steps).getByText("登录以后打开你的个人页，先读「收藏」")).toBeTruthy();
  expect(within(steps).getByText(/只用于例子的窗口/)).toBeTruthy();
  expect(within(steps).getByText(/每隔几秒自动向下滚动一屏/)).toBeTruthy();
  expect(within(steps).getByText(/不会拷贝到其他地方/)).toBeTruthy();
  expect(within(steps).getByText(/登录后的页面中会运行B站插件附带的脚本/)).toBeTruthy();
  expect(within(steps).getByText("/m/plugins/usuyo-fetch/bilibili/0.1.0/userscript/window.js")).toBeTruthy();
  // 浏览器标签页里开不了站点窗口：说一句，不装作开了
  fireEvent.click(within(steps).getByRole("button", { name: "登录例子" }));
  expect(await within(steps).findByText("此窗口只能在 usuyo 应用中打开，浏览器标签页中不可用。")).toBeTruthy();
  expect(within(steps).getByRole("link", { name: "导入" }).getAttribute("href")).toBe("/import");
  expect(screen.getByText(/已记录：自动往下翻是用你的账号做的自动操作/)).toBeTruthy();
  // 账号与访客身份里是这个站点窗口的那一行
  expect(screen.getByText("例子窗口登录")).toBeTruthy();
});

it("steps for a file reader: how to get the file, then 选文件 with this reader; one off by default says why and has its switch", async () => {
  const dump = method({ id: "dump", name: "接口结果文件", access: "private-api", account: { kind: "none" }, gets: { list: [] }, enabled: false });
  const reader: FileReader = { id: "usuyo-fetch/bilibili#dump", name: dump.name, plugin: "B站", reads: "接口返回的 json", patterns: ["*.json"], lists: [] };
  const served = serve({ plugins: [bili([guest, dump])], notLoaded: [] }, [SITE], [], "/plugins/bilibili", [reader]);
  const steps = await screen.findByRole("list", { name: "接口结果文件的步骤" });
  expect(await within(steps).findByText(/接口返回的 json/)).toBeTruthy();
  expect(within(steps).getByRole("link", { name: "选择文件" }).getAttribute("href")).toBe("/import?from=file&reader=usuyo-fetch%2Fbilibili%23dump");
  expect(screen.getByText(/此获取方式默认关闭：/)).toBeTruthy();
  fireEvent.click(screen.getByRole("switch", { name: "接口结果文件：已关闭" }));
  await waitFor(() => expect(served.switched).toEqual([{ path: "/api/plugins/usuyo-fetch/bilibili/methods/dump", body: { enabled: true } }]));
});

it("steps for a phone method point to the phone page", async () => {
  const phone = method({ id: "iphone", name: "iPhone", access: "vision-ui", account: { kind: "own-phone", where: "小红书" }, gets: { list: ["收藏"] } });
  serve({ plugins: [bili([phone], { id: "usuyo-fetch/rednote", name: "小红书", platforms: [{ id: "rednote", name: "小红书" }], sites: [] })], notLoaded: [] }, [], [], "/plugins/rednote");
  const steps = await screen.findByRole("list", { name: "iPhone 的步骤" });
  expect(within(steps).getByRole("link", { name: "前往「手机连接」页面" }).getAttribute("href")).toBe("/phone");
});

it("lists logins and visitor identities with the methods that use them, the pacing, and what the plugin calls", async () => {
  const visitor = method({ id: "visitor", name: "访客身份接口", access: "public-web", account: { kind: "visitor", sites: ["www.bilibili.com"], where: "B站" }, gets: { fetch: ["text"] } });
  serve(
    {
      plugins: [bili([guest, session, visitor], { upstream: [{ name: "parse-video", license: "MIT" }], pauses: [{ method: "guest", scope: "bilibili", byYou: false, until: "2099-01-01T15:00:00.000Z", code: "rate-limited" }] })],
      notLoaded: [],
    },
    [SITE],
    [{ site: "www.bilibili.com", name: "B站", url: "https://www.bilibili.com/", cookies: [], domains: [], state: "present", plugins: ["usuyo-fetch/bilibili"] }],
    "/plugins/bilibili",
  );
  const accounts = await screen.findByRole("region", { name: "账号与访客身份" });
  expect(within(accounts).getByText(/使用它的获取方式：会话接口/)).toBeTruthy();
  expect(within(accounts).getByText(/使用它的获取方式：访客身份接口/)).toBeTruthy();
  const pacing = screen.getByRole("region", { name: "请求频率" });
  expect(within(pacing).getByText("每两次请求至少间隔 5 秒，每天最多导入 500 次。")).toBeTruthy();
  expect(within(pacing).getByText(/平台提示请求过于频繁/)).toBeTruthy();
  expect(within(pacing).getByRole("button", { name: "立即重试" })).toBeTruthy();
  const about = screen.getByRole("region", { name: "关于" });
  expect(within(about).getByText(/使用的程序：parse-video（MIT）/)).toBeTruthy();
  expect(within(about).getByText("/m/logs/fetch-failures.jsonl")).toBeTruthy();
});

it("names the other platforms a multi-platform method's one switch moves, and offers no lone option that equals 自动", async () => {
  const multi: FetchPlugin = {
    ...bili([method({ id: "clip", name: "多平台", access: "public-web", account: { kind: "none" }, gets: { fetch: ["text", "meta"] } })]),
    id: "example/multi",
    name: "多平台",
    platforms: [
      { id: "bilibili", name: "B站" },
      { id: "youtube", name: "YouTube" },
    ],
    sites: [],
  };
  serve({ plugins: [bili([guest]), multi], notLoaded: [] }, [SITE], [], "/plugins/bilibili");
  expect(await screen.findByText(/的开关同时作用于 YouTube/)).toBeTruthy();
  cleanup();

  serve({ plugins: [bili([guest, unbuilt])], notLoaded: [] }, [SITE], [], "/plugins/bilibili");
  fireEvent.click(await screen.findByRole("button", { name: /获取方式/ }));
  expect(await screen.findByRole("option", { name: /^自动/ })).toBeTruthy();
  expect(screen.getAllByText("目前只提供访客接口").length).toBeGreaterThan(0);
  expect(screen.queryByRole("option", { name: /^访客接口/ })).toBeNull();
});
