import { daemonConnection } from "../shell";
import { DEFAULT_API_BASE, DEV_PROXY_PATH } from "./endpoints";

const REQUEST_TIMEOUT_MS = 8000;

/** 连不上内核：没启动、被拒绝或超时。 */
export class DaemonUnreachableError extends Error {
  constructor(cause: unknown) {
    super("daemon unreachable", { cause });
    this.name = "DaemonUnreachableError";
  }
}

/** 内核回了非 2xx。 */
export class ApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

/** 内核在哪、带什么令牌。开发服务器的代理在服务端带令牌，这里是 null。 */
export interface Connection {
  base: string;
  token: string | null;
}

/** 开发服务器的代理连不上内核时回 502，带着这个头（vite.config.ts）。 */
export const PROXY_UNREACHABLE_HEADER = "x-usuyo-unreachable";

let connecting: Promise<Connection> | null = null;

async function resolve(): Promise<Connection> {
  // 开发时（浏览器标签页或 `tauri dev`）都经 Vite 的代理走，同源。
  if (import.meta.env.DEV) return { base: new URL(DEV_PROXY_PATH, location.origin).href, token: null };
  const shell = await daemonConnection();
  if (shell) return shell;
  return { base: import.meta.env.VITE_USUYO_API ?? DEFAULT_API_BASE, token: null };
}

/** 第一次问桌面壳（它等内核起来才回），之后用同一个；问失败了下次再问。 */
export function connection(): Promise<Connection> {
  connecting ??= resolve().catch((error: unknown) => {
    connecting = null;
    throw new DaemonUnreachableError(error);
  });
  return connecting;
}

/** `base` 下的地址：`base` 可以带路径（开发代理的 `/api`）。 */
export function urlOf(c: Connection, path: string): URL {
  return new URL(path.replace(/^\//, ""), c.base.endsWith("/") ? c.base : `${c.base}/`);
}

export function authHeaders(c: Connection): Record<string, string> {
  return c.token === null ? {} : { Authorization: `Bearer ${c.token}` };
}

export function unreachable(response: Response): boolean {
  return response.status === 502 && response.headers.has(PROXY_UNREACHABLE_HEADER);
}

export async function request<T>(
  path: string,
  init: { method?: "GET" | "POST"; body?: unknown; query?: Record<string, string> } = {},
): Promise<T> {
  const c = await connection();
  const url = urlOf(c, path);
  for (const [key, value] of Object.entries(init.query ?? {})) {
    url.searchParams.set(key, value);
  }

  let response: Response;
  try {
    response = await fetch(url, {
      method: init.method ?? "GET",
      headers: {
        ...authHeaders(c),
        ...(init.body === undefined ? {} : { "Content-Type": "application/json" }),
      },
      body: init.body === undefined ? undefined : JSON.stringify(init.body),
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
  } catch (error) {
    throw new DaemonUnreachableError(error);
  }

  if (unreachable(response)) throw new DaemonUnreachableError(new Error("proxy could not reach the daemon"));
  if (!response.ok) {
    throw new ApiError(response.status, (await response.text()) || response.statusText);
  }
  // 响应形状由 endpoints.ts 的类型声明负责，这里不再校验。
  const body: unknown = await response.json();
  return body as T;
}
