import { Link, useNavigate } from "@tanstack/react-router";
import { useState, type FormEvent, type ReactNode } from "react";
import { Form, Input, Label, TextField } from "react-aria-components";
import { ApiError } from "../api/client";
import type { FetchMethod, FetchPlugin, FileReader, NewDeliverToken, SiteLogin, VisitorIdentity } from "../api/endpoints";
import { useCreateDeliverToken, useFileReaders, useMethodScript, usePlugins, useSites, useSources, useStartImport, useVisitors } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { rich, t } from "../i18n";
import { formatList } from "../i18n/format";
import { localized, useNames } from "../plugins/names";
import { needsOf, Needs, TryAgain } from "../plugins/Needs";
import { keyOf, platformPages, savedKind, scriptPages, type PlatformPage, type SavedKind, type Use } from "../plugins/platforms";
import { OffSwitch, ReadingButton } from "../plugins/SavedCard";
import { SiteLoginButton } from "../plugins/SiteLoginRow";
import { VisitorTake } from "../plugins/VisitorIdentity";
import { clauses, lastCallText, pausedAsBlocker, pauseText } from "../plugins/words";
import { Button } from "../ui/Button";
import { Fold } from "../ui/Fold";
import { ScriptSetup } from "./ScriptSetup";

/**
 * 列不出来：照内核给的 `hint` 说；插件报的错没带 `hint` 时，才按 `code` 说一句。
 * 要登录、要访客身份、停着的，就地给那一步的按钮，做完再点「查找」或「列出」。
 */
function SourcesError({ error, plugin, method }: { error: Error; plugin: FetchPlugin; method: FetchMethod }) {
  const visitors = useVisitors();
  const sites = useSites();
  if (!(error instanceof ApiError) || error.code === undefined) return <ErrorLine error={error} className="mt-2" />;
  const code = error.code;
  const visitor = visitors.data?.find((x) => plugin.visitors.includes(x.site));
  const text =
    error.hint ??
    (code === "login-required"
      ? t("imports.picker.login")
      : code === "visitor-required"
        ? t("imports.picker.visitor", { visitor: t("plugins.visitor.needed") })
        : code === "rate-limited" || code === "blocked"
          ? t("imports.picker.paused", { plugin: plugin.name })
          : t("imports.picker.failed", { code }));
  // 这种方式自己没写站点时（例如不用账号的方式被平台要求登录），给这个插件用到的账号站点。
  const want = method.account.sites ?? [];
  const login = code === "login-required" ? (sites.data ?? []).filter((s) => s.plugins.includes(plugin.id) && (want.length === 0 || want.includes(s.site))) : [];
  return (
    <div role="status" className="mt-2 flex flex-col items-start gap-1.5">
      <p className="m-0 text-small text-sub">{text}</p>
      {login.map((s) => (
        <SiteLoginButton key={s.site} site={s} />
      ))}
      {code === "visitor-required" && visitor ? <VisitorTake visitor={visitor} /> : null}
      {code === "rate-limited" || code === "blocked" ? <TryAgain pluginId={plugin.id} label={t("plugins.needs.retryNow")} /> : null}
    </div>
  );
}

/**
 * 一种能列的获取方式展开以后：问插件这种方式有哪些列表（会调插件，按平台的请求频率可能要等一会儿），选一个开始列。
 * 从一条链接开始的先填链接，交了才问。写死在清单里的列表先摆出来，问到了换成插件给的。
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
      {sources.error ? <SourcesError error={sources.error} plugin={plugin} method={method} /> : null}
      {sources.error && !fromLink ? (
        <Button variant="outline" size="sm" className="mt-2" isDisabled={sources.isFetching} onPress={() => void sources.refetch()}>
          {t("common.retry")}
        </Button>
      ) : null}
      {fromLink && sources.data && sources.data.length === 0 ? <p className="m-0 mt-1 text-small text-sub">{t("imports.picker.noneInLink")}</p> : null}
      {!fromLink && sources.data && sources.data.length === 0 ? <p className="m-0 mt-1 text-small text-sub">{t("imports.picker.noneListed")}</p> : null}
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

/** 一种导入方式在这一页上的一行：能导入什么（标题）、怎么导入（一句）。 */
function rowText(u: Use, kind: SavedKind, reader: FileReader | undefined, pages: string | null): { title: string; sub: string } {
  const m = u.method;
  const name = m.name;
  const lists = m.gets.list ?? [];
  switch (kind) {
    case "file":
      return { title: reader && reader.lists.length > 0 ? formatList(reader.lists) : name, sub: t("imports.rows.file", { name }) };
    case "window":
      return { title: name, sub: t("imports.rows.window") };
    case "script":
      return { title: name, sub: pages ? t("imports.rows.scriptPages", { pages }) : t("imports.rows.script") };
    case "phone":
      return { title: lists.length > 0 ? formatList(lists) : name, sub: t("imports.rows.phone", { name }) };
    case "link-list":
      return { title: name, sub: t("imports.rows.linkList", { what: m.gets.link ?? "" }) };
    default: {
      const fixed = u.plugin.sources.filter((s) => s.method === m.id).map((s) => s.title);
      const what = [...new Set([...lists, ...fixed])];
      return { title: what.length > 0 ? formatList(what) : name, sub: what.length > 0 ? t("imports.rows.list", { name }) : t("imports.rows.listAny") };
    }
  }
}

/** 主操作的名字：每一类一个动词。 */
function actionText(kind: SavedKind): string {
  const text: Record<SavedKind, () => string> = {
    file: () => t("imports.rows.chooseFile"),
    window: () => "",
    script: () => t("imports.rows.setUpScript"),
    phone: () => t("imports.rows.setUpPhone"),
    "link-list": () => t("imports.rows.enterLink"),
    list: () => t("imports.rows.chooseList"),
  };
  return text[kind]();
}

const BUTTON_LINK =
  "press inline-flex h-ctl-sm items-center rounded-btn border border-border2 bg-chip px-3.5 text-note font-medium text-text no-underline outline-none hover:border-label/60 focus-visible:outline-2 focus-visible:outline-accent";

/**
 * 还差一步时那一步：关着的给开关，缺登录、访客身份、密钥、停着的给按钮（`Needs`），别的照内核的原话。
 * 用得上、只是按网站停着的写一行暂停；上一次没成的写一行原因。
 */
function Blocked({ use, sites, visitors }: { use: Use; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  const m = use.method;
  if (!m.enabled) return <OffSwitch use={use} />;
  if (!m.usable) {
    return needsOf(use, sites, visitors).length > 0 ? <Needs use={use} sites={sites} visitors={visitors} /> : <p className="m-0 text-small text-sub">{clauses(m.blockers.map((b) => b.hint))}</p>;
  }
  const pause = pausedAsBlocker(m) ? undefined : m.pauses[0];
  return (
    <>
      {pause ? <p className={`m-0 text-small ${pause.byYou ? "text-sub" : "text-warn"}`}>{pauseText(pause, use.plugin.platforms)}</p> : null}
      {m.lastCall && !m.lastCall.ok ? <p className="m-0 text-small text-warn">{lastCallText(m.lastCall)}</p> : null}
    </>
  );
}

/** 一行：能导入什么、一个主操作；还差一步的在下面写清差什么并给那一步的按钮；展开的是列表、链接框或脚本的几步。 */
function Row({
  use,
  kind,
  page,
  reader,
  open,
  onOpen,
  onStarted,
  sites,
  visitors,
}: {
  use: Use;
  kind: SavedKind;
  page: PlatformPage;
  reader: FileReader | undefined;
  open: boolean;
  onOpen: (on: boolean) => void;
  onStarted: (id: string) => void;
  sites: SiteLogin[];
  visitors: VisitorIdentity[];
}) {
  const m = use.method;
  const key = keyOf(use);
  const script = useMethodScript(use.plugin.id, m.id, kind === "script" && m.script?.present === true);
  const pages = script.data ? scriptPages(script.data).pages : null;
  const platform = page.name;
  const { title, sub } = rowText(use, kind, reader, pages);
  const create = useCreateDeliverToken();
  const [token, setToken] = useState<NewDeliverToken | null>(null);
  const scriptMissing = kind === "script" && m.script?.present !== true;
  const ready = m.usable && !scriptMissing;

  let action: ReactNode;
  if (kind === "file") {
    action = (
      <Link to="/import" search={{ from: "file", reader: key }} className={BUTTON_LINK}>
        {actionText(kind)}
      </Link>
    );
  } else if (kind === "phone") {
    action = (
      <Link to="/phone" className={BUTTON_LINK}>
        {actionText(kind)}
      </Link>
    );
  } else if (kind === "window") {
    const siteId = m.account.sites?.[0];
    const site = sites.find((s) => s.site === siteId && s.plugins.includes(use.plugin.id));
    action = <ReadingButton use={use} site={site?.name ?? m.account.where ?? platform} signedIn={site?.login === "present"} />;
  } else if (kind === "script") {
    action = (
      <Button
        variant="soft"
        size="sm"
        isDisabled={!ready || create.isPending || token !== null}
        onPress={() =>
          create.mutate(
            { name: t("imports.script.tokenName", { platform }), kind: "extension" },
            {
              onSuccess: (made) => {
                setToken(made);
                // 回的令牌也别留在请求的记录里。
                create.reset();
                onOpen(true);
              },
            },
          )
        }
      >
        {actionText(kind)}
      </Button>
    );
  } else {
    action = (
      <Button variant="soft" size="sm" aria-expanded={open} isDisabled={!ready} onPress={() => onOpen(!open)}>
        {actionText(kind)}
      </Button>
    );
  }

  return (
    <li className="border-t border-border px-4 py-3 first:border-t-0">
      <div className="flex flex-wrap items-start justify-between gap-x-4 gap-y-2">
        <div className="min-w-0 flex-[1_1_220px]">
          <div className="text-body font-medium text-text">{title}</div>
          <div className="mt-0.5 text-small text-sub">{sub}</div>
        </div>
        <div className="flex max-w-full min-w-0 flex-none flex-col items-end gap-1">{action}</div>
      </div>
      <div className="mt-1.5 flex min-w-0 flex-col gap-1.5 empty:hidden">
        <Blocked use={use} sites={sites} visitors={visitors} />
        {scriptMissing && m.script ? (
          <p className="m-0 text-small text-warn">
            {t("imports.script.missing")}
            <Link
              to="/plugins/$owner/$repo"
              params={{ owner: use.plugin.id.split("/")[0] ?? "", repo: use.plugin.id.split("/")[1] ?? "" }}
              className="ml-1 text-label underline decoration-border2 underline-offset-4 hover:text-text"
            >
              {t("imports.script.openPlugin")}
            </Link>
          </p>
        ) : null}
        {create.error ? <ErrorLine error={create.error} /> : null}
      </div>
      {kind === "window" ? (
        <Fold title={t("common.details")} className="mt-1.5">
          <div className="flex flex-col gap-1 text-small text-sub">
            {m.guide ? <p className="m-0 text-label">{m.guide}</p> : null}
            <p className="m-0">{t("plugins.window.scrolling")}</p>
            <p className="m-0">{t("plugins.window.separate", { site: platform })}</p>
            <p className="m-0">{t("plugins.window.scriptBody", { plugin: use.plugin.name })}</p>
            <p className="m-0">{t("imports.rows.windowAfter")}</p>
          </div>
        </Fold>
      ) : null}
      {open && ready && (kind === "list" || kind === "link-list") ? <Sources plugin={use.plugin} method={m} onStarted={onStarted} /> : null}
      {/* 令牌只显示这一次：展开别的行时也留着，点「完成」才收起。 */}
      {kind === "script" && token ? (
        <ScriptSetup
          use={use}
          token={token}
          onDone={() => {
            setToken(null);
            onOpen(false);
          }}
        />
      ) : null}
    </li>
  );
}

/**
 * 导入页的「平台上已保存的内容」：已装的平台一个一组，每组列能导入的东西，每样一个主操作。
 * 只按链接一条条取的平台不分组，最后一行说一句。
 */
export function SourcePicker({ onStarted, initial }: { onStarted: (id: string) => void; initial?: string }) {
  const plugins = usePlugins();
  const sites = useSites();
  const visitors = useVisitors();
  const readers = useFileReaders().data ?? [];
  const names = useNames();
  const navigate = useNavigate();
  const [openKey, setOpenKey] = useState<string | null>(initial ?? null);
  const ids = new Set(readers.map((r) => r.id));
  const pages = plugins.data ? platformPages(localized(plugins.data, names)) : [];
  const groups = pages.map((page) => ({ page, uses: page.saved.filter((u) => u.method.built) })).filter((g) => g.uses.length > 0);
  const linkOnly = pages.filter((p) => !groups.some((g) => g.page.id === p.id));

  return (
    <section aria-label={t("imports.picker.title")} className="flex flex-col gap-4">
      {plugins.error ? <ErrorLine error={plugins.error} /> : null}
      {plugins.data && plugins.data.plugins.length === 0 ? (
        <div className="flex flex-wrap items-center gap-3">
          <p className="m-0 text-note text-sub">{t("imports.picker.noPlugins")}</p>
          <Button variant="soft" size="sm" onPress={() => void navigate({ to: "/plugins" })}>
            {t("imports.picker.getPlugins")}
          </Button>
        </div>
      ) : null}
      {groups.map(({ page, uses }) => (
        <section key={page.id} aria-label={page.name} className="min-w-0 rounded-card border border-border bg-card">
          <h2 className="m-0 px-4 pt-3.5 pb-1 text-section font-semibold text-text">{page.name}</h2>
          <ul className="m-0 flex list-none flex-col p-0">
            {uses.map((u) => (
              <Row
                key={keyOf(u)}
                use={u}
                kind={savedKind(u, ids)}
                page={page}
                reader={readers.find((r) => r.id === keyOf(u))}
                open={openKey === keyOf(u)}
                onOpen={(on) => setOpenKey(on ? keyOf(u) : null)}
                onStarted={onStarted}
                sites={sites.data ?? []}
                visitors={visitors.data ?? []}
              />
            ))}
          </ul>
        </section>
      ))}
      {linkOnly.length > 0 ? (
        <p className="m-0 text-small text-sub">
          {rich("imports.picker.linkOnly", {
            platforms: formatList(linkOnly.map((p) => p.name)),
            link: <Link to="/import" search={{ from: "links" }} className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text" />,
          })}
        </p>
      ) : null}
    </section>
  );
}
