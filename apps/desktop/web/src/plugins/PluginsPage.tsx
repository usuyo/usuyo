import { getRouteApi } from "@tanstack/react-router";
import { useState } from "react";
import { Button as AriaButton } from "react-aria-components";
import type { FetchPlugin, PluginsView, SiteLogin, VisitorIdentity as Visitor } from "../api/endpoints";
import { usePlugins, useSites, useVisitors } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { Choice } from "../ui/Choice";
import { PluginCard, shown, type MethodFilter } from "./PluginCard";
import { SiteLoginRow } from "./SiteLoginRow";
import { TransformsPanel } from "./TransformsPanel";
import { VisitorIdentity } from "./VisitorIdentity";
import { timeText } from "./words";

/** 左边一栏的一个平台：用到它的插件、要登录的站点和访客身份。 */
interface PlatformGroup {
  key: string;
  name: string;
  plugins: FetchPlugin[];
  sites: SiteLogin[];
  visitors: Visitor[];
}

/** 按平台分：一个插件管几个平台就在几个平台下面；不认平台的（通用网页）自己一组。 */
function groups(view: PluginsView, sites: SiteLogin[], visitors: Visitor[]): PlatformGroup[] {
  const out = new Map<string, PlatformGroup>();
  for (const p of view.plugins) {
    const targets = p.platforms.length > 0 ? p.platforms : [{ id: p.id, name: p.name }];
    for (const t of targets) {
      const g = out.get(t.id) ?? { key: t.id, name: t.name, plugins: [], sites: [], visitors: [] };
      g.plugins.push(p);
      for (const s of p.sites) {
        const site = sites.find((x) => x.site === s);
        if (site && !g.sites.includes(site)) g.sites.push(site);
      }
      for (const s of p.visitors) {
        const visitor = visitors.find((x) => x.site === s);
        if (visitor && !g.visitors.includes(visitor)) g.visitors.push(visitor);
      }
      out.set(t.id, g);
    }
  }
  return [...out.values()];
}

/** 左边那一行的小字：停着的先说停着；不然写现在用得上的那几种的名字，一种都用不上就这么说。不数数。 */
function groupSub(g: PlatformGroup, filter: MethodFilter): { text: string; warn: boolean } {
  const pauses = g.plugins.flatMap((p) => p.pauses);
  const pause = pauses.find((p) => p.byYou) ?? pauses[0];
  if (pause) return { text: pause.byYou ? "你让它先别试" : pause.until ? `有一种停着，${timeText(pause.until)} 再试` : "有一种停着", warn: !pause.byYou };
  const methods = g.plugins.flatMap((p) => p.methods);
  // 只看还用不上的时，写的是那几种的名字。
  if (filter === "blocked") return { text: `还用不上：${methods.filter((m) => !m.usable).map((m) => m.name).join("、")}`, warn: false };
  const usable = methods.filter((m) => m.usable);
  return { text: usable.length > 0 ? `能用：${usable.map((m) => m.name).join("、")}` : "现在都还用不上", warn: false };
}

const FILTERS: { id: MethodFilter; text: string }[] = [
  { id: "all", text: "全部" },
  { id: "usable", text: "现在能用的" },
  { id: "blocked", text: "还用不上的" },
];

const route = getRouteApi("/plugins");

export function PluginsPage() {
  const plugins = usePlugins();
  const sites = useSites();
  const visitors = useVisitors();
  const all = plugins.data ? groups(plugins.data, sites.data ?? [], visitors.data ?? []) : [];
  const [picked, setPicked] = useState<string | null>(null);
  const [filter, setFilter] = useState<MethodFilter>("all");
  // 「变成文字」那一栏和开着的平台放在地址里：设置页、「手机」页能直接链过来。
  const search = route.useSearch();
  const text = search.show === "text";
  const navigate = route.useNavigate();
  const pick = (key: string | null) => {
    setPicked(key);
    void navigate({ search: key === null ? { show: "text" } : {} });
  };
  // 只看一类时，各平台的那几种从上到下排成一列，一眼扫完；左边点平台跳到它那一段。
  const narrowed = filter === "all" || text ? null : all.filter((g) => g.plugins.some((p) => p.methods.some((m) => shown(m, filter))));
  const current = text ? undefined : (all.find((g) => g.key === (picked ?? search.platform)) ?? all[0]);
  const jump = (key: string) => document.getElementById(`platform-${key}`)?.scrollIntoView({ block: "start" });
  const notLoaded = plugins.data?.notLoaded ?? [];

  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航" />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-7 pb-10 lg:px-gutter">
        <h1 className="m-0 text-title font-semibold text-text">获取插件</h1>
        <p className="m-0 mt-1 max-w-[720px] text-note text-sub">
          每个平台的获取插件有哪几种取法、账号用在哪儿、按什么节奏取；还有把口播和图里的字转成文字的插件。usuyo 自己不带任何平台的插件。
        </p>
        {plugins.error ? <ErrorLine error={plugins.error} className="mt-4" /> : null}
        {all.length > 0 && !text ? (
          <div className="mt-5">
            <Choice label="看哪些取法" options={FILTERS} value={filter} onChange={setFilter} />
          </div>
        ) : null}
        <div className="mt-7 grid items-start gap-x-8 gap-y-6 @3xl:grid-cols-[232px_minmax(0,1fr)]">
          <nav aria-label="平台" className="flex min-w-0 flex-col gap-0.5 @3xl:sticky @3xl:top-0">
            <div className="px-3 pb-1.5 text-tag font-medium text-faint">获取</div>
            {plugins.data && all.length === 0 ? <div className="px-3 py-1.5 text-small text-sub">还没有装</div> : null}
            {(narrowed ?? all).map((g) => {
              const sub = groupSub(g, text ? "all" : filter);
              return (
                <NavRow
                  key={g.key}
                  name={g.name}
                  sub={sub.text}
                  warn={sub.warn}
                  current={narrowed === null && g.key === current?.key}
                  onPress={() => (narrowed === null ? pick(g.key) : jump(g.key))}
                />
              );
            })}
            <div className="px-3 pt-4 pb-1.5 text-tag font-medium text-faint">变成文字</div>
            <NavRow name="口播、图里的字、画面" sub="转换插件和模型" warn={false} current={current === undefined && plugins.data !== undefined} onPress={() => pick(null)} />
            {notLoaded.length > 0 ? <NotLoaded notLoaded={notLoaded} /> : null}
          </nav>
          <div className="min-w-0">
            {narrowed ? (
              <Narrowed matching={narrowed} filter={filter} />
            ) : current ? (
              <PlatformDetail key={current.key} group={current} />
            ) : plugins.data ? (
              <>
                {all.length === 0 && !text ? <NothingInstalled /> : null}
                <TransformsPanel />
              </>
            ) : null}
          </div>
        </div>
      </main>
    </div>
  );
}

function NavRow({ name, sub, warn, current, onPress }: { name: string; sub: string; warn: boolean; current: boolean; onPress: () => void }) {
  return (
    <AriaButton
      aria-current={current ? "true" : undefined}
      onPress={onPress}
      className={`press flex w-full min-w-0 cursor-pointer flex-col items-start gap-0.5 rounded-btn border-0 px-3 py-2 text-left outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60 ${current ? "bg-chip" : "bg-transparent"}`}
    >
      <span className={`text-body ${current ? "font-semibold text-text" : "font-medium text-label"}`}>{name}</span>
      <span className={`max-w-full truncate text-small ${warn ? "text-warn" : "text-sub"}`}>{sub}</span>
    </AriaButton>
  );
}

function PlatformDetail({ group }: { group: PlatformGroup }) {
  const pacing = group.plugins.flatMap((p) => p.pacing).filter((x) => x.platform === group.key);
  const strict = pacing.length > 0 ? { batch: pacing.every((x) => x.batch), intervalSec: Math.max(...pacing.map((x) => x.intervalSec)), daily: minDaily(pacing) } : null;
  return (
    <section aria-label={group.name} className="min-w-0 animate-fade">
      <h2 className="m-0 text-name font-semibold text-text">{group.name}</h2>
      <p className="m-0 mt-1.5 max-w-[720px] text-note text-sub">
        一种取法一行，做好了的各自开关，每一种写着账号在哪儿、这台电脑上现在用不用得上、差什么。不用账号能拿到的，一律不带账号去取。档位和记下的风险只是给你挑的，不拦什么。
      </p>
      <div className="mt-4 flex flex-col gap-3">
        {group.plugins.map((p) => (
          <PluginCard key={p.id} plugin={p} />
        ))}
      </div>

      {group.sites.length > 0 ? (
        <>
          <h3 className="m-0 mt-8 text-section font-semibold text-text">在 usuyo 里登录的</h3>
          <p className="m-0 mt-1.5 max-w-[720px] text-note text-sub">
            只有写着「要在 usuyo 里登录」的取法用到。在 usuyo 给每个站点单独开的窗口里，你自己登录（多半是扫码），和你日常用的浏览器分开。
          </p>
          <div className="mt-3 flex flex-col gap-2">
            {group.sites.map((s) => (
              <SiteLoginRow key={s.site} site={s} />
            ))}
          </div>
        </>
      ) : null}

      {group.visitors.map((v) => (
        <VisitorIdentity key={v.site} visitor={v} />
      ))}

      {strict ? (
        <>
          <h3 className="m-0 mt-8 text-section font-semibold text-text">节奏</h3>
          <p className="m-0 mt-1.5 max-w-[720px] text-note text-sub">
            {strict.batch
              ? `两次之间至少隔 ${strict.intervalSec} 秒${strict.daily ? `，导入每天最多 ${strict.daily} 次` : ""}。你手动收下的单条也照这个间隔排队，但不算进每天的次数。几种取法共用这一个节奏。`
              : `两次之间至少隔 ${strict.intervalSec} 秒。这个平台不做批量导入，只按链接一条条取。`}
          </p>
        </>
      ) : null}
    </section>
  );
}

/** 只看一类时：各平台一段，只列这一类的取法。 */
function Narrowed({ matching, filter }: { matching: PlatformGroup[]; filter: MethodFilter }) {
  if (matching.length === 0) {
    return <p className="m-0 animate-fade text-note text-sub">{filter === "usable" ? "现在没有用得上的取法。" : "装着的取法这台电脑上都用得上。"}</p>;
  }
  return (
    <div className="flex animate-fade flex-col gap-7">
      {matching.map((g) => (
        <section key={g.key} id={`platform-${g.key}`} aria-label={g.name} className="min-w-0 scroll-mt-4">
          <h2 className="m-0 text-name font-semibold text-text">{g.name}</h2>
          <div className="mt-3 flex flex-col gap-3">
            {g.plugins
              .filter((p) => p.methods.some((m) => shown(m, filter)))
              .map((p) => (
                <PluginCard key={p.id} plugin={p} filter={filter} />
              ))}
          </div>
        </section>
      ))}
    </div>
  );
}

function minDaily(pacing: { daily?: number }[]): number | undefined {
  const d = pacing.flatMap((x) => (x.daily === undefined ? [] : [x.daily]));
  return d.length > 0 ? Math.min(...d) : undefined;
}

function NotLoaded({ notLoaded }: { notLoaded: PluginsView["notLoaded"] }) {
  return (
    <>
      <div className="px-3 pt-4 pb-1.5 text-tag font-medium text-faint">装了但没加载</div>
      {notLoaded.map((n) => (
        <div key={n.dir} className="px-3 py-1.5">
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
    <div className="mb-8 max-w-[560px] animate-fade rounded-card border border-dashed border-border2 p-5">
      <div className="text-body text-label">还没有装获取插件。</div>
      <p className="m-0 mt-1 text-note text-sub">
        usuyo 自己不带平台的插件。现在只能在开发时装：在 usuyo 的仓库里跑下面这条命令，它把旁边 usuyo-fetch
        里的插件编好、装到这台机器上；重启 usuyo 以后这里就有了。
      </p>
      <code className="mono mt-3 block rounded-sm bg-chip px-3 py-2 text-tag text-text select-text">just dev-plugins</code>
    </div>
  );
}
