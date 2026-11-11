// 点阵雾：主页和空资料库共用一套画法，数照画布原型（Home.dc.html、Empty.dc.html 的 frame/draw/dropPhase）。
// 18px 格子上算两层噪声，9px 一格画点，8 档密度；鼠标拨开雾的光走弹簧；收下一条时雾往落点聚拢再出一圈强调色。
import { reducedMotion, subscribe, wake } from "../motion/clock";
import { currentPalette, onPaletteChange } from "../theme/apply";
import { approach, clamp, fbm, hash2, inOut, smooth } from "./noise";
import { foldRows, nameSize, textWidth, type Frame, type Place, type Region } from "./places";

export interface Rect {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

export interface LibraryScene {
  kind: "library";
  frame: Frame;
  regions: Region[];
  sources: Region[];
  places: Place[];
  arrangement: "topic" | "source";
  followed: ReadonlySet<string>;
  /** 搜到的条目；`searching` 时其余的点变淡。 */
  hits: ReadonlySet<string>;
  searching: boolean;
  selected: string | null;
  /** 列表里鼠标停着的那条（折起来的行、搜索结果）。 */
  rowHover: string | null;
  /** 左栏里鼠标停着的主题。 */
  railHover: string | null;
  /** 右侧面板，开着时它下面和四周的雾让开。 */
  panel: Rect | null;
  /** 刚收下、要放落下动画的条目。 */
  landing: ReadonlySet<string>;
}

export interface EmptyScene {
  kind: "empty";
  frame: Frame;
  /** 中间那块字，雾让开一个椭圆。 */
  clearing: { cx: number; cy: number; rx: number; ry: number };
  drops: { id: string; x: number; y: number }[];
}

export type FogScene = LibraryScene | EmptyScene;

export interface FogEvents {
  hover?: (id: string | null) => void;
  hoverTopic?: (id: string | null) => void;
  lampCount?: (n: number | null) => void;
}

interface Drop {
  x: number;
  y: number;
  t0: number;
}

interface Phase {
  g: number;
  on: number;
  ring: number;
  gathering: boolean;
  age: number;
}

const LAMP = 190;
const SPRING_K = 520;
const SPRING_C = 2 * 0.82 * Math.sqrt(SPRING_K);
const TAU = Math.PI * 2;

/** 一条当前在哪：按排布的进度在两个位置之间走弯路。 */
function placeAt(p: Place, u: number): { x: number; y: number } {
  const dx = p.sx - p.tx;
  const dy = p.sy - p.ty;
  const dl = Math.hypot(dx, dy) || 1;
  const b = Math.sin(Math.PI * u) * p.bend;
  return { x: p.tx + dx * u - (dy / dl) * b, y: p.ty + dy * u + (dx / dl) * b };
}

/** 悬停那一条旁边的一行字：哪边有地方放哪边。 */
export function labelBox(x: number, y: number, title: string, platform: string, right: number): Rect & { side: 1 | -1 } {
  const w = textWidth(title, 12) + textWidth(platform, 11) + 34;
  const x0 = x + 12 + w > right ? x - 12 - w : x + 12;
  return { x0, y0: y - 12, x1: x0 + w, y1: y + 12, side: x0 < x ? -1 : 1 };
}

function rectFalloff(r: Rect, x: number, y: number, soft: number): number {
  const dx = Math.max(r.x0 - x, 0, x - r.x1);
  const dy = Math.max(r.y0 - y, 0, y - r.y1);
  return 1 - smooth(0, soft, Math.sqrt(dx * dx + dy * dy));
}

export class FogField {
  private readonly canvas: HTMLCanvasElement;
  private readonly events: FogEvents;
  private scene: FogScene | null = null;
  private t = 0;
  private ft = 0;
  private dt = 1 / 60;
  private raw: { x: number; y: number } | null = null;
  private lamp = { x: 760, y: 450 };
  private lampV = { x: 0, y: 0 };
  private lampA = 0;
  private P = 0;
  private moving = false;
  private readonly eased = new Map<string, number>();
  private readonly drops = new Map<string, Drop>();
  private hovI: string | null = null;
  private hovT: string | null = null;
  private lastHov: string | null = null;
  private lampN: number | null = null;
  private ng = new Float32Array(0);
  private sg = new Float32Array(0);
  private ig = new Uint8Array(0);
  private dims = { nx: 0, ny: 0, cols: 0, rows: 0 };
  private readonly buckets: number[][] = Array.from({ length: 8 }, () => []);
  private readonly accentBuckets: number[][] = Array.from({ length: 8 }, () => []);
  private readonly unsubscribe: () => void;
  private readonly unpalette: () => void;

  constructor(canvas: HTMLCanvasElement, events: FogEvents) {
    this.canvas = canvas;
    this.events = events;
    this.unsubscribe = subscribe((dt) => this.tick(dt));
    this.unpalette = onPaletteChange(() => {
      this.moving = true;
      wake();
    });
  }

  destroy(): void {
    this.unsubscribe();
    this.unpalette();
  }

  setScene(scene: FogScene): void {
    this.scene = scene;
    const { width, height } = scene.frame;
    const nx = Math.ceil(width / 18) + 2;
    const ny = Math.ceil(height / 18) + 2;
    const cols = Math.ceil(width / 9);
    const rows = Math.ceil(height / 9);
    if (nx !== this.dims.nx || ny !== this.dims.ny || cols !== this.dims.cols || rows !== this.dims.rows) {
      this.dims = { nx, ny, cols, rows };
      this.ng = new Float32Array(nx * ny);
      this.sg = new Float32Array(nx * ny);
      this.ig = new Uint8Array(cols * rows);
    }
    const dpr = typeof window === "undefined" ? 1 : Math.min(2, window.devicePixelRatio || 1);
    const w = Math.round(width * dpr);
    const h = Math.round(height * dpr);
    if (this.canvas.width !== w || this.canvas.height !== h) {
      this.canvas.width = w;
      this.canvas.height = h;
    }
    this.moving = true;
    wake();
  }

  /** 鼠标在地图上的位置；离开地图、在左栏或面板上时传 null。 */
  pointer(p: { x: number; y: number } | null): void {
    this.raw = p;
    wake();
  }

  private ez(key: string, target: number, rate: number): number {
    const v = this.eased.get(key);
    if (v === undefined) {
      this.eased.set(key, target);
      return target;
    }
    if (v === target) return v;
    const next = approach(v, target, rate, this.dt);
    this.eased.set(key, next);
    this.moving = true;
    return next;
  }

  // 界面走得快，它让开的雾合拢得慢一些。
  private heal(key: string, a: number): number {
    const v = this.eased.get(key);
    if (v === undefined || a >= v) {
      this.eased.set(key, a);
      return a;
    }
    let next = v + (a - v) * (1 - Math.exp(-6 * this.dt));
    if (next - a < 0.003) next = a;
    this.eased.set(key, next);
    this.moving = true;
    return next;
  }

  private phase(d: Drop, red: boolean): Phase {
    const a = this.t - d.t0;
    const GT = red ? 0 : 0.9;
    return {
      g: GT ? inOut(clamp(a / GT, 0, 1)) : 1,
      on: smooth(0, red ? 0.3 : 0.25, a - GT),
      ring: red ? 0 : clamp((a - GT) / 0.7, 0, 1),
      gathering: a < GT,
      age: a,
    };
  }

  private tick(dt: number): boolean {
    const scene = this.scene;
    if (!scene) return false;
    const red = reducedMotion();
    this.dt = dt;
    this.t += dt;
    if (!red) this.ft += dt;
    this.moving = false;
    if (this.raw) {
      if (red) {
        this.lamp = { ...this.raw };
        this.lampV = { x: 0, y: 0 };
      } else {
        // 光跟着鼠标走的是弹簧：甩一下会带着速度，被打断时保留速度。
        this.lampV.x += (SPRING_K * (this.raw.x - this.lamp.x) - SPRING_C * this.lampV.x) * dt;
        this.lampV.y += (SPRING_K * (this.raw.y - this.lamp.y) - SPRING_C * this.lampV.y) * dt;
        this.lamp.x += this.lampV.x * dt;
        this.lamp.y += this.lampV.y * dt;
        if (Math.hypot(this.lampV.x, this.lampV.y) > 0.5) this.moving = true;
      }
    }
    this.lampA = this.ez("lampA", this.raw ? 1 : 0, this.raw ? 8 : 4);
    const ctx = this.canvas.getContext("2d");
    if (!ctx) return false;
    if (scene.kind === "library") this.drawLibrary(ctx, scene, red);
    else this.drawEmpty(ctx, scene, red);
    return !red || this.moving;
  }

  private begin(ctx: CanvasRenderingContext2D, f: Frame): void {
    const dpr = this.canvas.width / f.width;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.globalAlpha = 1;
    ctx.fillStyle = currentPalette().bg;
    ctx.fillRect(0, 0, f.width, f.height);
  }

  private noiseGrid(threshold: [number, number], withSplats: boolean): void {
    const { nx, ny } = this.dims;
    const t = this.ft;
    const NG = this.ng;
    const SG = this.sg;
    for (let j = 0; j < ny; j++) {
      for (let i = 0; i < nx; i++) {
        const sx = i * 18;
        const sy = j * 18;
        const n1 = fbm(sx * 0.0058 + t * 0.05, sy * 0.0058 - t * 0.018);
        const n2 = fbm(sx * 0.0026 - t * 0.016 + n1 * 1.5, sy * 0.0026 + n1 * 1.1 + 7.3);
        const base = smooth(threshold[0], threshold[1], n2);
        if (withSplats) {
          const s = 1 - Math.exp(-(SG[j * nx + i] ?? 0) / 3);
          NG[j * nx + i] = base * (0.26 + 0.9 * s);
        } else NG[j * nx + i] = base;
      }
    }
  }

  // 每条条目在它现在的位置半径 66px 内加一点雾：雾就是那一堆东西。
  private splat(x: number, y: number, w: number): void {
    const { nx, ny } = this.dims;
    const R = 66;
    const i0 = Math.max(0, Math.floor((x - R) / 18));
    const i1 = Math.min(nx - 1, Math.ceil((x + R) / 18));
    const j0 = Math.max(0, Math.floor((y - R) / 18));
    const j1 = Math.min(ny - 1, Math.ceil((y + R) / 18));
    for (let j = j0; j <= j1; j++) {
      for (let i = i0; i <= i1; i++) {
        const dx = i * 18 - x;
        const dy = j * 18 - y;
        const q = 1 - (dx * dx + dy * dy) / (R * R);
        if (q > 0) this.sg[j * nx + i] = (this.sg[j * nx + i] ?? 0) + w * q * q;
      }
    }
  }

  private sample(x: number, y: number): number {
    const { nx, ny } = this.dims;
    const vv = y / 18;
    const j0 = Math.min(ny - 2, Math.floor(vv));
    const fv = vv - j0;
    const uu = x / 18;
    const i0 = Math.min(nx - 2, Math.floor(uu));
    const fu = uu - i0;
    const b0 = j0 * nx + i0;
    const NG = this.ng;
    return (
      ((NG[b0] ?? 0) * (1 - fu) + (NG[b0 + 1] ?? 0) * fu) * (1 - fv) +
      ((NG[b0 + nx] ?? 0) * (1 - fu) + (NG[b0 + nx + 1] ?? 0) * fu) * fv
    );
  }

  private paint(ctx: CanvasRenderingContext2D, bk: number[][], rgb: string, aMul: number): void {
    for (let lv = 0; lv < 8; lv++) {
      const arr = bk[lv];
      if (!arr || arr.length === 0) continue;
      ctx.fillStyle = `rgba(${rgb},${Math.min(1, (0.16 + lv * 0.08) * aMul).toFixed(3)})`;
      ctx.beginPath();
      const rr = 0.75 + lv * 0.39;
      for (let k = 0; k < arr.length; k += 2) {
        const x = arr[k] ?? 0;
        const y = arr[k + 1] ?? 0;
        ctx.moveTo(x + rr, y);
        ctx.arc(x, y, rr, 0, TAU);
      }
      ctx.fill();
      arr.length = 0;
    }
  }

  private lampTerm(x: number, y: number): number {
    if (this.lampA <= 0.001) return 1;
    const dx = x - this.lamp.x;
    const dy = y - this.lamp.y;
    return 1 - this.lampA * (1 - smooth(LAMP * 0.32, LAMP, Math.sqrt(dx * dx + dy * dy)));
  }

  private drawEmpty(ctx: CanvasRenderingContext2D, scene: EmptyScene, red: boolean): void {
    const f = scene.frame;
    const pal = currentPalette();
    this.begin(ctx, f);
    this.noiseGrid([0.26, 0.8], false);
    for (const d of scene.drops) if (!this.drops.has(d.id)) this.drops.set(d.id, { x: d.x, y: d.y, t0: this.t });
    const drops = scene.drops.flatMap((d) => {
      const dr = this.drops.get(d.id);
      return dr ? [{ d: dr, ph: this.phase(dr, red) }] : [];
    });
    if (drops.some((x) => x.ph.age < 1.8)) this.moving = true;
    const B = scene.clearing;
    const bk = this.buckets;
    const { cols, rows } = this.dims;
    for (let gy = 0; gy < rows; gy++) {
      const y = gy * 9 + 4.5;
      for (let gx = 0; gx < cols; gx++) {
        const x = gx * 9 + 4.5;
        let px = x;
        let py = y;
        const ex = (x - B.cx) / B.rx;
        const ey = (y - B.cy) / B.ry;
        let k = smooth(1, 1.5, Math.sqrt(ex * ex + ey * ey));
        if (k < 0.004) continue;
        k *= this.lampTerm(x, y);
        for (const { d, ph } of drops) {
          const ddx = x - d.x;
          const ddy = y - d.y;
          const dd = Math.sqrt(ddx * ddx + ddy * ddy);
          if (dd > 120) continue;
          // 落下的条目给它的名字留一小块；聚拢时附近的格子往落点滑、变薄。
          k *= 1 - ph.on * (1 - smooth(26, 70, dd));
          if (ph.gathering) {
            const w = 1 - smooth(10, 110, dd);
            px += (d.x - px) * ph.g * w;
            py += (d.y - py) * ph.g * w;
            k *= 1 - ph.g * w * 0.85;
          }
        }
        const dens = this.sample(x, y) * k;
        if (dens < 0.07) continue;
        const lv = Math.min(7, Math.floor(dens * 8));
        bk[lv]?.push(px, py);
      }
    }
    this.paint(ctx, bk, pal.fogRgb, 1);
    for (const { d, ph } of drops) {
      if (ph.on > 0.01) {
        ctx.fillStyle = `rgba(${pal.itemRgb},${(0.95 * ph.on).toFixed(3)})`;
        ctx.beginPath();
        ctx.arc(d.x, d.y, 3, 0, TAU);
        ctx.fill();
      }
      // 圆环是手刚动过的地方，所以它是唯一用强调色的；往外走一次就没了。
      this.ring(ctx, d, ph, pal.accentRgb);
    }
  }

  private ring(ctx: CanvasRenderingContext2D, d: { x: number; y: number }, ph: Phase, rgb: string): void {
    if (ph.ring <= 0 || ph.ring >= 1) return;
    ctx.strokeStyle = `rgba(${rgb},${(0.7 * (1 - ph.ring)).toFixed(3)})`;
    ctx.lineWidth = 1.2;
    ctx.beginPath();
    ctx.arc(d.x, d.y, 5 + 30 * inOut(ph.ring), 0, TAU);
    ctx.stroke();
  }

  private itemU(p: Place, red: boolean): number {
    if (red) return this.P < 0.5 ? 0 : 1;
    return inOut(clamp((this.P - p.delay) / 0.75, 0, 1));
  }

  private drawLibrary(ctx: CanvasRenderingContext2D, scene: LibraryScene, red: boolean): void {
    const f = scene.frame;
    const pal = currentPalette();
    const scale = (f.width - f.mapX) / 1192;
    // 一个线性的钟管整次重排，每条在它自己那一段上缓动。
    const goal = scene.arrangement === "source" ? 1 : 0;
    const D = red ? 0.4 : 0.9;
    if (this.P !== goal) {
      this.P = goal > this.P ? Math.min(1, this.P + this.dt / D) : Math.max(0, this.P - this.dt / D);
      this.moving = true;
    }
    const P = this.P;
    const inTopic = 1 - smooth(0, 0.5, P);
    const tA = inTopic;
    const sA = smooth(0.5, 1, P);

    const pos = new Map<string, { x: number; y: number }>();
    for (const p of scene.places) pos.set(p.id, placeAt(p, this.itemU(p, red)));

    // 关注的主题把名下几条折成列表：点变成行首的圆点。
    const anchors = new Map<string, { x: number; y: number; rp: number }>();
    const masks: (Rect & { a: number; soft: number })[] = [];
    const focus: { x: number; y: number; rad: number; a: number; glow: number }[] = [];
    for (const r of scene.regions) {
      const on = scene.followed.has(r.id);
      const fa = this.ez(`fa:${r.id}`, on ? 1 : 0, on ? 5 : 8);
      const rh = scene.railHover === r.id ? 1 : 0;
      const ha = this.ez(`rh:${r.id}`, rh, rh ? 12 : 16);
      const clear = Math.max(fa, 0.7 * ha) * inTopic;
      if (clear > 0.002) focus.push({ x: r.x, y: r.y, rad: r.rad, a: clear, glow: fa * inTopic });
      const rp0 = fa * inTopic;
      if (rp0 < 0.002) continue;
      const rows = foldRows(r, scene.places, f);
      const first = rows[0];
      const lastRow = rows[rows.length - 1];
      if (!first || !lastRow) continue;
      masks.push({ x0: first.x - 8, y0: first.y - 6, x1: first.x + first.w + 8, y1: lastRow.y + 32, a: rp0, soft: 12 });
      for (const row of rows) {
        const rp = smooth(0, 1, (rp0 - row.index * 0.08) / 0.6);
        const A = pos.get(row.place.id) ?? { x: row.x, y: row.y };
        const ty = row.y + 12;
        anchors.set(row.place.id, { x: A.x + (row.x + 15 - A.x) * rp, y: A.y + (ty - A.y) * rp, rp });
      }
    }
    for (const p of scene.places) {
      if (anchors.has(p.id)) continue;
      const A = pos.get(p.id) ?? { x: p.tx, y: p.ty };
      anchors.set(p.id, { x: A.x, y: A.y, rp: 0 });
    }
    const nameMask = (c: Region, a: number) => {
      const fs = nameSize(c.rad, scale);
      const w = textWidth(c.name, fs) / 2 + 12;
      masks.push({ x0: c.x - w, y0: c.y - fs * 0.5 - 6, x1: c.x + w, y1: c.y + fs * 0.5 + 6, a, soft: 12 });
    };
    if (tA > 0.01) for (const r of scene.regions) nameMask(r, tA);
    if (sA > 0.01) for (const s of scene.sources) nameMask(s, sA);

    // 悬停：12px 以内最近的点，或鼠标落在一个主题名字那一片里。
    let hovI: string | null = null;
    if (this.raw) {
      let bd = 12;
      for (const p of scene.places) {
        const A = anchors.get(p.id);
        if (!A || A.rp > 0.5) continue;
        const d = Math.hypot(A.x - this.raw.x, A.y - this.raw.y);
        if (d < bd) {
          bd = d;
          hovI = p.id;
        }
      }
    }
    if (hovI !== this.hovI) {
      this.hovI = hovI;
      this.events.hover?.(hovI);
    }
    const hov = hovI ?? scene.rowHover;
    let hovT: string | null = null;
    if (this.raw && P < 0.5 && !hov) {
      const keep = scene.regions.find((r) => r.id === this.hovT);
      if (keep && Math.hypot(this.raw.x - keep.x, this.raw.y - keep.y) < keep.rad * 1.25) hovT = keep.id;
      else {
        for (const r of scene.regions) {
          if (Math.hypot(this.raw.x - r.x, this.raw.y - r.y) < r.rad * 0.9) {
            hovT = r.id;
            break;
          }
        }
      }
    }
    if (hovT !== this.hovT) {
      this.hovT = hovT;
      this.events.hoverTopic?.(hovT);
    }

    // 悬停那一行字下面的雾让开。
    if (hov) this.lastHov = hov;
    const labelOn = !!hov && (anchors.get(hov)?.rp ?? 0) <= 0.5;
    const hlA = this.ez("hl", labelOn ? 1 : 0, labelOn ? 18 : 26);
    let hl: (Rect & { a: number }) | null = null;
    if (this.lastHov && hlA > 0.01) {
      const p = scene.places.find((x) => x.id === this.lastHov);
      const A = p ? anchors.get(p.id) : undefined;
      if (p && A) {
        const right = scene.panel ? scene.panel.x0 - 12 : f.width - 12;
        hl = { ...labelBox(A.x, A.y, p.title, p.platform, right), a: hlA };
      }
    }

    const qa = this.ez("qa", scene.hits.size > 0 ? 1 : 0, scene.hits.size > 0 ? 10 : 14);
    const panelOn = scene.panel !== null;
    const panelA = this.ez("panelA", panelOn ? 1 : 0, panelOn ? 14 : 22);
    const pH = this.heal("panelH", panelA);
    if (scene.panel) this.lastPanel = scene.panel;
    const Pn = this.lastPanel;

    this.begin(ctx, f);
    this.sg.fill(0);
    // 新落下的条目从聚拢以后才开始加雾。
    for (const id of scene.landing) {
      const A = pos.get(id);
      if (A && !this.drops.has(id)) this.drops.set(id, { x: A.x, y: A.y, t0: this.t });
    }
    const phases = new Map<string, Phase>();
    for (const [id, d] of this.drops) {
      const ph = this.phase(d, red);
      if (ph.age > 3.4 && !scene.landing.has(id)) this.drops.delete(id);
      else {
        phases.set(id, ph);
        if (ph.age < 1.8) this.moving = true;
      }
    }
    for (const p of scene.places) {
      const w = phases.get(p.id)?.on ?? 1;
      if (w <= 0) continue;
      if (red && P > 0 && P < 1) {
        this.splat(p.tx, p.ty, w * (1 - P));
        this.splat(p.sx, p.sy, w * P);
      } else {
        const A = pos.get(p.id);
        if (A) this.splat(A.x, A.y, w);
      }
    }
    this.noiseGrid([0.18, 0.76], true);

    const { cols, rows } = this.dims;
    const ig = this.ig;
    ig.fill(0);
    for (const A of anchors.values()) {
      const gx = Math.round((A.x - 4.5) / 9);
      const gy = Math.round((A.y - 4.5) / 9);
      if (gx >= 0 && gx < cols && gy >= 0 && gy < rows) ig[gy * cols + gx] = 1;
    }
    const hitAnchors: { x: number; y: number }[] = [];
    if (qa > 0.01) for (const id of scene.hits) {
      const A = anchors.get(id);
      if (A) hitAnchors.push(A);
    }
    const dropList = [...phases.entries()].flatMap(([id, ph]) => {
      const A = pos.get(id);
      return A ? [{ A, ph }] : [];
    });

    const bN = this.buckets;
    const bA = this.accentBuckets;
    const gx0 = Math.floor(f.mapX / 9) + 1;
    for (let gy = 0; gy < rows; gy++) {
      const y = gy * 9 + 4.5;
      for (let gx = gx0; gx < cols; gx++) {
        if (ig[gy * cols + gx]) continue;
        const x = gx * 9 + 4.5;
        let px = x;
        let py = y;
        let k = 1;
        let glow = 0;
        if (pH > 0.002 && Pn) k *= 1 - pH * rectFalloff(Pn, x, y, 14);
        for (const M of masks) k *= 1 - M.a * rectFalloff(M, x, y, M.soft);
        if (hl) k *= 1 - hl.a * rectFalloff(hl, x, y, 10);
        if (k < 0.004) continue;
        k *= this.lampTerm(x, y);
        for (const F of focus) {
          const fd = Math.hypot(x - F.x, y - F.y);
          k *= 1 - F.a * (1 - smooth(F.rad * 0.8, F.rad * 1.35, fd));
          const g2 = F.glow * (1 - smooth(F.rad * 0.95, F.rad * 2, fd));
          if (g2 > glow) glow = g2;
        }
        // 搜到的条目各在雾里开一个小口子。
        for (const HA of hitAnchors) {
          const hd = Math.hypot(x - HA.x, y - HA.y);
          if (hd < 60) k *= 1 - qa * (1 - smooth(14, 56, hd));
        }
        for (const { A, ph } of dropList) {
          if (!ph.gathering) continue;
          const ddx = x - A.x;
          const ddy = y - A.y;
          const dd = Math.sqrt(ddx * ddx + ddy * ddy);
          if (dd > 120) continue;
          const w = 1 - smooth(10, 110, dd);
          px += (A.x - px) * ph.g * w;
          py += (A.y - py) * ph.g * w;
          k *= 1 - ph.g * w * 0.85;
        }
        const dens = this.sample(x, y) * k;
        if (dens < 0.07) continue;
        const lv = Math.min(7, Math.floor(dens * 8));
        (glow > 0.04 && hash2(gx * 3 + 1, gy * 7 + 5) < glow ? bA : bN)[lv]?.push(px, py);
      }
    }
    this.paint(ctx, bN, pal.fogRgb, 1);
    this.paint(ctx, bA, pal.accentRgb, 1.25);

    // 条目是能指着的点。关注的主题里的条目按来源摆时用强调色；面板里开着的那条带一圈中性的环。
    let lampN = 0;
    for (const p of scene.places) {
      const A = anchors.get(p.id);
      if (!A) continue;
      const ph = phases.get(p.id);
      const vis = ph ? ph.on : 1;
      if (A.rp < 0.5 && Math.hypot(A.x - this.lamp.x, A.y - this.lamp.y) < LAMP * 0.8) lampN++;
      if (vis <= 0.01) {
        if (ph) this.ring(ctx, A, ph, pal.accentRgb);
        continue;
      }
      const near = this.lampA * (1 - smooth(LAMP * 0.45, LAMP * 0.95, Math.hypot(A.x - this.lamp.x, A.y - this.lamp.y)));
      const isH = hov === p.id;
      const isS = scene.selected === p.id;
      const hA = this.ez(`dh:${p.id}`, isH ? 1 : 0, isH ? 16 : 24);
      const sA2 = this.ez(`ds:${p.id}`, isS ? 1 : 0, isS ? 14 : 22);
      const fo = p.topicIds.some((t) => scene.followed.has(t));
      const railOn = scene.railHover !== null && p.topicIds.includes(scene.railHover) && P > 0.5;
      const rgb = (fo && P > 0.02) || railOn ? pal.accentRgb : pal.itemRgb;
      const isHit = scene.hits.has(p.id);
      const qd = this.ez(`qd:${p.id}`, scene.searching && !isHit ? 1 : 0, 12);
      const qh = this.ez(`qh:${p.id}`, isHit ? 1 : 0, 14);
      const a = (0.6 + 0.35 * Math.max(near, hA, sA2, A.rp, qh)) * vis * (1 - 0.65 * qd);
      const rad = 2.5 + 0.7 * Math.max(near, hA);
      ctx.fillStyle = `rgba(${rgb},${a.toFixed(3)})`;
      ctx.beginPath();
      ctx.arc(A.x, A.y, rad, 0, TAU);
      ctx.fill();
      const ring = Math.max(hA, sA2, 0.55 * qh);
      if (ring > 0.01) {
        ctx.strokeStyle = `rgba(${pal.itemRgb},${(0.85 * ring).toFixed(3)})`;
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.arc(A.x, A.y, rad + 2.5 + 2.5 * ring, 0, TAU);
        ctx.stroke();
      }
      if (ph) this.ring(ctx, A, ph, pal.accentRgb);
    }
    const n = this.lampA > 0.5 ? lampN : null;
    if (n !== this.lampN) {
      this.lampN = n;
      this.events.lampCount?.(n);
    }
  }

  private lastPanel: Rect | null = null;
}
