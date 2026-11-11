// 样例：获取插件、登录站点、导入、投递令牌。和内核同一套接口（见 api/endpoints.ts），数据在内存里。
// 导入照时间往前走：新开的一次过几秒列完，确认以后隔一会儿收下一条，页面不用真的内核也能看到「来了」。
import type {
  DeliverToken,
  FetchPlugin,
  ImportEntry,
  ImportView,
  SiteLogin,
} from "../api/endpoints";
import type { Platforms } from "../api/source";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const inMinutes = (m: number) => new Date(Date.now() + m * 60_000).toISOString();

function basePlugins(): FetchPlugin[] {
  return [
    {
      id: "usuyo-fetch/xiaohongshu",
      name: "小红书",
      description: "按你存过的笔记链接取公开笔记页的正文、图片和视频，做个人备份",
      version: "0.1.0",
      platforms: [{ id: "xiaohongshu", name: "小红书" }],
      access: "public-web",
      login: "none",
      canList: false,
      sources: [],
      sites: [],
      settings: [
        { key: "visitor-cookie", label: "访客 cookie（可选，不是你的账号）", type: "string", secret: true, filled: false },
      ],
      pacing: [{ platform: "xiaohongshu", batch: true, intervalSec: 15, daily: 100 }],
      pauses: [],
    },
    {
      id: "usuyo-fetch/wechat-channels",
      name: "微信视频号",
      description: "用分享链接取一条：不登录只存作者、描述和封面；登录了腾讯元宝才取视频文件",
      version: "0.1.0",
      platforms: [{ id: "wechat-channels", name: "微信视频号" }],
      access: "session-api",
      login: "optional",
      canList: false,
      sources: [],
      sites: ["yuanbao.tencent.com"],
      settings: [],
      pacing: [{ platform: "wechat-channels", batch: true, intervalSec: 30, daily: 50 }],
      pauses: [],
    },
    {
      id: "usuyo-fetch/bilibili",
      name: "B站",
      description: "列出我的收藏夹和稍后再看，按视频链接取元数据、封面、字幕和低画质音视频",
      version: "0.1.0",
      platforms: [{ id: "bilibili", name: "B站" }],
      access: "session-api",
      login: "required",
      canList: true,
      sources: [{ source: "bilibili:toview", title: "稍后再看", action: "favorite" }],
      sites: ["www.bilibili.com"],
      settings: [],
      pacing: [{ platform: "bilibili", batch: true, intervalSec: 5, daily: 500 }],
      // 样例里 B站 刚被限流：页面上写原因和什么时候再试
      pauses: [{ scope: "bilibili", byYou: false, until: inMinutes(12), code: "rate-limited", hint: "B站说请求太频繁，过一会儿再试" }],
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
  /** 确认时勾了「回收站里的也拿回来」。 */
  alsoTrashed: boolean;
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
  const { alsoTrashed } = i;
  while (now - i.at >= i.everyMs) {
    const next = i.entries.find((e) => !e.taken && (e.standing === "new" || (alsoTrashed && e.standing === "trashed")));
    if (!next) break;
    next.taken = true;
    i.at += i.everyMs;
    i.view = { ...i.view, taken: i.view.taken + 1 };
  }
  const left = i.entries.some((e) => !e.taken && (e.standing === "new" || (alsoTrashed && e.standing === "trashed")));
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
      alsoTrashed: false,
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
      alsoTrashed: false,
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
  const setSecret = (pluginId: string, key: string, filled: boolean): FetchPlugin => {
    const p = plugin(pluginId);
    p.settings = p.settings.map((s) => (s.key === key && s.secret ? { ...s, filled } : s));
    return { ...p };
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
    async tryNow(pluginId) {
      await wait(40);
      const p = plugin(pluginId);
      p.pauses = [];
      return { ...p };
    },
    async hold(pluginId) {
      await wait(40);
      const p = plugin(pluginId);
      p.pauses = [{ scope: "*", byYou: true }];
      return { ...p };
    },
    async forgetSecret(pluginId, key) {
      await wait(40);
      return setSecret(pluginId, key, false);
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
        alsoTrashed: false,
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
      const wanted = i.entries.filter((e) => e.standing === "new" || (body.alsoTrashed === true && e.standing === "trashed"));
      i.at = Date.now();
      i.alsoTrashed = body.alsoTrashed === true;
      i.view = { ...i.view, state: wanted.length > 0 ? "taking" : "done" };
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
