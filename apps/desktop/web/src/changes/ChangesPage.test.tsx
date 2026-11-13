import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ChangesView } from "../api/endpoints";
import { createMockChanges } from "../mock/changes";
import { createAppRouter } from "../router";
// 路由是用到才载入这一页的；先载入，免得第一次转译的时间算进等页面出现的那一秒里。
import "./ChangesPage";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function serve(view: ChangesView | null) {
  const sent: { path: string; body: unknown }[] = [];
  vi.stubGlobal("scrollTo", vi.fn());
  vi.stubGlobal(
    "fetch",
    vi.fn((input: URL | string, init?: RequestInit) => {
      const path = new URL(input).pathname;
      if (init?.method === "POST") {
        sent.push({ path, body: typeof init.body === "string" ? JSON.parse(init.body) : null });
        const id = path.split("/").at(-2);
        const b = view?.batches.find((x) => x.id === id);
        return Promise.resolve(Response.json({ ...b, state: path.endsWith("/undo") ? "undone" : b?.state }));
      }
      if (view && path.endsWith("/changes")) return Promise.resolve(Response.json(view));
      if (path.endsWith("/events")) return new Promise(() => {});
      return Promise.resolve(Response.json({ code: "not-found", detail: "no such route" }, { status: 404 }));
    }),
  );
  const router = createAppRouter(createMemoryHistory({ initialEntries: ["/changes"] }));
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return sent;
}

const sample = () => createMockChanges(false).changes();

it("accepts one change of a proposal, or all of what is still open", async () => {
  const sent = serve(await sample());
  const [first] = await screen.findAllByRole("button", { name: "接受" });
  if (!first) throw new Error("no proposal");
  fireEvent.click(first);
  await waitFor(() => expect(sent).toContainEqual({ path: "/api/changes/b-draft-topic/accept", body: { ops: ["o1"] } }));
  fireEvent.click(screen.getByRole("button", { name: "接受其余更改" }));
  await waitFor(() => expect(sent).toContainEqual({ path: "/api/changes/b-draft-topic/accept", body: {} }));
});

it("asks once before undoing a batch that touches what you wrote", async () => {
  const sent = serve(await sample());
  const batch = (await screen.findAllByRole("article")).find((a) => a.textContent?.includes("Codex"));
  if (!batch) throw new Error("no batch");
  fireEvent.click(within(batch).getByRole("button", { name: "撤销这批更改" }));
  expect(within(batch).getByText(/你后来将其中的「这周的豆子」添加到了主题「咖啡」/)).toBeTruthy();
  expect(sent).toHaveLength(0);
  fireEvent.click(within(batch).getByRole("button", { name: "全部撤销" }));
  await waitFor(() => expect(sent.map((s) => s.path)).toEqual(["/api/changes/b-agent-take/undo"]));
});

it("says one calm sentence when the daemon keeps no change log yet", async () => {
  serve(null);
  expect(await screen.findByText(/此版本的 usuyo 尚未提供更改记录/)).toBeTruthy();
  expect(screen.queryByRole("radiogroup", { name: "显示" })).toBeNull();
});
