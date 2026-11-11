// 收下浮层后面那层：一个没对上焦的窗口，用点画出来（Capture.dc.html 的 drawBackdrop）。只画一次，换主题时重画。
import { fbm, smooth } from "../fog/noise";
import { currentPalette } from "../theme/apply";

function soft(x: number, y: number, x0: number, y0: number, x1: number, y1: number, blur: number): number {
  const dx = Math.max(x0 - x, 0, x - x1);
  const dy = Math.max(y0 - y, 0, y - y1);
  return 1 - smooth(0, blur, Math.sqrt(dx * dx + dy * dy));
}

export function drawBackdrop(canvas: HTMLCanvasElement): void {
  const W = canvas.clientWidth;
  const H = canvas.clientHeight;
  const dpr = Math.min(2, window.devicePixelRatio || 1);
  canvas.width = Math.round(W * dpr);
  canvas.height = Math.round(H * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  // 画布原型按 1440×900 摆，这里按窗口大小缩放。
  const sx = W / 1440;
  const sy = H / 900;
  const pal = currentPalette();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, W, H);
  const S = 12;
  const B: number[][] = [[], [], [], [], [], []];
  for (let y = S / 2; y < H; y += S) {
    for (let x = S / 2; x < W; x += S) {
      const u = x / sx;
      const w = y / sy;
      let v = 0.05 + 0.07 * fbm(u * 0.004, w * 0.005);
      v += 0.2 * soft(u, w, 170, 130, 800, 610, 70);
      v += 0.12 * soft(u, w, 640, 330, 1290, 790, 80);
      v += 0.1 * soft(u, w, 1050, 96, 1330, 250, 50);
      v += 0.05 * soft(u, w, 230, 200, 560, 214, 22) + 0.05 * soft(u, w, 230, 250, 700, 262, 22) + 0.05 * soft(u, w, 230, 296, 640, 308, 22);
      if (v < 0.09) continue;
      B[Math.min(5, Math.floor(v * 14))]?.push(x, y);
    }
  }
  for (let lv = 1; lv < 6; lv++) {
    const arr = B[lv];
    if (!arr || arr.length === 0) continue;
    const r = 0.7 + lv * 0.32;
    ctx.fillStyle = `rgba(${pal.fogRgb},${(0.1 + lv * 0.07).toFixed(3)})`;
    ctx.beginPath();
    for (let i = 0; i < arr.length; i += 2) {
      const x = arr[i] ?? 0;
      const y = arr[i + 1] ?? 0;
      ctx.moveTo(x + r, y);
      ctx.arc(x, y, r, 0, Math.PI * 2);
    }
    ctx.fill();
  }
}
