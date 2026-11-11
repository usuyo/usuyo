import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { createMockLibrary } from "../mock/library";
import { FacetColumn } from "./FacetColumn";
import type { Player } from "./usePlayer";

afterEach(cleanup);

function player(): Player {
  return {
    t: 0,
    playing: false,
    image: null,
    canPlay: true,
    video: undefined,
    attachVideo: () => undefined,
    toggle: () => undefined,
    go: vi.fn(),
    seek: () => undefined,
    showImage: () => undefined,
    onTime: () => undefined,
  };
}

it("shows every text facet side by side, each with its positions", async () => {
  const item = await createMockLibrary(false).item("c1");
  const p = player();
  render(<FacetColumn item={item} player={p} />);
  for (const name of ["口播", "图里的字", "作者写的", "评论"]) {
    expect(screen.getByRole("region", { name })).toBeTruthy();
  }
  screen.getByRole("button", { name: "从 01:42 播" }).click();
  expect(p.go).toHaveBeenCalledWith(expect.objectContaining({ kind: "time", sec: 102 }));
  expect(screen.getByRole("button", { name: "看封面" })).toBeTruthy();
});
