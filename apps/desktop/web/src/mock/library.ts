// 样例库：和内核同一套接口，数据在内存里，刷新页面就回到原样。
// empty 为真时从空资料库开始，看空资料库那一页用（地址栏带 ?mock=empty）。
import type {
  Facet,
  Item,
  MapPoint,
  MapRegion,
  MapView,
  Platform,
  Position,
  SearchHit,
  Segment,
  TakeRequest,
  TakeResponse,
  Topic,
} from "../api/endpoints";
import type { Library } from "../api/source";
import { r01 } from "../fog/noise";
import { PLATFORMS, RAW_ITEMS, TOPICS, type RawItem } from "./data";
import { dotPicture, PICTURE_SIZE, SCENES } from "./pictures";

const PAGE_W = 1440;
const PAGE_H = 900;
const MAP_X = 248;
const MAP_W = PAGE_W - MAP_X;

interface Entry {
  item: Item;
  /** 页面坐标（1440×900），返回时换成 0–1。 */
  x: number;
  y: number;
}

function snapshotOf(id: string): string {
  return `0193f2b0-${id.padEnd(4, "0").slice(0, 4)}-7f3a-8b4c-5d6e7f8a9b0c`;
}

function takenAt(raw: RawItem): string {
  const minute = String(Math.floor(r01(`${raw.id}:m`) * 50) + 5).padStart(2, "0");
  // 正午前后，哪个时区看都是同一天。
  return `2026-${raw.at}T1${Math.floor(r01(`${raw.id}:h`) * 3)}:${minute}:00.000Z`;
}

function buildFacets(raw: RawItem): Facet[] {
  if (raw.link) return [];
  const snapshotId = snapshotOf(raw.id);
  // 图到打开一条时才画（pictured），这里先记下画哪一张。
  const pic = (k: number | string) => `${PIC}${raw.scene}|${raw.same ? raw.id : `${raw.id}:${k}`}`;
  const position = (at: number | string): Position =>
    typeof at === "number" ? { kind: "time", sec: at, snapshotId } : { kind: "image", imageKey: at };
  const facets: Facet[] = [];
  if (raw.kind === "video") {
    const dur = raw.dur ?? 60;
    const frames = Array.from({ length: 8 }, (_, i) => {
      const sec = Math.round((dur * (i + 0.5)) / 8);
      return { sec, url: pic(i) };
    });
    facets.push({
      layer: "video",
      snapshotId,
      durationSec: dur,
      frames,
      quality: "1080p",
      cover: { key: "cover", url: pic("cover"), ...PICTURE_SIZE },
      ...(raw.same ? { remark: "画面一直是同一张图" } : {}),
    });
  } else if (raw.images) {
    const images = Array.from({ length: raw.images }, (_, i) => ({
      key: `p${i + 1}`,
      url: pic(i),
      ...PICTURE_SIZE,
    }));
    facets.push({ layer: "image", images, ...(raw.same ? { remark: "几张图是同一张" } : {}) });
  }
  if (raw.asr) {
    facets.push({
      layer: "asr",
      by: raw.asrByPlatform ? "platform" : "machine",
      segments: raw.asr.map(([sec, text]) => ({ text, position: position(sec) })),
    });
  }
  if (raw.ocr) {
    facets.push({ layer: "ocr", segments: raw.ocr.map(([at, text]) => ({ text, position: position(at) })) });
  }
  if (raw.text) facets.push({ layer: "text", segments: raw.text.map((text): Segment => ({ text })) });
  if (raw.comments) {
    facets.push({
      layer: "comments",
      comments: raw.comments.map(([author, text, likes]) => (likes === undefined ? { author, text } : { author, text, likes })),
    });
  }
  return facets;
}

const PIC = "mock-picture:";

function draw(url: string): string {
  if (!url.startsWith(PIC)) return url;
  const [scene, seed] = url.slice(PIC.length).split("|");
  return dotPicture(SCENES.find((s) => s === scene) ?? "plain", seed ?? "");
}

/** 把记下的图换成画好的 data URL；只在回一整条时做。 */
function pictured(item: Item): Item {
  return {
    ...item,
    facets: item.facets.map((f): Facet => {
      if (f.layer === "video") {
        return {
          ...f,
          frames: f.frames.map((fr) => ({ ...fr, url: draw(fr.url) })),
          ...(f.cover ? { cover: { ...f.cover, url: draw(f.cover.url) } } : {}),
        };
      }
      if (f.layer === "image") return { ...f, images: f.images.map((im) => ({ ...im, url: draw(im.url) })) };
      return f;
    }),
  };
}

function toItem(raw: RawItem): Item {
  const platform = PLATFORMS[raw.p];
  return {
    id: raw.id,
    title: raw.title,
    platform: { id: platform.id, name: platform.name },
    author: raw.author,
    takenAt: takenAt(raw),
    topicIds: [...raw.topics],
    url: `${platform.host}${raw.p === "bili" ? "BV1" : ""}${raw.id}us${Math.floor(r01(raw.id) * 1e6)}`,
    stored: raw.link ? "link" : "complete",
    ...(raw.link ? { missing: raw.link.missing } : {}),
    ...(raw.note ? { note: raw.note } : {}),
    facets: buildFacets(raw),
  };
}

// 位置照主页画布：一个主题里的条目沿黄金角螺旋排在名字下面，属于两个主题的落在两者之间。
function place(raws: RawItem[]): Map<string, { x: number; y: number }> {
  const byTopic = new Map<string, (typeof TOPICS)[number]>(TOPICS.map((t) => [t.id, t]));
  const counts = new Map<string, number>();
  const out = new Map<string, { x: number; y: number }>();
  for (const raw of raws) {
    const h = r01(`${raw.id}a`);
    const h2 = r01(`${raw.id}b`);
    const a = byTopic.get(raw.topics[0] ?? "");
    const b = byTopic.get(raw.topics[1] ?? "");
    if (!a) continue;
    if (b) {
      const u = 0.42 + 0.16 * h;
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const dl = Math.hypot(dx, dy) || 1;
      out.set(raw.id, {
        x: a.x + dx * u - (dy / dl) * (h2 - 0.5) * 40,
        y: a.y + dy * u + (dx / dl) * (h2 - 0.5) * 40,
      });
      continue;
    }
    const k = (counts.get(a.id) ?? 0) + 1;
    counts.set(a.id, k);
    const rr = a.rad * 0.5 * Math.sqrt((k - 0.5) / 6);
    const an = k * 2.39996 + h;
    out.set(raw.id, { x: a.x + Math.cos(an) * rr * 1.2, y: a.y + 22 + Math.abs(Math.sin(an)) * rr * 0.9 });
  }
  return out;
}

const PLATFORM_RULES: [RegExp, Platform][] = [
  [/xiaohongshu|xhslink/, { id: "xiaohongshu", name: "小红书" }],
  [/channels\.weixin/, { id: "wechat-channels", name: "视频号" }],
  [/bilibili|b23\.tv/, { id: "bilibili", name: "B站" }],
  [/mp\.weixin/, { id: "wechat-mp", name: "公众号" }],
];

function parseShare(input: string): { url: string; title: string; platform: Platform } {
  const url = /https?:\/\/\S+/i.exec(input)?.[0] ?? "";
  const host = url.replace(/^https?:\/\//i, "").split("/")[0]?.replace(/^www\./i, "") ?? "";
  const platform = PLATFORM_RULES.find(([re]) => re.test(host))?.[1] ?? { id: "web", name: url ? "网页" : "一段文字" };
  const before = url ? input.slice(0, input.indexOf(url)) : input;
  let title = before.replace(/[【】]/g, "").replace(/-哔哩哔哩.*$/, "").trim() || host || input.trim();
  if (title.length > 40) title = `${title.slice(0, 40)}…`;
  return { url, title, platform };
}

const LAYER_ORDER = ["asr", "ocr", "text", "comments"] as const;

function searchItem(item: Item, terms: string[]): SearchHit | null {
  const fields: { layer: SearchHit["layer"]; text: string; position?: Position }[] = [];
  for (const layer of LAYER_ORDER) {
    for (const facet of item.facets) {
      if (facet.layer !== layer) continue;
      if (facet.layer === "comments") {
        for (const c of facet.comments) fields.push({ layer: "comments", text: c.text });
      } else if (facet.layer === "asr" || facet.layer === "ocr" || facet.layer === "text") {
        for (const s of facet.segments) fields.push({ layer, text: s.text, ...(s.position ? { position: s.position } : {}) });
      }
    }
  }
  if (item.note) fields.push({ layer: "note", text: item.note });
  fields.push({ layer: "title", text: item.title });
  if (item.author) fields.push({ layer: "author", text: item.author });
  for (const f of fields) {
    const low = f.text.toLowerCase();
    if (!terms.every((t) => low.includes(t))) continue;
    const first = terms[0] ?? "";
    const at = low.indexOf(first);
    return {
      itemId: item.id,
      title: item.title,
      platform: item.platform,
      layer: f.layer,
      ...(f.position ? { position: f.position } : {}),
      snippet: f.text,
      match: [at, at + first.length],
    };
  }
  return null;
}

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export function createMockLibrary(empty: boolean): Library {
  const raws = empty ? [] : RAW_ITEMS;
  const places = place(raws);
  const entries = new Map<string, Entry>();
  for (const raw of raws) {
    const p = places.get(raw.id) ?? { x: 700, y: 450 };
    entries.set(raw.id, { item: toItem(raw), x: p.x, y: p.y });
  }
  const topics: Topic[] = TOPICS.map((t) => ({ id: t.id, name: t.name, followed: t.id === "film" }));
  let landed = 0;

  const get = (id: string): Entry => {
    const entry = entries.get(id);
    if (!entry) throw new Error(`样例库里没有 ${id}`);
    return entry;
  };
  const update = async (id: string, change: (item: Item) => Item): Promise<Item> => {
    await wait(60);
    const entry = get(id);
    entry.item = change(entry.item);
    return pictured(entry.item);
  };

  return {
    health: async () => ({ status: "ok", version: "mock", library: "~/Documents/usuyo（样例库，不在磁盘上）" }),
    async map(): Promise<MapView> {
      await wait(40);
      const live = [...entries.values()].filter((e) => !e.item.trashedAt);
      const regions: MapRegion[] = TOPICS.map((t) => ({
        topicId: t.id,
        x: (t.x - MAP_X) / MAP_W,
        y: t.y / PAGE_H,
        r: t.rad / MAP_W,
        count: live.filter((e) => e.item.topicIds.includes(t.id)).length,
      })).filter((r) => r.count > 0);
      const points: MapPoint[] = live.map(({ item, x, y }) => ({
        itemId: item.id,
        x: (x - MAP_X) / MAP_W,
        y: y / PAGE_H,
        title: item.title,
        platform: item.platform,
        topicIds: item.topicIds,
        takenAt: item.takenAt,
      }));
      return { regions, points };
    },
    topics: async () => topics.map((t) => ({ ...t })),
    async createTopic(name) {
      await wait(40);
      const trimmed = name.trim();
      if (trimmed === "") throw new Error("topic name is empty");
      const found = topics.find((t) => t.name === trimmed);
      if (found) return { ...found };
      const topic: Topic = { id: `t${topics.length + 1}-${Math.floor(Math.random() * 1e6).toString(36)}`, name: trimmed, followed: false };
      topics.push(topic);
      return { ...topic };
    },
    async follow(topicId, followed) {
      await wait(40);
      const topic = topics.find((t) => t.id === topicId);
      if (!topic) throw new Error(`样例库里没有主题 ${topicId}`);
      topic.followed = followed;
      return { ...topic };
    },
    async item(id) {
      await wait(40);
      return pictured(get(id).item);
    },
    async search(q) {
      await wait(30);
      const terms = q.toLowerCase().split(/\s+/).filter(Boolean);
      if (terms.length === 0) return [];
      return [...entries.values()]
        .filter((e) => !e.item.trashedAt)
        .map((e) => searchItem(e.item, terms))
        .filter((h): h is SearchHit => h !== null);
    },
    async take(body: TakeRequest): Promise<TakeResponse> {
      await wait(120);
      const share =
        body.kind === "file"
          ? { url: "", title: body.input.split(/[\\/]/).pop() ?? body.input, platform: { id: "file", name: "文件" } }
          : parseShare(body.input);
      const seen = [...entries.values()].find(
        (e) => (share.url !== "" && e.item.url === share.url) || body.input.includes(e.item.title),
      );
      if (seen) {
        return { id: seen.item.id, seen: true, title: seen.item.title, platform: seen.item.platform, takenAt: seen.item.takenAt };
      }
      // 还没取到文字，没法按内容摆，先排在地图下沿（照主页画布）。
      const n = landed++;
      const id = `n${n}-${Math.floor(Math.random() * 1e6).toString(36)}`;
      const now = new Date().toISOString();
      const item: Item = {
        id,
        title: share.title,
        platform: share.platform,
        takenAt: now,
        topicIds: [],
        url: share.url,
        stored: "link",
        facets: [],
      };
      entries.set(id, { item, x: 330 + (n % 9) * 46 + (r01(id) - 0.5) * 14, y: 846 - Math.floor(n / 9) * 30 });
      return { id, seen: false, title: item.title, platform: item.platform, takenAt: now };
    },
    setNote: (id, text) =>
      update(id, (item) => {
        const { note: _old, ...rest } = item;
        return text.trim() ? { ...rest, note: text.trim() } : rest;
      }),
    setTopic: (id, topicId, member) =>
      update(id, (item) => ({
        ...item,
        topicIds: member ? [...new Set([...item.topicIds, topicId])] : item.topicIds.filter((t) => t !== topicId),
      })),
    trash: (id) => update(id, (item) => ({ ...item, trashedAt: new Date().toISOString() })),
    restore: (id) =>
      update(id, (item) => {
        const { trashedAt: _t, ...rest } = item;
        return rest;
      }),
    restoreMedia: (id) => update(id, (item) => ({ ...item, stored: "complete" })),
    textOnly: (id) => update(id, (item) => ({ ...item, stored: "text-only" })),
    // 和内核一样：只是排上再取一次，存着的不动。
    refetch: (id) => update(id, (item) => item),
  };
}
