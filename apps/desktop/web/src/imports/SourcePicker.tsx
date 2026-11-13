import { Link } from "@tanstack/react-router";
import { useState, type FormEvent } from "react";
import { Button as AriaButton, Form, Input, Label, TextField } from "react-aria-components";
import { ApiError } from "../api/client";
import type { FetchMethod, FetchPlugin } from "../api/endpoints";
import { useFileReaders, usePlugins, useSources, useStartImport, useVisitors } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import { accountText, canList, methodStatus, pausedAsBlocker, pauseText, visitorText } from "../plugins/words";
import { Button } from "../ui/Button";

/**
 * 列不出来：照内核给的 `hint` 说；插件报的错没带 `hint` 时，才按 `code` 说一句。
 * 要你去「获取插件」动手的几种，后面带个链接。
 */
function SourcesError({ error, plugin }: { error: Error; plugin: FetchPlugin }) {
  const visitors = useVisitors();
  if (!(error instanceof ApiError) || error.code === undefined) return <ErrorLine error={error} className="mt-2" />;
  const code = error.code;
  const text =
    error.hint ??
    (code === "login-required"
      ? t("imports.picker.login", { plugin: plugin.name })
      : code === "visitor-required"
        ? t("imports.picker.visitor", { visitor: visitorText(visitors.data?.find((x) => plugin.visitors.includes(x.site))?.state) })
        : code === "rate-limited" || code === "blocked"
          ? t("imports.picker.paused", { plugin: plugin.name })
          : t("imports.picker.failed", { code }));
  return (
    <p role="status" className="m-0 mt-2 text-small text-sub">
      {text}
      {code === "login-required" || code === "visitor-required" || code === "rate-limited" || code === "blocked" || code === "unsupported-input" ? (
        <Link to="/plugins/$platform" params={{ platform: plugin.platforms[0]?.id ?? plugin.id }} className="ml-1 text-label underline-offset-2 hover:underline">
          {t("imports.picker.go")}
        </Link>
      ) : null}
    </p>
  );
}

/**
 * 一种能列的取法展开以后：问插件这种取法有哪些来源（会调插件，按平台的节奏可能要等一会儿），选一个开始列。
 * 从一条链接开始的取法先填链接，交了才问。
 */
export function Sources({ plugin, method, onStarted }: { plugin: FetchPlugin; method: FetchMethod; onStarted: (id: string) => void }) {
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
              placeholder={t("imports.picker.linkPlaceholder")}
              className="h-ctl-sm w-full rounded-input border border-border2 bg-transparent px-3 font-sans text-note text-text outline-none focus:border-label"
            />
          </TextField>
          <Button type="submit" variant="soft" size="sm" isDisabled={draft.trim() === "" || sources.isFetching}>
            {t("imports.picker.find")}
          </Button>
        </Form>
      ) : null}
      {sources.isFetching ? <p className="m-0 text-small text-sub">{t("imports.picker.asking", { plugin: plugin.name })}</p> : null}
      {sources.error ? <SourcesError error={sources.error} plugin={plugin} /> : null}
      {fromLink && sources.data && sources.data.length === 0 ? <p className="m-0 mt-1 text-small text-sub">{t("imports.picker.noneInLink")}</p> : null}
      <ul className="m-0 mt-1 flex list-none flex-col p-0">
        {list.map((s) => (
          <li key={s.source} className="flex items-center gap-3 py-1.5">
            <span className="min-w-0 flex-1 truncate text-body text-text">{s.title}</span>
            {s.count !== undefined ? <span className="mono text-tag text-faint">{t("imports.picker.count", { count: s.count })}</span> : null}
            <Button
              variant="soft"
              size="tight"
              isDisabled={start.isPending}
              onPress={() => start.mutate({ pluginId: plugin.id, method: s.method, source: s.source, title: s.title }, { onSuccess: (v) => onStarted(v.id) })}
            >
              {t("imports.picker.list")}
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
  if (m.gets.link !== undefined) return t("imports.picker.fromLink", { what: m.gets.link });
  const fixed = plugin.sources.filter((s) => s.method === m.id).map((s) => s.title);
  return fixed.length > 0 ? t("imports.picker.saved", { list: formatList(fixed) }) : t("imports.picker.savedPlugin");
}

/**
 * 从哪里导入：每种能列的、做好了的取法一张卡片。这台电脑上现在用不上的照样摆着，写差什么，展开不了；
 * 只按链接一条条取的插件写清楚清单从哪交进来。
 */
export function SourcePicker({ onStarted, initial }: { onStarted: (id: string) => void; initial?: string }) {
  const plugins = usePlugins();
  const readers = new Set((useFileReaders().data ?? []).map((r) => r.id));
  const [openKey, setOpenKey] = useState<string | null>(initial ?? null);
  const all = plugins.data?.plugins ?? [];
  // 读文件的方法在「别的工具导出的文件」那一半，这里不列。
  const lists = (p: FetchPlugin, m: FetchMethod) => m.built && canList(m) && !readers.has(`${p.id}#${m.id}`);
  const cards = all.flatMap((p) => p.methods.filter((m) => lists(p, m)).map((m) => ({ plugin: p, method: m, key: `${p.id}#${m.id}` })));
  const linkOnly = all.filter((p) => !p.methods.some((m) => lists(p, m)));

  return (
    <section aria-label={t("imports.picker.title")}>
      <h2 className="m-0 text-section font-semibold text-text">{t("imports.picker.title")}</h2>
      <p className="m-0 mt-1 text-small text-sub">{t("imports.picker.note")}</p>
      {plugins.error ? <ErrorLine error={plugins.error} className="mt-3" /> : null}
      {plugins.data && all.length === 0 ? (
        <p className="m-0 mt-3 text-note text-sub">
          {t("imports.picker.noPlugins")}
          <Link to="/plugins" className="ml-1 text-label underline-offset-2 hover:underline">
            {t("imports.picker.howInstall")}
          </Link>
        </p>
      ) : null}
      <div className="mt-3.5 grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3">
        {cards.map(({ plugin: p, method: m, key }) => {
          const expanded = openKey === key && m.usable;
          const status = methodStatus(m);
          // 停着已经是用不上的一条原因时，不再另写一行。
          const pause = pausedAsBlocker(m) ? undefined : m.pauses[0];
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
                <Link to="/plugins/$platform" params={{ platform: p.platforms[0]?.id ?? p.id }} className="mt-1 inline-block text-small text-label underline-offset-2 hover:underline">
                  {t("imports.picker.goPlugins")}
                </Link>
              ) : null}
              {expanded ? <Sources plugin={p} method={m} onStarted={onStarted} /> : null}
            </div>
          );
        })}
      </div>
      {linkOnly.length > 0 ? (
        <p className="m-0 mt-3 text-small text-sub">
          {t("imports.picker.linkOnly", { platforms: formatList(linkOnly.map((p) => p.name)) })}
        </p>
      ) : null}
    </section>
  );
}
