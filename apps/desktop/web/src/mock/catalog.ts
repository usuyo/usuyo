// 样例库的插件目录：订阅源 usuyo-fetch/index 里的几个获取插件（和 mock/platforms.ts 装着的对得上），
// 加随 usuyo 带的转写与识别插件。装、更新、删除照内核的样子一步步走，几秒就完。
import type { CatalogMethod, CatalogPlugin, CatalogSource, CatalogView } from "../api/endpoints";
import type { Catalog } from "../api/source";
import { currentLocale } from "../i18n";
import { said } from "./said";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

const SOURCE = "usuyo-fetch/index";
const TARGET = "macos-arm64";

const m = (id: string, name: string, access: string, account = "none", login = "none", built = true): CatalogMethod => ({ id, name, built, access, login, account });

const fetchPlugin = (id: string, name: string, description: string, platforms: { id: string; name: string }[], methods: CatalogMethod[], version: string | null, installed?: string): CatalogPlugin => ({
  id,
  kind: "fetch",
  name,
  description,
  platforms,
  repository: `https://github.com/${id}`,
  license: "MIT",
  methods,
  source: SOURCE,
  ...(version ? { latest: { version, publishedAt: "2026-11-12T08:00:00.000Z", url: `https://github.com/${id}/releases/tag/v${version}`, asset: { name: `${id.split("/")[1]}_${version}_${TARGET}.tar.gz`, size: 3_400_000 } } } : {}),
  ...(installed ? { installed: { version: installed, origin: "source" as const, source: SOURCE, dir: `/m/plugins/${id}/${installed}` } } : {}),
  update: Boolean(installed && version && version !== installed),
  ...(version ? {} : { blocked: { code: "no-release" as const, hint: said("这个插件还没有发布的版本。", "This plugin has no released version yet.") } }),
});

const builtin = (id: string, name: string, description: string): CatalogPlugin => ({
  id,
  kind: "transform",
  name,
  description,
  platforms: [],
  license: "MIT",
  methods: [],
  installed: { version: "0.1.0", origin: "development", dir: `/m/plugins/${id}/0.1.0` },
  update: false,
  blocked: { code: "builtin", hint: said("这个插件随 usuyo 提供，不从订阅源安装或删除。", "This plugin comes with usuyo. It isn't installed or removed from a source.") },
});

function start(): CatalogPlugin[] {
  return [
    fetchPlugin("usuyo-fetch/bilibili", "B站", "按视频链接取元数据、封面和低画质音视频（不用账号）；列公开的收藏夹；列我的收藏夹和稍后再看（用账号）", [{ id: "bilibili", name: "B站" }], [m("view", "公开接口", "public-web"), m("favorites-public", "公开收藏夹", "public-web"), m("favorites", "我的收藏夹和稍后再看", "session-api", "in-usuyo", "required")], "0.1.0", "0.1.0"),
    fetchPlugin(
      "usuyo-fetch/x",
      "X",
      "按链接取帖子的文字和图片；读官方归档里的书签和喜欢",
      [{ id: "x", name: "X" }],
      [
        m("guest", "访客接口", "public-web"),
        m("embed", "嵌入接口", "public-web"),
        m("userscript", "浏览器脚本", "browser-session", "own-browser", "required"),
        m("archive", "官方归档", "official-export"),
        m("deliver", "内容投递", "browser-session", "own-browser", "required", false),
        m("session", "会话接口", "session-api", "in-usuyo", "required", false),
        m("signed", "签名接口", "private-api", "in-usuyo", "required", false),
        m("mirror", "手机镜像", "vision-ui", "own-phone", "required", false),
      ],
      "0.2.0",
      "0.1.0",
    ),
    fetchPlugin(
      "usuyo-fetch/rednote",
      "小红书",
      "按分享链接取笔记的文字、图片和视频",
      [{ id: "rednote", name: "小红书" }],
      [m("note-page", "笔记页", "public-web", "visitor"), m("in-usuyo", "在 usuyo 中登录并读取", "browser-session", "in-usuyo", "required"), m("userscript", "浏览器脚本", "browser-session", "own-browser", "required"), m("iphone", "iPhone", "vision-ui", "own-phone", "required", false)],
      "0.1.0",
      "0.1.0",
    ),
    fetchPlugin(
      "usuyo-fetch/wechat-channels",
      "微信视频号",
      "取视频号视频的作者、描述和封面",
      [{ id: "wechat-channels", name: "微信视频号" }],
      [m("feed-info", "分享信息", "public-web"), m("yuanbao", "腾讯元宝", "session-api", "in-usuyo", "required"), m("pc-client", "电脑微信", "client-hook", "own-app", "required", false), m("iphone", "iPhone", "vision-ui", "own-phone", "required", false)],
      "0.1.0",
      "0.1.0",
    ),
    fetchPlugin("usuyo-fetch/douyin", "抖音", "按分享链接取作品的文字、作者和视频（不用账号）", [{ id: "douyin", name: "抖音" }], [m("share-page", "分享页", "public-web"), m("iphone", "iPhone 镜像", "vision-ui", "own-phone", "required", false)], "0.1.0"),
    fetchPlugin("usuyo-fetch/zhihu", "知乎", "按链接取回答和文章的正文", [{ id: "zhihu", name: "知乎" }], [m("page", "公开页", "public-web", "visitor")], "0.1.0"),
    fetchPlugin("usuyo-fetch/telegram", "Telegram", "读 Telegram Desktop 导出的聊天记录", [{ id: "telegram", name: "Telegram" }], [m("export", "导出文件", "official-export")], null),
    builtin("builtin/media", "媒体", "探测媒体、合并音视频、抽音轨"),
    builtin("builtin/asr-sherpa", "语音转写", "本机把口播转写成文字"),
    builtin("builtin/ocr-vision", "文字识别", "用系统的文字识别读出图中文字和画面中文字"),
  ];
}

/**
 * 订阅源目录里的英文名（照 usuyo-fetch/index 的 index.json）：内核按设置的语言挑，样例库照当前语言换上。
 * 插件清单本身只有一种语言的名字，`/plugins` 里的不换。
 */
const EN: Record<string, { name: string; description: string; platforms?: Record<string, string>; methods?: Record<string, string> }> = {
  "usuyo-fetch/bilibili": {
    name: "Bilibili",
    description: "Fetch a video by link: metadata, cover and low-quality audio and video (no account). List public favorites. List my favorites and Watch Later (account).",
    platforms: { bilibili: "Bilibili" },
    methods: { view: "Public API", "favorites-public": "Public Favorites", favorites: "My Favorites and Watch Later" },
  },
  "usuyo-fetch/x": {
    name: "X",
    description: "Fetch a post by link: text and images. Read bookmarks and likes from the official archive.",
    methods: { guest: "Guest API", embed: "Embed API", userscript: "Browser Script (Bookmarks, Likes)", archive: "Official Archive", deliver: "Content Delivery", session: "Session API", signed: "Signed API", mirror: "Phone Mirroring" },
  },
  "usuyo-fetch/rednote": {
    name: "rednote",
    description: "Fetch the text, images and video of a note from a share link.",
    platforms: { rednote: "rednote" },
    methods: { "note-page": "Public Note Page", "in-usuyo": "Sign In and Read in usuyo", userscript: "Browser Script (Favorites, Likes)", iphone: "iPhone Mirroring" },
  },
  "usuyo-fetch/wechat-channels": {
    name: "WeChat Channels",
    description: "Fetch the author, description and cover of a Channels video.",
    platforms: { "wechat-channels": "WeChat Channels" },
    methods: { "feed-info": "Share Page API", yuanbao: "Tencent Yuanbao Video Exchange", "pc-client": "WeChat for Mac", iphone: "iPhone Mirroring" },
  },
  "usuyo-fetch/douyin": {
    name: "Douyin",
    description: "Fetch the text, author and video of a post from a share link (no account).",
    platforms: { douyin: "Douyin" },
    methods: { "share-page": "Share Page", iphone: "iPhone Mirroring" },
  },
  "usuyo-fetch/zhihu": { name: "Zhihu", description: "Fetch the body of an answer or article by link.", platforms: { zhihu: "Zhihu" }, methods: { page: "Public Page" } },
  "usuyo-fetch/telegram": { name: "Telegram", description: "Read chats exported with Telegram Desktop.", methods: { export: "Telegram Desktop Export" } },
  "builtin/media": { name: "Media", description: "Probe media, combine audio and video, extract the audio track." },
  "builtin/asr-sherpa": { name: "Speech Transcription", description: "Transcribe speech on this computer." },
  "builtin/ocr-vision": { name: "Text Recognition", description: "Recognize text in images and video with the system's text recognition." },
};

function localized(p: CatalogPlugin): CatalogPlugin {
  const en = EN[p.id];
  if (currentLocale() !== "en" || !en) return p;
  return {
    ...p,
    name: en.name,
    description: en.description,
    platforms: p.platforms.map((x) => ({ ...x, name: en.platforms?.[x.id] ?? x.name })),
    methods: p.methods.map((x) => ({ ...x, name: en.methods?.[x.id] ?? x.name })),
  };
}

const SOURCES = (): CatalogSource[] => [
  { address: SOURCE, indexUrl: `https://raw.githubusercontent.com/${SOURCE}/HEAD/index.json`, default: true, refreshedAt: "2026-11-13T08:00:00.000Z", plugins: 12, refreshing: false },
];

/** `empty` 时一个都没装，订阅源也还没读到。 */
export function createMockCatalog(empty = false): Catalog {
  let plugins = empty ? start().map((p) => (p.kind === "fetch" ? { ...p, installed: undefined, update: false } : p)) : start();
  let sources = SOURCES();
  const view = (): CatalogView => structuredClone({ target: TARGET, sources, plugins: plugins.map(localized) });
  const set = (id: string, change: (p: CatalogPlugin) => CatalogPlugin) => {
    plugins = plugins.map((p) => (p.id === id ? change(p) : p));
  };
  const one = (id: string): CatalogPlugin => {
    const p = plugins.find((x) => x.id === id);
    if (!p) throw new Error(`no plugin ${id}`);
    return structuredClone(localized(p));
  };
  // 下载分几步走完，再「载入」。
  const run = (id: string, kind: "install" | "update") => {
    const total = 3_400_000;
    [0.2, 0.5, 0.8, 1].forEach((share, i) => {
      setTimeout(() => set(id, (p) => ({ ...p, action: { kind, phase: "downloading", received: Math.round(total * share), total } })), 300 * (i + 1));
    });
    setTimeout(() => set(id, (p) => ({ ...p, action: { kind, phase: "verifying" } })), 1500);
    setTimeout(() => {
      set(id, (p) => {
        const version = p.latest?.version ?? "0.1.0";
        const { action: _done, ...rest } = p;
        return { ...rest, installed: { version, origin: "source", source: SOURCE, dir: `/m/plugins/${id}/${version}` }, update: false };
      });
    }, 2000);
  };
  return {
    async catalog() {
      await wait(60);
      return view();
    },
    async installPlugin(id) {
      await wait(80);
      set(id, (p) => ({ ...p, action: { kind: "install", phase: "downloading", received: 0, total: 3_400_000 } }));
      run(id, "install");
      return one(id);
    },
    async updatePlugin(id) {
      await wait(80);
      set(id, (p) => ({ ...p, action: { kind: "update", phase: "downloading", received: 0, total: 3_400_000 } }));
      run(id, "update");
      return one(id);
    },
    async removePlugin(id) {
      await wait(300);
      set(id, (p) => ({ ...p, installed: undefined, update: false }));
      return view();
    },
    async addSource(address) {
      await wait(200);
      sources = [...sources, { address, indexUrl: `https://raw.githubusercontent.com/${address}/HEAD/index.json`, default: false, refreshing: false, error: said("找不到这个仓库里的 index.json。检查地址，或确认仓库是公开的。", "Couldn't find index.json in this repository. Check the address, or make sure the repository is public.") }];
      return structuredClone(sources);
    },
    async removeSource(address) {
      await wait(100);
      sources = sources.filter((s) => s.address !== address);
      return structuredClone(sources);
    },
    async refreshSources() {
      await wait(400);
      sources = sources.map((s) => (s.error ? s : { ...s, refreshedAt: new Date().toISOString() }));
      return structuredClone(sources);
    },
  };
}
