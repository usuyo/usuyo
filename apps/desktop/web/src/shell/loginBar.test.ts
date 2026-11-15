import { expect, it, vi } from "vitest";
import { detailText, phaseText, reasonText, type PanelView } from "./loginBar";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const view = (v: Partial<PanelView>): PanelView => ({
  phase: "reading",
  reason: null,
  handed: null,
  auto: true,
  paused: false,
  requests: 0,
  recognised: 0,
  noted: { favorite: 0, like: 0, all: 0 },
  skipped: {},
  over: 0,
  diagnosis: "",
  ...v,
});

it("says where the reading is, in plain words", () => {
  expect(phaseText(view({ phase: "signing-in" }), "例子")).toContain("在下方登录例子");
  expect(phaseText(view({}), "例子")).toBe("正在读取，每隔几秒自动向下滚动一屏。");
  expect(phaseText(view({ auto: false }), "例子")).toBe("正在读取。在下方页面中向下滚动。");
  expect(phaseText(view({ paused: true }), "例子")).toBe("已暂停。");
  expect(phaseText(view({ phase: "stopped", reason: "verify" }), "例子")).toBe("已停止：页面要求验证。完成验证后点按「继续」。");
  expect(phaseText(view({ phase: "handed", handed: { imports: 2, noted: 300, dropped: 1 } }), "例子")).toBe(
    "已发送到 usuyo：300 条链接。在 usuyo 的导入页面确认后开始获取。可以关闭此窗口。 1 条不符合插件的链接格式，已略过。",
  );
  expect(phaseText(view({ phase: "handed", handed: { imports: 0, noted: 0, dropped: 0 } }), "例子")).toBe("没有找到链接。可以关闭此窗口。");
});

it("names the stop reasons the protocol fixes and shows any other word as it is", () => {
  expect(reasonText("signin")).toBe("页面要求登录。登录后点按「继续」。");
  expect(reasonText(null)).toContain("无法识别");
  expect(reasonText("rateLimited")).toBe("rateLimited");
});

it("counts what was skipped by kind", () => {
  expect(detailText(view({ skipped: { noId: 2, noToken: 1 } }))).toBe("略过 3 条：缺少 ID 2, 缺少令牌 1");
  expect(detailText(view({ skipped: { noId: 0 } }))).toBe("");
  expect(detailText(view({ over: 5 }))).toBe("已达到上限，另有 5 条未记录。");
});
