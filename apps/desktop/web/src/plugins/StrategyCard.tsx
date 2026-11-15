import { Switch } from "react-aria-components";
import type { FunctionRoute, PlatformStrategy, SiteLogin, StrategyView, VisitorIdentity } from "../api/endpoints";
import { useSetStrategy, useStrategy, useSwitchMethod } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { openExternal } from "../shell";
import { Card, CardRow } from "../ui/Card";
import { Select, type SelectOption } from "../ui/Select";
import { Needs } from "./Needs";
import { accessOn, enabledOn, onByDefault, type PlatformPage, type Use } from "./platforms";
import { functionName, presetName, wayUse, wayKey, wayName, withPick, withPlatformPreset } from "./strategy";
import { t } from "../i18n";
import { ACCESS_TEXT, lastCallText } from "./words";

const FOLLOW = "follow";
/** 一个平台能另选的预设：「自定义」只在总策略上。 */
const PLATFORM_PRESETS: NonNullable<PlatformStrategy["preset"]>[] = ["no-account-only", "no-account-first", "account-first"];
const FROM: Record<FunctionRoute["from"], () => string> = {
  function: () => t("plugins.strategy.from.function"),
  platform: () => t("plugins.strategy.from.platform"),
  strategy: () => t("plugins.strategy.from.strategy"),
};
const SKIP = "skip";

/**
 * 「获取策略」：这个平台跟随总策略还是另选一个预设；下面一项功能一行，写现在照先后会用哪几种方式、带不带账号，
 * 和一个选择框（跟随策略、指定一种、不获取）。先后由内核照规则算好（`GET /strategy`），这里只照着摆。
 */
export function StrategyCard({ page, sites, visitors }: { page: PlatformPage; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  const strategy = useStrategy();
  const set = useSetStrategy();
  const v = strategy.data;
  if (!v) {
    return <Card title={t("plugins.strategy.title")}>{strategy.error ? <CardRow title={<ErrorLine error={strategy.error} />} /> : null}</Card>;
  }
  const routes = v.routes.find((r) => r.platform.id === page.id)?.functions ?? [];
  const own = v.platforms?.[page.id];
  const options: SelectOption[] = [
    { id: FOLLOW, text: t("plugins.strategy.follow"), sub: t("plugins.strategy.followSub", { name: presetName(v.preset) }) },
    ...PLATFORM_PRESETS.map((p) => ({ id: p, text: presetName(p) })),
  ];
  return (
    <Card title={t("plugins.strategy.title")} sub={t("plugins.strategy.sub")}>
      <CardRow
        title={t("plugins.strategy.platform")}
        sub={t("plugins.strategy.platformSub")}
        control={
          <Select
            label={t("plugins.strategy.platform")}
            options={options}
            value={own?.preset ?? FOLLOW}
            isDisabled={set.isPending}
            onChange={(id) => set.mutate(withPlatformPreset(v, page.id, PLATFORM_PRESETS.find((p) => p === id) ?? null))}
          />
        }
      >
        {set.error ? <ErrorLine error={set.error} /> : null}
      </CardRow>
      {routes.length === 0 ? <CardRow title={t("plugins.strategy.none")} /> : null}
      {routes.map((r) => (
        <FunctionRow key={r.function} route={r} view={v} page={page} sites={sites} visitors={visitors} pending={set.isPending} onPick={(pick) => set.mutate(withPick(v, page.id, r.function, pick))} />
      ))}
    </Card>
  );
}

function FunctionRow({
  route,
  view,
  page,
  sites,
  visitors,
  pending,
  onPick,
}: {
  route: FunctionRoute;
  view: StrategyView;
  page: PlatformPage;
  sites: SiteLogin[];
  visitors: VisitorIdentity[];
  pending: boolean;
  onPick: (pick: string | null) => void;
}) {
  const name = functionName(route.function);
  const plugins = page.plugins;
  const ways = route.ways.map((w) => wayUse(plugins, w)).filter((u): u is Use => u !== undefined);
  const picked = view.platforms?.[page.id]?.functions?.[route.function];
  const nameOf = (u: Use) => (plugins.length > 1 ? t("plugins.link.optionName", { method: wayName(u), plugin: u.plugin.name }) : wayName(u));
  // 跟随策略时会用的第一种：选择框里那一项下面的小字。指定了的时候不知道策略会给什么，不写。
  const first = ways[0];
  const now = route.skip ? t("plugins.strategy.skip") : first ? nameOf(first) : t("plugins.strategy.skip");
  const options: SelectOption[] = [
    { id: FOLLOW, text: t("plugins.strategy.followPick"), ...(picked === undefined ? { sub: t("plugins.strategy.followPickSub", { way: now }) } : {}) },
    ...route.choices.flatMap((w) => {
      const u = wayUse(plugins, w);
      return u ? [{ id: wayKey(w), text: nameOf(u), sub: ACCESS_TEXT[accessOn(u, page.id)] }] : [];
    }),
    { id: SKIP, text: t("plugins.strategy.skip"), sub: t("plugins.strategy.skipSub") },
  ];
  return (
    <CardRow
      title={name}
      sub={FROM[route.from]()}
      control={<Select label={t("plugins.strategy.pick", { name })} options={options} value={picked ?? FOLLOW} isDisabled={pending} onChange={(id) => onPick(id === FOLLOW ? null : id)} />}
    >
      {route.skip ? (
        <p className="m-0 text-small text-sub">{t("plugins.strategy.skipped")}</p>
      ) : ways.length === 0 ? (
        <p className="m-0 text-small text-sub">{t("plugins.strategy.noWay")}</p>
      ) : (
        <div className="flex flex-col gap-2.5">
          {ways.length > 1 ? <p className="m-0 text-small text-sub">{t("plugins.strategy.order")}</p> : null}
          <ol aria-label={t("plugins.strategy.orderLabel", { name })} className="m-0 flex list-none flex-col gap-3 p-0">
            {ways.map((u, i) => (
              <li key={`${u.plugin.id}#${u.method.id}`} className="flex gap-2.5">
                <span className="mono w-4 shrink-0 pt-px text-tag text-faint">{i + 1}</span>
                <WayLine use={u} name={nameOf(u)} platform={page.id} sites={sites} visitors={visitors} />
              </li>
            ))}
          </ol>
        </div>
      )}
    </CardRow>
  );
}

/** 一种方式：名字和带不带账号、层级；用得上写上一次的结果，关着的带开关，缺东西的照 `Needs` 一样一行。 */
function WayLine({ use, name, platform, sites, visitors }: { use: Use; name: string; platform: string; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  const switcher = useSwitchMethod(use.plugin.id);
  const m = use.method;
  const access = accessOn(use, platform);
  return (
    <div className="min-w-0 flex-1">
      <div className="flex flex-wrap items-baseline gap-x-2">
        <span className="text-note font-medium text-text">{name}</span>
        <span className="text-small text-sub">{ACCESS_TEXT[access]}</span>
      </div>
      {!enabledOn(use, platform) ? (
        <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1">
          <span className="text-small text-label">{onByDefault(access) ? t("plugins.strategy.off") : t("plugins.strategy.offTier")}</span>
          <SmallSwitch label={t("plugins.strategy.turnOn", { name: m.name })} isSelected={false} isDisabled={switcher.isPending} onChange={(on) => switcher.mutate({ methodId: m.id, enabled: on })} />
        </div>
      ) : m.usable ? (
        <div className="text-small text-label">{m.lastCall ? lastCallText(m.lastCall) : t("plugins.usable")}</div>
      ) : (
        <div className="mt-1.5">
          <Needs use={use} sites={sites} visitors={visitors} />
          {m.lastCall ? <div className={`mt-1 text-small ${m.lastCall.ok ? "text-label" : "text-warn"}`}>{lastCallText(m.lastCall)}</div> : null}
        </div>
      )}
      {m.riskNote ? (
        <p className="m-0 mt-1 text-small text-sub">
          {m.riskNote.text}
          <button
            type="button"
            onClick={() => void openExternal(m.riskNote?.source.url ?? "")}
            className="ml-1 cursor-pointer border-0 bg-transparent p-0 font-sans text-small text-label underline decoration-border2 underline-offset-4 hover:text-text"
          >
            {m.riskNote.source.label}
          </button>
        </p>
      ) : null}
      {switcher.error ? <ErrorLine error={switcher.error} className="mt-1.5" /> : null}
    </div>
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
