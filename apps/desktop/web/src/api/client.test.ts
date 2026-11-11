import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError, connection, DaemonUnreachableError, request } from "./client";

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

  it("asks for the connection again after a 401 or a network failure, not after other errors", async () => {
    // 内核重启过：令牌换了，旧的回 401；桌面壳会给新的。
    const fetchMock = vi.fn<typeof fetch>();
    vi.stubGlobal("fetch", fetchMock);
    const first = connection();
    fetchMock.mockResolvedValueOnce(new Response("nope", { status: 500 }));
    await expect(request("/map")).rejects.toMatchObject({ status: 500 });
    expect(connection()).toBe(first);

    fetchMock.mockResolvedValueOnce(new Response("nope", { status: 401 }));
    await expect(request("/map")).rejects.toMatchObject({ status: 401 });
    const second = connection();
    expect(second).not.toBe(first);

    fetchMock.mockRejectedValueOnce(new TypeError("Failed to fetch"));
    await expect(request("/map")).rejects.toBeInstanceOf(DaemonUnreachableError);
    expect(connection()).not.toBe(second);
  });

  it("sends PUT and DELETE, and reads a 204 as no content", async () => {
    const fetchMock = vi.fn<typeof fetch>().mockImplementation(() => Promise.resolve(new Response(null, { status: 204 })));
    vi.stubGlobal("fetch", fetchMock);
    await expect(request<void>("/sites/a/login", { method: "DELETE" })).resolves.toBeUndefined();
    await request<void>("/plugins/o/r/secrets/k", { method: "PUT", body: { value: "v" } });
    expect(fetchMock.mock.calls.map((c) => c[1]?.method)).toEqual(["DELETE", "PUT"]);
  });

  it("keeps the shell's reason for the user", () => {
    const e = new DaemonUnreachableError("x", "端口 47615 被别的程序占着，usuyo 的后台没能启动");
    expect(e.reason).toBe("端口 47615 被别的程序占着，usuyo 的后台没能启动");
    expect(new DaemonUnreachableError(new Error("x")).reason).toBeUndefined();
  });
});
