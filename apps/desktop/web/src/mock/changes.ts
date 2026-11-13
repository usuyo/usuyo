// 样例：改动记录与改动提议。和内核同一套接口，数据在内存里。标题和原话都是样例库里有的。
import type { ChangeBatch, ChangeOp } from "../api/endpoints";
import type { Changes } from "../api/source";
import { said } from "./said";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
/** 今天几点几分；`days` 往前推几天。 */
function at(days: number, h: number, m: number): string {
  const d = new Date();
  d.setDate(d.getDate() - days);
  d.setHours(h, m, 0, 0);
  return d.toISOString();
}

const MAC = "MacBook Air";

function seed(): ChangeBatch[] {
  return [
    {
      id: "b-draft-topic",
      at: at(0, 14, 20),
      by: { kind: "agent", name: "Claude Code", device: MAC },
      state: "proposed",
      get via() {
        return said("经 MCP，在终端中要求补充「咖啡」主题页", "Through MCP, asked in Terminal to fill in the Coffee page");
      },
      ops: [
        {
          id: "o1",
          target: { kind: "topic", id: "coffee", title: "咖啡" },
          op: "edit-page",
          before: "比例我现在固定 1:15，不再改它；要调就只动水温和研磨，一次只动一样。",
          after: "比例我现在固定 1:15，不再改它；要调就只动水温和研磨，一次只动一样。研磨先动，水温后动。",
          decision: "open",
        },
        {
          id: "o2",
          target: { kind: "topic", id: "coffee", title: "咖啡" },
          op: "add-citation",
          after: "「先看刀盘，锥刀比平刀更适合手冲入门」　磨豆机怎么选：三百块以内　口播 00:26",
          decision: "open",
        },
        {
          id: "o3",
          target: { kind: "item", id: "c4", title: "磨豆机怎么选：三百块以内" },
          op: "set-note",
          before: "",
          after: "入门先买手摇，有刻度的",
          decision: "open",
        },
      ],
    },
    {
      id: "b-note",
      at: at(0, 11, 2),
      by: { kind: "you", device: MAC },
      state: "applied",
      get via() {
        return said("在条目页中", "On the item page");
      },
      ops: [{ id: "o4", target: { kind: "item", id: "c1", title: "4:6 冲煮法复刻（一）：比例" }, op: "set-note", before: "92 度", after: "92 度，闷蒸 30 秒" }],
    },
    {
      id: "b-agent-take",
      at: at(0, 9, 40),
      by: { kind: "agent", name: "Codex", device: MAC },
      state: "applied",
      get via() {
        return said("经 MCP 添加", "Added through MCP");
      },
      get undoNote() {
        return said("你后来将其中的「这周的豆子」添加到了主题「咖啡」。撤销会将这 3 个条目都移到废纸篓，30 天内可以恢复。", "You later added “这周的豆子” to the topic “咖啡”. Undoing moves all 3 items to the Trash, where they can be restored for 30 days.");
      },
      ops: [
        { id: "o5", target: { kind: "item", id: "c7", title: "这周的豆子" }, op: "add-item" },
        { id: "o6", target: { kind: "item", id: "w2", title: "梧桐区的门牌号怎么看" }, op: "add-item" },
        { id: "o7", target: { kind: "item", id: "k4", title: "一周备餐：周日两小时" }, op: "add-item" },
      ],
    },
    {
      id: "b-readback",
      at: at(1, 22, 15),
      by: { kind: "readback", target: "Obsidian", device: MAC },
      state: "applied",
      get via() {
        return said("导出前从 Obsidian 读回", "Read back from Obsidian before an export");
      },
      ops: [
        {
          id: "o8",
          target: { kind: "draft", id: "d-pour", title: "手冲只动三个数" },
          op: "edit-page",
          before: "比例定下来就不再动。",
          after: "比例定下来就不再动。我用的是 1:15，冰的另算。",
        },
      ],
    },
    {
      id: "b-partly",
      at: at(1, 18, 3),
      by: { kind: "agent", name: "Claude Code", device: MAC },
      state: "applied",
      get via() {
        return said("经 MCP", "Through MCP");
      },
      ops: [
        { id: "o9", target: { kind: "item", id: "f2", title: "Portra 400 过曝一档的肤色" }, op: "add-to-topic", after: "胶片摄影", decision: "accepted" },
        { id: "o10", target: { kind: "item", id: "f2", title: "Portra 400 过曝一档的肤色" }, op: "set-note", before: "", after: "过曝一档肤色最好", decision: "declined" },
      ],
    },
    {
      id: "b-keep-text",
      at: at(3, 20, 30),
      by: { kind: "you", device: MAC },
      state: "undone",
      get via() {
        return said("在条目页中", "On the item page");
      },
      ops: [{ id: "o11", target: { kind: "item", id: "w3", title: "苏州河沿岸 12 公里，拍完一卷" }, op: "keep-text" }],
    },
    {
      id: "b-sync",
      at: at(4, 8, 12),
      by: { kind: "sync", device: "家里的 iMac" },
      state: "applied",
      get via() {
        return said("同步", "Sync");
      },
      ops: [{ id: "o12", target: { kind: "item", id: "r1", title: "读《置身事内》的笔记" }, op: "add-to-topic", after: "读书" }],
    },
  ];
}

/** 定完一处以后：提议里还有没定的就还是提议；都定了，接受过的生效，一处也没接受就是都不要了。 */
function settle(b: ChangeBatch): ChangeBatch {
  if (b.ops.some((o) => o.decision === "open")) return b;
  return { ...b, state: b.ops.some((o) => o.decision === "accepted") ? "applied" : "declined" };
}

export function createMockChanges(empty: boolean): Changes {
  let batches = empty ? [] : seed();
  const update = (id: string, change: (b: ChangeBatch) => ChangeBatch): ChangeBatch => {
    const b = batches.find((x) => x.id === id);
    if (!b) throw new Error("no such batch");
    const next = change(b);
    batches = batches.map((x) => (x.id === id ? next : x));
    return next;
  };
  const decide = (decision: "accepted" | "declined") => async (id: string, ops?: string[]) => {
    await wait(80);
    return update(id, (b) => {
      if (b.state !== "proposed") throw new Error("this batch is not a proposal");
      const pick = (o: ChangeOp) => o.decision === "open" && (ops === undefined || ops.includes(o.id));
      return settle({ ...b, ops: b.ops.map((o) => (pick(o) ? { ...o, decision } : o)) });
    });
  };
  return {
    async changes() {
      await wait(50);
      return { batches: batches.map((b) => ({ ...b, ops: b.ops.map((o) => ({ ...o })) })) };
    },
    acceptChange: decide("accepted"),
    declineChange: decide("declined"),
    async undoChange(id) {
      await wait(80);
      return update(id, (b) => ({ ...b, state: "undone" }));
    },
    async redoChange(id) {
      await wait(80);
      return update(id, (b) => ({ ...b, state: "applied" }));
    },
  };
}
