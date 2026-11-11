import { DEFAULT_API_BASE } from "./endpoints";

export const API_BASE: string = import.meta.env.VITE_USUYO_API ?? DEFAULT_API_BASE;

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

export async function request<T>(
  path: string,
  init: { method?: "GET" | "POST"; body?: unknown; query?: Record<string, string> } = {},
): Promise<T> {
  const url = new URL(path, API_BASE);
  for (const [key, value] of Object.entries(init.query ?? {})) {
    url.searchParams.set(key, value);
  }

  let response: Response;
  try {
    response = await fetch(url, {
      method: init.method ?? "GET",
      headers: init.body === undefined ? undefined : { "Content-Type": "application/json" },
      body: init.body === undefined ? undefined : JSON.stringify(init.body),
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
  } catch (error) {
    throw new DaemonUnreachableError(error);
  }

  if (!response.ok) {
    throw new ApiError(response.status, (await response.text()) || response.statusText);
  }
  // 响应形状由 endpoints.ts 的类型声明负责，这里不再校验。
  const body: unknown = await response.json();
  return body as T;
}
