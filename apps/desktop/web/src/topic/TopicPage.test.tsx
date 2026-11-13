import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { TopicPage as TopicView } from "../api/endpoints";
import { createMockLibrary } from "../mock/library";
import { createMockPages } from "../mock/pages";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./TopicPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const sample = () => createMockPages(createMockLibrary(false), false).topicPage("coffee");

function serve(page: TopicView | null) {
  const sent: { path: string; method: string; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      const method = init?.method ?? "GET";
      if (method !== "GET") sent.push({ path, method, body: typeof init?.body === "string" ? JSON.parse(init.body) : null });
      if (page && path.endsWith("/topics/coffee")) return Promise.resolve(Response.json(page));
      if (page && (path.endsWith("/topics/coffee/cite") || path.endsWith("/topics/coffee/page"))) return Promise.resolve(Response.json(page));
      if (page && path.includes("/threads/")) return Promise.resolve(new Response(null, { status: 204 }));
      if (path.endsWith("/topics")) return Promise.resolve(Response.json([{ id: "coffee", name: "咖啡", followed: false }]));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/topics/coffee"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return sent;
}

it("every sample quotation, thread and angle points at a segment the sample library really has", async () => {
  const page = await sample();
  expect(page.page.citations).toHaveLength(6);
  expect(page.page.citations.filter((c) => c.found === "gone")).toHaveLength(1);
  expect(page.threads.flatMap((t) => t.segments).length).toBeGreaterThan(5);
  expect(page.angles.length).toBe(2);
});

it("shows the page's own text with its quotations, each jumping back to its second", async () => {
  serve(await sample());
  // 这句也被一个角度引了：页上的那一处排在最前。
  const [quote] = await screen.findAllByText("「前 40% 的水决定酸甜，后 60% 决定浓淡」");
  expect(quote?.closest("a")?.getAttribute("href")).toBe("/items/c1?t=150");
  expect(screen.getByText("此处现在为「粉 20 克，水 300 克，就是 1 比 15」")).toBeTruthy();
  // 清掉了的来源不能点。
  const gone = screen.getByText("「深烘的豆子我一般 86 度，浅烘再往上加」");
  expect(gone.closest("a")).toBeNull();
  expect(screen.getByText("来源已删除，这里显示的是引用时的文字")).toBeTruthy();
});

it("labels what the machine picked, quotes a segment from it and drops a thread without a trace", async () => {
  const sent = serve(await sample());
  const threads = await screen.findAllByRole("region", { name: /^自动选取的 \d+ 段$/ });
  const first = threads[0];
  if (!first) throw new Error("no thread");
  expect(within(first).getByText("自动选取")).toBeTruthy();
  fireEvent.click(within(first).getByRole("button", { name: "将「水温控制在 92 度左右」引用到此页" }));
  await waitFor(() => expect(sent.some((s) => s.path.endsWith("/topics/coffee/cite"))).toBe(true));
  const body = sent.find((s) => s.path.endsWith("/cite"))?.body;
  expect(body).toMatchObject({ segment: { itemId: "c1", layer: "asr", text: "水温控制在 92 度左右", position: { kind: "time", sec: 102 } } });

  fireEvent.click(within(first).getByRole("button", { name: "忽略这组" }));
  await waitFor(() => expect(screen.getAllByRole("region", { name: /^自动选取的 \d+ 段$/ })).toHaveLength(threads.length - 1));
});

it("says one calm sentence and shows no sample text when the daemon has no topic pages", async () => {
  serve(null);
  expect(await screen.findByText(/此版本的 usuyo 尚未提供主题页/)).toBeTruthy();
  expect(screen.queryByText("页面")).toBeNull();
});

it("edits a paragraph in place and sends the whole page back", async () => {
  const sent = serve(await sample());
  const [edit] = await screen.findAllByRole("button", { name: "编辑" });
  if (!edit) throw new Error("no paragraph");
  fireEvent.click(edit);
  const box = screen.getByRole("textbox", { name: /编辑段落/ });
  fireEvent.change(box, { target: { value: "同一包豆子，先看前面那几十克水。" } });
  fireEvent.click(screen.getByRole("button", { name: "保存" }));
  await waitFor(() => expect(sent.some((s) => s.method === "PUT" && s.path.endsWith("/topics/coffee/page"))).toBe(true));
  const body = sent.find((s) => s.path.endsWith("/page"))?.body;
  expect(body).toMatchObject({ blocks: [{ id: "b1", kind: "text", text: "同一包豆子，先看前面那几十克水。", cites: ["k3f7q", "m2x8d"] }, {}, {}, {}, {}] });
});

it("labels keyword hits as machine-picked and lets you put one in", async () => {
  const page = await sample();
  const [member] = page.members;
  if (!member) throw new Error("no member");
  const sent = serve({ ...page, followed: true, hits: [{ item: { ...member, id: "f1", title: "命中的一条", topicIds: [] }, words: ["手冲"] }] });
  expect(await screen.findByText("标题或内容中包含「手冲」")).toBeTruthy();
  const match = screen.getByRole("region", { name: "匹配规则" });
  expect(within(match).getByText("自动选取")).toBeTruthy();
  fireEvent.click(within(match).getByRole("button", { name: "添加" }));
  await waitFor(() => expect(sent).toContainEqual({ path: "/api/items/f1/topics", method: "POST", body: { topicId: "coffee", member: true } }));
});
