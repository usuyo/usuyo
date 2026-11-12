import { Link } from "@tanstack/react-router";
import { Button as AriaButton, Switch } from "react-aria-components";
import type { FetchMethod, FetchPlugin } from "../api/endpoints";
import { usePluginChange, useSwitchMethod } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { openExternal } from "../shell";
import { Button } from "../ui/Button";
import { accountText, ACCESS_TEXT, getsText, lastCallText, methodStatus, pauseText } from "./words";

/** 看哪些取法：全部、现在用得上的、还用不上的。 */
export type MethodFilter = "all" | "usable" | "blocked";

export function shown(m: FetchMethod, filter: MethodFilter): boolean {
  return filter === "all" || (filter === "usable") === m.usable;
}

/**
 * 一个平台的获取插件：它的几种取法一行一种，做好了的各一个开关。每行写档位、账号用在哪、取得到什么、
 * 现在用不用得上（用不上的照内核的原因说）、上一次调用怎样。都只是给你挑的信息：不弹确认，只有调用没成的那句标颜色。
 */
export function PluginCard({ plugin, filter = "all" }: { plugin: FetchPlugin; filter?: MethodFilter }) {
  const change = usePluginChange(plugin.id);
  const switcher = useSwitchMethod(plugin.id);
  const held = plugin.pauses.some((p) => p.byYou);
  const paused = plugin.pauses.length > 0;
  const methods = plugin.methods.filter((m) => shown(m, filter));
  // 停着的写在各自的取法上；插件这一行只写「先别试」和没落在哪种取法上的。
  const own = plugin.pauses.filter((p) => p.byYou || !plugin.methods.some((m) => m.id === p.method));
  return (
    <article aria-label={plugin.name} className="rounded-card border border-border bg-card">
      <header className="flex flex-wrap items-start gap-x-3 gap-y-2 px-4 pt-3.5 pb-3">
        <div className="min-w-0 flex-[1_1_240px]">
          <div className="flex flex-wrap items-baseline gap-x-2">
            <span className="text-section font-semibold text-text">{plugin.name}</span>
            <span className="mono text-tag text-faint">
              {plugin.id} · {plugin.version}
            </span>
          </div>
          {plugin.description ? <p className="m-0 mt-1 text-note text-sub">{plugin.description}</p> : null}
          {own.map((p) => (
            <p key={`${p.method}-${p.scope}-${p.byYou ? "you" : "platform"}`} className={`m-0 mt-1 text-small ${p.byYou ? "text-sub" : "text-warn"}`}>
              {pauseText(p, plugin.platforms)}
            </p>
          ))}
        </div>
        <div className="flex items-center gap-1.5">
          {paused ? (
            <Button variant="outline" size="sm" onPress={() => change.mutate({ kind: "try-now" })} isDisabled={change.isPending}>
              现在再试
            </Button>
          ) : null}
          {!held ? (
            <Button variant="quiet" size="sm" onPress={() => change.mutate({ kind: "hold" })} isDisabled={change.isPending}>
              先别试
            </Button>
          ) : null}
        </div>
      </header>
      {change.error ? <ErrorLine error={change.error} className="px-4 pb-2" /> : null}
      <ul aria-label={`${plugin.name}的取法`} className="m-0 list-none border-t border-border p-0">
        {methods.map((m) => (
          <MethodRow
            key={m.id}
            plugin={plugin}
            method={m}
            {...(m.built ? { onSwitch: (on: boolean) => switcher.mutate({ methodId: m.id, enabled: on }) } : {})}
            busy={switcher.isPending}
          />
        ))}
      </ul>
      {switcher.error ? <ErrorLine error={switcher.error} className="px-4 pb-3" /> : null}
    </article>
  );
}

function MethodRow({ plugin, method, onSwitch, busy }: { plugin: FetchPlugin; method: FetchMethod; onSwitch?: (on: boolean) => void; busy: boolean }) {
  const status = methodStatus(method);
  const last = method.lastCall;
  const gets = getsText(method.gets);
  const lines = [
    ...(status ? [{ key: "status", text: status, tone: method.usable ? "text-label" : "text-sub" }] : []),
    ...method.pauses.filter((p) => !p.byYou && method.built).map((p) => ({ key: `${p.method}-${p.scope}-${p.byYou ? "you" : "platform"}`, text: pauseText(p, plugin.platforms), tone: p.byYou ? "text-sub" : "text-warn" })),
    ...(last ? [{ key: "last", text: lastCallText(last), tone: last.ok ? "text-sub" : "text-warn" }] : []),
  ];
  return (
    <li className="flex gap-3.5 border-t border-border px-4 py-3 first:border-t-0">
      <div className="pt-0.5">
        {onSwitch ? (
          <Switch
            aria-label={`${method.name}：${method.enabled ? "开着" : "关着"}`}
            isSelected={method.enabled}
            isDisabled={busy}
            onChange={onSwitch}
            className="group flex cursor-pointer outline-none"
          >
            <span className="flex h-5 w-8 items-center rounded-full border border-border2 bg-chip px-0.5 transition-colors duration-150 group-data-[focus-visible]:outline-2 group-data-[focus-visible]:outline-offset-2 group-data-[focus-visible]:outline-accent group-data-[selected]:border-label group-data-[selected]:bg-label">
              <span className="size-3.5 rounded-full bg-text transition-transform duration-150 ease-out group-data-[selected]:translate-x-3 group-data-[selected]:bg-bg motion-reduce:transition-none" />
            </span>
          </Switch>
        ) : (
          <span className="block w-8 text-center text-tag text-faint" aria-hidden="true">
            —
          </span>
        )}
      </div>
      <div className={`min-w-0 flex-1 ${method.built ? "" : "opacity-75"}`}>
        <div className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
          <span className="text-body font-medium text-text">{method.name}</span>
          <span className="text-small text-label">{ACCESS_TEXT[method.access]}</span>
          <span className="mono text-tag text-faint">{method.access}</span>
          {method.built ? null : <span className="rounded-chip border border-dashed border-border2 px-1.5 text-tag text-sub">还没做好</span>}
        </div>
        <p className="m-0 mt-0.5 text-note text-label">
          {accountText(method.account, method.account.sites ?? plugin.platforms.map((x) => x.name))}
          {method.interaction === "foreground" ? <span className="text-sub"> · 用的时候你看得见它在动</span> : null}
        </p>
        {method.account.kind === "own-phone" ? (
          <p className="m-0 mt-0.5 text-small text-sub">
            要一台手机：驱动、授权和连没连上在
            <Link to="/phone" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text">
              手机
            </Link>
            那一页。
          </p>
        ) : null}
        {gets.length > 0 ? (
          <div className="mt-1 flex flex-wrap gap-x-3 text-small text-sub">
            {gets.map((g) => (
              <span key={g}>{g}</span>
            ))}
          </div>
        ) : null}
        {lines.length > 0 ? (
          <p className="m-0 mt-1 text-small">
            {lines.map((l, i) => (
              <span key={l.key}>
                {i > 0 ? <span className="text-sub"> · </span> : null}
                <span className={l.tone}>{l.text}</span>
              </span>
            ))}
          </p>
        ) : null}
        {method.riskNote ? (
          <p className="m-0 mt-1 text-small text-faint">
            记下的：{method.riskNote.text}
            <AriaButton
              onPress={() => void openExternal(method.riskNote?.source.url ?? "")}
              className="ml-1 cursor-pointer border-0 bg-transparent p-0 font-sans text-small text-label underline decoration-border2 underline-offset-4 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text"
            >
              {method.riskNote.source.label}
            </AriaButton>
          </p>
        ) : null}
      </div>
    </li>
  );
}
