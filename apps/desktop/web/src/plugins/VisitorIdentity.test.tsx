import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { VisitorIdentity as Visitor } from "../api/endpoints";
import { VisitorIdentity } from "./VisitorIdentity";

const { openVisitor, clearVisitorData } = vi.hoisted(() => ({
  openVisitor: vi.fn<(site: string, replace: boolean) => Promise<boolean>>(),
  clearVisitorData: vi.fn<(site: string) => Promise<void>>(),
}));
vi.mock("../shell", () => ({ openVisitor, clearVisitorData }));
const forget = vi.fn<(input: URL, init?: RequestInit) => Promise<Response>>();

afterEach(() => {
  cleanup();
  openVisitor.mockReset();
  clearVisitorData.mockReset();
  forget.mockReset();
  vi.unstubAllGlobals();
});

const visitor: Visitor = {
  site: "www.example.com",
  name: "例子",
  url: "https://www.example.com/",
  cookies: [],
  domains: [],
  state: "missing",
  plugins: ["someone/example"],
};

function show(state: Visitor["state"]) {
  openVisitor.mockResolvedValue(true);
  render(
    <QueryClientProvider client={new QueryClient()}>
      <VisitorIdentity visitor={{ ...visitor, state }} />
    </QueryClientProvider>,
  );
}

it("takes the first identity from a fresh window and asks for a clean one when replacing", () => {
  show("missing");
  fireEvent.click(screen.getByRole("button", { name: "取访客身份" }));
  expect(openVisitor).toHaveBeenLastCalledWith("www.example.com", false);
  cleanup();

  show("present");
  fireEvent.click(screen.getByRole("button", { name: "换一份" }));
  expect(openVisitor).toHaveBeenLastCalledWith("www.example.com", true);
  cleanup();

  show("expired");
  fireEvent.click(screen.getByRole("button", { name: "换一份" }));
  expect(openVisitor).toHaveBeenLastCalledWith("www.example.com", true);
});

it("clears the window's data before the daemon forgets, and keeps the identity when clearing fails", async () => {
  const order: string[] = [];
  clearVisitorData.mockImplementation(() => {
    order.push("clear");
    return Promise.resolve();
  });
  forget.mockImplementation(() => {
    order.push("forget");
    return Promise.resolve(new Response(null, { status: 204 }));
  });
  vi.stubGlobal("fetch", forget);
  show("present");
  fireEvent.click(screen.getByRole("button", { name: "去掉" }));
  await waitFor(() => expect(order).toEqual(["clear", "forget"]));
  expect(forget.mock.calls[0]?.[1]?.method).toBe("DELETE");
  cleanup();

  order.length = 0;
  clearVisitorData.mockRejectedValue("没能清掉");
  show("present");
  fireEvent.click(screen.getByRole("button", { name: "去掉" }));
  expect(await screen.findByText("没能清掉")).toBeTruthy();
  expect(order).toEqual([]);
});
