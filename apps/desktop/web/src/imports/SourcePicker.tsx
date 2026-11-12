import { Link } from "@tanstack/react-router";
import { useState, type FormEvent } from "react";
import { Button as AriaButton, Form, Input, Label, TextField } from "react-aria-components";
import { ApiError } from "../api/client";
import type { FetchMethod, FetchPlugin } from "../api/endpoints";
import { usePlugins, useSources, useStartImport, useVisitors } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { accountText, canList, methodStatus, pauseText, visitorText } from "../plugins/words";
import { Button } from "../ui/Button";

/** 内核回的错误里只按 `code` 分。 */
function problemCode(error: Error): string | null {
  return error instanceof ApiError ? (error.code ?? null) : null;
}

function SourcesError({ error, plugin, fromLink }: { error: Error; plugin: FetchPlugin; fromLink: boolean }) {
  const code = problemCode(error);
  const visitors = useVisitors();
  if (code === null) return <ErrorLine error={error} className="mt-2" />;
  const text =
    code === "invalid-args" && fromLink
      ? "这条链接用不了：要一条 http 或 https 开头的完整链接。"
      : code === "unsupported-input"
        ? `${plugin.name}的这种取法现在列不了：可能在按节奏等，或者这台电脑上缺了什么，「获取插件」里写着。`
        : code === "login-required"
      ? `要先登录：在「获取插件」的${plugin.name}里登录好了，再回来列。`
      : code === "visitor-required"
        ? `${visitorText(visitors.data?.find((x) => plugin.visitors.includes(x.site))?.state)}；好了再回来列。`
        : code === "rate-limited" || code === "blocked" || code === "paused"
          ? `${plugin.name}的插件停着，原因写在「获取插件」里。`
          : `没能列出来（${code}）。`;
  return (
    <p role="status" className="m-0 mt-2 text-small text-sub">
      {text}
      {code === "login-required" || code === "visitor-required" || code === "paused" || code === "unsupported-input" ? (
        <Link to="/plugins" className="ml-1 text-label underline-offset-2 hover:underline">
          去看看
        </Link>
      ) : null}
    </p>
  );
}

/**
 * 一种能列的取法展开以后：问插件这种取法有哪些来源（会调插件，按平台的节奏可能要等一会儿），选一个开始列。
 * 从一条链接开始的取法先填链接，交了才问。
 */
function Sources({ plugin, method, onStarted }: { plugin: FetchPlugin; method: FetchMethod; onStarted: (id: string) => void }) {
  const [draft, setDraft] = useState("");
  const [link, setLink] = useState<string | null>(null);
  const fromLink = method.gets.link !== undefined;
  const sources = useSources(fromLink && link === null ? null : { pluginId: plugin.id, method: method.id, ...(link === null ? {} : { link }) });
  const start = useStartImport();
  const fixed = fromLink ? [] : plugin.sources.filter((s) => s.method === method.id);
  const list = sources.data ?? fixed;
  const submit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const v = draft.trim();
    if (v === "") return;
    // 同一条再交一次（例如上次没列成）就再问一次。
    if (v === link) void sources.refetch();
    else setLink(v);
  };
  return (
    <div className="mt-3 border-t border-border pt-3">
      {fromLink ? (
        <Form onSubmit={submit} className="mb-2 flex items-end gap-2">
          <TextField value={draft} onChange={setDraft} className="flex min-w-0 flex-1 flex-col gap-1">
            <Label className="text-small text-label">{method.gets.link}</Label>
            <Input
              type="url"
              placeholder="https://"
              className="h-ctl-sm w-full rounded-input border border-border2 bg-transparent px-3 font-sans text-note text-text outline-none focus:border-label"
            />
          </TextField>
          <Button type="submit" variant="soft" size="sm" isDisabled={draft.trim() === "" || sources.isFetching}>
            看看有什么
          </Button>
        </Form>
      ) : null}
      {sources.isFetching ? <p className="m-0 text-small text-sub">在问{plugin.name}有哪些可以列的……按平台的节奏，可能要等一会儿。</p> : null}
      {sources.error ? <SourcesError error={sources.error} plugin={plugin} fromLink={fromLink} /> : null}
      {fromLink && sources.data && sources.data.length === 0 ? <p className="m-0 mt-1 text-small text-sub">这条链接里没有能列的。</p> : null}
      <ul className="m-0 mt-1 flex list-none flex-col p-0">
        {list.map((s) => (
          <li key={s.source} className="flex items-center gap-3 py-1.5">
            <span className="min-w-0 flex-1 truncate text-body text-text">{s.title}</span>
            {s.count !== undefined ? <span className="mono text-tag text-faint">{s.count} 条</span> : null}
            <Button
              variant="soft"
              size="tight"
              isDisabled={start.isPending}
              onPress={() => start.mutate({ pluginId: plugin.id, method: s.method, source: s.source, title: s.title }, { onSuccess: (v) => onStarted(v.id) })}
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

/** 卡片上那句能列什么：写死的来源名，或从链接开始，或插件自己的清单。 */
function listsText(plugin: FetchPlugin, m: FetchMethod): string {
  if (m.gets.link !== undefined) return `从一条链接列出：${m.gets.link}`;
  const fixed = plugin.sources.filter((s) => s.method === m.id).map((s) => s.title);
  return fixed.length > 0 ? `列出你存过的：${fixed.join("、")} 和别的` : "列出你存过的：插件自己的清单";
}

/**
 * 从哪里导入：每种能列的、做好了的取法一张卡片。这台电脑上现在用不上的照样摆着，写差什么，展开不了；
 * 只按链接一条条取的插件写清楚清单从哪交进来。
 */
export function SourcePicker({ onStarted }: { onStarted: (id: string) => void }) {
  const plugins = usePlugins();
  const [openKey, setOpenKey] = useState<string | null>(null);
  const all = plugins.data?.plugins ?? [];
  const cards = all.flatMap((p) => p.methods.filter((m) => m.built && canList(m)).map((m) => ({ plugin: p, method: m, key: `${p.id}#${m.id}` })));
  const linkOnly = all.filter((p) => !p.methods.some((m) => m.built && canList(m)));

  return (
    <section aria-label="从哪里导入">
      <h2 className="m-0 text-section font-semibold text-text">从哪里导入</h2>
      <p className="m-0 mt-1 text-small text-sub">选一种取法和它能列的一份清单。先列出来给你看，你点了才收。</p>
      {plugins.error ? <ErrorLine error={plugins.error} className="mt-3" /> : null}
      {plugins.data && all.length === 0 ? (
        <p className="m-0 mt-3 text-note text-sub">
          还没有装获取插件。
          <Link to="/plugins" className="ml-1 text-label underline-offset-2 hover:underline">
            去「获取插件」看怎么装
          </Link>
        </p>
      ) : null}
      <div className="mt-3.5 grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3">
        {cards.map(({ plugin: p, method: m, key }) => {
          const expanded = openKey === key && m.usable;
          const status = methodStatus(m);
          const pause = m.pauses[0];
          return (
            <div key={key} className={`rounded-card border bg-card p-4 ${expanded ? "col-span-full border-border2" : "border-border"}`}>
              <AriaButton
                aria-expanded={m.usable ? expanded : undefined}
                isDisabled={!m.usable}
                onPress={() => setOpenKey(expanded ? null : key)}
                className="block w-full cursor-pointer border-0 bg-transparent p-0 text-left outline-none data-[disabled]:cursor-default data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
              >
                <div className="flex flex-wrap items-baseline gap-x-2">
                  <span className="text-body font-medium text-text">{p.name}</span>
                  <span className="text-small text-sub">{m.name}</span>
                </div>
                <div className="mt-1 text-small text-label">{listsText(p, m)}</div>
                <div className="mt-0.5 text-small text-sub">{accountText(m.account, m.account.sites ?? p.platforms.map((x) => x.name))}</div>
                {m.usable ? null : <div className="mt-0.5 text-small text-sub">{status}</div>}
                {pause ? <div className={`mt-0.5 text-small ${pause.byYou ? "text-sub" : "text-warn"}`}>{pauseText(pause, p.platforms)}</div> : null}
              </AriaButton>
              {!m.usable ? (
                <Link to="/plugins" search={{ platform: p.platforms[0]?.id ?? p.id }} className="mt-1 inline-block text-small text-label underline-offset-2 hover:underline">
                  去「获取插件」看看
                </Link>
              ) : null}
              {expanded ? <Sources plugin={p} method={m} onStarted={onStarted} /> : null}
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
