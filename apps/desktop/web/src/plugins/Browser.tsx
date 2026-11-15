import { Link } from "@tanstack/react-router";
import { LayoutGrid, List, Search, X } from "lucide-react";
import { Button as AriaButton, Input, SearchField, ToggleButton, ToggleButtonGroup, type Key } from "react-aria-components";
import type { CatalogPlugin } from "../api/endpoints";
import { useCatalog, usePlugins } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Choice } from "../ui/Choice";
import { Select } from "../ui/Select";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import { filterPlugins, platformsOf, sharedName, stateOf, useBrowserStore, type StateFilter, type ViewMode } from "./catalog";
import { NotLoaded } from "./PluginsPage";
import { SourcesCard } from "./SourcesCard";

const KINDS: CatalogPlugin["kind"][] = ["fetch", "transform", "export"];

/** `/plugins`：订阅源里的和装着的插件，卡片或列表，可以搜索、按平台和状态筛选；下面是订阅源。 */
export function BrowserRoute() {
  const catalog = useCatalog();
  const plugins = usePlugins();
  const { view, query, platform, state, setView, setQuery, setPlatform, setState } = useBrowserStore();
  const all = catalog.data?.plugins ?? [];
  const shown = filterPlugins(all, { query, platform, state });
  const platforms = platformsOf(all);
  return (
    <div className="flex min-w-0 animate-fade flex-col">
      <h1 className="m-0 text-title font-semibold text-text">{t("plugins.page.title")}</h1>
      <p className="m-0 mt-1 max-w-[720px] text-note text-sub">{t("plugins.page.intro")}</p>
      {catalog.error ? <ErrorLine error={catalog.error} className="mt-4" /> : null}

      <div role="toolbar" aria-label={t("plugins.catalog.searchLabel")} className="mt-6 flex flex-wrap items-center gap-x-3 gap-y-2.5">
        <SearchField value={query} onChange={setQuery} aria-label={t("plugins.catalog.searchLabel")} className="flex h-ctl-sm min-w-0 flex-[1_1_240px] items-center gap-2 rounded-input border border-border2 px-3 text-sub data-[focus-within]:border-label">
          <Search size={15} aria-hidden="true" className="shrink-0" />
          <Input placeholder={t("plugins.catalog.searchPlaceholder")} className="h-full min-w-0 flex-1 border-0 bg-transparent p-0 text-note text-text outline-none [&::-webkit-search-cancel-button]:hidden" />
          {query ? (
            <AriaButton aria-label={t("plugins.catalog.clearSearch")} className="press -mr-1.5 flex size-ctl-xs shrink-0 cursor-pointer items-center justify-center rounded-sm border-0 bg-transparent text-sub outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text">
              <X size={14} aria-hidden="true" />
            </AriaButton>
          ) : null}
        </SearchField>
        <div className="w-[180px] max-w-full">
          <Select label={t("plugins.catalog.platformLabel")} value={platform === "" ? "*" : platform} onChange={(id) => setPlatform(id === "*" ? "" : id)} options={[{ id: "*", text: t("plugins.catalog.allPlatforms") }, ...platforms.map((p) => ({ id: p.id, text: p.name }))]} />
        </div>
        <Choice<StateFilter>
          label={t("plugins.catalog.stateLabel")}
          value={state}
          onChange={setState}
          options={(["all", "installed", "update", "available"] as const).map((id) => ({ id, text: t(`plugins.catalog.state.${id}`) }))}
        />
        <ViewSwitch view={view} onChange={setView} />
      </div>

      {catalog.data && all.length === 0 ? <p className="m-0 mt-6 text-note text-sub">{t("plugins.catalog.emptyAll")}</p> : null}
      {catalog.data && all.length > 0 && shown.length === 0 ? <p className="m-0 mt-6 text-note text-sub">{t("plugins.catalog.empty")}</p> : null}

      {KINDS.map((kind) => {
        const group = shown.filter((p) => p.kind === kind);
        if (group.length === 0) return null;
        const title = t(`plugins.catalog.group.${kind}`);
        return (
          <section key={kind} aria-label={title} className="mt-7">
            <h2 className="m-0 text-section font-semibold text-text">{title}</h2>
            {view === "cards" ? (
              <ul className="m-0 mt-3 grid list-none grid-cols-1 gap-3 p-0 @2xl:grid-cols-2 @5xl:grid-cols-3">
                {group.map((p) => (
                  <li key={p.id} className="min-w-0">
                    <PluginCard plugin={p} />
                  </li>
                ))}
              </ul>
            ) : (
              <ul className="m-0 mt-3 flex list-none flex-col rounded-card border border-border bg-card p-0">
                {group.map((p) => (
                  <li key={p.id} className="min-w-0 border-t border-border first:border-t-0">
                    <PluginRow plugin={p} />
                  </li>
                ))}
              </ul>
            )}
          </section>
        );
      })}

      {plugins.data ? <NotLoaded notLoaded={plugins.data.notLoaded} /> : null}
      {catalog.data ? <SourcesCard sources={catalog.data.sources} /> : null}
    </div>
  );
}

function ViewSwitch({ view, onChange }: { view: ViewMode; onChange: (v: ViewMode) => void }) {
  const options: { id: ViewMode; text: string; icon: typeof LayoutGrid }[] = [
    { id: "cards", text: t("plugins.catalog.viewCards"), icon: LayoutGrid },
    { id: "list", text: t("plugins.catalog.viewList"), icon: List },
  ];
  return (
    <ToggleButtonGroup
      aria-label={t("plugins.catalog.viewLabel")}
      selectionMode="single"
      disallowEmptySelection
      selectedKeys={[view]}
      onSelectionChange={(keys: Set<Key>) => {
        const picked = options.find((o) => keys.has(o.id));
        if (picked) onChange(picked.id);
      }}
      className="ml-auto flex gap-0.5 rounded-btn border border-border2 p-0.5"
    >
      {options.map((o) => (
        <ToggleButton
          key={o.id}
          id={o.id}
          aria-label={o.text}
          className="press flex size-ctl-xs cursor-pointer items-center justify-center rounded-sm border-0 bg-transparent text-sub outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text data-[selected]:bg-chip data-[selected]:text-text"
        >
          <o.icon size={15} aria-hidden="true" />
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}

/** 卡片和列表上的一句状态：在做的先说在做什么，再是装着的、有没有更新、装得上装不上。 */
export function statusText(p: CatalogPlugin): string {
  if (p.action) return p.action.kind === "update" ? t("plugins.catalog.status.update_") : t(`plugins.catalog.status.${p.action.kind}`);
  if (p.installed) {
    if (p.update && p.latest) return t("plugins.catalog.status.update", { version: p.latest.version });
    if (p.blocked?.code === "builtin") return t("plugins.catalog.status.builtin");
    return p.installed.origin === "development" ? t("plugins.catalog.status.development", { version: p.installed.version }) : t("plugins.catalog.status.installed", { version: p.installed.version });
  }
  if (p.blocked?.code === "no-release") return t("plugins.catalog.status.notReleased");
  if (p.blocked?.code === "no-asset") return t("plugins.catalog.status.noAsset");
  return t("plugins.catalog.status.available");
}

function linkProps(p: CatalogPlugin) {
  const [owner = "", repo = ""] = p.id.split("/");
  return { to: "/plugins/$owner/$repo" as const, params: { owner, repo } };
}

const STATE_TONE: Record<ReturnType<typeof stateOf>, string> = {
  update: "text-accent",
  installed: "text-label",
  available: "text-sub",
};

/** 一个插件的卡片：整张是一个链接，点按或回车打开详情。 */
function PluginCard({ plugin: p }: { plugin: CatalogPlugin }) {
  return (
    <Link
      {...linkProps(p)}
      aria-label={p.name}
      title={p.id}
      style={{ viewTransitionName: sharedName(p.id) }}
      className="press flex h-full min-w-0 flex-col rounded-card border border-border bg-card px-4 py-3.5 no-underline outline-none hover:border-border2 focus-visible:outline-2 focus-visible:outline-accent"
    >
      <div className="flex items-baseline justify-between gap-3">
        <h3 className="m-0 min-w-0 truncate text-section font-semibold text-text">{p.name}</h3>
        <span className={`shrink-0 text-small ${STATE_TONE[stateOf(p)]}`}>{statusText(p)}</span>
      </div>
      {p.description ? <p className="m-0 mt-1.5 line-clamp-2 text-note text-label">{p.description}</p> : null}
      <div className="flex-1" />
      <div className="mt-3 flex min-w-0 items-center justify-between gap-3 border-t border-border pt-2.5 text-tag text-faint">
        <span className="min-w-0 truncate">{p.platforms.length > 0 ? formatList(p.platforms.map((x) => x.name)) : t(`plugins.catalog.group.${p.kind}`)}</span>
      </div>
    </Link>
  );
}

/** 列表里的一行：名字和一句说明，平台，状态。 */
function PluginRow({ plugin: p }: { plugin: CatalogPlugin }) {
  return (
    <Link
      {...linkProps(p)}
      aria-label={p.name}
      style={{ viewTransitionName: sharedName(p.id) }}
      className="press grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-0.5 px-4 py-2.5 no-underline outline-none hover:bg-chip/40 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-accent @3xl:grid-cols-[minmax(0,2fr)_minmax(0,1fr)_auto]"
    >
      <span className="min-w-0">
        <span className="block truncate text-body font-medium text-text">{p.name}</span>
        {p.description ? <span className="block truncate text-small text-sub">{p.description}</span> : null}
      </span>
      <span className="hidden min-w-0 truncate text-small text-sub @3xl:block">{formatList(p.platforms.map((x) => x.name))}</span>
      <span className={`text-small whitespace-nowrap ${STATE_TONE[stateOf(p)]}`}>{statusText(p)}</span>
    </Link>
  );
}
