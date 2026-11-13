import type { ReactNode } from "react";
import { Switch } from "react-aria-components";
import type { SiteLogin, VisitorIdentity } from "../api/endpoints";
import { useSwitchMethod, useSwitchMethods } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { openExternal } from "../shell";
import { Card, CardRow } from "../ui/Card";
import { Select, type SelectOption } from "../ui/Select";
import { Needs } from "./Needs";
import { accessOn, alsoMoves, autoOrder, enabledOn, linkChoice, linkSwitches, onByDefault, keyOf, type PlatformPage, type Use } from "./platforms";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import { accountText, ACCESS_TEXT, clauses, getsText, lastCallText, OUTPUT_TEXT, TIER_TEXT } from "./words";

const AUTO = "auto";

/** 这种方法的账号用在哪，名字照内核给的。 */
function accountOf(u: Use): string {
  return accountText(u.method.account, u.method.account.sites ?? u.plugin.platforms.map((x) => x.name));
}

/** 选项和说明里的名字：一个平台有几个插件时带上插件的名字。 */
function nameOf(u: Use, page: PlatformPage): string {
  return page.plugins.length > 1 ? t("plugins.link.optionName", { method: u.method.name, plugin: u.plugin.name }) : u.method.name;
}

/** 一种方法现在怎样，一句：用得上，或内核说的第一个原因。 */
function stateOf(u: Use, platform: string): { text: string; ok: boolean } {
  if (!enabledOn(u, platform)) return { text: onByDefault(accessOn(u, platform)) ? t("plugins.link.offByYou") : t("plugins.link.offByTier"), ok: false };
  if (u.method.usable) return { text: t("plugins.usable"), ok: true };
  return { text: clauses(u.method.blockers.map((b) => b.hint)), ok: false };
}

/**
 * 「收一条链接」：一个选择框选用哪种方法，下面是选中那一项的说明。「自动」是内核自己的先后（默认开着的都用，层级低、不带账号的先，
 * 缺的问下一种）；选一种是这个平台上收链接只用它。选了就照它拨开关：选一种开它、关掉这个平台上别的收链接的方法，「自动」把每种拨回它那一层的默认。
 */
export function LinkCard({ page, sites, visitors }: { page: PlatformPage; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  const switches = useSwitchMethods();
  const choice = linkChoice(page.link, page.id);
  const value = choice.kind === "auto" ? AUTO : choice.kind === "only" ? choice.key : null;
  const built = page.link.filter((u) => u.method.built);
  // 只有一种做好了、默认开着：选它和「自动」是一回事，不另列。
  const lone = built.length === 1 && built[0] !== undefined && onByDefault(accessOn(built[0], page.id)) ? built[0] : undefined;
  const multi = built.filter((u) => u.plugin.platforms.length > 1);
  const options: SelectOption[] = [
    { id: AUTO, text: t("plugins.link.auto"), sub: lone ? t("plugins.link.autoLone", { name: nameOf(lone, page) }) : t("plugins.link.autoSub") },
    ...(lone ? [] : built).map((u) => ({ id: keyOf(u), text: nameOf(u, page), sub: t("plugins.link.optionSub", { first: accountOf(u), second: ACCESS_TEXT[accessOn(u, page.id)] }) })),
    ...page.link.filter((u) => !u.method.built).map((u) => ({ id: keyOf(u), text: nameOf(u, page), sub: t("plugins.link.notBuilt"), disabled: true })),
  ];
  const chosen = choice.kind === "only" ? built.find((u) => keyOf(u) === choice.key) : undefined;

  if (page.link.length === 0) {
    return (
      <Card title={t("plugins.link.title")}>
        <CardRow title={t("plugins.link.noneTitle")} sub={t("plugins.link.noneSub")} />
      </Card>
    );
  }

  return (
    <Card title={t("plugins.link.title")} sub={t("plugins.link.sub")}>
      <CardRow
        title={t("plugins.link.wayTitle")}
        sub={choice.kind === "mixed" ? t("plugins.link.wayMixedSub") : t("plugins.link.waySub")}
        control={
          <Select
            label={t("plugins.link.wayTitle")}
            options={options}
            value={value}
            placeholder={t("plugins.link.wayMixed")}
            isDisabled={switches.isPending}
            onChange={(id) => switches.mutate(linkSwitches(page.link, page.id, id))}
          />
        }
      >
        {switches.error ? <ErrorLine error={switches.error} className="mb-2" /> : null}
        {multi.map((u) => (
          <p key={keyOf(u)} className="m-0 mb-2.5 text-small text-sub">
            {t("plugins.link.alsoMoves", { name: nameOf(u, page), others: formatList(alsoMoves(u, page.id)) })}
          </p>
        ))}
        {chosen ? (
          <MethodExplain use={chosen} page={page} sites={sites} visitors={visitors} />
        ) : (
          <AutoExplain page={page} sites={sites} visitors={visitors} mixed={choice.kind === "mixed"} />
        )}
      </CardRow>
      {page.parts.length > 0 && chosen === undefined ? <Parts page={page} sites={sites} visitors={visitors} /> : null}
    </Card>
  );
}

/** 「自动」：开着的几种照内核的先后排，各写现在怎样；关着的一行带过。 */
function AutoExplain({ page, sites, visitors, mixed }: { page: PlatformPage; sites: SiteLogin[]; visitors: VisitorIdentity[]; mixed: boolean }) {
  const order = autoOrder(page.link, page.id);
  const on = order.filter((u) => enabledOn(u, page.id));
  const off = order.filter((u) => !enabledOn(u, page.id));
  return (
    <div className="flex flex-col gap-3">
      <p className="m-0 text-note text-label">
        {mixed ? t("plugins.link.mixedIntro") : t("plugins.link.autoIntro")}
      </p>
      {on.length > 0 ? (
        <ol className="m-0 flex list-none flex-col gap-3 p-0">
          {on.map((u, i) => {
            const state = stateOf(u, page.id);
            return (
              <li key={keyOf(u)} className="flex gap-2.5">
                <span className="mono w-4 shrink-0 pt-px text-tag text-faint">{i + 1}</span>
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-baseline gap-x-2">
                    <span className="text-note font-medium text-text">{nameOf(u, page)}</span>
                    <span className="text-small text-sub">{ACCESS_TEXT[accessOn(u, page.id)]}</span>
                  </div>
                  {/* 用不上的原因由下面那几行一样一样写，这里不重复。 */}
                  {u.method.usable ? <div className="text-small text-label">{state.text}</div> : (
                    <div className="mt-1.5">
                      <Needs use={u} sites={sites} visitors={visitors} />
                    </div>
                  )}
                </div>
              </li>
            );
          })}
        </ol>
      ) : (
        <p className="m-0 text-small text-sub">{t("plugins.link.noneOn")}</p>
      )}
      {off.length > 0 ? (
        <p className="m-0 text-small text-sub">
          {t("plugins.link.offList", { list: formatList(off.map((u) => t("plugins.link.offItem", { name: nameOf(u, page), state: stateOf(u, page.id).text }))) })}
        </p>
      ) : null}
    </div>
  );
}

/** 选中的那一种：取得到什么、账号用在哪、这一层的意思、要的几样、记下的风险、上一次的结果。 */
function MethodExplain({ use, page, sites, visitors }: { use: Use; page: PlatformPage; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  const m = use.method;
  const access = accessOn(use, page.id);
  const gets = getsText(m.gets);
  return (
    <dl className="m-0 grid gap-x-4 gap-y-2.5 @xl:grid-cols-[96px_minmax(0,1fr)]">
      <Term name={t("plugins.link.termGets")}>{gets.length > 0 ? clauses(gets) : t("plugins.link.getsUnknown")}</Term>
      <Term name={t("plugins.link.termAccount")}>{accountOf(use)}</Term>
      <Term name={t("plugins.link.termTier")}>
        {ACCESS_TEXT[access]}
        <span className="mono ml-1.5 text-tag text-faint">{access}</span>
        <span className="block text-small text-sub">{TIER_TEXT[access]}</span>
      </Term>
      <Term name={t("plugins.link.termNeeds")}>
        <Needs use={use} sites={sites} visitors={visitors} />
      </Term>
      {m.riskNote ? (
        <Term name={t("plugins.link.termRisk")}>
          {m.riskNote.text}
          <button
            type="button"
            onClick={() => void openExternal(m.riskNote?.source.url ?? "")}
            className="ml-1 cursor-pointer border-0 bg-transparent p-0 font-sans text-small text-label underline decoration-border2 underline-offset-4 hover:text-text"
          >
            {m.riskNote.source.label}
          </button>
        </Term>
      ) : null}
      <Term name={t("plugins.link.termLast")}>{m.lastCall ? <span className={m.lastCall.ok ? "" : "text-warn"}>{lastCallText(m.lastCall)}</span> : t("plugins.link.neverUsed")}</Term>
    </dl>
  );
}

function Term({ name, children }: { name: string; children: ReactNode }) {
  return (
    <>
      <dt className="text-small text-faint">{name}</dt>
      <dd className="m-0 min-w-0 text-note text-label">{children}</dd>
    </>
  );
}

/** 「自动」下只补上一样的几种（例如只取视频的）：各自一行，写补上什么、要的几样，和自己的开关。 */
function Parts({ page, sites, visitors }: { page: PlatformPage; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  return (
    <>
      {page.parts.map((u) => (
        <PartRow key={keyOf(u)} use={u} page={page} sites={sites} visitors={visitors} />
      ))}
    </>
  );
}

function PartRow({ use, page, sites, visitors }: { use: Use; page: PlatformPage; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  const switcher = useSwitchMethod(use.plugin.id);
  const m = use.method;
  const on = enabledOn(use, page.id);
  const what = formatList((m.gets.fetch ?? []).map((k) => OUTPUT_TEXT[k]));
  return (
    <CardRow
      title={t("plugins.link.partTitle", { what })}
      sub={t("plugins.link.partSub", { name: nameOf(use, page), account: accountOf(use) })}
      control={
        m.built ? (
          <SmallSwitch label={t("plugins.link.partSwitch", { name: m.name })} isSelected={on} isDisabled={switcher.isPending} onChange={(v) => switcher.mutate({ methodId: m.id, enabled: v })} />
        ) : (
          <span className="text-small text-faint">{t("plugins.link.notBuilt")}</span>
        )
      }
    >
      {m.built && on && !m.usable ? <Needs use={use} sites={sites} visitors={visitors} /> : null}
      {switcher.error ? <ErrorLine error={switcher.error} className="mt-1.5" /> : null}
    </CardRow>
  );
}

/** 小开关：拨了马上生效。 */
export function SmallSwitch({ label, isSelected, isDisabled, onChange }: { label: string; isSelected: boolean; isDisabled?: boolean; onChange: (on: boolean) => void }) {
  return (
    <Switch aria-label={label} isSelected={isSelected} isDisabled={isDisabled} onChange={onChange} className="group flex cursor-pointer items-center gap-2 font-sans text-small text-sub outline-none data-[disabled]:cursor-default">
      <span>{isSelected ? t("plugins.link.switchOn") : t("plugins.link.switchOff")}</span>
      <span className="flex h-5 w-8 items-center rounded-full border border-border2 bg-chip px-0.5 transition-colors duration-150 group-data-[focus-visible]:outline-2 group-data-[focus-visible]:outline-offset-2 group-data-[focus-visible]:outline-accent group-data-[selected]:border-label group-data-[selected]:bg-label">
        <span className="size-3.5 rounded-full bg-text transition-transform duration-150 ease-out group-data-[selected]:translate-x-3 group-data-[selected]:bg-bg motion-reduce:transition-none" />
      </span>
    </Switch>
  );
}
