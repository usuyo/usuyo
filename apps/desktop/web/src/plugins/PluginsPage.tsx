import { Link, Outlet, useMatchRoute, useParams } from "@tanstack/react-router";
import type { PluginsView } from "../api/endpoints";
import { usePlugins, useSites, useVisitors } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { PlatformPage } from "./PlatformPage";
import { autoOrder, enabledOn, keyOf, linkChoice, platformPages, type PlatformPage as Page } from "./platforms";
import { TransformsPanel } from "./TransformsPanel";
import { PAGE, timeText } from "./words";

/**
 * 左边那一行的一句：停着的先说停着；不然说收链接现在怎么取。不数数，没出错的不标颜色。
 */
function navLine(page: Page): string {
  const pauses = page.plugins.flatMap((p) => p.pauses).filter((x) => x.byYou || x.scope === page.id || x.scope === "*");
  const pause = pauses.find((x) => x.byYou) ?? pauses[0];
  if (pause) return pause.byYou ? PAGE.navHeld : pause.until ? PAGE.navPausedUntil(timeText(pause.until)) : PAGE.navPaused;
  if (page.link.length === 0) return PAGE.navSavedOnly;
  const choice = linkChoice(page.link, page.id);
  if (choice.kind === "mixed") return PAGE.navMixed;
  if (choice.kind === "only") {
    const only = page.link.find((u) => keyOf(u) === choice.key);
    return only ? PAGE.navOnly(only.method.name, only.method.usable) : PAGE.navMixed;
  }
  const first = autoOrder(page.link, page.id).find((u) => enabledOn(u, page.id) && u.method.usable);
  return first ? PAGE.navAuto(first.method.name) : PAGE.navAutoNone;
}

// 窄的时候平台排成几行小块、只写名字，页面不被一长列推到下面去；宽了是一列，带那一句。
const NAV_ROW =
  "press flex min-w-0 flex-col items-start gap-0.5 rounded-btn px-3 py-1.5 text-left no-underline outline-none hover:bg-chip/60 focus-visible:outline-2 focus-visible:outline-accent @3xl:w-full @3xl:py-2";
const NAV_LINE = "hidden max-w-full truncate text-small text-sub @3xl:block";
const NAV_GROUP = "w-full px-3 pb-1.5 text-tag font-medium text-faint";

/** 获取插件：左边是平台（和「变成文字」），右边是选中的那一页；每个平台一页，有自己的地址。 */
export function PluginsPage() {
  const plugins = usePlugins();
  const pages = plugins.data ? platformPages(plugins.data) : [];
  const params = useParams({ strict: false });
  const matchRoute = useMatchRoute();
  const onText = matchRoute({ to: "/plugins/text" }) !== false;
  const current = onText ? null : (params.platform ?? pages[0]?.id ?? null);
  const notLoaded = plugins.data?.notLoaded ?? [];

  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航" />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-7 pb-10 lg:px-gutter">
        <h1 className="m-0 text-title font-semibold text-text">{PAGE.title}</h1>
        <p className="m-0 mt-1 max-w-[720px] text-note text-sub">{PAGE.intro}</p>
        {plugins.error ? <ErrorLine error={plugins.error} className="mt-4" /> : null}
        <div className="mt-6 grid items-start gap-x-8 gap-y-6 @3xl:grid-cols-[220px_minmax(0,1fr)]">
          <nav aria-label={PAGE.navLabel} className="flex min-w-0 flex-wrap gap-0.5 @3xl:sticky @3xl:top-0 @3xl:flex-col @3xl:flex-nowrap">
            <div className={NAV_GROUP}>{PAGE.navPlatforms}</div>
            {plugins.data && pages.length === 0 ? <div className="px-3 py-1.5 text-small text-sub">{PAGE.navEmpty}</div> : null}
            {pages.map((p) => (
              <Link
                key={p.id}
                to="/plugins/$platform"
                params={{ platform: p.id }}
                aria-current={p.id === current ? "page" : undefined}
                className={`${NAV_ROW} ${p.id === current ? "bg-chip" : ""}`}
              >
                <span className={`text-body ${p.id === current ? "font-semibold text-text" : "font-medium text-label"}`}>{p.name}</span>
                <span className={NAV_LINE}>{navLine(p)}</span>
              </Link>
            ))}
            <div className={`${NAV_GROUP} pt-3 @3xl:pt-4`}>{PAGE.navText}</div>
            <Link to="/plugins/text" aria-current={onText ? "page" : undefined} className={`${NAV_ROW} ${onText ? "bg-chip" : ""}`}>
              <span className={`text-body ${onText ? "font-semibold text-text" : "font-medium text-label"}`}>{PAGE.textEntryNav}</span>
              <span className={NAV_LINE}>{PAGE.textEntryLine}</span>
            </Link>
            {notLoaded.length > 0 ? <NotLoaded notLoaded={notLoaded} /> : null}
          </nav>
          <div className="min-w-0 max-w-[760px]">
            <Outlet />
          </div>
        </div>
      </main>
    </div>
  );
}

/** `/plugins`：第一个平台的那一页；一个都没装时说怎么装。 */
export function FirstPlatform() {
  const plugins = usePlugins();
  if (!plugins.data) return null;
  const first = platformPages(plugins.data)[0];
  return first ? <Platform id={first.id} /> : <NothingInstalled />;
}

/** `/plugins/$platform`。 */
export function PlatformRoute() {
  const { platform } = useParams({ from: "/plugins/$platform" });
  return <Platform id={platform} />;
}

/** `/plugins/text`。 */
export function TextRoute() {
  return <TransformsPanel />;
}

function Platform({ id }: { id: string }) {
  const plugins = usePlugins();
  const sites = useSites();
  const visitors = useVisitors();
  if (!plugins.data) return null;
  const page = platformPages(plugins.data).find((p) => p.id === id);
  if (!page) return <p className="m-0 animate-fade text-note text-sub">{PAGE.notFound}</p>;
  return <PlatformPage key={page.id} page={page} sites={sites.data ?? []} visitors={visitors.data ?? []} failureLog={plugins.data.failureLog} />;
}

function NotLoaded({ notLoaded }: { notLoaded: PluginsView["notLoaded"] }) {
  return (
    <>
      <div className={`${NAV_GROUP} pt-3 @3xl:pt-4`}>{PAGE.navNotLoaded}</div>
      {notLoaded.map((n) => (
        <div key={n.dir} className="w-full px-3 py-1.5">
          <div className="mono truncate text-tag text-label" title={n.dir}>
            {n.dir}
          </div>
          <div className="text-small text-warn">{n.reason}</div>
        </div>
      ))}
    </>
  );
}

/** 一个获取插件都没装：说清楚现在插件从哪来，不假装有商店。 */
function NothingInstalled() {
  return (
    <div className="max-w-[560px] animate-fade rounded-card border border-dashed border-border2 p-5">
      <div className="text-body text-label">{PAGE.nothingTitle}</div>
      <p className="m-0 mt-1 text-note text-sub">
        {PAGE.nothingBody}
      </p>
      <code className="mono mt-3 block rounded-sm bg-chip px-3 py-2 text-tag text-text select-text">just dev-plugins</code>
    </div>
  );
}
