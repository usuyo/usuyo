import type { FetchPlugin, SiteLogin } from "../api/endpoints";
import { usePluginChange } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { ACCESS_TEXT, loginText, pauseText } from "./words";

/** 标签的小块：怎么取、要不要账号、能做什么。都是给你挑的信息，不用颜色。 */
function Tag({ children }: { children: string }) {
  return <span className="flex h-6 items-center rounded-full border border-border2 px-[9px] text-small leading-none text-label">{children}</span>;
}

/** 一个获取插件：标签、停着没有；停着的写原因和什么时候自己再试，能「现在再试」「先别试」。 */
export function PluginCard({ plugin, sites }: { plugin: FetchPlugin; sites: SiteLogin[] }) {
  const change = usePluginChange(plugin.id);
  const names = plugin.sites.map((s) => sites.find((x) => x.site === s)?.name ?? s);
  const held = plugin.pauses.some((p) => p.byYou);
  return (
    <article className="rounded-card border border-border bg-card px-4 py-3.5">
      <div className="flex items-start gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-2">
            <span className="text-body font-medium text-text">{plugin.name}</span>
            <span className="mono text-tag text-faint">{plugin.version}</span>
          </div>
          <div className="mono mt-0.5 text-tag text-faint">
            {plugin.id} · {plugin.access}
          </div>
        </div>
        {plugin.pauses.length > 0 ? (
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
      <div className="mt-2.5 flex flex-wrap gap-1.5">
        <Tag>{ACCESS_TEXT[plugin.access]}</Tag>
        <Tag>{loginText(plugin.login, names)}</Tag>
        <Tag>按链接取一条</Tag>
        {plugin.canList ? <Tag>列出你存过的</Tag> : null}
      </div>
      {plugin.description ? <p className="m-0 mt-2.5 text-note text-sub">{plugin.description}</p> : null}
      {plugin.pauses.length > 0 ? (
        <div className="mt-2 flex flex-col gap-0.5">
          {plugin.pauses.map((p) => (
            <p key={`${p.scope}-${p.byYou ? "you" : "platform"}`} className={`m-0 text-small ${p.byYou ? "text-sub" : "text-warn"}`}>
              {pauseText(p, plugin.platforms)}
            </p>
          ))}
        </div>
      ) : (
        <p className="m-0 mt-2 text-small text-sub">照常在用</p>
      )}
      {change.error ? <ErrorLine error={change.error} className="mt-2" /> : null}
    </article>
  );
}
