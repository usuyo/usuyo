// 样例库的获取策略：照内核的求值（crates/usuyo-core/src/strategy.rs）算每个平台每项功能会用哪几种方式，
// 只给样例库用；真的窗口照内核回的结果摆。
import type { Access, FetchFunction, FetchMethod, FetchPlugin, FunctionRoute, MethodAccount, OutputKind, Strategy, StrategyRule, StrategyStep, StrategyView } from "../api/endpoints";
import { ApiError } from "../api/client";

const ACCESS: Access[] = ["official-export", "public-web", "vision-ui", "browser-session", "session-api", "private-api", "client-hook"];
const ACCOUNT: MethodAccount["kind"][] = ["none", "visitor", "own-browser", "own-phone", "own-app", "in-usuyo", "developer"];
const NO_ACCOUNT: MethodAccount["kind"][] = ["none", "visitor"];
const WITH_ACCOUNT: MethodAccount["kind"][] = ["own-browser", "own-phone", "own-app", "in-usuyo", "developer"];
const ORDER: FetchFunction[] = ["text", "images", "video", "audio", "comments", "list:favorite", "list:like", "list:share", "list:view", "list"];

const OF_OUTPUT: Record<OutputKind, FetchFunction> = {
  text: "text",
  html: "text",
  meta: "text",
  images: "images",
  "media:video": "video",
  subtitle: "video",
  "media:audio": "audio",
  comments: "comments",
};

const OF_ACTION: Record<"favorite" | "like" | "share" | "view", FetchFunction> = {
  favorite: "list:favorite",
  like: "list:like",
  share: "list:share",
  view: "list:view",
};

export const PRESETS: StrategyView["presets"] = [
  { preset: "no-account-only", rules: [{ use: [{ accounts: NO_ACCOUNT }] }] },
  { preset: "no-account-first", rules: [{ use: [{}] }] },
  { preset: "account-first", rules: [{ use: [{ accounts: WITH_ACCOUNT }, { accounts: NO_ACCOUNT }] }] },
];

interface Way {
  key: string;
  account: MethodAccount["kind"];
  access: Access;
  serves: Set<FetchFunction>;
}

function serves(p: FetchPlugin, m: FetchMethod): Set<FetchFunction> {
  const out = new Set<FetchFunction>((m.gets.fetch ?? []).map((k) => OF_OUTPUT[k]));
  if (m.gets.list !== undefined || m.gets.link !== undefined) {
    const actions = p.sources.flatMap((s) => (s.method === m.id && s.action && s.action !== "unknown" ? [OF_ACTION[s.action]] : []));
    if (actions.length === 0) out.add("list");
    for (const a of actions) out.add(a);
  }
  return out;
}

function admits(s: StrategyStep, w: Way): boolean {
  if (s.method) return s.method === w.key;
  const accounts = s.accounts ?? [];
  const access = s.access ?? [];
  return (accounts.length === 0 || accounts.includes(w.account)) && (access.length === 0 ? ACCESS.indexOf(w.access) < ACCESS.indexOf("private-api") : access.includes(w.access));
}

function decide(v: Strategy, platform: string, f: FetchFunction, ways: Way[]): Pick<FunctionRoute, "from" | "skip"> & { order: number[] } {
  const own = v.platforms?.[platform];
  const pick = own?.functions?.[f];
  const serving = ways.map((w, i) => ({ w, i })).filter((x) => x.w.serves.has(f));
  if (pick === "skip") return { from: "function", skip: true, order: [] };
  const picked = serving.find((x) => x.w.key === pick);
  if (picked) return { from: "function", skip: false, order: [picked.i] };
  const from = own?.preset ? "platform" : "strategy";
  const preset = own?.preset ?? v.preset;
  const rules: StrategyRule[] = preset === "custom" ? (v.rules ?? []) : (PRESETS.find((p) => p.preset === preset)?.rules ?? []);
  const rule = rules.find((r) => (r.platform === undefined || r.platform === platform) && (r.function === undefined || r.function === f));
  if (!rule) return { from, skip: true, order: [] };
  const order: number[] = [];
  for (const step of rule.use) for (const x of serving) if (admits(step, x.w) && !order.includes(x.i)) order.push(x.i);
  return { from, skip: rule.use.length === 0, order };
}

/** 存着的策略和照它算出的每个平台每项功能。 */
export function strategyView(v: Strategy, plugins: FetchPlugin[]): StrategyView {
  const platforms = [...new Map(plugins.flatMap((p) => p.platforms.map((x) => [x.id, x] as const))).values()];
  const routes = platforms.map((platform) => {
    const ranked = plugins
      .filter((p) => p.platforms.some((x) => x.id === platform.id))
      .flatMap((p, pi) => p.methods.map((m, mi) => ({ p, m, pi, mi })))
      .filter(({ p, m }) => m.built && m.gets.delivers === undefined && serves(p, m).size > 0)
      .map((x) => ({ ...x, access: x.m.perPlatform?.find((t) => t.platform === platform.id)?.access ?? x.m.access }))
      .sort((a, b) => ACCESS.indexOf(a.access) - ACCESS.indexOf(b.access) || ACCOUNT.indexOf(a.m.account.kind) - ACCOUNT.indexOf(b.m.account.kind) || a.pi - b.pi || a.mi - b.mi);
    const ways: Way[] = ranked.map((x) => ({ key: `${x.p.id}#${x.m.id}`, account: x.m.account.kind, access: x.access, serves: serves(x.p, x.m) }));
    const ref = (i: number) => ({ plugin: ranked[i]?.p.id ?? "", method: ranked[i]?.m.id ?? "" });
    const functions = ORDER.filter((f) => ways.some((w) => w.serves.has(f))).map((f) => {
      const d = decide(v, platform.id, f, ways);
      return {
        function: f,
        from: d.from,
        skip: d.skip,
        ways: d.order.map(ref),
        choices: ways.flatMap((w, i) => (w.serves.has(f) ? [ref(i)] : [])),
      };
    });
    return { platform, functions };
  });
  return { ...v, presets: PRESETS, routes };
}

/** 同内核的检查：自定义要有规则、最后一条是兜底，平台另选的不能是「自定义」。 */
export function checked(v: Strategy): Strategy {
  const rules = v.rules ?? [];
  const last = rules[rules.length - 1];
  if (v.preset === "custom" && rules.length === 0) throw new ApiError(400, "a custom strategy needs at least one rule", "invalid-args");
  if (last && (last.platform !== undefined || last.function !== undefined)) throw new ApiError(400, "the last rule must match any platform and any function", "invalid-args");
  return v;
}
