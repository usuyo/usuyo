// 样例库的转换插件和模型：和内核同一套接口（feature 003 §8.1），转换在内存里「跑」一两秒就出结果。
// 两个是 usuyo 自带的（builtin/），两个是装上的别人的插件，好在条目页上「换一个转」。
import type { Facet, Item, Pending, Segment, TransformLayer, TransformPlugin, TransformRequest, TransformsView } from "../api/endpoints";
import type { Transforms } from "../api/source";

const MB = 1_000_000;

function basePlugins(): TransformPlugin[] {
  return [
    {
      id: "builtin/media",
      name: "视频和声音",
      version: "0.1.0",
      tool: "ffmpeg",
      turns: "视频 → 能播的视频、关键帧；声音 → 给转口播用的音频",
      jobs: ["play", "keyframes"],
      bundled: [{ name: "ffmpeg 9.0.2", license: "LGPL-2.1" }],
      models: [],
      health: "ok",
    },
    {
      id: "builtin/asr-sherpa",
      name: "口播转文字",
      version: "0.1.0",
      tool: "SenseVoice",
      turns: "声音 → 口播文字，带每一句的时间",
      jobs: ["asr"],
      models: [
        {
          name: "sense-voice-int8-2024-07-17",
          label: "SenseVoice",
          size: 163_002_883,
          license: "FunASR Model Open Source License（可商用，署名并写明模型名）",
          state: "absent",
        },
      ],
      health: "ok",
    },
    {
      id: "builtin/ocr-vision",
      name: "认图里的字",
      version: "0.1.0",
      tool: "Vision",
      turns: "图、封面、关键帧 → 图里的字，带每一行在图上的位置",
      jobs: ["ocr", "frames"],
      models: [],
      health: "ok",
      hint: "用 macOS 自带的文字识别，不用下载模型",
    },
    {
      id: "lin/asr-firered",
      name: "FireRedASR2 转口播",
      version: "0.3.1",
      tool: "FireRedASR2",
      turns: "声音 → 口播文字，带每一句的时间",
      jobs: ["asr"],
      models: [{ name: "firered-asr2-aed-l", label: "FireRedASR2-AED-L", size: 912 * MB, license: "Apache-2.0", state: "present" }],
      health: "ok",
    },
    {
      id: "lin/ocr-ppocr",
      name: "PP-OCR 认字",
      version: "0.2.0",
      tool: "PP-OCRv6",
      turns: "图、封面、关键帧 → 图里的字，带每一行在图上的位置",
      jobs: ["ocr", "frames"],
      models: [{ name: "pp-ocrv6-small", label: "PP-OCRv6 small", size: 16 * MB, license: "Apache-2.0", state: "absent" }],
      health: "unavailable",
      hint: "这台电脑上缺 onnxruntime，插件没能启动",
    },
  ];
}

/** 每一层默认用哪个插件。 */
const DEFAULT_PLUGIN: Record<TransformLayer, string> = { asr: "builtin/asr-sherpa", ocr: "builtin/ocr-vision", frames: "builtin/ocr-vision" };

/** 样例库的条目：读、写一条，和还收着没放出来的几面。 */
export interface MockItems {
  get(id: string): Item;
  set(id: string, item: Item): void;
  /** 拿走这一层收着的结果（只给一次）。 */
  release(id: string, layer: Pending["layer"]): Facet[];
  ids(): string[];
}

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const RUN_MS = 1800;
const DOWNLOAD_MS = 3200;

/** 再转一次的口播：同一段声音，有几句断句和写法不一样。 */
function reheard(segments: Segment[]): Segment[] {
  return segments.map((s, i) => (i % 3 === 1 ? { ...s, text: s.text.replace(/，/g, "。").replace(/ 度/g, "°") } : s));
}

export function createMockTransforms(items: MockItems, empty: boolean): Transforms {
  const plugins = basePlugins();
  if (empty) plugins.splice(3);
  let seq = 0;

  const model = (pluginId: string, name: string) => {
    const m = plugins.find((p) => p.id === pluginId)?.models.find((x) => x.name === name);
    if (!m) throw new Error(`样例库里没有模型 ${pluginId} ${name}`);
    return m;
  };
  const setPending = (id: string, change: (p: Pending[]) => Pending[]) => {
    const item = items.get(id);
    items.set(id, { ...item, pending: change(item.pending) });
  };

  /** 这一层「转好了」：有收着的结果就放出来；没有就照最新那份转一份新的，原来的留着。 */
  const finish = (id: string, layer: Pending["layer"], pluginId: string) => {
    const item = items.get(id);
    const released = items.release(id, layer);
    const plugin = plugins.find((p) => p.id === pluginId);
    let added: Facet[] = released;
    if (released.length === 0 && plugin && (layer === "asr" || layer === "ocr" || layer === "frames")) {
      const base = item.facets.find((f) => f.layer === layer);
      if (base && "source" in base) {
        const runId = `0193f2b2-0000-7000-8000-${String(++seq).padStart(12, "0")}`;
        const copy = { ...base, key: `${layer}:${runId}`, source: { by: "machine" as const, tool: plugin.tool, runId, at: new Date().toISOString(), ...(layer === "asr" ? { lang: "zh" } : {}) } };
        added = [{ ...copy, segments: layer === "asr" ? reheard(base.segments) : base.segments }];
      }
    }
    // 新的那份放在同一层的平台字幕后面、别的机器那几份前面。
    const facets = [...item.facets];
    const at = facets.findIndex((f) => f.layer === layer && "source" in f && f.source.by === "machine");
    const end = facets.reduce((k, f, i) => (f.layer === layer ? i + 1 : k), -1);
    facets.splice(at >= 0 ? at : end >= 0 ? end : facets.length, 0, ...added);
    items.set(id, { ...item, facets, pending: item.pending.filter((p) => p.layer !== layer) });
  };

  const runLater = (id: string, layer: Pending["layer"], pluginId: string) => {
    setPending(id, (ps) => [...ps.filter((p) => p.layer !== layer), { layer, state: "now" }]);
    void wait(RUN_MS).then(() => finish(id, layer, pluginId));
  };

  return {
    async transforms(): Promise<TransformsView> {
      await wait(40);
      return { plugins: plugins.map((p) => ({ ...p, models: p.models.map((m) => ({ ...m })) })), auto: true };
    },
    async transform(id, body: TransformRequest) {
      await wait(60);
      const item = items.get(id);
      if ("jobs" in body) {
        // 打开了这一条：排着的挪到最前，在电池上也转；等模型的接着等。
        for (const p of item.pending) {
          if (p.state === "later" || p.state === "on-power") runLater(id, p.layer, p.layer === "video" ? "builtin/media" : DEFAULT_PLUGIN[p.layer]);
        }
        return;
      }
      const pluginId = body.plugin ?? DEFAULT_PLUGIN[body.job];
      const plugin = plugins.find((p) => p.id === pluginId);
      if (!plugin?.jobs.includes(body.job)) throw new Error(`invalid-args: ${pluginId} 不做 ${body.job}`);
      if (item.pending.some((p) => p.layer === body.job && p.state === "now")) throw new Error("conflict: 这一层正在转");
      const missing = plugin.models.find((m) => m.state !== "present");
      if (missing) {
        setPending(id, (ps) => [
          ...ps.filter((p) => p.layer !== body.job),
          { layer: body.job, state: "needs-model", model: { plugin: plugin.id, name: missing.name, size: missing.size, license: missing.license } },
        ]);
        return;
      }
      runLater(id, body.job, pluginId);
    },
    async downloadModel(pluginId, name) {
      await wait(60);
      const m = model(pluginId, name);
      if (m.state === "present") return;
      m.state = "downloading";
      void wait(DOWNLOAD_MS).then(() => {
        m.state = "present";
        // 等着这个模型的条目照常转。
        for (const id of waitingFor(pluginId, name)) {
          const p = items.get(id).pending.find((x) => x.model?.name === name);
          if (p) runLater(id, p.layer, pluginId);
        }
      });
    },
    async removeModel(pluginId, name) {
      await wait(60);
      model(pluginId, name).state = "absent";
    },
  };

  function waitingFor(pluginId: string, name: string): string[] {
    return items.ids().filter((id) => items.get(id).pending.some((p) => p.state === "needs-model" && p.model?.plugin === pluginId && p.model.name === name));
  }
}
