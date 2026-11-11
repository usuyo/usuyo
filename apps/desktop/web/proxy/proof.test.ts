// @vitest-environment node
/// <reference types="node" />
import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { afterEach, describe, expect, it } from "vitest";
import { proofOf, prove, verifyProof } from "./proof";

const servers: { close(): void }[] = [];
afterEach(() => {
  for (const s of servers.splice(0)) s.close();
});

/** 起一个本机服务，记下它收到的每个请求（头和请求体）。 */
async function serve(answer: (req: IncomingMessage, body: string, res: ServerResponse) => void) {
  const seen: string[] = [];
  const server = createServer((req, res) => {
    let body = "";
    req.on("data", (chunk: Buffer) => (body += chunk.toString()));
    req.on("end", () => {
      seen.push(`${req.method} ${req.url}\n${JSON.stringify(req.headers)}\n${body}`);
      answer(req, body, res);
    });
  });
  servers.push(server);
  await new Promise<void>((done) => server.listen(0, "127.0.0.1", done));
  const address = server.address();
  if (address === null || typeof address === "string") throw new Error("no port");
  return { port: address.port, seen };
}

describe("daemon proof", () => {
  it("matches the Rust side on a fixed vector", () => {
    // 同 crates/usuyo-api/src/handshake.rs 的 fixed_vector。
    expect(proofOf("test-token", 47615, "0".repeat(64)).toString("hex")).toBe(
      "fc597e72ae56d3d9ba293b2e557a6df8a9d0736968d89fed4b7bc79571475776",
    );
  });

  it("verifies with the right token only", () => {
    const nonce = "ab".repeat(32);
    const proof = proofOf("right", 47615, nonce).toString("hex");
    expect(verifyProof("right", 47615, nonce, proof)).toBe(true);
    expect(verifyProof("wrong", 47615, nonce, proof)).toBe(false);
    expect(verifyProof("right", 47616, nonce, proof)).toBe(false);
    expect(verifyProof("right", 47615, nonce, "zz")).toBe(false);
  });

  it("accepts a listener that signs with the token", async () => {
    const token = "the-token";
    const { port } = await serve((_req, body, res) => {
      const parsed: unknown = JSON.parse(body);
      const nonce = typeof parsed === "object" && parsed !== null && "nonce" in parsed ? String(parsed.nonce) : "";
      res.setHeader("content-type", "application/json");
      res.end(JSON.stringify({ proof: proofOf(token, port, nonce).toString("hex") }));
    });
    expect(await prove(port, token)).toBe("daemon");
  });

  it("rejects an impostor that answers everything, without sending it the token", async () => {
    const token = "secret-token-for-this-test";
    const { port, seen } = await serve((_req, _body, res) => {
      res.setHeader("content-type", "application/json");
      res.end(JSON.stringify({ status: "ok", proof: "0".repeat(64) }));
    });
    expect(await prove(port, token)).toBe("stranger");
    expect(seen).toHaveLength(1);
    expect(seen[0]).toContain("POST /handshake");
    expect(seen.join("\n")).not.toContain(token);
    expect(seen.join("\n").toLowerCase()).not.toContain("authorization");
  });

  it("tells an empty port apart", async () => {
    const { port } = await serve(() => {});
    servers.splice(0).forEach((s) => s.close());
    await new Promise((r) => setTimeout(r, 50));
    expect(await prove(port, "t")).toBe("unreachable");
  });
});
