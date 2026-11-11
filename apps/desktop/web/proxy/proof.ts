/// <reference types="node" />
// 开发代理带令牌以前先确认对面是内核：发一个随机数，内核用令牌签回来，这边用自己读到的令牌核对。
// 令牌不上线路。消息格式和 crates/usuyo-api/src/handshake.rs 一致。
import { createHmac, randomBytes, timingSafeEqual } from "node:crypto";

export const HANDSHAKE_PATH = "/handshake";
const PROVE_TIMEOUT_MS = 2000;

export function proofOf(token: string, port: number, nonce: string): Buffer {
  return createHmac("sha256", token).update(`usuyo-daemon-proof/v1\n127.0.0.1:${port}\n${nonce}`).digest();
}

/** 对面回的十六进制证明对不对；比较用常数时间。 */
export function verifyProof(token: string, port: number, nonce: string, proof: string): boolean {
  if (!/^[0-9a-f]{64}$/.test(proof)) return false;
  return timingSafeEqual(proofOf(token, port, nonce), Buffer.from(proof, "hex"));
}

/** 握手的结果：是内核、没人在听、端口上是别的程序。 */
export type Proven = "daemon" | "unreachable" | "stranger";

export async function prove(port: number, token: string): Promise<Proven> {
  const nonce = randomBytes(32).toString("hex");
  let response: Response;
  try {
    response = await fetch(`http://127.0.0.1:${port}${HANDSHAKE_PATH}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ nonce }),
      signal: AbortSignal.timeout(PROVE_TIMEOUT_MS),
    });
  } catch {
    return "unreachable";
  }
  if (!response.ok) return "stranger";
  try {
    const body: unknown = await response.json();
    const proof = typeof body === "object" && body !== null && "proof" in body ? body.proof : null;
    return typeof proof === "string" && verifyProof(token, port, nonce, proof) ? "daemon" : "stranger";
  } catch {
    return "stranger";
  }
}
