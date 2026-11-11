import { afterEach, describe, expect, it, vi } from "vitest";
import { openExternal, webUrl } from "./index";

const openUrl = vi.fn<(url: string) => Promise<void>>();
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl }));

const hostile = ["javascript:alert(1)", "file:///etc/passwd", "data:text/html,<b>x</b>", "not a url", "", "//evil.test/x"];

afterEach(() => {
  vi.unstubAllGlobals();
  openUrl.mockReset();
  Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
});

describe("openExternal", () => {
  it("only lets http and https through", () => {
    expect(webUrl("https://example.test/a?b=1")).toBe("https://example.test/a?b=1");
    expect(webUrl("http://example.test")).toBe("http://example.test/");
    for (const url of hostile) expect(webUrl(url)).toBeNull();
  });

  it("opens web links in a new tab outside the desktop shell, and nothing else", async () => {
    const open = vi.fn();
    vi.stubGlobal("open", open);
    for (const url of hostile) expect(await openExternal(url)).toBe(false);
    expect(open).not.toHaveBeenCalled();
    expect(await openExternal("https://example.test/a")).toBe(true);
    expect(open).toHaveBeenCalledWith("https://example.test/a", "_blank", "noopener,noreferrer");
  });

  it("goes through the opener plugin inside the desktop shell", async () => {
    Object.defineProperty(window, "__TAURI_INTERNALS__", { value: {}, configurable: true });
    const open = vi.fn();
    vi.stubGlobal("open", open);
    openUrl.mockResolvedValue();
    for (const url of hostile) expect(await openExternal(url)).toBe(false);
    expect(openUrl).not.toHaveBeenCalled();
    expect(await openExternal("https://example.test/a")).toBe(true);
    expect(openUrl).toHaveBeenCalledWith("https://example.test/a");
    expect(open).not.toHaveBeenCalled();

    openUrl.mockRejectedValue(new Error("forbidden"));
    vi.spyOn(console, "warn").mockImplementation(() => {});
    expect(await openExternal("https://example.test/b")).toBe(false);
  });
});
