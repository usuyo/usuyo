// 样例：跟着收。和内核同一套接口，数据在内存里。
// 能跟着的取法照 docs/platforms/routes.md：B站的收藏夹和稍后再看能列；按创作者列的（RSSHub）还没做；关键词搜索没有不签名的路子，不列。
// 订阅源用官方预装的通用网页获取读。最近收下的指向样例库里真有的条目。
import type { FollowOption, FollowsView, FollowTaken, FollowTrial, FollowView } from "../api/endpoints";
import type { Follows, Library } from "../api/source";
import { said } from "./said";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const HOUR = 3_600_000;
const at = (hours: number) => new Date(Date.now() + hours * HOUR).toISOString();

const OPTIONS: FollowOption[] = [
  {
    kind: "list",
    pluginId: "usuyo-fetch/bilibili",
    pluginName: "B站",
    methodName: "收藏夹和稍后再看",
    built: true,
    sources: [
      { source: "bilibili:toview", title: "稍后再看", method: "favorites" },
      { source: "bilibili:fav:1", title: "默认收藏夹", method: "favorites" },
      { source: "bilibili:fav:2", title: "胶片", method: "favorites" },
    ],
    minHours: 6,
  },
  { kind: "creator", pluginId: "usuyo-fetch/xiaohongshu", pluginName: "小红书", methodName: "RSSHub", built: false, minHours: 12 },
  { kind: "feed", pluginId: "usuyo/web", pluginName: "网页", methodName: "订阅源", built: true, minHours: 6 },
];

/** 订阅源试列时读到的几篇：有一篇说到了胶片，按关注的主题会收。 */
const FEED_POSTS = [
  { title: "这个月拍完的三卷", words: ["胶片"] },
  { title: "换了一台新的扫描仪", words: [] },
  { title: "周末在城里走了一圈", words: [] },
  { title: "胶片相机的快门为什么会慢", words: ["胶片"] },
  { title: "读完了一本小说", words: [] },
];

export function createMockFollows(library: Library, empty: boolean): Follows {
  const follows: FollowView[] = empty
    ? []
    : [
        {
          id: "f-toview",
          kind: "list",
          title: "稍后再看",
          pluginId: "usuyo-fetch/bilibili",
          pluginName: "B站",
          source: "bilibili:toview",
          take: { all: true },
          everyHours: 6,
          on: true,
          lastAt: at(-2),
          nextAt: at(4),
        },
        {
          id: "f-blog",
          kind: "feed",
          title: "每周胶片简报",
          pluginId: "usuyo/web",
          pluginName: "网页",
          url: "https://example.org/film-weekly/feed.xml",
          take: { all: false, topics: ["film"], words: ["C41"] },
          everyHours: 24,
          on: true,
          lastAt: at(-9),
          nextAt: at(15),
        },
        {
          id: "f-film",
          kind: "list",
          title: "胶片",
          pluginId: "usuyo-fetch/bilibili",
          pluginName: "B站",
          source: "bilibili:fav:2",
          take: { all: true },
          everyHours: 12,
          on: true,
          lastAt: at(-1),
          waiting: {
            code: "rate-limited",
            get hint() {
              return said("平台提示请求过于频繁", "The platform says there are too many requests");
            },
            until: at(5),
          },
        },
        {
          id: "f-old",
          kind: "list",
          title: "默认收藏夹",
          pluginId: "usuyo-fetch/bilibili",
          pluginName: "B站",
          source: "bilibili:fav:1",
          take: { all: true },
          everyHours: 24,
          on: false,
          lastAt: at(-80),
        },
      ];
  let powerOnly = true;
  let seq = 0;

  /** 最近收下的：c5、c1 从稍后再看来，f6、f3 是订阅源里按「胶片摄影」和另加的「C41」命中的。 */
  async function recent(): Promise<FollowTaken[]> {
    if (empty) return [];
    const rows: { id: string; followId: string; hours: number; words?: string[] }[] = [
      { id: "c5", followId: "f-toview", hours: -2 },
      { id: "f6", followId: "f-blog", hours: -9, words: ["胶片"] },
      { id: "c1", followId: "f-toview", hours: -26 },
      { id: "f3", followId: "f-blog", hours: -33, words: ["C41"] },
    ];
    return Promise.all(
      rows.map(async (r) => {
        const item = await library.item(r.id);
        return {
          itemId: item.id,
          title: item.title,
          platform: item.platform,
          takenAt: at(r.hours),
          followId: r.followId,
          ...(r.words ? { picked: { words: r.words, topicIds: r.words.includes("胶片") ? ["film"] : [] } } : {}),
        };
      }),
    );
  }

  async function view(): Promise<FollowsView> {
    return { follows: follows.map((f) => ({ ...f })), recent: await recent(), options: empty ? [] : OPTIONS, powerOnly };
  }

  function get(id: string): FollowView {
    const f = follows.find((x) => x.id === id);
    if (!f) throw new Error("no such follow");
    return f;
  }

  return {
    async follows() {
      await wait(50);
      return view();
    },
    async trialFollow(body): Promise<FollowTrial> {
      await wait(900);
      const option = OPTIONS.find((o) => o.kind === body.kind && o.pluginId === body.pluginId);
      if (!option?.built) throw new Error("this plugin cannot list this way yet");
      if (body.kind === "list") {
        const title = option.sources?.find((s) => s.source === body.source)?.title ?? body.source ?? "";
        const would = ["一周备餐的思路", "城市骑行路线整理"].map((t, i) => ({ title: t, url: `https://www.bilibili.com/video/BV1n${i}xx411c7mD`, actedAt: at(-3 - i) }));
        return { title, firstRun: "take", listed: 24, would, left: { stored: 21, trashed: 1, unmatched: 0 } };
      }
      const words = body.take.all ? [] : [...body.take.words, ...(body.take.topics.includes("film") ? ["胶片"] : [])];
      const would = FEED_POSTS.filter((p) => body.take.all || p.words.some((w) => words.includes(w))).map((p, i) => ({
        title: p.title,
        url: `${body.url ?? "https://example.org"}#${i}`,
        actedAt: at(-24 * (i + 1)),
        ...(body.take.all ? {} : { picked: p.words.filter((w) => words.includes(w)) }),
      }));
      return { title: "每周胶片简报", firstRun: "record", listed: FEED_POSTS.length, would, left: { stored: 0, trashed: 0, unmatched: FEED_POSTS.length - would.length } };
    },
    async newFollow(body) {
      await wait(80);
      const option = OPTIONS.find((o) => o.kind === body.kind && o.pluginId === body.pluginId);
      if (!option) throw new Error("no such plugin");
      const f: FollowView = {
        id: `f-new${++seq}`,
        kind: body.kind,
        title: body.title ?? option.sources?.find((s) => s.source === body.source)?.title ?? body.url ?? "关键词",
        pluginId: body.pluginId,
        pluginName: option.pluginName,
        ...(body.source ? { source: body.source } : {}),
        ...(body.url ? { url: body.url } : {}),
        take: body.take,
        everyHours: Math.max(body.everyHours, option.minHours),
        on: true,
        nextAt: at(Math.max(body.everyHours, option.minHours)),
      };
      follows.unshift(f);
      return { ...f };
    },
    async changeFollow(id, patch) {
      await wait(50);
      const f = get(id);
      if (patch.on !== undefined) f.on = patch.on;
      if (patch.everyHours !== undefined) f.everyHours = patch.everyHours;
      if (patch.take) f.take = patch.take;
      if (f.on) f.nextAt = at(f.everyHours);
      else delete f.nextAt;
      return { ...f };
    },
    async removeFollow(id) {
      await wait(50);
      const i = follows.findIndex((f) => f.id === id);
      if (i >= 0) follows.splice(i, 1);
    },
    async setFollowPower(on) {
      await wait(40);
      powerOnly = on;
      return view();
    },
  };
}
