import { Link } from "@tanstack/react-router";
import { useState } from "react";
import { Button as AriaButton } from "react-aria-components";
import { ApiError } from "../api/client";
import type { FetchPlugin } from "../api/endpoints";
import { usePlugins, useSites, useSources, useStartImport } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { pauseText } from "../plugins/words";
import { Button } from "../ui/Button";

/** 内核回的错误里只按 `code` 分。 */
function problemCode(error: Error): string | null {
  if (!(error instanceof ApiError)) return null;
  try {
    const body: unknown = JSON.parse(error.message);
    return typeof body === "object" && body !== null && "code" in body && typeof body.code === "string" ? body.code : null;
  } catch {
    return null;
  }
}

function SourcesError({ error, plugin }: { error: Error; plugin: FetchPlugin }) {
  const code = problemCode(error);
  if (code === null) return <ErrorLine error={error} className="mt-2" />;
  const text =
    code === "login-required"
      ? `要先登录：在「获取插件」的${plugin.name}里登录好了，再回来列。`
      : code === "rate-limited" || code === "blocked" || code === "paused"
        ? `${plugin.name}的插件停着，原因写在「获取插件」里。`
        : `没能列出来（${code}）。`;
  return (
    <p role="status" className="m-0 mt-2 text-small text-sub">
      {text}
      {code === "login-required" || code === "paused" ? (
        <Link to="/plugins" className="ml-1 text-label underline-offset-2 hover:underline">
          去看看
        </Link>
      ) : null}
    </p>
  );
}

/** 一个能列的插件展开以后：问插件有哪些来源（会调插件，按平台的节奏可能要等一会儿），选一个开始列。 */
function Sources({ plugin, onStarted }: { plugin: FetchPlugin; onStarted: (id: string) => void }) {
  const sources = useSources(plugin.id);
  const start = useStartImport();
  const list = sources.data ?? plugin.sources;
  return (
    <div className="mt-3 border-t border-border pt-3">
      {sources.isFetching ? <p className="m-0 text-small text-sub">在问{plugin.name}有哪些可以列的……按平台的节奏，可能要等一会儿。</p> : null}
      {sources.error ? <SourcesError error={sources.error} plugin={plugin} /> : null}
      <ul className="m-0 mt-1 flex list-none flex-col p-0">
        {list.map((s) => (
          <li key={s.source} className="flex items-center gap-3 py-1.5">
            <span className="min-w-0 flex-1 truncate text-body text-text">{s.title}</span>
            {s.count !== undefined ? <span className="mono text-tag text-faint">{s.count} 条</span> : null}
            <Button
              variant="soft"
              size="tight"
              isDisabled={start.isPending}
              onPress={() => start.mutate({ pluginId: plugin.id, source: s.source, title: s.title }, { onSuccess: (v) => onStarted(v.id) })}
            >
              列出来
            </Button>
          </li>
        ))}
      </ul>
      {start.error ? <ErrorLine error={start.error} className="mt-2" /> : null}
    </div>
  );
}

/** 从哪里导入：能列出你存过的插件各一张卡片；只按链接取的写清楚清单从哪交进来。 */
export function SourcePicker({ onStarted }: { onStarted: (id: string) => void }) {
  const plugins = usePlugins();
  const sites = useSites();
  const [openId, setOpenId] = useState<string | null>(null);
  const all = plugins.data?.plugins ?? [];
  const listing = all.filter((p) => p.canList);
  const linkOnly = all.filter((p) => !p.canList);

  return (
    <section aria-label="从哪里导入">
      <h2 className="m-0 text-section font-semibold text-text">从哪里导入</h2>
      <p className="m-0 mt-1 text-small text-sub">选一个插件和它能列的一份清单。先列出来给你看，你点了才收。</p>
      {plugins.error ? <ErrorLine error={plugins.error} className="mt-3" /> : null}
      {plugins.data && all.length === 0 ? (
        <p className="m-0 mt-3 text-note text-sub">
          还没有装获取插件。
          <Link to="/plugins" className="ml-1 text-label underline-offset-2 hover:underline">
            去「获取插件」看怎么装
          </Link>
        </p>
      ) : null}
      <div className="mt-3.5 grid grid-cols-2 gap-3">
        {listing.map((p) => {
          const expanded = openId === p.id;
          const site = p.sites.map((s) => sites.data?.find((x) => x.site === s)).find((s) => s !== undefined);
          const pause = p.pauses[0];
          return (
            <div key={p.id} className={`rounded-card border bg-card p-4 ${expanded ? "col-span-2 border-border2" : "border-border"}`}>
              <AriaButton
                aria-expanded={expanded}
                onPress={() => setOpenId(expanded ? null : p.id)}
                className="block w-full cursor-pointer border-0 bg-transparent p-0 text-left outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
              >
                <div className="text-body font-medium text-text">{p.name}</div>
                <div className="mt-1 text-small text-label">
                  列出你存过的：{p.sources.length > 0 ? p.sources.map((s) => s.title).join("、") : "插件自己的清单"}
                  {p.sources.length > 0 ? " 和别的" : ""}
                </div>
                <div className="mt-0.5 text-small text-sub">
                  {site ? (site.login === "present" ? `用你登录着的${site.name}账号` : `要先登录${site.name}`) : "不用账号"}
                  {pause ? ` · ${pauseText(pause, p.platforms)}` : ""}
                </div>
              </AriaButton>
              {expanded ? <Sources plugin={p} onStarted={onStarted} /> : null}
            </div>
          );
        })}
      </div>
      {linkOnly.length > 0 ? (
        <p className="m-0 mt-3 text-small text-sub">
          {linkOnly.map((p) => p.name).join("、")}的插件只按链接一条条取，自己列不出清单：在浏览器里用脚本把清单交进来（下面发一个令牌），交来的会出现在右边的记录里，一样先给你看。
        </p>
      ) : null}
    </section>
  );
}
