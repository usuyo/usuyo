// 样例库的图：本地生成的点阵 SVG，不用网上的图。灰点在深浅两种底上都看得见。
import { fbm, hash2, r01, smooth, vnoise } from "../fog/noise";

export const SCENES = ["pourover", "film", "street", "dish", "page", "screen", "cat", "mountain", "plain"] as const;
export type Scene = (typeof SCENES)[number];

const W = 620;
const H = 348;
const STEP = 6;

function seg(px: number, py: number, ax: number, ay: number, bx: number, by: number): number {
  const dx = bx - ax;
  const dy = by - ay;
  let t = ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy);
  t = t < 0 ? 0 : t > 1 ? 1 : t;
  return Math.hypot(px - ax - dx * t, py - ay - dy * t);
}

function soft(x: number, y: number, x0: number, y0: number, x1: number, y1: number, blur: number): number {
  const dx = Math.max(x0 - x, 0, x - x1);
  const dy = Math.max(y0 - y, 0, y - y1);
  return 1 - smooth(0, blur, Math.sqrt(dx * dx + dy * dy));
}

function ring(x: number, y: number, cx: number, cy: number, r: number, w: number): number {
  return 1 - smooth(0, w, Math.abs(Math.hypot(x - cx, y - cy) - r));
}

// 条目页画布上的那张手冲（Item.dc.html 的 L.pic），t 固定为 0。
function pourover(x: number, y: number): number {
  let v = 0.05 + 0.07 * vnoise(x * 0.05, y * 0.05);
  const vx = (x - 300) / 360;
  const vy = (y - 170) / 230;
  v *= 1 - 0.6 * (vx * vx + vy * vy);
  if (y > 302) v = Math.max(v, 0.15 + 0.07 * vnoise(x * 0.04, y * 0.25));
  if (Math.abs(y - 302) < 2.6) v = Math.max(v, 0.34);
  const sx = (x - 270) / 80;
  const sy = (y - 262) / 44;
  const sd = Math.sqrt(sx * sx + sy * sy);
  if (y < 302 && sd < 1.07) {
    if (sd > 0.93) v = Math.max(v, 0.6);
    else if (y > 270) v = Math.max(v, 0.18 + 0.08 * vnoise(x * 0.12, y * 0.12));
    else v = Math.max(v, 0.1);
  }
  if (y >= 212 && y <= 222 && Math.abs(x - 270) < 40) v = Math.max(v, 0.62);
  if (y >= 150 && y < 212) {
    const hw = 76 - (y - 150) * (50 / 62);
    const dx = x - 270;
    if (Math.abs(dx) < hw) v = Math.max(v, 0.66 + (0.14 * (dx + hw)) / (2 * hw) - (Math.abs(dx) > hw - 4 ? 0.12 : 0));
  }
  if (y >= 143 && y < 151 && Math.abs(x - 270) < 82) v = Math.max(v, 0.92);
  const kd = Math.hypot(x - 566, y - 26);
  if (kd < 100) v = Math.max(v, kd > 92 ? 0.68 : 0.44 + 0.08 * vnoise(x * 0.06, y * 0.06));
  if (seg(x, y, 480, 74, 304, 104) < 6.5) v = Math.max(v, 0.62);
  const stx = 302 - (y - 106) * 0.22;
  if (y > 104 && y < 146 && Math.abs(x - stx) < 2.6) v = Math.max(v, 0.82 + 0.18 * vnoise(y * 0.3, 3.1));
  return v;
}

function base(x: number, y: number, seed: number): number {
  return 0.05 + 0.08 * fbm(x * 0.012 + seed * 7, y * 0.014 - seed * 3);
}

function sceneValue(scene: Scene, x: number, y: number, seed: number): number {
  const j = (k: string) => r01(`${seed}:${k}`);
  switch (scene) {
    case "pourover":
      return pourover(x, y);
    case "film": {
      // 一台相机：机身、镜头的两圈、取景窗。
      const ox = (j("x") - 0.5) * 80;
      let v = base(x, y, seed);
      v = Math.max(v, 0.42 * soft(x, y, 170 + ox, 110, 450 + ox, 260, 8));
      v = Math.max(v, 0.85 * ring(x, y, 310 + ox, 190, 62, 5), 0.6 * ring(x, y, 310 + ox, 190, 38, 4));
      v = Math.max(v, 0.7 * soft(x, y, 200 + ox, 124, 250 + ox, 150, 3));
      return v;
    }
    case "street": {
      // 一条街：两侧的楼，地平线，远处一个点。
      let v = base(x, y, seed) + 0.1 * smooth(200, 348, y);
      for (let i = 0; i < 5; i++) {
        const bx = 20 + i * 52 + j(`b${i}`) * 20;
        v = Math.max(v, (0.3 + 0.3 * j(`h${i}`)) * soft(x, y, bx, 60 + j(`t${i}`) * 90, bx + 40, 300, 4));
        const rx = 600 - bx - 40;
        v = Math.max(v, (0.25 + 0.3 * j(`k${i}`)) * soft(x, y, rx, 80 + j(`u${i}`) * 80, rx + 40, 300, 4));
      }
      return Math.max(v, 0.5 * (1 - smooth(0, 3, Math.abs(y - 300))));
    }
    case "dish": {
      // 一只碗，从上面看。
      const cx = 310 + (j("x") - 0.5) * 120;
      let v = base(x, y, seed);
      v = Math.max(v, 0.8 * ring(x, y, cx, 174, 120, 5));
      const inner = 1 - smooth(90, 112, Math.hypot(x - cx, y - 174));
      v = Math.max(v, inner * (0.25 + 0.3 * fbm(x * 0.03 + seed, y * 0.03)));
      return v;
    }
    case "page": {
      // 一页字：一行行的横条。
      let v = 0.04 + 0.04 * vnoise(x * 0.05, y * 0.05);
      v = Math.max(v, 0.18 * soft(x, y, 120, 30, 500, 318, 6));
      for (let row = 0; row < 11; row++) {
        const y0 = 54 + row * 24;
        const len = row % 4 === 3 ? 160 + j(`r${row}`) * 120 : 330 + j(`r${row}`) * 30;
        v = Math.max(v, 0.62 * soft(x, y, 146, y0, 146 + len, y0 + 4, 2));
      }
      return v;
    }
    case "screen": {
      // 一块屏幕上的代码。
      let v = base(x, y, seed);
      v = Math.max(v, 0.3 * soft(x, y, 60, 30, 560, 318, 4));
      for (let row = 0; row < 10; row++) {
        const ind = Math.floor(j(`i${row}`) * 4) * 24;
        const len = 80 + j(`l${row}`) * 240;
        v = Math.max(v, 0.78 * soft(x, y, 100 + ind, 60 + row * 24, 100 + ind + len, 64 + row * 24, 2));
      }
      return v;
    }
    case "cat": {
      // 一只猫的轮廓：身子、头、两只耳朵。
      let v = base(x, y, seed);
      const body = 1 - smooth(0.9, 1.05, Math.hypot((x - 330) / 130, (y - 240) / 70));
      const head = 1 - smooth(0.9, 1.05, Math.hypot((x - 220) / 52, (y - 168) / 46));
      v = Math.max(v, 0.55 * body, 0.62 * head);
      if (seg(x, y, 186, 140, 196, 106) < 7 || seg(x, y, 250, 140, 244, 106) < 7) v = Math.max(v, 0.7);
      return v;
    }
    case "mountain": {
      // 远山和红叶。
      const ridge = 190 + 50 * Math.sin(x * 0.012 + seed) + 30 * fbm(x * 0.02, seed);
      let v = base(x, y, seed);
      if (y > ridge) v = Math.max(v, 0.3 + 0.2 * fbm(x * 0.03, y * 0.03));
      if (Math.abs(y - ridge) < 3) v = Math.max(v, 0.75);
      return Math.max(v, 0.6 * smooth(0.62, 0.8, fbm(x * 0.02 + 9, y * 0.02)) * smooth(250, 348, y));
    }
    default:
      return base(x, y, seed) + 0.35 * smooth(0.5, 0.8, fbm(x * 0.008 + seed, y * 0.01));
  }
}

const cache = new Map<string, string>();

/** 一张点阵图的 data URL；同一个 scene 和 seed 总是同一张。 */
export function dotPicture(scene: Scene, seed: string): string {
  const key = `${scene}:${seed}`;
  const hit = cache.get(key);
  if (hit) return hit;
  const s = Math.floor(r01(seed) * 1000);
  const levels: string[][] = [[], [], [], [], [], [], [], []];
  for (let y = STEP / 2; y < H; y += STEP) {
    for (let x = STEP / 2; x < W; x += STEP) {
      const v = sceneValue(scene, x, y, s) + (hash2(x, y) - 0.5) * 0.02;
      if (v < 0.06) continue;
      levels[Math.min(7, Math.floor(v * 8))]?.push(`M${x} ${y}h0`);
    }
  }
  const paths = levels
    .map((d, lv) =>
      d.length === 0
        ? ""
        : `<path stroke-width="${((0.6 + lv * 0.3) * 2).toFixed(2)}" stroke-opacity="${(0.2 + lv * 0.105).toFixed(3)}" d="${d.join("")}"/>`,
    )
    .join("");
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${W} ${H}" width="${W}" height="${H}"><g fill="none" stroke="rgb(140,148,160)" stroke-linecap="round">${paths}</g></svg>`;
  const url = `data:image/svg+xml,${encodeURIComponent(svg)}`;
  cache.set(key, url);
  return url;
}

export const PICTURE_SIZE = { width: W, height: H };
