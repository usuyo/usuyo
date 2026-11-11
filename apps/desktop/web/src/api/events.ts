// 内核推的事件：一条连接，收到就让对应的数据重新拉一次，地图、列表和开着的条目跟着变。
// 事件流要带令牌，EventSource 带不了请求头，所以用 fetch 读流。
import type { QueryClient } from "@tanstack/react-query";
import { authHeaders, connection, forgetConnection, unreachable, urlOf } from "./client";
import { paths, type LibraryEvent } from "./endpoints";
import { queryKeys } from "./hooks";
import { usingMockLibrary } from "./source";
import { SseParser } from "./sse";

const RETRY_MIN_MS = 1000;
const RETRY_MAX_MS = 30_000;

export function applyEvent(queryClient: QueryClient, e: LibraryEvent): void {
  const refresh = (queryKey: readonly unknown[]) => void queryClient.invalidateQueries({ queryKey });
  switch (e.type) {
    case "topic-updated":
      refresh(queryKeys.topics);
      refresh(queryKeys.map);
      return;
    case "item-fetch-failed":
      refresh(queryKeys.item(e.id));
      return;
    default:
      // 收下、取到内容、改了、删掉：地图、搜索结果和这一条都可能变了。
      refresh(queryKeys.map);
      refresh(["search"]);
      refresh(queryKeys.item(e.id));
  }
}

/** 连上事件流，断了就过一会儿再连；连上时（含重连）全部重拉一次，补上断开时漏掉的。回一个停下的函数。 */
export function startLiveUpdates(queryClient: QueryClient): () => void {
  const stop = new AbortController();
  void (async () => {
    let delay = RETRY_MIN_MS;
    let connectedBefore = false;
    while (!stop.signal.aborted && !usingMockLibrary()) {
      try {
        const c = await connection();
        const response = await fetch(urlOf(c, paths.events), { headers: authHeaders(c), signal: stop.signal });
        if (response.ok && response.body && !unreachable(response)) {
          if (connectedBefore) void queryClient.invalidateQueries();
          connectedBefore = true;
          delay = RETRY_MIN_MS;
          await read(response.body, (e) => applyEvent(queryClient, e));
        }
        // 流断了或进不去：内核可能重启过，下次重新问连接。
        forgetConnection();
      } catch (error) {
        if (stop.signal.aborted) return;
        forgetConnection();
        console.info("usuyo: 事件流断了，过会儿再连", error);
      }
      await sleep(delay, stop.signal);
      delay = Math.min(delay * 2, RETRY_MAX_MS);
    }
  })();
  return () => stop.abort();
}

const EVENT_TYPES: ReadonlySet<string> = new Set<LibraryEvent["type"]>([
  "item-added",
  "item-updated",
  "item-fetch-failed",
  "item-trashed",
  "topic-updated",
]);

/** 只看刷新要用的 `type` 和 `id`；别的字段照 endpoints.ts。 */
function isLibraryEvent(v: unknown): v is LibraryEvent {
  return (
    typeof v === "object" &&
    v !== null &&
    "type" in v &&
    "id" in v &&
    typeof v.type === "string" &&
    EVENT_TYPES.has(v.type) &&
    typeof v.id === "string"
  );
}

async function read(body: ReadableStream<Uint8Array>, on: (e: LibraryEvent) => void): Promise<void> {
  const parser = new SseParser();
  const decoder = new TextDecoder();
  const reader = body.getReader();
  for (;;) {
    const { done, value } = await reader.read();
    if (done) return;
    for (const m of parser.push(decoder.decode(value, { stream: true }))) {
      const e: unknown = JSON.parse(m.data);
      if (isLibraryEvent(e)) on(e);
      else console.info("usuyo: 不认识的事件", m.event);
    }
  }
}

function sleep(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    const id = setTimeout(resolve, ms);
    signal.addEventListener(
      "abort",
      () => {
        clearTimeout(id);
        resolve();
      },
      { once: true },
    );
  });
}
