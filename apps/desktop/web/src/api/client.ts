import { daemonConnection } from "../shell";
import { DEFAULT_API_BASE, DEV_PROXY_PATH } from "./endpoints";

const REQUEST_TIMEOUT_MS = 8000;

/** 连不上内核：没启动、被拒绝或超时。 */
export class DaemonUnreachableError extends Error {
  /** 桌面壳说的原因，给用户看的一句话（比如端口被别的程序占着）；没有就是 undefined。 */
  readonly reason: string | undefined;

  constructor(cause: unknown, reason?: string) {
    super("daemon unreachable", { cause });
    this.name = "DaemonUnreachableError";
    this.reason = reason;
  }
}

/**
 * 内核回了非 2xx。`code` 是 problem+json 里的 `code`（SPEC §0.3），回的不是 problem 时没有；
 * `hint` 是给人看的一句话，按设置的语言写，界面照它显示；`message` 是 `detail`，给日志。
 */
export class ApiError extends Error {
  readonly status: number;
  readonly code: string | undefined;
  readonly hint: string | undefined;

  constructor(status: number, message: string, code?: string, hint?: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
    this.hint = hint;
  }
}

/** 内核的兜底路由回的那一句：这个内核还没有这条路（`routes.rs` 的 fallback）。 */
const NO_SUCH_ROUTE = "no such route";

/** 这个内核还没有这条接口：兜底的 404，或这条路不收这个方法的 405。 */
export function isNotServed(error: unknown): boolean {
  return error instanceof ApiError && (error.status === 405 || (error.status === 404 && error.message === NO_SUCH_ROUTE));
}

function problemOf(text: string): { detail?: string; code?: string; hint?: string } {
  try {
    const body: unknown = JSON.parse(text);
    if (typeof body !== "object" || body === null) return {};
    return {
      ...("detail" in body && typeof body.detail === "string" ? { detail: body.detail } : {}),
      ...("code" in body && typeof body.code === "string" ? { code: body.code } : {}),
      ...("hint" in body && typeof body.hint === "string" ? { hint: body.hint } : {}),
    };
  } catch {
    // 不是 JSON（例如代理回的纯文字），整段当原因。
    return {};
  }
}

function apiError(status: number, statusText: string, text: string): ApiError {
  const problem = problemOf(text);
  return new ApiError(status, problem.detail ?? (text || statusText), problem.code, problem.hint);
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
    // 壳回的错是给用户看的一句话。
    throw new DaemonUnreachableError(error, typeof error === "string" ? error : undefined);
  });
  return connecting;
}

/** 连不上或令牌不对（内核重启过，令牌换了）：下次再问桌面壳，它重新核对、给新的令牌。 */
export function forgetConnection(): void {
  connecting = null;
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
  init: {
    method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
    body?: unknown;
    query?: Record<string, string>;
    /** 调插件的慢请求（列来源）要等得久一些。 */
    timeoutMs?: number;
  } = {},
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
      signal: AbortSignal.timeout(init.timeoutMs ?? REQUEST_TIMEOUT_MS),
    });
  } catch (error) {
    forgetConnection();
    throw new DaemonUnreachableError(error);
  }

  if (unreachable(response)) throw new DaemonUnreachableError(new Error("proxy could not reach the daemon"));
  if (response.status === 401) forgetConnection();
  if (!response.ok) {
    throw apiError(response.status, response.statusText, await response.text());
  }
  // 204（退出登录、撤销令牌、不要这次导入）没有内容；调用处的类型写 `void`。
  if (response.status === 204) return undefined as T;
  // 响应形状由 endpoints.ts 的类型声明负责，这里不再校验。
  const body: unknown = await response.json();
  return body as T;
}

/** 交文件的时限：内核那边 30 分钟，这边多等一分钟，让内核的那句先到。 */
const UPLOAD_TIMEOUT_MS = 31 * 60_000;

/**
 * 原样交一个文件，边交边报交了多少字节。fetch 报不了上传了多少，这里用 XMLHttpRequest。
 * `signal` 停下时回 `AbortError`；别的失败和 `request` 一样：连不上是 `DaemonUnreachableError`，非 2xx 是 `ApiError`。
 */
export async function upload<T>(
  path: string,
  file: Blob,
  query: Record<string, string>,
  onSent: (bytes: number) => void,
  signal: AbortSignal,
): Promise<T> {
  const c = await connection();
  const url = urlOf(c, path);
  for (const [key, value] of Object.entries(query)) url.searchParams.set(key, value);
  signal.throwIfAborted();
  return new Promise<T>((done, fail) => {
    const xhr = new XMLHttpRequest();
    xhr.open("POST", url);
    for (const [key, value] of Object.entries(authHeaders(c))) xhr.setRequestHeader(key, value);
    xhr.setRequestHeader("Content-Type", "application/octet-stream");
    xhr.timeout = UPLOAD_TIMEOUT_MS;
    xhr.upload.onprogress = (e) => onSent(e.loaded);
    const stop = () => xhr.abort();
    signal.addEventListener("abort", stop, { once: true });
    const settle = () => signal.removeEventListener("abort", stop);
    xhr.onabort = () => {
      settle();
      fail(signal.reason instanceof Error ? signal.reason : new DOMException("stopped", "AbortError"));
    };
    const lost = () => {
      settle();
      forgetConnection();
      fail(new DaemonUnreachableError(new Error("upload did not reach the daemon")));
    };
    xhr.onerror = lost;
    xhr.ontimeout = lost;
    xhr.onload = () => {
      settle();
      if (xhr.status === 502 && xhr.getResponseHeader(PROXY_UNREACHABLE_HEADER) !== null) {
        fail(new DaemonUnreachableError(new Error("proxy could not reach the daemon")));
        return;
      }
      if (xhr.status === 401) forgetConnection();
      if (xhr.status < 200 || xhr.status >= 300) {
        fail(apiError(xhr.status, xhr.statusText, xhr.responseText));
        return;
      }
      try {
        // 响应形状由 endpoints.ts 的类型声明负责，这里不再校验。
        done(JSON.parse(xhr.responseText) as T);
      } catch (error) {
        fail(error);
      }
    };
    xhr.send(file);
  });
}
