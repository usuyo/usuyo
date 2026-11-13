import { Link, useNavigate } from "@tanstack/react-router";
import { Copy, FolderOpen } from "lucide-react";
import { useState, type ReactNode } from "react";
import type { FileReader, NewDeliverToken, SiteLogin, VisitorIdentity } from "../api/endpoints";
import { useCreateDeliverToken, useDeliverTokens, useMethodScript, useRevealScript, useSwitchMethod } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Sources } from "../imports/SourcePicker";
import { Button } from "../ui/Button";
import { Card, CardRow } from "../ui/Card";
import { FlashLine, useFlash } from "../ui/Flash";
import { Select, type SelectOption } from "../ui/Select";
import { SmallSwitch } from "./LinkCard";
import { Needs } from "./Needs";
import { onByDefault, savedKind, scriptPages, keyOf, type PlatformPage, type SavedKind, type Use } from "./platforms";
import { rich, t } from "../i18n";
import { formatList } from "../i18n/format";
import { accountText, ACCESS_TEXT, clauses, getsText, lastCallText, TIER_TEXT } from "./words";

function kindText(kind: SavedKind): string {
  const text: Record<SavedKind, () => string> = {
    file: () => t("plugins.saved.kind.file"),
    script: () => t("plugins.saved.kind.script"),
    phone: () => t("plugins.saved.kind.phone"),
    "link-list": () => t("plugins.saved.kind.linkList"),
    list: () => t("plugins.saved.kind.list"),
  };
  return text[kind]();
}

/**
 * 「拿回存过的」：一个选择框选一种（做好了的在前，没做的灰着），下面是照这一种走的几步，每步写现在怎样、在哪动手。
 * 选它不拨开关：拿回存过的是你点了才去做的。
 */
export function SavedCard({ page, sites, visitors, readers }: { page: PlatformPage; sites: SiteLogin[]; visitors: VisitorIdentity[]; readers: FileReader[] }) {
  const ids = new Set(readers.map((r) => r.id));
  const built = page.saved.filter((u) => u.method.built);
  const later = page.saved.filter((u) => !u.method.built);
  const [picked, setPicked] = useState<string | null>(null);
  const chosen = built.find((u) => keyOf(u) === picked) ?? built.find((u) => u.method.usable) ?? built[0];
  const name = (u: Use) => (page.plugins.length > 1 ? t("plugins.link.optionName", { method: u.method.name, plugin: u.plugin.name }) : u.method.name);
  const options: SelectOption[] = [
    ...built.map((u) => ({ id: keyOf(u), text: name(u), sub: t("plugins.link.optionSub", { first: kindText(savedKind(u, ids)), second: ACCESS_TEXT[u.method.access] }) })),
    ...later.map((u) => ({ id: keyOf(u), text: name(u), sub: t("plugins.link.notBuilt"), disabled: true })),
  ];

  if (page.saved.length === 0) {
    return (
      <Card title={t("plugins.saved.title")}>
        <CardRow title={t("plugins.saved.noneTitle")} sub={t("plugins.saved.noneSub")} />
      </Card>
    );
  }

  return (
    <Card title={t("plugins.saved.title")} sub={t("plugins.saved.sub")}>
      <CardRow
        title={t("plugins.saved.way")}
        sub={
          chosen
            ? t("plugins.saved.waySub", { kind: kindText(savedKind(chosen, ids)), account: accountText(chosen.method.account, chosen.method.account.sites ?? chosen.plugin.platforms.map((x) => x.name)) })
            : t("plugins.link.notBuilt")
        }
        control={<Select label={t("plugins.saved.way")} options={options} value={chosen ? keyOf(chosen) : null} placeholder={t("plugins.link.notBuilt")} onChange={setPicked} />}
      >
        {chosen ? <Steps key={keyOf(chosen)} use={chosen} kind={savedKind(chosen, ids)} reader={readers.find((r) => r.id === keyOf(chosen))} sites={sites} visitors={visitors} /> : null}
      </CardRow>
    </Card>
  );
}

function Steps({ use, kind, reader, sites, visitors }: { use: Use; kind: SavedKind; reader: FileReader | undefined; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  const m = use.method;
  const key = keyOf(use);
  const navigate = useNavigate();
  const ready = <Needs use={use} sites={sites} visitors={visitors} />;
  const toImport = (
    <Link to="/import" search={{ pick: key }} className="press inline-flex h-ctl-sm items-center rounded-btn border border-border2 bg-chip px-3.5 text-note font-medium text-text no-underline hover:border-label/60">
      {t("plugins.saved.goImport")}
    </Link>
  );
  const steps: { key: string; title: ReactNode; body?: ReactNode }[] =
    kind === "file"
      ? [
          { key: "get", title: t("plugins.saved.stepGetFile"), body: <FileHow reader={reader} /> },
          {
            key: "pick",
            title: t("plugins.saved.stepPickFile"),
            body: (
              <Link to="/import" search={{ from: "file", reader: key }} className="press inline-flex h-ctl-sm items-center rounded-btn border border-border2 bg-chip px-3.5 text-note font-medium text-text no-underline hover:border-label/60">
                {t("plugins.saved.pickFile")}
              </Link>
            ),
          },
        ]
      : kind === "script"
        ? scriptSteps(use)
        : kind === "phone"
          ? [
              { key: "what", title: t("plugins.saved.stepWhat"), body: <p className="m-0 text-note text-label">{clauses(getsText(m.gets)) || m.name}</p> },
              { key: "phone", title: t("plugins.saved.stepPhone"), body: ready },
            ]
          : kind === "link-list"
            ? [
                { key: "ready", title: t("plugins.saved.stepReady"), body: ready },
                { key: "link", title: t("plugins.saved.stepLink"), body: <Sources plugin={use.plugin} method={m} onStarted={(id) => void navigate({ to: "/import", search: { open: id } })} /> },
              ]
            : [
                { key: "ready", title: t("plugins.saved.stepReady"), body: ready },
                {
                  key: "go",
                  title: (m.gets.list ?? []).length > 0 ? t("plugins.saved.stepList", { lists: formatList(m.gets.list ?? []) }) : t("plugins.saved.stepListAny"),
                  body: toImport,
                },
              ];
  return (
    <div className="flex flex-col gap-3">
      <OffSwitch use={use} />
      <ol aria-label={t("plugins.saved.steps", { name: m.name })} className="m-0 flex list-none flex-col gap-3.5 p-0">
        {steps.map((s, i) => (
          <li key={s.key} className="flex gap-2.5">
            <span className="mono flex size-5 shrink-0 items-center justify-center rounded-full border border-border2 text-tag text-label">{i + 1}</span>
            <div className="min-w-0 flex-1">
              <div className="text-note font-medium text-text">{s.title}</div>
              {s.body ? <div className="mt-1">{s.body}</div> : null}
            </div>
          </li>
        ))}
      </ol>
      {m.riskNote ? <p className="m-0 text-small text-faint">{t("plugins.saved.risk", { text: m.riskNote.text, source: m.riskNote.source.label })}</p> : null}
      {m.lastCall ? <p className={`m-0 text-small ${m.lastCall.ok ? "text-sub" : "text-warn"}`}>{lastCallText(m.lastCall)}</p> : null}
    </div>
  );
}

/** 这种方法关着时：为什么关着（默认关着的那一层，或你关的），和它的开关。 */
function OffSwitch({ use }: { use: Use }) {
  const switcher = useSwitchMethod(use.plugin.id);
  const m = use.method;
  if (m.enabled) return null;
  return (
    <div className="flex flex-wrap items-start justify-between gap-x-4 gap-y-1.5 rounded-btn bg-card2 px-3 py-2.5">
      <p className="m-0 min-w-0 flex-[1_1_240px] text-small text-label">{onByDefault(m.access) ? t("plugins.saved.offByYou") : t("plugins.saved.offByTier", { tier: TIER_TEXT[m.access] })}</p>
      <SmallSwitch label={t("plugins.saved.offSwitch", { name: m.name })} isSelected={false} isDisabled={switcher.isPending} onChange={(v) => switcher.mutate({ methodId: m.id, enabled: v })} />
      {switcher.error ? <ErrorLine error={switcher.error} className="w-full" /> : null}
    </div>
  );
}

function FileHow({ reader }: { reader: FileReader | undefined }) {
  if (!reader) return null;
  return (
    <div className="text-note text-label">
      {reader.reads}
      {reader.patterns.length > 0 ? <span className="mono mt-0.5 block text-tag text-faint">{reader.patterns.join("  ")}</span> : null}
      {reader.lists.length > 0 ? <span className="mt-0.5 block text-small text-sub">{t("plugins.saved.readsAlso", { lists: formatList(reader.lists) })}</span> : null}
    </div>
  );
}

/** 浏览器脚本的几步：装脚本管理器、装这个脚本、给它一个令牌、打开你自己的那一页、点「交给 usuyo」。 */
function scriptSteps(use: Use): { key: string; title: ReactNode; body?: ReactNode }[] {
  return [
    {
      key: "manager",
      title: t("plugins.script.stepManager"),
      body: <p className="m-0 text-small text-sub">{t("plugins.script.stepManagerBody")}</p>,
    },
    { key: "script", title: t("plugins.script.stepScript"), body: <ScriptFile use={use} /> },
    { key: "token", title: t("plugins.script.stepToken"), body: <TokenStep name={t("plugins.script.tokenName", { plugin: use.plugin.name })} /> },
    { key: "page", title: t("plugins.script.stepPage"), body: <ScriptPage use={use} /> },
    {
      key: "hand",
      title: t("plugins.script.stepSend"),
      body: (
        <p className="m-0 text-small text-sub">
          {rich("plugins.script.sendBody", {
            sendButton: t("plugins.quoted.sendButton"),
            diagnoseButton: t("plugins.quoted.diagnoseButton"),
            link: <Link to="/import" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text" />,
          })}
        </p>
      ),
    },
  ];
}

function ScriptFile({ use }: { use: Use }) {
  const script = use.method.script;
  const reveal = useRevealScript();
  const text = useMethodScript(use.plugin.id, use.method.id, script?.present === true);
  const flash = useFlash();
  if (!script) return <p className="m-0 text-small text-sub">{t("plugins.script.none")}</p>;
  if (!script.present) return <p className="m-0 text-small text-warn">{t("plugins.script.missing", { path: script.path })}</p>;
  return (
    <div className="min-w-0">
      <p className="m-0 text-small text-sub">{t("plugins.script.stepScriptBody")}</p>
      <code className="mono mt-1.5 block truncate rounded-sm bg-chip px-2.5 py-1.5 text-tag text-label select-text" title={script.path}>
        {script.path}
      </code>
      <div className="mt-1.5 flex flex-wrap items-center gap-1.5">
        <Button variant="outline" size="sm" isDisabled={reveal.isPending} onPress={() => reveal.mutate({ pluginId: use.plugin.id, methodId: use.method.id })}>
          <FolderOpen size={14} aria-hidden="true" />
          {t("plugins.script.reveal")}
        </Button>
        <Button
          variant="outline"
          size="sm"
          isDisabled={!text.data}
          onPress={() => {
            if (!text.data) return;
            navigator.clipboard.writeText(text.data).then(
              () => flash.show(t("plugins.script.copied")),
              () => flash.show(t("plugins.script.copyFailed")),
            );
          }}
        >
          <Copy size={14} aria-hidden="true" />
          {t("plugins.script.copy")}
        </Button>
        <FlashLine flash={flash} className="text-small text-sub" />
      </div>
      {reveal.error ? <ErrorLine error={reveal.error} className="mt-1" /> : null}
      {text.error ? <ErrorLine error={text.error} className="mt-1" /> : null}
    </div>
  );
}

/** 脚本自己写的那一页（说明里的「你自己的……页」和 `@match`）。 */
function ScriptPage({ use }: { use: Use }) {
  const text = useMethodScript(use.plugin.id, use.method.id, use.method.script?.present === true);
  const found = text.data ? scriptPages(text.data) : null;
  const platform = formatList(use.plugin.platforms.map((p) => p.name));
  return (
    <div className="text-small text-sub">
      {found?.pages ? (
        <p className="m-0 text-note text-label">
          {t("plugins.script.pageBody", { platform, pages: found.pages })}
        </p>
      ) : (
        <p className="m-0 text-note text-label">{t("plugins.script.pageFallback", { platform })}</p>
      )}
      {found && found.matches.length > 0 ? <span className="mono mt-0.5 block text-tag text-faint">{found.matches.join("  ")}</span> : null}
    </div>
  );
}

/** 就地发一个投递令牌：只能往 usuyo 交，读不了资料库；只显示这一次。 */
function TokenStep({ name }: { name: string }) {
  const tokens = useDeliverTokens();
  const create = useCreateDeliverToken();
  const [fresh, setFresh] = useState<NewDeliverToken | null>(null);
  const flash = useFlash();
  const scripts = (tokens.data ?? []).filter((x) => x.kind === "extension");
  return (
    <div className="min-w-0">
      <p className="m-0 text-small text-sub">
        {scripts.length > 0 ? t("plugins.script.tokenIntroIssued", { names: formatList(scripts.map((x) => x.name)) }) : t("plugins.script.tokenIntro")}
      </p>
      {fresh ? (
        <div className="mt-1.5 rounded-btn border border-border2 bg-card2 px-3 py-2.5">
          <div className="text-small text-label">{t("plugins.script.tokenOnce")}</div>
          <div className="mt-1.5 flex items-center gap-1.5">
            <code className="mono min-w-0 flex-1 truncate rounded-sm bg-chip px-2.5 py-1.5 text-tag text-text select-all">{fresh.token}</code>
            <Button
              variant="outline"
              size="sm"
              square
              aria-label={t("plugins.script.copyToken")}
              onPress={() => {
                navigator.clipboard.writeText(fresh.token).then(
                  () => flash.show(t("common.copied")),
                  () => flash.show(t("common.copyFailed")),
                );
              }}
            >
              <Copy size={14} aria-hidden="true" />
            </Button>
            <Button variant="quiet" size="sm" onPress={() => setFresh(null)}>
              {t("common.done")}
            </Button>
          </div>
          <FlashLine flash={flash} className="mt-1 block text-small text-sub" />
        </div>
      ) : (
        <Button
          variant="soft"
          size="sm"
          className="mt-1.5"
          isDisabled={create.isPending}
          onPress={() =>
            create.mutate(
              { name, kind: "extension" },
              {
                onSuccess: (made) => {
                  setFresh(made);
                  create.reset();
                },
              },
            )
          }
        >
          {t("plugins.script.issueToken")}
        </Button>
      )}
      {create.error ? <ErrorLine error={create.error} className="mt-1" /> : null}
    </div>
  );
}
