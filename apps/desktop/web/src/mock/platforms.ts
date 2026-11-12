// 样例：获取插件、登录站点、访客身份、导入、投递令牌。和内核同一套接口（见 api/endpoints.ts），数据在内存里。
// 导入照时间往前走：新开的一次过几秒列完，确认以后隔一会儿收下一条，页面不用真的内核也能看到「来了」。
import type {
  DeliverToken,
  FetchMethod,
  FetchPlugin,
  FileReader,
  ImportEntry,
  ImportView,
  SiteLogin,
  VisitorIdentity,
} from "../api/endpoints";
import { ApiError } from "../api/client";
import type { Platforms } from "../api/source";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const inMinutes = (m: number) => new Date(Date.now() + m * 60_000).toISOString();

type MethodSpec = Pick<FetchMethod, "id" | "name" | "access" | "account" | "gets"> & Partial<FetchMethod>;

/** 停着的写成一条原因，同内核：限流、被拦写到几点，「先别试」写到你点「现在再试」为止。 */
function pausedHint(p: FetchMethod["pauses"][number]): string {
  if (p.byYou) return "你让它先别试，到你点「现在再试」为止";
  const d = p.until ? new Date(p.until) : null;
  const at = d ? `${d.getMonth() + 1} 月 ${d.getDate()} 日 ${d.getHours()}:${String(d.getMinutes()).padStart(2, "0")}` : null;
  const why = p.code === "rate-limited" ? "平台限流了" : p.code === "blocked" ? "平台拦住了" : "停着";
  return at ? `${why}，${at}以前不用它` : `${why}，等你点「现在再试」`;
}

/** `off`、`not-built`、`paused` 由开关、做没做好、暂停算出来；别的原因（登录、访客身份、本机检查）照样例写的留着。 */
function settle(m: FetchMethod): FetchMethod {
  const kept = m.blockers.filter((b) => b.code !== "off" && b.code !== "not-built" && b.code !== "paused");
  const paused = m.pauses[0];
  const blockers = !m.built
    ? [{ code: "not-built", hint: "这种方法还没做" }]
    : [...(m.enabled ? [] : [{ code: "off", hint: "你关掉了这种方法" }]), ...kept, ...(paused ? [{ code: "paused", hint: pausedHint(paused) }] : [])];
  return { ...m, blockers, usable: blockers.length === 0 };
}

/** 一种取法的样例：没写的照常用默认值（做好了、开着、没停、不用登录、在后台）。 */
export function method(m: MethodSpec): FetchMethod {
  return settle({ login: "none", interaction: "background", built: true, enabled: true, usable: true, blockers: [], pauses: [], ...m });
}

// 没做好的取法，清单里写了能力的，内核照样给 `gets`：做好以后会取得什么。
const notBuilt = (m: MethodSpec) => method({ ...m, built: false, enabled: false });

const minutesAgo = (m: number) => new Date(Date.now() - m * 60_000).toISOString();

/**
 * 样例的四个平台插件：一个平台一个插件，插件里几种取法。几种样子都有：用得上的、你关掉的、
 * 要先登录的、要先取访客身份的、本机检查没过的、还没做好的、上一次调用没成的、被限流停着的。
 */
function basePlugins(): FetchPlugin[] {
  const base = {
    version: "0.1.0",
    settings: [],
    pauses: [],
    sources: [],
    sites: [],
    visitors: [],
  };
  return [
    {
      ...base,
      id: "usuyo-fetch/x",
      name: "X",
      description: "按链接取一条帖子的文字、图和视频；书签和点过赞的由你浏览器里的脚本交来，账号只留在你的浏览器里",
      platforms: [{ id: "x", name: "X" }],
      access: "official-export",
      login: "none",
      canList: true,
      pacing: [{ platform: "x", batch: true, intervalSec: 10, daily: 200 }],
      methods: [
        // 照 usuyo-fetch/x 的 README：四种做好了，五种写了做法、还在等。
        method({
          id: "guest",
          name: "访客接口",
          access: "public-web",
          account: { kind: "none" },
          gets: { fetch: ["text", "images", "media:video", "meta"] },
          lastCall: { at: minutesAgo(40), ok: true },
          riskNote: { text: "没有账号参与。访客令牌会被拒或被限流，此时改走嵌入接口。", source: { label: "gallery-dl #9505", url: "https://github.com/mikf/gallery-dl/issues/9505" } },
        }),
        // 你关掉的一种。
        method({ id: "embed", name: "嵌入接口", access: "public-web", account: { kind: "none" }, gets: { fetch: ["text", "images"] }, enabled: false }),
        method({
          id: "userscript",
          name: "浏览器脚本",
          access: "browser-session",
          login: "required",
          account: { kind: "own-browser" },
          gets: { delivers: "links" },
        }),
        method({ id: "archive", name: "官方归档", access: "official-export", account: { kind: "none" }, gets: { list: [] } }),
        notBuilt({ id: "deliver", name: "内容投递", access: "browser-session", login: "required", account: { kind: "own-browser" }, gets: { fetch: ["text", "images", "media:video"] } }),
        notBuilt({ id: "session", name: "会话接口", access: "session-api", login: "required", account: { kind: "in-usuyo", sites: ["x.com"], where: "X" }, gets: { list: ["书签", "喜欢"] } }),
        notBuilt({
          id: "signed",
          name: "签名接口",
          access: "private-api",
          login: "required",
          account: { kind: "in-usuyo", sites: ["x.com"], where: "X" },
          gets: { list: ["搜索", "用户时间线"] },
          riskNote: { text: "注册三年的账号一次取 800 个关注后被封。", source: { label: "twikit #338", url: "https://github.com/d60/twikit/issues/338" } },
        }),
        notBuilt({ id: "api", name: "官方 API", access: "official-export", login: "required", account: { kind: "developer", where: "X" }, gets: { list: ["书签", "喜欢"] } }),
        notBuilt({ id: "mirror", name: "手机镜像", access: "vision-ui", login: "required", account: { kind: "own-phone", where: "X" }, drives: "iphone", interaction: "foreground", gets: { list: ["书签", "喜欢"], delivers: "links" } }),
      ],
    },
    {
      ...base,
      id: "usuyo-fetch/xiaohongshu",
      name: "小红书",
      description: "按你存过的笔记链接取公开笔记页的正文、图片和视频，做个人备份；存过的由你浏览器里的脚本把链接交进来",
      platforms: [{ id: "xiaohongshu", name: "小红书" }],
      access: "public-web",
      login: "none",
      canList: true,
      visitors: ["www.xiaohongshu.com"],
      pacing: [{ platform: "xiaohongshu", batch: true, intervalSec: 15, daily: 100 }],
      methods: [
        // 要访客身份、这台电脑上还没有的一种。
        method({
          id: "note-page",
          name: "笔记页",
          access: "public-web",
          account: { kind: "visitor", sites: ["www.xiaohongshu.com"], where: "小红书" },
          gets: { fetch: ["text", "images", "media:video"] },
          blockers: [{ code: "visitor-missing", hint: "要先开一次小红书首页，取一份访客身份" }],
          riskNote: { text: "不填 cookie 批量取约 108 条后，复制链接的账号被下线。", source: { label: "XHS-Downloader #305", url: "https://github.com/JoeanAmier/XHS-Downloader/issues/305" } },
        }),
        method({
          id: "userscript",
          name: "浏览器脚本",
          access: "browser-session",
          login: "required",
          account: { kind: "own-browser" },
          gets: { delivers: "links" },
          riskNote: { text: "网页对采集类扩展弹「有害插件」；只截获、不发请求的脚本没有处罚报告。", source: { label: "XHS-Downloader #283", url: "https://github.com/JoeanAmier/XHS-Downloader/issues/283" } },
        }),
        notBuilt({
          id: "iphone",
          name: "iPhone",
          access: "vision-ui",
          login: "required",
          account: { kind: "own-phone", where: "小红书" },
          drives: "iphone",
          interaction: "foreground",
          gets: { list: ["收藏", "赞过"], delivers: "links" },
        }),
      ],
    },
    {
      ...base,
      id: "usuyo-fetch/wechat-channels",
      name: "微信视频号",
      description: "用分享链接取一条：不登录只存作者、描述和封面；登录了腾讯元宝才取视频文件",
      platforms: [{ id: "wechat-channels", name: "微信视频号" }],
      access: "session-api",
      login: "required",
      canList: false,
      sites: ["yuanbao.tencent.com"],
      pacing: [{ platform: "wechat-channels", batch: true, intervalSec: 30, daily: 50 }],
      methods: [
        // 插件的本机检查没过的一种。
        method({
          id: "feed-info",
          name: "分享信息",
          access: "public-web",
          account: { kind: "none" },
          gets: { fetch: ["meta", "images"] },
          blockers: [{ code: "check-failed", hint: "这台电脑上没找到 Node 22，插件跑不起来" }],
        }),
        // 要在 usuyo 里登录、还没登录的一种。
        method({
          id: "yuanbao",
          name: "腾讯元宝",
          access: "session-api",
          login: "required",
          account: { kind: "in-usuyo", sites: ["yuanbao.tencent.com"], where: "腾讯元宝" },
          gets: { fetch: ["media:video"] },
          blockers: [{ code: "login-missing", hint: "要先在 usuyo 里登录腾讯元宝" }],
          riskNote: { text: "公开部署的解析服务被大量调用后，元宝账号被封。", source: { label: "wx_channels_download #495", url: "https://github.com/ltaoo/wx_channels_download/issues/495" } },
        }),
        notBuilt({ id: "pc-client", name: "电脑微信", access: "client-hook", login: "required", account: { kind: "own-app", where: "微信视频号" }, drives: "desktop-app", gets: { fetch: ["media:video"] } }),
        notBuilt({
          id: "iphone",
          name: "iPhone",
          access: "vision-ui",
          login: "required",
          account: { kind: "own-phone", where: "微信视频号" },
          drives: "iphone",
          interaction: "foreground",
          gets: { list: ["赞和收藏"], delivers: "links" },
        }),
      ],
    },
    {
      ...base,
      id: "usuyo-fetch/bilibili",
      name: "B站",
      description: "按视频链接取元数据、封面、字幕和低画质音视频；公开收藏夹从链接列，自己的收藏夹和稍后再看要登录",
      platforms: [{ id: "bilibili", name: "B站" }],
      access: "session-api",
      login: "required",
      canList: true,
      sources: [{ source: "bilibili:toview", title: "稍后再看", action: "favorite", method: "favorites" }],
      sites: ["www.bilibili.com"],
      pacing: [{ platform: "bilibili", batch: true, intervalSec: 5, daily: 500 }],
      methods: [
        method({
          id: "view",
          name: "公开接口",
          access: "public-web",
          account: { kind: "none" },
          gets: { fetch: ["meta", "images", "subtitle", "media:video", "media:audio"] },
          lastCall: { at: minutesAgo(8), ok: true },
          riskNote: { text: "2026-06 起缺指纹参数的部分接口回 412。", source: { label: "BiliNote #397", url: "https://github.com/JefferyHcool/BiliNote/issues/397" } },
        }),
        // 上一次调用没成的一种。
        method({
          id: "favorites-public",
          name: "公开收藏夹",
          access: "public-web",
          account: { kind: "none" },
          gets: { list: [], link: "收藏夹或个人空间的链接" },
          lastCall: { at: minutesAgo(25), ok: false, code: "not-found", hint: "这个收藏夹不公开，或者已经删掉了" },
        }),
        method({
          id: "favorites",
          name: "我的收藏夹和稍后再看",
          access: "session-api",
          login: "required",
          account: { kind: "in-usuyo", sites: ["www.bilibili.com"], where: "B站" },
          gets: { list: ["稍后再看"] },
          // 样例里这一种刚被限流：页面上写原因和什么时候再试
          pauses: [{ method: "favorites", scope: "bilibili", byYou: false, until: inMinutes(12), code: "rate-limited", hint: "B站说请求太频繁，过一会儿再试" }],
          riskNote: { text: "一例大会员被冻结，因果不明。", source: { label: "yutto #452", url: "https://github.com/yutto-dev/yutto/issues/452" } },
        }),
        notBuilt({ id: "yutto", name: "yutto（高清音视频、字幕）", access: "session-api", login: "required", account: { kind: "in-usuyo", sites: ["www.bilibili.com"], where: "B站" }, gets: { fetch: ["media:video", "subtitle"] } }),
      ],
    },
    // 一个插件管几个平台、在其中几个上层级更高的一种（照 usuyo-fetch/ytdlp）：开关只有一个，那几个平台没拨过时关着。
    {
      ...base,
      id: "usuyo-fetch/ytdlp",
      name: "yt-dlp",
      description: "用 yt-dlp 按链接取一条视频：标题、作者、封面、字幕和视频文件",
      platforms: [
        { id: "bilibili", name: "B站" },
        { id: "youtube", name: "YouTube" },
        { id: "weibo", name: "微博" },
      ],
      access: "private-api",
      login: "none",
      canList: false,
      pacing: [{ platform: "youtube", batch: true, intervalSec: 10, daily: 200 }],
      methods: [
        method({
          id: "ytdlp",
          name: "yt-dlp",
          access: "public-web",
          account: { kind: "none" },
          gets: { fetch: ["meta", "images", "subtitle", "media:video"] },
          perPlatform: [
            { platform: "youtube", access: "private-api", login: "none", enabled: false },
            { platform: "weibo", access: "private-api", login: "none", enabled: false },
          ],
        }),
      ],
    },
  ];
}

function baseVisitors(): VisitorIdentity[] {
  return [
    {
      site: "www.xiaohongshu.com",
      name: "小红书",
      url: "https://www.xiaohongshu.com/",
      cookies: [],
      domains: [],
      state: "missing",
      plugins: ["usuyo-fetch/xiaohongshu"],
    },
  ];
}

function baseSites(): SiteLogin[] {
  return [
    {
      site: "www.bilibili.com",
      name: "B站",
      loginUrl: "https://passport.bilibili.com/login",
      cookies: ["SESSDATA"],
      domains: [".bilibili.com"],
      login: "present",
      capturedAt: "2026-11-10T12:30:00.000Z",
      plugins: ["usuyo-fetch/bilibili"],
    },
    {
      site: "yuanbao.tencent.com",
      name: "腾讯元宝",
      loginUrl: "https://yuanbao.tencent.com/",
      cookies: [],
      domains: [],
      login: "missing",
      plugins: ["usuyo-fetch/wechat-channels"],
    },
  ];
}

/** 读别的工具导出的文件的两样：X 插件的官方归档（照 docs/platforms/routes.md 的 `archive`），和官方的链接清单。 */
const X_ARCHIVE: FileReader = {
  id: "usuyo-fetch/x#archive",
  name: "官方归档",
  plugin: "X",
  reads: "X 设置里「下载你的数据」给的 zip，在你自己的电脑上读，不连 X",
  patterns: ["twitter-*.zip"],
  lists: ["喜欢", "书签"],
  note: "归档里有没有书签还没核过；没有的话只列得出喜欢。",
};
const LINK_LIST: FileReader = {
  id: "usuyo/link-list#file",
  name: "链接清单",
  plugin: "链接清单导入",
  reads: "一份 JSON 或 CSV，一行一个链接，可以带标题和日期；书签工具、阅读器导出的多半是这样",
  patterns: ["*.json", "*.csv"],
  lists: ["清单里的链接"],
};
const FILE_READERS = [X_ARCHIVE, LINK_LIST] as const;

/** 只认 `*` 的文件名通配，不分大小写。 */
function globMatch(pattern: string, name: string): boolean {
  const re = new RegExp(`^${pattern.split("*").map((p) => p.replace(/[.+?^${}()|[\]\\]/g, "\\$&")).join(".*")}$`, "i");
  return re.test(name);
}

const POSTS = ["关于慢慢写东西的一条长帖", "胶片扫描的一个小技巧", "一张东京的夜景", "读书笔记的模板", "做饭时的一个顺序", "前端动效的几条规矩"];

/** 归档里读出来的喜欢和书签：前两条库里已经存着，第三条在回收站里；每十条有一条只在文件里，没有链接。 */
function archiveEntries(n: number): ImportEntry[] {
  return Array.from({ length: n }, (_, i): ImportEntry => {
    const standing = i < 2 ? "stored" : i === 2 ? "trashed" : "new";
    return {
      seq: i + 1,
      ...(i % 10 === 9 ? {} : { url: `https://x.com/i/status/18${(5_300_000_000_000_000 + i * 7_919).toString()}` }),
      title: i % 10 === 9 ? "私信里转来的一张图" : (POSTS[i % POSTS.length] ?? ""),
      action: i % 4 === 0 ? "favorite" : "like",
      actedAt: new Date(Date.UTC(2026, 9, 28 - i, 9)).toISOString(),
      ...(standing === "new" ? {} : { itemId: `x-mock${i}` }),
      standing,
      taken: false,
    };
  });
}

const TITLES = ["4:6 冲煮法复刻", "手冲水温到底要多少", "胶片相机入门：先买哪台", "城市骑行路线整理", "一周备餐的思路", "读《置身事内》的笔记"];

function entriesFor(n: number, stored: number, trashed: number): ImportEntry[] {
  return Array.from({ length: n }, (_, i): ImportEntry => {
    const standing = i < stored ? "stored" : i < stored + trashed ? "trashed" : "new";
    const bv = `BV1${(1000 + i).toString(36)}xx411c7mD`;
    return {
      seq: i + 1,
      url: `https://www.bilibili.com/video/${bv}`,
      title: TITLES[i % TITLES.length] ?? "",
      action: "favorite",
      actedAt: new Date(Date.UTC(2026, 9, 30 - i, 13)).toISOString(),
      ...(standing === "new" ? {} : { itemId: `bilibili-mock${i}` }),
      standing,
      taken: false,
    };
  });
}

interface MockImport {
  view: ImportView;
  /** 已经列出来的。 */
  entries: ImportEntry[];
  /** 还没列到的（在列的时候一页页挪进 `entries`）。 */
  later: ImportEntry[];
  /** 收下一条要隔多久（毫秒），照插件的节奏，样例里缩短了。 */
  everyMs: number;
  /** 上次动过的时间。 */
  at: number;
  /** 确认时挑了要收的条目（`seq`）。 */
  chosen: Set<number>;
}

function counts(entries: ImportEntry[]): ImportView["preview"] {
  const n = (s: ImportEntry["standing"]) => entries.filter((e) => e.standing === s).length;
  return { new: n("new"), stored: n("stored"), trashed: n("trashed") };
}

const PAGE = 6;
const LIST_EVERY_MS = 1200;

/** 照时间往前走一步：在列的翻页，在收的收下轮到的。 */
function advance(i: MockImport, now: number): void {
  const v = i.view;
  if (v.state === "listing") {
    while (now - i.at >= LIST_EVERY_MS && i.later.length > 0) {
      i.entries.push(...i.later.splice(0, PAGE));
      i.at += LIST_EVERY_MS;
      i.view = { ...i.view, pages: i.view.pages + 1, preview: counts(i.entries) };
    }
    if (i.later.length === 0) i.view = { ...i.view, state: "ready", preview: counts(i.entries) };
    return;
  }
  if (v.state !== "taking") return;
  while (now - i.at >= i.everyMs) {
    const next = i.entries.find((e) => !e.taken && i.chosen.has(e.seq));
    if (!next) break;
    next.taken = true;
    i.at += i.everyMs;
    i.view = { ...i.view, taken: i.view.taken + 1 };
  }
  const left = i.entries.some((e) => !e.taken && i.chosen.has(e.seq));
  if (!left) {
    const { nextAt: _n, waiting: _w, ...rest } = i.view;
    i.view = { ...rest, state: "done" };
  } else {
    const nextAt = new Date(i.at + i.everyMs).toISOString();
    i.view = { ...i.view, nextAt, waiting: { code: "pacing", until: nextAt } };
  }
}

export function createMockPlatforms(empty = false): Platforms {
  // 插件上的 `pauses` 是各取法的合起来，同内核。
  const plugins = empty ? [] : basePlugins().map((p) => ({ ...p, pauses: p.methods.flatMap((m) => m.pauses) }));
  // 拨过开关以后，各平台都照它。
  const switched = (m: FetchMethod, enabled: boolean): FetchMethod =>
    m.perPlatform ? { ...m, enabled, perPlatform: m.perPlatform.map((t) => ({ ...t, enabled })) } : { ...m, enabled };
  const sites = empty ? [] : baseSites();
  const visitors = empty ? [] : baseVisitors();
  const imports = new Map<string, MockImport>();
  const tokens: (DeliverToken & { token: string })[] = empty
    ? []
    : [{ id: "6f1c0a9e2b7d4c31", name: "小红书油猴脚本", kind: "extension", createdAt: "2026-11-09T08:00:00.000Z", token: "" }];
  const now = Date.now();

  if (!empty) {
    const ready = entriesFor(24, 5, 1);
    imports.set("0193f2b0-0001-7f3a-8b4c-5d6e7f8a9b0c", {
      entries: ready,
      later: [],
      everyMs: 4000,
      at: now,
      chosen: new Set(),
      view: {
        id: "0193f2b0-0001-7f3a-8b4c-5d6e7f8a9b0c",
        kind: "source",
        title: "稍后再看",
        pluginId: "usuyo-fetch/bilibili",
        method: "favorites",
        source: "bilibili:toview",
        createdAt: "2026-11-11T09:00:00.000Z",
        state: "ready",
        pages: 2,
        preview: counts(ready),
        taken: 0,
        notStored: 0,
        skipped: 0,
      },
    });
    // 交进来的一批小红书：收下的几条指向样例库里真有的条目，取不到视频的那条在条目页写着原因。
    const linked = ["c2", "c7", "f2", "j5", "f7", "w1", "w4", "k3", "e4", "r4", "r5", "c1"];
    const taking = entriesFor(40, 3, 0).map((e, i) => ({
      ...e,
      url: e.url?.replace("bilibili.com/video", "xiaohongshu.com/explore"),
      taken: i >= 3 && i < 15,
      ...(i >= 3 && i < 15 ? { itemId: linked[i - 3] ?? `xhs-mock${i}` } : {}),
    }));
    imports.set("0193f2b0-0002-7f3a-8b4c-5d6e7f8a9b0c", {
      entries: taking,
      later: [],
      everyMs: 15_000,
      at: now,
      chosen: new Set(taking.filter((e) => e.standing === "new").map((e) => e.seq)),
      view: {
        id: "0193f2b0-0002-7f3a-8b4c-5d6e7f8a9b0c",
        kind: "delivered",
        title: "小红书收藏夹 2026-11-10",
        client: "小红书油猴脚本",
        createdAt: "2026-11-10T21:00:00.000Z",
        state: "taking",
        pages: 0,
        preview: counts(taking),
        taken: 12,
        notStored: 1,
        skipped: 0,
        nextAt: new Date(now + 15_000).toISOString(),
        waiting: { code: "pacing", until: new Date(now + 15_000).toISOString() },
      },
    });
  }

  const plugin = (id: string): FetchPlugin => {
    const p = plugins.find((x) => x.id === id);
    if (!p) throw new Error(`样例里没有插件 ${id}`);
    return p;
  };
  const getImport = (id: string): MockImport => {
    const i = imports.get(id);
    if (!i) throw new Error(`样例里没有导入 ${id}`);
    advance(i, Date.now());
    return i;
  };

  return {
    async plugins() {
      await wait(40);
      return { plugins: plugins.map((p) => ({ ...p, methods: p.methods.map((m) => ({ ...m })) })), notLoaded: [] };
    },
    async sources(pluginId, methodId, link) {
      await wait(600);
      const p = plugin(pluginId);
      const m = p.methods.find((x) => x.id === methodId);
      if (!m || (m.gets.list === undefined && m.gets.link === undefined)) throw new ApiError(422, `${methodId} 不能列`, "unsupported-input");
      if (!m.usable) throw new ApiError(422, m.blockers.map((b) => b.hint).join("；"), "unsupported-input");
      if (m.gets.link !== undefined) {
        if (link === undefined) throw new ApiError(422, "这种取法要先给一条链接", "unsupported-input");
        return [{ source: `bilibili:fav-public:${link.length}`, title: "胶片（公开收藏夹）", action: "favorite", count: 36, method: m.id }];
      }
      if (link !== undefined) throw new ApiError(422, "这种取法不从链接开始", "unsupported-input");
      if (p.id !== "usuyo-fetch/bilibili") return [];
      return [
        ...p.sources.filter((s) => s.method === m.id),
        { source: "bilibili:fav:50948568", title: "默认收藏夹", action: "favorite", count: 128, method: m.id },
        { source: "bilibili:fav:50948571", title: "做饭", action: "favorite", count: 36, method: m.id },
      ];
    },
    async switchMethod(pluginId, methodId, enabled) {
      await wait(60);
      const p = plugin(pluginId);
      const at = p.methods.findIndex((x) => x.id === methodId);
      const m = p.methods[at];
      if (!m) throw new ApiError(404, `样例里 ${pluginId} 没有取法 ${methodId}`, "not-found");
      if (!m.built) throw new ApiError(422, "这种取法还没做好", "unsupported-input");
      p.methods[at] = settle(switched(m, enabled));
      return { ...p, methods: p.methods.map((x) => ({ ...x })) };
    },
    async tryNow(pluginId) {
      await wait(40);
      const p = plugin(pluginId);
      p.pauses = [];
      // 每次换新的对象，同内核每次回新的一份。
      p.methods = p.methods.map((m) => settle({ ...m, pauses: [] }));
      return { ...p };
    },
    async hold(pluginId) {
      await wait(40);
      const p = plugin(pluginId);
      const hold = { method: "*", scope: "*", byYou: true };
      p.pauses = [...p.pauses.filter((x) => !x.byYou), hold];
      p.methods = p.methods.map((m) => settle({ ...m, pauses: [...m.pauses.filter((x) => !x.byYou), hold] }));
      return { ...p };
    },
    async sites() {
      await wait(40);
      return sites.map((s) => ({ ...s }));
    },
    async forgetLogin(site) {
      await wait(40);
      const s = sites.find((x) => x.site === site);
      if (s) {
        s.login = "missing";
        delete s.capturedAt;
      }
    },
    async visitors() {
      await wait(40);
      return visitors.map((v) => ({ ...v }));
    },
    async forgetVisitor(site) {
      await wait(40);
      const v = visitors.find((x) => x.site === site);
      if (v) {
        v.state = "missing";
        delete v.capturedAt;
        delete v.expiresAt;
      }
    },
    async imports() {
      await wait(40);
      return [...imports.keys()]
        .map((id) => ({ ...getImport(id).view }))
        .sort((a, b) => b.createdAt.localeCompare(a.createdAt));
    },
    async startImport(body) {
      await wait(80);
      const p = plugin(body.pluginId);
      const id = `0193f2b0-${(imports.size + 1).toString().padStart(4, "0")}-7f3a-8b4c-5d6e7f8a9b0c`;
      const all = entriesFor(18, 2, 1);
      imports.set(id, {
        entries: [],
        later: all,
        everyMs: Math.max(p.pacing[0]?.intervalSec ?? 5, 1) * 1000,
        at: Date.now(),
        chosen: new Set(),
        view: {
          id,
          kind: "source",
          title: body.title?.trim() || p.sources.find((s) => s.source === body.source)?.title || body.source,
          pluginId: body.pluginId,
          method: body.method,
          source: body.source,
          createdAt: new Date().toISOString(),
          state: "listing",
          pages: 0,
          preview: { new: 0, stored: 0, trashed: 0 },
          taken: 0,
          notStored: 0,
          skipped: 0,
        },
      });
      return { ...getImport(id).view };
    },
    async import(id) {
      await wait(40);
      return { ...getImport(id).view };
    },
    async importEntries(id, standing, offset, limit) {
      await wait(40);
      return getImport(id)
        .entries.filter((e) => e.standing === standing)
        .slice(offset, offset + limit)
        .map((e) => ({ ...e }));
    },
    async confirmImport(id, body) {
      await wait(80);
      const i = getImport(id);
      if (i.view.state !== "ready") throw new Error("这次导入不在等确认");
      const except = new Set(body.except ?? []);
      const group = (e: ImportEntry) => (e.standing === "new" ? body.new : e.standing === "trashed" ? body.trashed : null);
      if ([...except].some((seq) => { const e = i.entries.find((x) => x.seq === seq); return !e || group(e) === null; })) {
        throw new Error("except 里有不是新的、也不在回收站里的条目");
      }
      i.chosen = new Set(i.entries.filter((e) => { const g = group(e); return g !== null && g !== except.has(e.seq); }).map((e) => e.seq));
      i.at = Date.now();
      i.view = { ...i.view, state: i.chosen.size > 0 ? "taking" : "done" };
      advance(i, i.at);
      return { ...i.view };
    },
    async holdImport(id) {
      await wait(40);
      const i = getImport(id);
      const { nextAt: _n, waiting: _w, ...rest } = i.view;
      i.view = { ...rest, state: "held" };
      return { ...i.view };
    },
    async resumeImport(id) {
      await wait(40);
      const i = getImport(id);
      i.at = Date.now();
      i.view = { ...i.view, state: "taking" };
      advance(i, i.at);
      return { ...i.view };
    },
    async deleteImport(id) {
      await wait(40);
      imports.delete(id);
    },
    async deliverTokens() {
      await wait(40);
      return tokens.map(({ token: _t, ...t }) => t);
    },
    async createDeliverToken(body) {
      await wait(60);
      const id = Math.floor(Math.random() * 2 ** 48)
        .toString(16)
        .padStart(16, "0");
      const t = { id, name: body.name.trim(), kind: body.kind, createdAt: new Date().toISOString(), token: `${id}.${"0".repeat(64)}` };
      tokens.push(t);
      return { ...t };
    },
    async revokeDeliverToken(id) {
      await wait(40);
      const at = tokens.findIndex((t) => t.id === id);
      if (at >= 0) tokens.splice(at, 1);
    },
    inbox: async () => ({ path: "~/Library/Application Support/usuyo/inbox（样例）" }),
    async fileReaders() {
      await wait(40);
      return empty ? [LINK_LIST] : [...FILE_READERS];
    },
    async importFile(readerId, file, onSent, signal) {
      const reader = [...FILE_READERS].find((r) => r.id === readerId);
      if (!reader) throw new ApiError(404, "no such file reader", "not-found", "没有这种读文件的方法");
      // 照约 200 MB/s 交，一步 120 毫秒，能停下。
      for (let sent = 0; sent < file.size; ) {
        await wait(120);
        signal.throwIfAborted();
        sent = Math.min(file.size, sent + 24_000_000);
        onSent(sent);
      }
      await wait(300);
      // 样例里，名字对不上的当作插件认不出的文件；内核自己不按名字拒。
      if (!reader.patterns.some((pattern) => globMatch(pattern, file.name))) {
        throw new ApiError(400, "the plugin could not read the file", "invalid-args", `这不像${reader.plugin}的${reader.name}：里面没有它认得的文件`);
      }
      const id = `0193f2b0-${(imports.size + 1).toString().padStart(4, "0")}-7f3a-8b4c-5d6e7f8a9b0c`;
      const all = archiveEntries(reader.id === LINK_LIST.id ? 9 : 31);
      imports.set(id, {
        entries: [],
        later: all,
        everyMs: 5000,
        at: Date.now(),
        chosen: new Set(),
        view: {
          id,
          kind: "file",
          title: file.name,
          file: { name: file.name, reader: reader.plugin },
          createdAt: new Date().toISOString(),
          state: "listing",
          pages: 0,
          preview: { new: 0, stored: 0, trashed: 0 },
          taken: 0,
          notStored: 0,
          skipped: 0,
        },
      });
      return { ...getImport(id).view };
    },
  };
}
