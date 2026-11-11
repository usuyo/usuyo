/// <reference types="node" />
import { readFileSync } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { homedir } from "node:os";
import { join } from "node:path";
import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import type { Plugin } from "vite";
import { defineConfig } from "vitest/config";

// 端口固定：src-tauri/tauri.conf.json 的 devUrl 指向它。
const DEV_PORT = 5173;
const DAEMON = `http://127.0.0.1:${process.env.USUYO_PORT ?? "47615"}`;
// 和 src/api/endpoints.ts 的 DEV_PROXY_PATH、client.ts 的 PROXY_UNREACHABLE_HEADER 一致。
const PROXY_PATH = "/api";
const UNREACHABLE_HEADER = "x-usuyo-unreachable";

/** 按机器的目录，同内核（SPEC §0.1）：`USUYO_HOME`，没有就是系统的位置。 */
function machineRoot(): string {
  if (process.env.USUYO_HOME) return process.env.USUYO_HOME;
  if (process.platform === "darwin") return join(homedir(), "Library/Application Support/usuyo");
  if (process.platform === "win32") return join(process.env.LOCALAPPDATA ?? join(homedir(), "AppData/Local"), "usuyo");
  return join(process.env.XDG_STATE_HOME ?? join(homedir(), ".local/state"), "usuyo");
}

/** 内核第一次跑时才生成令牌，所以每个请求现读一次；还没有就不带，内核回 401。 */
function token(): string | null {
  try {
    return readFileSync(join(machineRoot(), "api-token"), "utf8").trim() || null;
  } catch {
    return null;
  }
}

/**
 * 代理会替请求带上令牌，所以只给这个开发服务器自己的页面用：
 * Host 得是本机这个端口，浏览器标了同源（Sec-Fetch-Site），带了 Origin 的也得是自己。
 */
function sameOriginOnly(): Plugin {
  const own = [`localhost:${DEV_PORT}`, `127.0.0.1:${DEV_PORT}`];
  return {
    name: "usuyo-api-same-origin",
    configureServer(server) {
      server.middlewares.use(PROXY_PATH, (req: IncomingMessage, res: ServerResponse, next: () => void) => {
        const host = req.headers.host ?? "";
        const origin = req.headers.origin;
        const ok =
          own.includes(host) &&
          req.headers["sec-fetch-site"] === "same-origin" &&
          (origin === undefined || origin === `http://${host}`);
        if (ok) return next();
        res.statusCode = 403;
        res.end("forbidden");
      });
    },
  };
}

export default defineConfig({
  plugins: [sameOriginOnly(), tailwindcss(), react(), babel({ presets: [reactCompilerPreset()] })],
  server: {
    port: DEV_PORT,
    strictPort: true,
    proxy: {
      [PROXY_PATH]: {
        target: DAEMON,
        changeOrigin: true,
        rewrite: (path) => path.slice(PROXY_PATH.length) || "/",
        configure(proxy) {
          proxy.on("proxyReq", (proxyReq) => {
            // 内核只认桌面壳的 Origin；代理发出的请求和命令行一样，不带 Origin，带令牌。
            proxyReq.removeHeader("origin");
            proxyReq.removeHeader("referer");
            const t = token();
            if (t) proxyReq.setHeader("authorization", `Bearer ${t}`);
          });
          proxy.on("error", (_error, _req, res) => {
            // 内核没在跑：让前端照「连不上」办，开发时换成样例库。
            if ("writeHead" in res && !res.headersSent) {
              res.writeHead(502, { [UNREACHABLE_HEADER]: "1", "content-type": "text/plain" });
              res.end("daemon unreachable");
            }
          });
        },
      },
    },
  },
  test: { environment: "jsdom" },
});
