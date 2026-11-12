import { expect, it } from "vitest";
import type { MapView } from "../api/endpoints";
import { buildPlaces, sourceCenter } from "./places";

const point = (itemId: string, id: string, name: string): MapView["points"][number] => ({
  itemId,
  x: 0.5,
  y: 0.5,
  title: itemId,
  platform: { id, name },
  topicIds: [],
  takenAt: "2026-11-01T12:00:00.000Z",
});
const FRAME = { width: 1280, height: 800, mapX: 248 };

it("keeps the sample platforms apart on the source map", () => {
  const ids = ["xiaohongshu", "wechat-channels", "bilibili", "wechat-mp", "web", "file", "x"];
  const { sources } = buildPlaces({ regions: [], points: ids.map((id, i) => point(`i${i}`, id, id)) }, new Map(), { width: 1440, height: 900, mapX: 248 });
  for (const a of sources) for (const b of sources) if (a !== b) expect(Math.hypot(a.x - b.x, a.y - b.y)).toBeGreaterThan(150);
});

it("places a source by its platform id alone, whatever its name and whoever else is there", () => {
  const one = buildPlaces({ regions: [], points: [point("a", "mastodon", "长毛象")] }, new Map(), FRAME).sources;
  const two = buildPlaces({ regions: [], points: [point("b", "web", "网页"), point("a", "mastodon", "另一个名字")] }, new Map(), FRAME).sources;
  const m1 = one.find((s) => s.id === "mastodon");
  const m2 = two.find((s) => s.id === "mastodon");
  expect(m1 && m2 && [m1.x, m1.y]).toEqual(m2 && [m2.x, m2.y]);
  expect(m2?.name).toBe("另一个名字");
  for (const id of ["web", "x", "some-new-platform"]) {
    const [x, y] = sourceCenter(id);
    expect(x).toBeGreaterThanOrEqual(330);
    expect(x).toBeLessThanOrEqual(1340);
    expect(y).toBeGreaterThanOrEqual(150);
    expect(y).toBeLessThanOrEqual(770);
  }
});
