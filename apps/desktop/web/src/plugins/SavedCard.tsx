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
import { accountText, ACCESS_TEXT, getsText, lastCallText, PAGE, TIER_TEXT } from "./words";

const KIND_TEXT: Record<SavedKind, string> = {
  file: PAGE.kindFile,
  script: PAGE.kindScript,
  phone: PAGE.kindPhone,
  "link-list": PAGE.kindLinkList,
  list: PAGE.kindList,
};

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
  const name = (u: Use) => (page.plugins.length > 1 ? `${u.method.name} · ${u.plugin.name}` : u.method.name);
  const options: SelectOption[] = [
    ...built.map((u) => ({ id: keyOf(u), text: name(u), sub: `${KIND_TEXT[savedKind(u, ids)]} · ${ACCESS_TEXT[u.method.access]}` })),
    ...later.map((u) => ({ id: keyOf(u), text: name(u), sub: PAGE.notBuilt, disabled: true })),
  ];

  if (page.saved.length === 0) {
    return (
      <Card title={PAGE.savedTitle}>
        <CardRow title={PAGE.savedNoneTitle} sub={PAGE.savedNoneSub} />
      </Card>
    );
  }

  return (
    <Card title={PAGE.savedTitle} sub={PAGE.savedSub}>
      <CardRow
        title={PAGE.importWay}
        sub={chosen ? `${KIND_TEXT[savedKind(chosen, ids)]}；${accountText(chosen.method.account, chosen.method.account.sites ?? chosen.plugin.platforms.map((x) => x.name))}` : PAGE.notBuilt}
        control={<Select label={PAGE.importWay} options={options} value={chosen ? keyOf(chosen) : null} placeholder={PAGE.notBuilt} onChange={setPicked} />}
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
      {PAGE.goImport}
    </Link>
  );
  const steps: { key: string; title: ReactNode; body?: ReactNode }[] =
    kind === "file"
      ? [
          { key: "get", title: PAGE.stepGetFile, body: <FileHow reader={reader} /> },
          {
            key: "pick",
            title: PAGE.stepPickFile,
            body: (
              <Link to="/import" search={{ from: "file", reader: key }} className="press inline-flex h-ctl-sm items-center rounded-btn border border-border2 bg-chip px-3.5 text-note font-medium text-text no-underline hover:border-label/60">
                {PAGE.pickFile}
              </Link>
            ),
          },
        ]
      : kind === "script"
        ? scriptSteps(use)
        : kind === "phone"
          ? [
              { key: "what", title: PAGE.stepWhat, body: <p className="m-0 text-note text-label">{getsText(m.gets).join("；") || m.name}</p> },
              { key: "phone", title: PAGE.stepPhone, body: ready },
            ]
          : kind === "link-list"
            ? [
                { key: "ready", title: PAGE.stepReady, body: ready },
                { key: "link", title: PAGE.stepLink, body: <Sources plugin={use.plugin} method={m} onStarted={(id) => void navigate({ to: "/import", search: { open: id } })} /> },
              ]
            : [
                { key: "ready", title: PAGE.stepReady, body: ready },
                { key: "go", title: PAGE.stepList((m.gets.list ?? []).join("、")), body: toImport },
              ];
  return (
    <div className="flex flex-col gap-3">
      <OffSwitch use={use} />
      <ol aria-label={PAGE.steps(m.name)} className="m-0 flex list-none flex-col gap-3.5 p-0">
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
      {m.riskNote ? <p className="m-0 text-small text-faint">{PAGE.risk(m.riskNote.text, m.riskNote.source.label)}</p> : null}
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
      <p className="m-0 min-w-0 flex-[1_1_240px] text-small text-label">{onByDefault(m.access) ? PAGE.offByYouSaved : PAGE.offByTierSaved(TIER_TEXT[m.access])}</p>
      <SmallSwitch label={PAGE.offSwitch(m.name)} isSelected={false} isDisabled={switcher.isPending} onChange={(v) => switcher.mutate({ methodId: m.id, enabled: v })} />
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
      {reader.lists.length > 0 ? <span className="mt-0.5 block text-small text-sub">{PAGE.readsAlso(reader.lists.join("、"))}</span> : null}
    </div>
  );
}

/** 浏览器脚本的几步：装脚本管理器、装这个脚本、给它一个令牌、打开你自己的那一页、点「交给 usuyo」。 */
function scriptSteps(use: Use): { key: string; title: ReactNode; body?: ReactNode }[] {
  return [
    {
      key: "manager",
      title: PAGE.stepManager,
      body: <p className="m-0 text-small text-sub">{PAGE.stepManagerBody}</p>,
    },
    { key: "script", title: PAGE.stepScript, body: <ScriptFile use={use} /> },
    { key: "token", title: PAGE.stepToken, body: <TokenStep name={PAGE.tokenName(use.plugin.name)} /> },
    { key: "page", title: PAGE.stepPage, body: <ScriptPage use={use} /> },
    {
      key: "hand",
      title: PAGE.stepSend,
      body: (
        <p className="m-0 text-small text-sub">
          {PAGE.sendBefore}
          <Link to="/import" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text">
            {PAGE.sendLink}
          </Link>
          {PAGE.sendAfter}
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
  if (!script) return <p className="m-0 text-small text-sub">{PAGE.noScript}</p>;
  if (!script.present) return <p className="m-0 text-small text-warn">{PAGE.scriptMissing(script.path)}</p>;
  return (
    <div className="min-w-0">
      <p className="m-0 text-small text-sub">{PAGE.stepScriptBody}</p>
      <code className="mono mt-1.5 block truncate rounded-sm bg-chip px-2.5 py-1.5 text-tag text-label select-text" title={script.path}>
        {script.path}
      </code>
      <div className="mt-1.5 flex flex-wrap items-center gap-1.5">
        <Button variant="outline" size="sm" isDisabled={reveal.isPending} onPress={() => reveal.mutate({ pluginId: use.plugin.id, methodId: use.method.id })}>
          <FolderOpen size={14} aria-hidden="true" />
          {PAGE.reveal}
        </Button>
        <Button
          variant="outline"
          size="sm"
          isDisabled={!text.data}
          onPress={() => {
            if (!text.data) return;
            navigator.clipboard.writeText(text.data).then(
              () => flash.show(PAGE.copiedScript),
              () => flash.show(PAGE.copyScriptFailed),
            );
          }}
        >
          <Copy size={14} aria-hidden="true" />
          {PAGE.copyScript}
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
  const platform = use.plugin.platforms.map((p) => p.name).join("、");
  return (
    <div className="text-small text-sub">
      {found?.pages ? (
        <p className="m-0 text-note text-label">
          {PAGE.pageBody(platform, found.pages)}
        </p>
      ) : (
        <p className="m-0 text-note text-label">{PAGE.pageFallback(platform)}</p>
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
  const scripts = (tokens.data ?? []).filter((t) => t.kind === "extension");
  return (
    <div className="min-w-0">
      <p className="m-0 text-small text-sub">
        {PAGE.tokenBody}
        {scripts.length > 0 ? PAGE.tokensIssued(scripts.map((t) => t.name).join("、")) : PAGE.tokensNone}
      </p>
      {fresh ? (
        <div className="mt-1.5 rounded-btn border border-border2 bg-card2 px-3 py-2.5">
          <div className="text-small text-label">{PAGE.tokenOnce}</div>
          <div className="mt-1.5 flex items-center gap-1.5">
            <code className="mono min-w-0 flex-1 truncate rounded-sm bg-chip px-2.5 py-1.5 text-tag text-text select-all">{fresh.token}</code>
            <Button
              variant="outline"
              size="sm"
              square
              aria-label={PAGE.copyToken}
              onPress={() => {
                navigator.clipboard.writeText(fresh.token).then(
                  () => flash.show(PAGE.copied),
                  () => flash.show(PAGE.copyFailed),
                );
              }}
            >
              <Copy size={14} aria-hidden="true" />
            </Button>
            <Button variant="quiet" size="sm" onPress={() => setFresh(null)}>
              {PAGE.done}
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
                onSuccess: (t) => {
                  setFresh(t);
                  create.reset();
                },
              },
            )
          }
        >
          {PAGE.issueToken}
        </Button>
      )}
      {create.error ? <ErrorLine error={create.error} className="mt-1" /> : null}
    </div>
  );
}
