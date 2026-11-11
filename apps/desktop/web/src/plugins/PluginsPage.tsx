import { useState } from "react";
import { Button as AriaButton } from "react-aria-components";
import type { FetchPlugin, PluginsView, SiteLogin, VisitorIdentity as Visitor } from "../api/endpoints";
import { usePlugins, useSites, useVisitors } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { PluginCard } from "./PluginCard";
import { SiteLoginRow } from "./SiteLoginRow";
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

/** 左边那一行的小字：几个插件、登录着没有、停着没有。 */
function groupSub(g: PlatformGroup): { text: string; warn: boolean } {
  const parts = [`${g.plugins.length} 个获取插件`];
  const pause = g.plugins.flatMap((p) => p.pauses)[0];
  if (pause) {
    parts.push(pause.byYou ? "你让它先别试" : pause.until ? `停着，${timeText(pause.until)} 再试` : "停着");
    return { text: parts.join(" · "), warn: !pause.byYou };
  }
  const states = g.sites.map((s) => s.login);
  if (states.includes("expired")) return { text: [...parts, "登录过期了"].join(" · "), warn: true };
  if (states.length > 0) parts.push(states.every((s) => s === "present") ? "登录着" : "还没登录");
  else parts.push("不用账号");
  return { text: parts.join(" · "), warn: false };
}

export function PluginsPage() {
  const plugins = usePlugins();
  const sites = useSites();
  const visitors = useVisitors();
  const all = plugins.data ? groups(plugins.data, sites.data ?? [], visitors.data ?? []) : [];
  const [picked, setPicked] = useState<string | null>(null);
  const current = all.find((g) => g.key === picked) ?? all[0];
  const notLoaded = plugins.data?.notLoaded ?? [];

  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航" />
      <main className="absolute inset-y-0 right-0 left-rail overflow-hidden">
        <div className="absolute top-7 left-gutter w-[860px]">
          <h1 className="m-0 text-title font-semibold text-text">获取插件</h1>
          <p className="m-0 mt-1 text-note text-sub">
            每个平台装了哪些获取插件、怎么登录、按什么节奏取。usuyo 自己不带任何平台的插件。
          </p>
        </div>
        {plugins.error ? <ErrorLine error={plugins.error} className="absolute top-[104px] left-gutter" /> : null}
        {plugins.data && all.length === 0 ? <NothingInstalled notLoaded={notLoaded} /> : null}
        {current ? (
          <>
            <nav aria-label="平台" className="absolute top-[104px] bottom-6 left-gutter flex w-[272px] flex-col gap-0.5 overflow-y-auto">
              <div className="px-3 pb-1.5 text-tag font-medium text-faint">装着的</div>
              {all.map((g) => {
                const sub = groupSub(g);
                const cur = g.key === current.key;
                return (
                  <AriaButton
                    key={g.key}
                    aria-current={cur ? "true" : undefined}
                    onPress={() => setPicked(g.key)}
                    className={`press flex w-full cursor-pointer flex-col items-start gap-0.5 rounded-btn border-0 px-3 py-2 text-left outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60 ${cur ? "bg-chip" : "bg-transparent"}`}
                  >
                    <span className={`text-body ${cur ? "font-semibold text-text" : "font-medium text-label"}`}>{g.name}</span>
                    <span className={`max-w-full truncate text-small ${sub.warn ? "text-warn" : "text-sub"}`}>{sub.text}</span>
                  </AriaButton>
                );
              })}
              {notLoaded.length > 0 ? <NotLoaded notLoaded={notLoaded} /> : null}
            </nav>
            <PlatformDetail key={current.key} group={current} />
          </>
        ) : null}
      </main>
    </div>
  );
}

function PlatformDetail({ group }: { group: PlatformGroup }) {
  const owners = [...new Set(group.plugins.map((p) => p.id.split("/")[0] ?? p.id))];
  const pacing = group.plugins.flatMap((p) => p.pacing).find((x) => x.platform === group.key);
  return (
    <section
      aria-label={group.name}
      className="absolute top-[104px] right-0 bottom-0 left-[calc(var(--spacing-gutter)+304px)] overflow-y-auto pr-gutter pb-10"
    >
      <div className="w-[808px] max-w-full animate-fade">
        <div className="flex items-baseline gap-2.5">
          <h2 className="m-0 text-name font-semibold text-text">{group.name}</h2>
          <span className="text-note text-sub">
            {group.plugins.length} 个获取插件 · 来自 {owners.join("、")}
          </span>
        </div>

        {group.sites.length > 0 ? (
          <>
            <h3 className="m-0 mt-7 text-section font-semibold text-text">登录</h3>
            <p className="m-0 mt-1.5 max-w-[700px] text-note text-sub">
              在 usuyo 给每个站点单独开的窗口里，你自己登录（多半是扫码），和你日常用的浏览器分开。用到同一个站点的插件共用这一份，登录一次就行。
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

        <h3 className="m-0 mt-8 text-section font-semibold text-text">获取方式</h3>
        <p className="m-0 mt-1.5 text-note text-sub">每个插件怎么取、要不要你的账号。标签只是给你挑的，不拦什么。</p>
        <div className="mt-3 flex flex-col gap-2.5">
          {group.plugins.map((p) => (
            <PluginCard key={p.id} plugin={p} sites={group.sites} />
          ))}
        </div>

        {pacing ? (
          <>
            <h3 className="m-0 mt-8 text-section font-semibold text-text">节奏</h3>
            <p className="m-0 mt-1.5 max-w-[700px] text-note text-sub">
              {pacing.batch
                ? `两次之间至少隔 ${pacing.intervalSec} 秒${pacing.daily ? `，导入每天最多 ${pacing.daily} 次` : ""}。你手动收下的单条也照这个间隔排队，但不算进每天的次数。`
                : `两次之间至少隔 ${pacing.intervalSec} 秒。这个平台不做批量导入，只按链接一条条取。`}
            </p>
          </>
        ) : null}
      </div>
    </section>
  );
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

/** 一个都没装：说清楚现在插件从哪来，不假装有商店。 */
function NothingInstalled({ notLoaded }: { notLoaded: PluginsView["notLoaded"] }) {
  return (
    <div className="absolute top-[104px] left-gutter w-[560px] animate-fade rounded-card border border-dashed border-border2 p-5">
      <div className="text-body text-label">还没有装获取插件。</div>
      <p className="m-0 mt-1 text-note text-sub">
        usuyo 自己不带平台的插件。现在只能在开发时装：在 usuyo 的仓库里跑下面这条命令，它把旁边 usuyo-fetch
        里的小红书、微信视频号、B站插件编好、装到这台机器上；重启 usuyo 以后这里就有了。
      </p>
      <code className="mono mt-3 block rounded-sm bg-chip px-3 py-2 text-tag text-text select-text">just dev-plugins</code>
      {notLoaded.length > 0 ? (
        <div className="mt-4 flex flex-col gap-1">
          <NotLoaded notLoaded={notLoaded} />
        </div>
      ) : null}
    </div>
  );
}
