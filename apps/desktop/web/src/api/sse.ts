// 事件流（text/event-stream）的解析：一块块喂进来，切成一条条消息。
// 只认 `event:` 和 `data:`，`:` 开头的是保活注释；不处理 `id:`、`retry:`（内核不发）。

export interface SseMessage {
  event: string;
  data: string;
}

export class SseParser {
  private buffer = "";
  private event = "";
  private data: string[] = [];

  push(chunk: string): SseMessage[] {
    this.buffer += chunk;
    const out: SseMessage[] = [];
    let nl = this.buffer.indexOf("\n");
    while (nl >= 0) {
      const line = this.buffer.slice(0, nl).replace(/\r$/, "");
      this.buffer = this.buffer.slice(nl + 1);
      this.line(line, out);
      nl = this.buffer.indexOf("\n");
    }
    return out;
  }

  private line(line: string, out: SseMessage[]): void {
    if (line === "") {
      if (this.data.length > 0) out.push({ event: this.event || "message", data: this.data.join("\n") });
      this.event = "";
      this.data = [];
      return;
    }
    if (line.startsWith(":")) return;
    const colon = line.indexOf(":");
    const field = colon < 0 ? line : line.slice(0, colon);
    const value = colon < 0 ? "" : line.slice(colon + 1).replace(/^ /, "");
    if (field === "event") this.event = value;
    else if (field === "data") this.data.push(value);
  }
}
