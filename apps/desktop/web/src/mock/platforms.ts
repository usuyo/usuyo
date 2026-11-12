// 样例：获取插件、登录站点、访客身份、导入、投递令牌。和内核同一套接口（见 api/endpoints.ts），数据在内存里。
// 导入照时间往前走：新开的一次过几秒列完，确认以后隔一会儿收下一条，页面不用真的内核也能看到「来了」。
import type {
  DeliverToken,
  FetchMethod,
  FetchPlugin,
  ImportEntry,
  ImportView,
  SiteLogin,
  VisitorIdentity,
} from "../api/endpoints";
import type { Platforms } from "../api/source";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const inMinutes = (m: number) => new Date(Date.now() + m * 60_000).toISOString();

/** 一种取法的样例：没写的照常用默认值（做好了、开着、没停）。 */
function method(m: Pick<FetchMethod, "id" | "name" | "access" | "account" | "gets"> & Partial<FetchMethod>): FetchMethod {
  return { built: true, enabled: true, pauses: [], ...m };
}

const notBuilt = (m: Pick<FetchMethod, "id" | "name" | "access" | "account" | "gets"> & Partial<FetchMethod>) => method({ ...m, built: false, enabled: false });

/** 样例的四个平台插件：一个平台一个插件，插件里几种取法。 */
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
      access: "public-web",
      login: "optional",
      canList: true,
      pacing: [{ platform: "x", batch: true, intervalSec: 10, daily: 200 }],
      methods: [
        method({
          id: "guest",
          name: "访客接口",
          access: "public-web",
          account: { kind: "none" },
          gets: { fetch: ["正文", "图", "视频"] },
          riskNote: { text: "不带账号；风险在出口上：访客令牌可能被拒、被限流。", source: { label: "gallery-dl #9505", url: "https://github.com/mikf/gallery-dl/issues/9505" } },
        }),
        method({ id: "embed", name: "嵌入接口", access: "public-web", account: { kind: "none" }, gets: { fetch: ["正文", "图"] }, riskNote: { text: "访客令牌被限流时退到这里；只给截断的正文，长帖只拿到前一段。", source: { label: "vercel/react-tweet", url: "https://github.com/vercel/react-tweet" } } }),
        method({
          id: "userscript",
          name: "浏览器脚本",
          access: "browser-session",
          account: { kind: "own-browser", where: "X" },
          gets: { list: ["书签", "点过赞的"], delivers: "links" },
          riskNote: { text: "只截获页面自己拿回的回答，不发额外请求，一屏屏由你自己翻；没有找到使用者被锁的报告。", source: { label: "twitter-web-exporter #120", url: "https://github.com/prinsss/twitter-web-exporter/issues/120" } },
        }),
        method({
          id: "archive",
          name: "官方归档",
          access: "official-export",
          account: { kind: "none" },
          gets: { list: ["你发过的", "点过赞的"] },
          riskNote: { text: "读你从 X 设置里申请下载的 zip，不碰账号；归档里有没有书签还要核。" },
        }),
        notBuilt({
          id: "session",
          name: "会话接口",
          access: "session-api",
          account: { kind: "in-usuyo", sites: ["x.com"] },
          gets: { list: ["书签"] },
          riskNote: { text: "要在 usuyo 里登录 X，cookie 出了浏览器、按程序的节奏翻页；只多一样跟着收。", source: { label: "fieldtheory-cli", url: "https://github.com/afar1/fieldtheory-cli" } },
        }),
        notBuilt({ id: "graphql", name: "网页内部接口", access: "private-api", account: { kind: "in-usuyo", sites: ["x.com"] }, gets: { list: ["书签"], fetch: ["正文", "图", "视频"] } }),
        notBuilt({ id: "mirror", name: "手机镜像", access: "vision-ui", account: { kind: "own-phone", where: "X" }, gets: { list: ["书签"], delivers: "links" } }),
        notBuilt({ id: "share", name: "手机分享", access: "public-web", account: { kind: "own-phone", where: "X" }, gets: { delivers: "links" } }),
        notBuilt({ id: "hook", name: "客户端注入", access: "client-hook", account: { kind: "own-phone", where: "X" }, gets: { list: ["书签", "点过赞的"] } }),
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
      canList: false,
      visitors: ["www.xiaohongshu.com"],
      pacing: [{ platform: "xiaohongshu", batch: true, intervalSec: 15, daily: 100 }],
      methods: [
        method({
          id: "page",
          name: "访客页面",
          access: "public-web",
          account: { kind: "visitor", sites: ["www.xiaohongshu.com"] },
          gets: { fetch: ["正文", "图", "视频"] },
          riskNote: { text: "读公开笔记页；访客身份过一阵会失效，换一份就行。" },
        }),
        method({
          id: "userscript",
          name: "浏览器脚本",
          access: "browser-session",
          account: { kind: "own-browser", where: "小红书" },
          gets: { list: ["存过的笔记", "点过赞的"], delivers: "links" },
          riskNote: { text: "脚本只在你自己打开那一页时读链接，不替你翻页。" },
        }),
        notBuilt({ id: "iphone", name: "iPhone 镜像", access: "vision-ui", account: { kind: "own-phone", where: "小红书" }, gets: { list: ["存过的笔记"], delivers: "links" } }),
      ],
    },
    {
      ...base,
      id: "usuyo-fetch/wechat-channels",
      name: "微信视频号",
      description: "用分享链接取一条：不登录只存作者、描述和封面；登录了腾讯元宝才取视频文件",
      platforms: [{ id: "wechat-channels", name: "微信视频号" }],
      access: "session-api",
      login: "optional",
      canList: false,
      sites: ["yuanbao.tencent.com"],
      pacing: [{ platform: "wechat-channels", batch: true, intervalSec: 30, daily: 50 }],
      methods: [
        method({ id: "preview", name: "分享预览", access: "public-web", account: { kind: "none" }, gets: { fetch: ["作者", "描述", "封面"] } }),
        // 要在 usuyo 里登录的，登录了才开：样例里还没登录，所以关着。
        method({
          id: "yuanbao",
          name: "腾讯元宝",
          access: "session-api",
          account: { kind: "in-usuyo", sites: ["yuanbao.tencent.com"] },
          gets: { fetch: ["视频"] },
          enabled: false,
          riskNote: { text: "经腾讯元宝的网页解析视频号链接，用的是你的元宝账号。" },
        }),
        notBuilt({ id: "iphone", name: "iPhone 镜像", access: "vision-ui", account: { kind: "own-phone", where: "微信" }, gets: { list: ["赞过的", "存过的"], delivers: "links" } }),
      ],
    },
    {
      ...base,
      id: "usuyo-fetch/bilibili",
      name: "B站",
      description: "按视频链接取元数据、封面、字幕和低画质音视频；列出存过的和稍后再看要登录",
      platforms: [{ id: "bilibili", name: "B站" }],
      access: "session-api",
      login: "optional",
      canList: true,
      sources: [{ source: "bilibili:toview", title: "稍后再看", action: "favorite" }],
      sites: ["www.bilibili.com"],
      pacing: [{ platform: "bilibili", batch: true, intervalSec: 5, daily: 500 }],
      methods: [
        method({ id: "public", name: "公开接口", access: "public-web", account: { kind: "none" }, gets: { fetch: ["标题", "封面", "字幕", "音视频"] } }),
        method({
          id: "lists",
          name: "收藏夹和稍后再看",
          access: "session-api",
          account: { kind: "in-usuyo", sites: ["www.bilibili.com"] },
          gets: { list: ["收藏夹", "稍后再看"] },
          // 样例里这一种刚被限流：页面上写原因和什么时候再试
          pauses: [{ scope: "bilibili", byYou: false, until: inMinutes(12), code: "rate-limited", hint: "B站说请求太频繁，过一会儿再试" }],
          riskNote: { text: "用你的登录调 B站的接口列出存过的；只列，不改。" },
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
  const plugins = empty ? [] : basePlugins();
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
      url: e.url.replace("bilibili.com/video", "xiaohongshu.com/explore"),
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
      return { plugins: plugins.map((p) => ({ ...p })), notLoaded: [] };
    },
    async sources(pluginId) {
      await wait(600);
      const p = plugin(pluginId);
      if (!p.canList) throw new Error(`${p.name} 不能列出你存过的`);
      return [
        ...p.sources,
        { source: "bilibili:fav:50948568", title: "默认收藏夹", action: "favorite", count: 128 },
        { source: "bilibili:fav:50948571", title: "做饭", action: "favorite", count: 36 },
      ];
    },
    async switchMethod(pluginId, methodId, enabled) {
      await wait(60);
      const p = plugin(pluginId);
      const m = p.methods.find((x) => x.id === methodId);
      if (!m) throw new Error(`样例里 ${pluginId} 没有取法 ${methodId}`);
      if (!m.built) throw new Error("invalid-args: 这种取法还没做好");
      m.enabled = enabled;
      return { ...p, methods: p.methods.map((x) => ({ ...x })) };
    },
    async tryNow(pluginId) {
      await wait(40);
      const p = plugin(pluginId);
      p.pauses = [];
      for (const m of p.methods) m.pauses = [];
      return { ...p };
    },
    async hold(pluginId) {
      await wait(40);
      const p = plugin(pluginId);
      p.pauses = [{ scope: "*", byYou: true }];
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
  };
}
