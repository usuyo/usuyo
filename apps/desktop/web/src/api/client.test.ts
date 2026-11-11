import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError, DaemonUnreachableError, request } from "./client";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("request", () => {
  it("maps a network failure to DaemonUnreachableError", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new TypeError("Failed to fetch")));
    await expect(request("/health")).rejects.toBeInstanceOf(DaemonUnreachableError);
  });

  it("maps a non-2xx answer to ApiError with its status", async () => {
    vi.stubGlobal("fetch", vi.fn().mockImplementation(() => Promise.resolve(new Response("nope", { status: 500 }))));
    await expect(request("/items")).rejects.toMatchObject({ status: 500 });
    await expect(request("/items")).rejects.toBeInstanceOf(ApiError);
  });

  it("sends the query and a JSON body, with a timeout signal", async () => {
    const fetchMock = vi.fn<typeof fetch>().mockImplementation(() => Promise.resolve(Response.json({ id: "a" })));
    vi.stubGlobal("fetch", fetchMock);
    await request("/search", { query: { q: "雾" } });
    await request("/items", { method: "POST", body: { url: "https://example.com" } });

    const searchUrl = fetchMock.mock.calls[0]?.[0];
    expect(searchUrl).toMatchObject({ search: "?q=%E9%9B%BE" });
    const postInit = fetchMock.mock.calls[1]?.[1];
    expect(postInit?.body).toBe('{"url":"https://example.com"}');
    expect(postInit?.signal).toBeInstanceOf(AbortSignal);
  });
});
