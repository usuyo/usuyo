// 样例：导出到 Obsidian。和内核同一套接口，数据在内存里。上一次导出读回了两处、有一处两边都改过、有一篇读不了。
import type { ExportRun, ExportView } from "../api/endpoints";
import type { Exports } from "../api/source";
import { said } from "./said";

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const ago = (hours: number) => new Date(Date.now() - hours * 3_600_000).toISOString();

const FIRST: ExportRun = {
  id: "e1",
  at: ago(52),
  state: "done",
  written: 46,
  readBack: [
    { file: "条目/4:6 冲煮法复刻（一）：比例.md", title: "4:6 冲煮法复刻（一）：比例", get what() {
        return said("备注", "Note");
      },
      batchId: "b-readback",
    },
    { file: "成稿/手冲只动三个数.md", title: "手冲只动三个数", get what() {
        return said("正文中的一段", "A paragraph of the text");
      },
      batchId: "b-readback",
    },
  ],
  both: [
    {
      file: "主题/咖啡.md",
      title: "咖啡",
      kept: "冰手冲是另一套数：热水少一半，冰放在下面接着，研磨细一档。",
      theirs: "冰手冲：热水 150，冰 100，研磨细一档，冰块用纯净水冻。",
    },
  ],
  left: [
    {
      file: "条目/胶片摄影师的书单.md",
      get reason() {
        return said("开头的属性格式有误，无法读取", "The properties at the top are malformed and can't be read");
      },
    },
  ],
};

const OLDER: ExportRun = { id: "e0", at: ago(24 * 9), state: "done", written: 44, readBack: [], both: [], left: [] };

export function createMockExports(empty: boolean): Exports {
  let folder: string | null = empty ? null : "~/Documents/Obsidian/笔记库/usuyo";
  const runs: ExportRun[] = empty ? [] : [FIRST, OLDER];
  const view = (): ExportView => ({
    target: { pluginId: "usuyo/obsidian", name: "Obsidian" },
    folder,
    will: empty ? { items: 0, topics: 0, drafts: 0, privateLeft: 0 } : { items: 39, topics: 7, drafts: 1, privateLeft: 1 },
    sample: {
      file: "条目/4:6 冲煮法复刻（一）：比例.md",
      yours: "> 92 度，闷蒸 30 秒\n\n第一段少倒真的会甜一点，明早再试一次 88 度。\n\n#咖啡 #手冲",
      ours: "B站 · 豆研所 · 10 月 21 日收下\n原帖：https://www.bilibili.com/video/…\n\n## 口播\n00:00 大家好，这里是豆研所\n00:12 这期复刻卡斯哲的四六手冲，先说比例\n……",
    },
    runs: runs.map((r) => ({ ...r })),
  });
  return {
    async exportView() {
      await wait(50);
      return view();
    },
    async setExportFolder(next) {
      await wait(60);
      if (next.trim() === "") throw new Error("invalid-args: folder");
      folder = next.trim();
      return view();
    },
    async exportNow() {
      await wait(1200);
      if (folder === null) throw new Error("no export folder chosen");
      const run: ExportRun = { id: `e${runs.length + 1}`, at: new Date().toISOString(), state: "done", written: 48, readBack: [], both: [], left: [] };
      runs.unshift(run);
      return { ...run };
    },
  };
}
