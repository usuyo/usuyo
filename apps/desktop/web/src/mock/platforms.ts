// 样例：获取插件、登录站点、导入、投递令牌。和内核同一套接口（见 api/endpoints.ts），数据在内存里。
// 页面接上以后放进 api/source.ts 的 Library；这里先单独给出，「获取插件」「导入」两页照它搭。
import type {
  ConfirmImportRequest,
  CreateDeliverTokenRequest,
  DeliverToken,
  FetchPlugin,
  ImportEntry,
  ImportView,
  NewDeliverToken,
  PluginSource,
  PluginsView,
  SiteLogin,
  StartImportRequest,
  StorageState,
} from "../api/endpoints";

/** 「获取插件」「导入」两页要的读写。真的内核照 `paths` 的同名路由实现。 */
export interface Platforms {
  plugins(): Promise<PluginsView>;
  sources(pluginId: string): Promise<PluginSource[]>;
  tryNow(pluginId: string): Promise<FetchPlugin>;
  hold(pluginId: string): Promise<FetchPlugin>;
  setSecret(pluginId: string, key: string, value: string): Promise<FetchPlugin>;
  forgetSecret(pluginId: string, key: string): Promise<FetchPlugin>;
  sites(): Promise<SiteLogin[]>;
  saveLogin(site: string, state: StorageState): Promise<SiteLogin>;
  forgetLogin(site: string): Promise<void>;
  imports(): Promise<ImportView[]>;
  startImport(body: StartImportRequest): Promise<ImportView>;
  import(id: string): Promise<ImportView>;
  importEntries(id: string, standing?: ImportEntry["standing"]): Promise<ImportEntry[]>;
  confirmImport(id: string, body: ConfirmImportRequest): Promise<ImportView>;
  holdImport(id: string): Promise<ImportView>;
  resumeImport(id: string): Promise<ImportView>;
  deleteImport(id: string): Promise<void>;
  deliverTokens(): Promise<DeliverToken[]>;
  createDeliverToken(body: CreateDeliverTokenRequest): Promise<NewDeliverToken>;
  revokeDeliverToken(id: string): Promise<void>;
  inbox(): Promise<{ path: string }>;
}

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
  entries: ImportEntry[];
}

function counts(entries: ImportEntry[]): ImportView["preview"] {
  const n = (s: ImportEntry["standing"]) => entries.filter((e) => e.standing === s).length;
  return { new: n("new"), stored: n("stored"), trashed: n("trashed") };
}

export function createMockPlatforms(): Platforms {
  const plugins = basePlugins();
  const sites = baseSites();
  const imports = new Map<string, MockImport>();
  const tokens: (DeliverToken & { token: string })[] = [
    { id: "6f1c0a9e2b7d4c31", name: "小红书油猴脚本", kind: "extension", createdAt: "2026-11-09T08:00:00.000Z", token: "" },
  ];

  const ready = entriesFor(24, 5, 1);
  imports.set("0193f2b0-0001-7f3a-8b4c-5d6e7f8a9b0c", {
    entries: ready,
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
  const taking = entriesFor(40, 3, 0).map((e, i) => ({ ...e, url: e.url.replace("bilibili.com/video", "xiaohongshu.com/explore"), taken: i >= 3 && i < 15 }));
  imports.set("0193f2b0-0002-7f3a-8b4c-5d6e7f8a9b0c", {
    entries: taking,
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
      nextAt: inMinutes(0.25),
      waiting: { code: "pacing", until: inMinutes(0.25) },
    },
  });

  const plugin = (id: string): FetchPlugin => {
    const p = plugins.find((x) => x.id === id);
    if (!p) throw new Error(`样例里没有插件 ${id}`);
    return p;
  };
  const getImport = (id: string): MockImport => {
    const i = imports.get(id);
    if (!i) throw new Error(`样例里没有导入 ${id}`);
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
      await wait(300);
      const p = plugin(pluginId);
      if (!p.canList) throw new Error(`${p.name} 不能列出你存过的`);
      return [...p.sources, { source: "bilibili:fav:50948568", title: "默认收藏夹", action: "favorite", count: 128 }];
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
    async setSecret(pluginId, key, value) {
      await wait(40);
      return setSecret(pluginId, key, value.trim() !== "");
    },
    async forgetSecret(pluginId, key) {
      await wait(40);
      return setSecret(pluginId, key, false);
    },
    async sites() {
      await wait(40);
      return sites.map((s) => ({ ...s }));
    },
    async saveLogin(site, state) {
      await wait(120);
      const s = sites.find((x) => x.site === site);
      if (!s) throw new Error(`没有插件用到 ${site}`);
      const names = new Set(state.cookies.map((c) => c.name));
      const ok = s.cookies.length === 0 ? names.size > 0 : s.cookies.every((c) => names.has(c));
      s.login = ok ? "present" : "missing";
      s.capturedAt = new Date().toISOString();
      return { ...s };
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
      return [...imports.values()].map((i) => ({ ...i.view })).sort((a, b) => b.createdAt.localeCompare(a.createdAt));
    },
    async startImport(body) {
      await wait(80);
      const p = plugin(body.pluginId);
      const id = `0193f2b0-${(imports.size + 1).toString().padStart(4, "0")}-7f3a-8b4c-5d6e7f8a9b0c`;
      const entries = entriesFor(18, 2, 0);
      // 样例不真的翻页：建出来就是列完、等你看的样子
      imports.set(id, {
        entries,
        view: {
          id,
          kind: "source",
          title: body.title?.trim() || p.sources.find((s) => s.source === body.source)?.title || body.source,
          pluginId: body.pluginId,
          source: body.source,
          createdAt: new Date().toISOString(),
          state: "ready",
          pages: 1,
          preview: counts(entries),
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
    async importEntries(id, standing) {
      await wait(40);
      return getImport(id).entries.filter((e) => standing === undefined || e.standing === standing);
    },
    async confirmImport(id, body) {
      await wait(80);
      const i = getImport(id);
      if (i.view.state !== "ready") throw new Error("这次导入不在等确认");
      const wanted = i.entries.filter((e) => e.standing === "new" || (body.alsoTrashed === true && e.standing === "trashed"));
      i.view = { ...i.view, state: wanted.length > 0 ? "taking" : "done", nextAt: inMinutes(0.1) };
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
      i.view = { ...i.view, state: "taking", nextAt: inMinutes(0.1) };
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
