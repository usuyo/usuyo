// 样例：你写的那一层（主题页、成稿）。和内核同一套接口，数据在内存里。
// 引用、线头、角度里的每一段都从样例库真有的条目里按「哪一条、哪一层、哪句话」找出来，找不到就报错，不编原话。
import type {
  Angle,
  CiteRequest,
  Citation,
  Draft,
  FacetLayer,
  Item,
  ItemSummary,
  PageBlock,
  SegmentRef,
  Thread,
  TopicMatch,
  TopicPage,
} from "../api/endpoints";
import type { Drafts, Library, Topics } from "../api/source";
import { TOPICS } from "./data";
import { said } from "./said";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** 一段原话的位置：哪一条、哪一层、这句话（开头几个字也行）。 */
type At = [itemId: string, layer: FacetLayer, starts: string];

/** 引用：原话照找到的抄；`changed` 的原话是当时抄下的，`now` 由现在那一层给。 */
interface CiteSeed {
  label: string;
  at: At;
  /** 当时抄下的原话和它出自哪一份（平台的字幕），现在那一层（机器转的）写法变了。 */
  then?: string;
}

/** 主题「咖啡」的页：你写的几段，带着引用；一处引用的条目已经清掉了。 */
const COFFEE_BLOCKS: (Omit<PageBlock, "id"> & { id: string })[] = [
  { id: "b1", kind: "text", text: "同一包豆子，冲出来酸还是甜，先看前面那几十克水怎么倒，再看水温。", cites: ["k3f7q", "m2x8d"] },
  { id: "b2", kind: "text", text: "比例我现在固定 1:15，不再改它；要调就只动水温和研磨，一次只动一样。", cites: ["p9w4z"] },
  { id: "b3", kind: "heading", text: "冰的", cites: [] },
  { id: "b4", kind: "text", text: "冰手冲是另一套数：热水少一半，冰放在下面接着，研磨细一档。", cites: ["r5t1n"] },
  { id: "b5", kind: "text", text: "觉得太酸的时候，先降水温，不急着换豆子。", cites: ["h8q2c", "z0z0z"] },
];

const COFFEE_CITES: CiteSeed[] = [
  { label: "k3f7q", at: ["c1", "asr", "前 40% 的水决定酸甜"] },
  { label: "m2x8d", at: ["c5", "asr", "92 度最平衡"] },
  // 当时引的是平台字幕；现在最前的是机器转的那一份，同一秒写成了「1 比 15」。
  { label: "p9w4z", at: ["c1", "asr", "粉 20 克，水 300 克，就是 1 比 15"], then: "粉 20 克，水 300 克，就是 1:15" },
  { label: "r5t1n", at: ["c2", "ocr", "15g 粉 / 150ml 热水 / 100g 冰"] },
  { label: "h8q2c", at: ["c3", "text", "如果觉得太酸"] },
];

/** 已经从资料库清掉的那条：原话、标题照留，链接换成「已删掉」。 */
const GONE_CITE: Citation = {
  label: "z0z0z",
  itemId: null,
  layer: "asr",
  quote: "深烘的豆子我一般 86 度，浅烘再往上加",
  title: "深烘和浅烘怎么调水温",
  author: "拿铁三分糖",
  date: "2026-08-14",
  found: "gone",
};

const COFFEE_THREADS: { id: string; at: At[] }[] = [
  {
    id: "t-temp",
    at: [
      ["c1", "asr", "水温控制在 92 度左右"],
      ["c5", "asr", "88 度这一杯"],
      ["c5", "asr", "96 度苦味开始出来"],
      ["c1", "frames", "水温 90–96°C"],
      ["c5", "comments", "深烘我一般 86 度"],
    ],
  },
  {
    id: "t-grind",
    at: [
      ["c1", "asr", "磨粗一点"],
      ["c2", "ocr", "研磨比热冲细一档"],
      ["c4", "asr", "先看刀盘"],
      ["c4", "asr", "细粉多不多"],
    ],
  },
];

const COFFEE_ANGLES: { id: string; question: string; runs: { id: string; at: string; model: string; points: { text: string; cites: { at: At; checked: boolean }[] }[] }[] }[] = [
  {
    id: "a-sour",
    question: "同样的豆子，什么让它冲出来更酸？",
    runs: [
      {
        id: "r-sour-2",
        at: "2026-11-08T13:20:00.000Z",
        model: said("Qwen3 8B · 本机", "Qwen3 8B · Local"),
        points: [
          { text: "注水的前一部分偏向酸甜，前段倒得少会更甜。", cites: [{ at: ["c1", "asr", "前 40% 的水决定酸甜"], checked: true }, { at: ["c1", "asr", "想甜一点"], checked: true }] },
          { text: "水温低时酸更明显，几位都在 88 度附近看到这一点。", cites: [{ at: ["c5", "asr", "88 度这一杯"], checked: true }, { at: ["c3", "text", "如果觉得太酸"], checked: true }] },
          { text: "浅烘的豆子本来就更酸。", cites: [{ at: ["c3", "text", "浅烘的时候酸会比较明显"], checked: false }] },
        ],
      },
      {
        id: "r-sour-1",
        at: "2026-11-02T09:05:00.000Z",
        model: said("Qwen3 8B · 本机", "Qwen3 8B · Local"),
        points: [{ text: "水温和注水比例都会改变酸感。", cites: [{ at: ["c5", "asr", "88 度这一杯"], checked: true }] }],
      },
    ],
  },
  {
    id: "a-gear",
    question: "器具上先在哪儿花钱？",
    runs: [
      {
        id: "r-gear-1",
        at: "2026-11-08T13:24:00.000Z",
        model: said("Qwen3 8B · 本机", "Qwen3 8B · Local"),
        points: [
          { text: "先升级磨豆机，不先换壶。", cites: [{ at: ["c4", "asr", "预算多一点"], checked: true }] },
          { text: "入门手冲选锥刀、有刻度的手摇。", cites: [{ at: ["c4", "asr", "先看刀盘"], checked: true }, { at: ["c4", "asr", "再看调节"], checked: true }, { at: ["c4", "asr", "三百块以内"], checked: true }] },
        ],
      },
    ],
  },
];

const COFFEE_MATCH: TopicMatch = { keywords: ["手冲", "水温", "研磨"], similarity: false };

/** 一篇成稿：从「咖啡」那一页和别的段里引过来，拼成一条要发的帖子。 */
const DRAFT_SEED = {
  id: "d-pour",
  title: "手冲只动三个数",
  kind: "post" as const,
  createdAt: "2026-11-09T12:00:00.000Z",
  updatedAt: "2026-11-11T14:30:00.000Z",
  blocks: [
    { id: "d1", kind: "text" as const, text: "这半年冲咖啡，最后只剩下三个数要管：比例、水温、研磨。别的都是这三个数换了个说法。", cites: [] },
    { id: "d2", kind: "heading" as const, text: "比例", cites: [] },
    { id: "d3", kind: "text" as const, text: "比例定下来就不再动。我用的是 1:15，冰的另算。", cites: ["dq1", "dq2"] },
    { id: "d4", kind: "heading" as const, text: "水温", cites: [] },
    { id: "d5", kind: "text" as const, text: "酸了降温，苦了也降温；浅烘往上走，深烘往下走。", cites: ["dq3"] },
    { id: "d6", kind: "heading" as const, text: "研磨", cites: [] },
    { id: "d7", kind: "text" as const, text: "", cites: ["dq4"] },
  ],
  cites: [
    { label: "dq1", at: ["c1", "asr", "粉 20 克，水 300 克"] as At },
    { label: "dq2", at: ["c2", "ocr", "热水 150 + 冰 100"] as At },
    { label: "dq3", at: ["c5", "asr", "浅烘可以往高走"] as At },
    { label: "dq4", at: ["c4", "asr", "预算多一点"] as At },
  ],
};

/** 一条里某一层的某句话，连同位置。 */
function segmentIn(item: Item, [, layer, starts]: At): SegmentRef {
  for (const f of item.facets) {
    if (f.layer !== layer) continue;
    if (f.layer === "comments") {
      const c = f.comments.find((x) => x.text.startsWith(starts));
      if (c) return { ...refOf(item), layer, text: c.text };
      continue;
    }
    if (f.layer === "image" || f.layer === "video") continue;
    const s = f.segments.find((x) => x.text.startsWith(starts));
    if (s) return { ...refOf(item), layer, text: s.text, ...(s.position ? { position: s.position } : {}) };
  }
  throw new Error(`样例库里没有这句：${item.id} ${layer} ${starts}`);
}

function refOf(item: Item): Omit<SegmentRef, "layer" | "text"> {
  return { itemId: item.id, title: item.title, platform: item.platform, ...(item.author ? { author: item.author } : {}) };
}

const summaryOf = (i: Item): ItemSummary => ({
  id: i.id,
  title: i.title,
  platform: i.platform,
  ...(i.author ? { author: i.author } : {}),
  takenAt: i.takenAt,
  topicIds: i.topicIds,
});

/** 一条里看得到的字（标题层和内容层），按关键词命中用。 */
function wordsOf(item: Item): string {
  const parts = [item.title];
  for (const f of item.facets) {
    if (f.layer === "comments" || f.layer === "image" || f.layer === "video") continue;
    for (const s of f.segments) parts.push(s.text);
  }
  return parts.join("\n");
}

/** 把一段原话引进一页：页尾记一处引用，原话、标题、作者、日期这时抄下；正文里接一段只放这处引用的段。 */
function insertCite(page: { blocks: PageBlock[]; citations: Citation[] }, body: CiteRequest, it: Item, seq: number): void {
  const label = `n${String(seq).padStart(4, "0")}`;
  page.citations.push({
    label,
    itemId: it.id,
    layer: body.segment.layer,
    ...(body.segment.position ? { position: body.segment.position } : {}),
    quote: body.segment.text,
    title: it.title,
    ...(it.author ? { author: it.author } : {}),
    date: it.takenAt.slice(0, 10),
    found: "same",
  });
  const block: PageBlock = { id: `b${seq}`, kind: "text", text: "", cites: [label] };
  const at = body.after === undefined ? -1 : page.blocks.findIndex((b) => b.id === body.after);
  if (at < 0) page.blocks.push(block);
  else page.blocks.splice(at + 1, 0, block);
}

interface PageState {
  blocks: PageBlock[];
  citations: Citation[];
  match: TopicMatch;
  threads: { id: string; at: At[] }[];
  angles: Angle[];
  /** 样例里起新段、新引用的号。 */
  seq: number;
}

export function createMockPages(library: Library, empty: boolean): Topics & Drafts {
  const pages = new Map<string, PageState>();
  const drafts = new Map<string, { draft: Omit<Draft, "page" | "from">; blocks: PageBlock[]; citations: Citation[]; from: Draft["from"] }>();
  let draftSeq = 0;
  let seeded = empty;
  // 样例的原话只看标题和各层的字，这些不会变；取一次留着（每次取都要重画图，慢）。
  const items = new Map<string, Promise<Item>>();
  const item = (id: string) => {
    let have = items.get(id);
    if (!have) {
      have = library.item(id);
      items.set(id, have);
    }
    return have;
  };

  async function resolve(seed: CiteSeed): Promise<Citation> {
    const it = await item(seed.at[0]);
    const s = segmentIn(it, seed.at);
    const day = it.takenAt.slice(0, 10);
    return {
      label: seed.label,
      itemId: it.id,
      layer: s.layer,
      ...(s.position ? { position: s.position } : {}),
      quote: seed.then ?? s.text,
      title: it.title,
      ...(it.author ? { author: it.author } : {}),
      date: day,
      found: seed.then ? "changed" : "same",
      ...(seed.then ? { now: s.text } : {}),
    };
  }

  async function state(id: string): Promise<PageState> {
    const have = pages.get(id);
    if (have) return have;
    if (!TOPICS.some((t) => t.id === id) && !(await library.topics()).some((t) => t.id === id)) throw new Error("没有这个主题");
    const rich = id === "coffee" && !empty;
    const fresh: PageState = {
      blocks: rich ? COFFEE_BLOCKS.map((b) => ({ ...b })) : [],
      citations: rich ? [...(await Promise.all(COFFEE_CITES.map(resolve))), GONE_CITE] : [],
      match: rich ? { ...COFFEE_MATCH } : { keywords: [], similarity: false },
      threads: rich ? COFFEE_THREADS : [],
      angles: rich
        ? await Promise.all(
            COFFEE_ANGLES.map(async (a) => ({
              id: a.id,
              question: a.question,
              runs: await Promise.all(
                a.runs.map(async (r) => ({
                  ...r,
                  points: await Promise.all(
                    r.points.map(async (p) => ({
                      text: p.text,
                      cites: await Promise.all(p.cites.map(async (c) => ({ segment: segmentIn(await item(c.at[0]), c.at), checked: c.checked }))),
                    })),
                  ),
                })),
              ),
            })),
          )
        : [],
      seq: 0,
    };
    pages.set(id, fresh);
    return fresh;
  }

  async function view(id: string): Promise<TopicPage> {
    const s = await state(id);
    const topic = (await library.topics()).find((t) => t.id === id);
    if (!topic) throw new Error("没有这个主题");
    // 成员照地图上的点（放进、拿出主题都在那里变）；命中只在关注着时算。
    const points = (await library.map()).points;
    const inTopic = points.filter((p) => p.topicIds.includes(id));
    const members = (await Promise.all(inTopic.map((p) => item(p.itemId)))).map((i) => ({ ...summaryOf(i), topicIds: inTopic.find((p) => p.itemId === i.id)?.topicIds ?? i.topicIds }));
    const others = topic.followed && s.match.keywords.length > 0 ? await Promise.all(points.filter((p) => !p.topicIds.includes(id)).map((p) => item(p.itemId))) : [];
    const hits = others.flatMap((i) => {
      const text = wordsOf(i);
      const words = s.match.keywords.filter((w) => text.includes(w));
      return words.length > 0 ? [{ item: summaryOf(i), words }] : [];
    });
    const threads: Thread[] = await Promise.all(
      s.threads.map(async (t) => ({ id: t.id, segments: await Promise.all(t.at.map(async (a) => segmentIn(await item(a[0]), a))) })),
    );
    return {
      ...topic,
      page: { file: `topics/${topic.name}.md`, blocks: s.blocks.map((b) => ({ ...b, cites: [...b.cites] })), citations: [...s.citations] },
      match: { ...s.match, keywords: [...s.match.keywords] },
      members,
      hits,
      threads,
      angles: s.angles,
    };
  }

  async function seedDrafts(): Promise<void> {
    if (seeded) return;
    seeded = true;
    const { blocks, cites, ...draft } = DRAFT_SEED;
    drafts.set(draft.id, { draft, blocks: blocks.map((b) => ({ ...b, cites: [...b.cites] })), citations: await Promise.all(cites.map(resolve)), from: [{ id: "coffee", name: "咖啡" }] });
  }

  function getDraft(id: string) {
    const d = drafts.get(id);
    if (!d) throw new Error("没有这篇");
    return d;
  }

  function draftView(id: string): Draft {
    const d = getDraft(id);
    return { ...d.draft, page: { file: `drafts/${d.draft.title}.md`, blocks: d.blocks.map((b) => ({ ...b, cites: [...b.cites] })), citations: [...d.citations] }, from: [...d.from] };
  }

  return {
    async topicPage(id) {
      await wait(60);
      return view(id);
    },
    async editTopicPage(id, patch) {
      await wait(60);
      const s = await state(id);
      s.blocks = patch.blocks.map((b) => ({ ...b, id: b.id ?? `n${++s.seq}` }));
      const used = new Set(s.blocks.flatMap((b) => b.cites));
      s.citations = s.citations.filter((c) => used.has(c.label));
      return view(id);
    },
    async citeInTopic(id, body) {
      await wait(60);
      const s = await state(id);
      s.seq += 2;
      insertCite(s, body, await item(body.segment.itemId), s.seq);
      return view(id);
    },
    async setTopicMatch(id, match) {
      await wait(60);
      const s = await state(id);
      s.match = { keywords: [...new Set(match.keywords.map((w) => w.trim()).filter((w) => w !== ""))], similarity: match.similarity };
      return view(id);
    },
    async dismissThread(id, threadId) {
      await wait(40);
      const s = await state(id);
      s.threads = s.threads.filter((t) => t.id !== threadId);
    },
    async addAngle(id, question) {
      await wait(60);
      const s = await state(id);
      s.angles = [...s.angles, { id: `a${++s.seq}`, question: question.trim(), runs: [] }];
      return view(id);
    },
    async runAngle(id, angleId) {
      await wait(900);
      const s = await state(id);
      const angle = s.angles.find((a) => a.id === angleId);
      if (!angle) throw new Error("没有这个角度");
      // 样例里没有模型：照第一个线头的原话写一份「读过了」，点和引用都是线头里真有的段。
      const first = s.threads[0];
      const segments = first ? await Promise.all(first.at.slice(0, 2).map(async (a) => segmentIn(await item(a[0]), a))) : [];
      const run = {
        id: `r${++s.seq}`,
        at: new Date().toISOString(),
        model: said("Qwen3 8B · 本机", "Qwen3 8B · Local"),
        points: segments.map((seg) => ({ text: `「${seg.text}」和这个问题有关。`, cites: [{ segment: seg, checked: true }] })),
      };
      s.angles = s.angles.map((a) => (a.id === angleId ? { ...a, runs: [run, ...a.runs] } : a));
      return view(id);
    },
    async drafts() {
      await wait(40);
      await seedDrafts();
      return [...drafts.values()].map((d) => ({ ...d.draft })).sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
    },
    async newDraft(body) {
      await wait(60);
      await seedDrafts();
      const now = new Date().toISOString();
      const id = `d${++draftSeq}`;
      drafts.set(id, { draft: { id, title: body.title.trim() || said("未命名文稿", "Untitled Draft"), kind: body.kind, createdAt: now, updatedAt: now }, blocks: [], citations: [], from: [] });
      return draftView(id);
    },
    async draft(id) {
      await wait(40);
      await seedDrafts();
      return draftView(id);
    },
    async changeDraft(id, patch) {
      await wait(40);
      const d = getDraft(id);
      d.draft = { ...d.draft, ...(patch.title !== undefined ? { title: patch.title.trim() || d.draft.title } : {}), ...(patch.kind ? { kind: patch.kind } : {}), updatedAt: new Date().toISOString() };
      return draftView(id);
    },
    async editDraftPage(id, patch) {
      await wait(60);
      const d = getDraft(id);
      d.blocks = patch.blocks.map((b) => ({ ...b, id: b.id ?? `dn${++draftSeq}` }));
      const used = new Set(d.blocks.flatMap((b) => b.cites));
      d.citations = d.citations.filter((c) => used.has(c.label));
      d.draft = { ...d.draft, updatedAt: new Date().toISOString() };
      return draftView(id);
    },
    async citeInDraft(id, body) {
      await wait(60);
      const d = getDraft(id);
      insertCite(d, body, await item(body.segment.itemId), (draftSeq += 1) + 500);
      d.draft = { ...d.draft, updatedAt: new Date().toISOString() };
      return draftView(id);
    },
  };
}
