import { Button as AriaButton, Switch } from "react-aria-components";
import { isNotServed } from "../api/client";
import type { FetchMethod, FetchPlugin, SiteLogin, VisitorIdentity } from "../api/endpoints";
import { usePluginChange, useSwitchMethod } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { openExternal } from "../shell";
import { Button } from "../ui/Button";
import { accountText, ACCESS_TEXT, getsText, loginStateText, pauseText } from "./words";

/**
 * 一个平台的获取插件：它的几种取法一行一种，各自一个开关。档位、账号用在哪、取得到什么、记下的一句风险，
 * 都只是给你挑的信息：不弹确认，不标红。还没做好的那几种列在后面，标明，开不了。
 */
export function PluginCard({ plugin, sites, visitors }: { plugin: FetchPlugin; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  const change = usePluginChange(plugin.id);
  const switcher = useSwitchMethod(plugin.id);
  const held = plugin.pauses.some((p) => p.byYou);
  const paused = plugin.pauses.length > 0 || plugin.methods.some((m) => m.pauses.length > 0);
  const built = plugin.methods.filter((m) => m.built);
  const later = plugin.methods.filter((m) => !m.built);
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
          {plugin.pauses.map((p) => (
            <p key={`${p.scope}-${p.byYou ? "you" : "platform"}`} className={`m-0 mt-1 text-small ${p.byYou ? "text-sub" : "text-warn"}`}>
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
      <div className="border-t border-border px-4 pt-2.5 text-tag font-medium text-faint">取法</div>
      <ul aria-label={`${plugin.name}的取法`} className="m-0 list-none p-0">
        {built.map((m) => (
          <MethodRow key={m.id} plugin={plugin} method={m} sites={sites} visitors={visitors} onSwitch={(on) => switcher.mutate({ methodId: m.id, enabled: on })} busy={switcher.isPending} />
        ))}
      </ul>
      {isNotServed(switcher.error) ? (
        <NotServedLine className="px-4 pb-3">这台电脑上的 usuyo 内核还不能分开开关取法，现在几种做好了的都照插件自己的来。</NotServedLine>
      ) : switcher.error ? (
        <ErrorLine error={switcher.error} className="px-4 pb-3" />
      ) : null}
      {later.length > 0 ? (
        <>
          <div className="border-t border-border px-4 pt-2.5 text-tag font-medium text-faint">插件写了、还没做好的 {later.length} 种</div>
          <ul className="m-0 list-none p-0">
            {later.map((m) => (
              <MethodRow key={m.id} plugin={plugin} method={m} sites={sites} visitors={visitors} />
            ))}
          </ul>
        </>
      ) : null}
    </article>
  );
}

function MethodRow({
  plugin,
  method,
  sites,
  visitors,
  onSwitch,
  busy,
}: {
  plugin: FetchPlugin;
  method: FetchMethod;
  sites: SiteLogin[];
  visitors: VisitorIdentity[];
  onSwitch?: (on: boolean) => void;
  busy?: boolean;
}) {
  const hosts = method.account.sites ?? [];
  const logins = method.account.kind === "in-usuyo" ? hosts.map((h) => sites.find((s) => s.site === h)) : [];
  const ids = method.account.kind === "visitor" ? hosts.map((h) => visitors.find((v) => v.site === h)) : [];
  const names = [...logins, ...ids].map((x, i) => x?.name ?? hosts[i] ?? "");
  const uses =
    method.account.kind === "in-usuyo"
      ? `用到：你在 usuyo 里登录的${names.join("、")}（${loginStateText(logins.map((l) => l?.login ?? "missing"))}）`
      : method.account.kind === "visitor"
        ? `用到：${names.join("、")}的访客身份（${ids.every((v) => v?.state === "present") ? "有一份" : ids.some((v) => v?.state === "expired") ? "不管用了" : "还没有"}）`
        : null;
  return (
    <li className={`flex gap-3.5 border-t border-border px-4 py-3 first:border-t-0 ${method.built ? "" : "opacity-70"}`}>
      <div className="pt-0.5">
        {method.built && onSwitch ? (
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
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
          <span className="text-body font-medium text-text">{method.name}</span>
          <span className="text-small text-label">{ACCESS_TEXT[method.access]}</span>
          <span className="mono text-tag text-faint">{method.access}</span>
          {method.built ? (
            <span className="text-small text-sub">{method.enabled ? "开着" : "关着"}</span>
          ) : (
            <span className="rounded-chip border border-dashed border-border2 px-1.5 text-tag text-sub">还没做好</span>
          )}
        </div>
        <p className="m-0 mt-0.5 text-note text-label">{accountText(method.account, names)}</p>
        <div className="mt-1 flex flex-wrap gap-x-3 text-small text-sub">
          {getsText(method.gets).map((g) => (
            <span key={g}>{g}</span>
          ))}
        </div>
        {uses && method.built ? <p className="m-0 mt-1 text-small text-sub">{uses}</p> : null}
        {method.riskNote ? (
          <p className="m-0 mt-1 text-small text-faint">
            记下的：{method.riskNote.text}
            {method.riskNote.source ? (
              <AriaButton
                onPress={() => void openExternal(method.riskNote?.source?.url ?? "")}
                className="ml-1 cursor-pointer border-0 bg-transparent p-0 font-sans text-small text-label underline decoration-border2 underline-offset-4 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text"
              >
                {method.riskNote.source.label}
              </AriaButton>
            ) : null}
          </p>
        ) : null}
        {method.pauses.map((p) => (
          <p key={`${p.scope}-${p.byYou ? "you" : "platform"}`} className={`m-0 mt-1 text-small ${p.byYou ? "text-sub" : "text-warn"}`}>
            {pauseText(p, plugin.platforms)}
          </p>
        ))}
      </div>
    </li>
  );
}
