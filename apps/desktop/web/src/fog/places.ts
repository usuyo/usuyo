// 把地图数据换成画面上的像素位置。按主题摆的位置来自内核；按来源摆的在前端按平台算，
// 每条两个位置都留着，切回来各回原位（Home.dc.html 的 placeSource）。
import type { MapView } from "../api/endpoints";
import { r01 } from "./noise";

export interface Region {
  id: string;
  name: string;
  x: number;
  y: number;
  rad: number;
  count: number;
}

export interface Place {
  id: string;
  title: string;
  platform: string;
  topicIds: string[];
  /** 按主题摆的位置。 */
  tx: number;
  ty: number;
  /** 按来源摆的位置。 */
  sx: number;
  sy: number;
  /** 两个位置之间各走各的弯路，和出发的先后。 */
  bend: number;
  delay: number;
}

export interface Frame {
  width: number;
  height: number;
  /** 地图从这里开始，左边是左栏。 */
  mapX: number;
}

// 来源的中心，照主页画布的页面坐标（1440×900）；视频号放在画布上抖音的位置。
const SOURCE_CENTERS: Record<string, [number, number]> = {
  小红书: [520, 300],
  B站: [900, 330],
  公众号: [720, 610],
  视频号: [380, 560],
  网页: [1130, 170],
  文件: [1290, 760],
};

function toPx(f: Frame, nx: number, ny: number): [number, number] {
  return [f.mapX + nx * (f.width - f.mapX), ny * f.height];
}

/** 画布坐标换算：页面上的 1440×900 位置按地图区域缩放。 */
function pagePx(f: Frame, x: number, y: number): [number, number] {
  return toPx(f, (x - 248) / 1192, y / 900);
}

export function buildPlaces(
  map: MapView,
  topicNames: Map<string, string>,
  f: Frame,
): { regions: Region[]; sources: Region[]; places: Place[] } {
  const scale = (f.width - f.mapX) / 1192;
  const regions = map.regions.map((r) => {
    const [x, y] = toPx(f, r.x, r.y);
    return { id: r.topicId, name: topicNames.get(r.topicId) ?? "", x, y, rad: r.r * (f.width - f.mapX), count: r.count };
  });
  const counts = new Map<string, number>();
  for (const p of map.points) counts.set(p.platform.name, (counts.get(p.platform.name) ?? 0) + 1);
  let spare = 0;
  const sources: Region[] = [...counts.entries()].map(([name, n]) => {
    const c = SOURCE_CENTERS[name] ?? [1100 + 60 * spare++, 820];
    const [x, y] = pagePx(f, c[0], c[1]);
    return { id: name, name, x, y, rad: (26 + Math.sqrt(n) * 8) * scale, count: n };
  });
  const byName = new Map(sources.map((s) => [s.id, s]));
  const k = new Map<string, number>();
  const places = map.points.map((p): Place => {
    const [tx, ty] = toPx(f, p.x, p.y);
    const s = byName.get(p.platform.name);
    let sx = tx;
    let sy = ty;
    if (s) {
      const n = (k.get(s.id) ?? 0) + 1;
      k.set(s.id, n);
      const rr = s.rad * 0.55 * Math.sqrt((n - 0.5) / 7);
      const an = n * 2.39996 + r01(`${p.itemId}c`);
      sx = s.x + Math.cos(an) * rr * 1.2;
      sy = s.y + 22 + Math.abs(Math.sin(an)) * rr * 0.9;
    }
    return {
      id: p.itemId,
      title: p.title,
      platform: p.platform.name,
      topicIds: p.topicIds,
      tx,
      ty,
      sx,
      sy,
      bend: (r01(`${p.itemId}f`) - 0.5) * 120,
      delay: r01(`${p.itemId}g`) * 0.25,
    };
  });
  return { regions, sources, places };
}

/** 字的宽度，按画布原型粗算：汉字一个字号宽，其余 0.56。 */
export function textWidth(s: string, px: number): number {
  let w = 0;
  for (let i = 0; i < s.length; i++) w += s.charCodeAt(i) > 255 ? px : px * 0.56;
  return w;
}

export function nameSize(rad: number, scale: number): number {
  return 13 + (rad / scale) * 0.035;
}

export interface FoldRow {
  place: Place;
  x: number;
  y: number;
  w: number;
  index: number;
}

/** 关注的主题把名下的前五条折成一张短列表，放在名字下面（放不下就放上面）。 */
export function foldRows(region: Region, places: Place[], f: Frame): FoldRow[] {
  const scale = (f.width - f.mapX) / 1192;
  const list = places.filter((p) => p.topicIds[0] === region.id).slice(0, 5);
  const W = 270;
  const fs = nameSize(region.rad, scale);
  const left = Math.min(Math.max(region.x - W / 2, f.mapX + 16), f.width - 16 - W);
  let top = region.y + fs * 0.5 + 14;
  if (top + list.length * 28 > f.height - 20) top = region.y - fs * 0.5 - 14 - list.length * 28;
  return list.map((place, index) => ({ place, x: left, y: top + index * 28, w: W, index }));
}
