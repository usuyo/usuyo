// 画布原型里的噪声和缓动，数一个不改（usuyo-canvas/project/Home.dc.html）。

export function hash2(x: number, y: number): number {
  let n = (Math.imul(x | 0, 374761393) + Math.imul(y | 0, 668265263)) | 0;
  n = Math.imul(n ^ (n >>> 13), 1274126177);
  n = n ^ (n >>> 16);
  return (n >>> 0) / 4294967296;
}

export function vnoise(x: number, y: number): number {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const xf = x - xi;
  const yf = y - yi;
  const u = xf * xf * (3 - 2 * xf);
  const v = yf * yf * (3 - 2 * yf);
  const a = hash2(xi, yi);
  const b = hash2(xi + 1, yi);
  const c = hash2(xi, yi + 1);
  const d = hash2(xi + 1, yi + 1);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}

/** 三个八度：0.55 / 0.30 / 0.15。 */
export function fbm(x: number, y: number): number {
  return (
    0.55 * vnoise(x, y) +
    0.3 * vnoise(x * 2.03 + 17.1, y * 2.03 - 9.2) +
    0.15 * vnoise(x * 4.1 - 3.7, y * 4.1 + 11.3)
  );
}

export function smooth(e0: number, e1: number, x: number): number {
  let t = (x - e0) / (e1 - e0);
  t = t < 0 ? 0 : t > 1 ? 1 : t;
  return t * t * (3 - 2 * t);
}

export function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}

export function inOut(p: number): number {
  return 0.5 - 0.5 * Math.cos(Math.PI * p);
}

export function strHash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

export function r01(s: string): number {
  return strHash(s) / 4294967296;
}

/** 指数趋近：每秒按 rate 靠近目标，差得很小就直接到。 */
export function approach(v: number, target: number, rate: number, dt: number): number {
  if (v === target) return v;
  const next = v + (target - v) * (1 - Math.exp(-rate * dt));
  return Math.abs(target - next) < 0.003 ? target : next;
}
