import { Navigate, Outlet, useParams } from "@tanstack/react-router";
import type { PluginsView } from "../api/endpoints";
import { usePlugins, useSites, useVisitors } from "../api/hooks";
import { Rail } from "../frame/Rail";
import { PlatformPage } from "./PlatformPage";
import { platformPages } from "./platforms";
import { TransformsPanel } from "./TransformsPanel";
import { t } from "../i18n";

/**
 * 获取插件：左边是导航，右边整块是插件目录（`/plugins`）或一个插件的详情（`/plugins/<owner>/<repo>`）。
 * 目录和详情放在同一个滚动区里换，换的时候外框不动：以后加从卡片到详情的过渡，只要接在这一层上。
 */
export function PluginsPage() {
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label={t("common.nav.main")} />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-7 pb-10 lg:px-gutter">
        <Outlet />
      </main>
    </div>
  );
}

/** `/plugins/$platform`：以前一个平台一页的地址，换到管这个平台的第一个插件的详情，选好这个平台。 */
export function PlatformRoute() {
  const { platform } = useParams({ from: "/plugins/$platform" });
  const plugins = usePlugins();
  if (!plugins.data) return null;
  const owner = plugins.data.plugins.find((p) => p.platforms.some((x) => x.id === platform));
  if (!owner) return <p className="m-0 animate-fade text-note text-sub">{t("plugins.page.notFound")}</p>;
  const [o, r] = owner.id.split("/");
  return <Navigate to="/plugins/$owner/$repo" params={{ owner: o ?? "", repo: r ?? "" }} search={{ platform }} replace />;
}

/** `/plugins/text`：所有转写与识别插件和它们的模型（设置页链到这里）。 */
export function TextRoute() {
  return <TransformsPanel />;
}

/** 一个获取插件在某个平台上的几张卡片（添加链接、导入已保存的内容、账号与访客身份、请求频率、关于）。 */
export function PlatformCards({ id }: { id: string }) {
  const plugins = usePlugins();
  const sites = useSites();
  const visitors = useVisitors();
  // 登录和访客身份到了再画：账号那几行照它们写，先画空的会闪一下
  if (!plugins.data || (sites.isPending && !sites.error) || (visitors.isPending && !visitors.error)) return null;
  const page = platformPages(plugins.data).find((p) => p.id === id);
  if (!page) return <p className="m-0 animate-fade text-note text-sub">{t("plugins.page.notFound")}</p>;
  return <PlatformPage key={page.id} page={page} sites={sites.data ?? []} visitors={visitors.data ?? []} failureLog={plugins.data.failureLog} />;
}

/** 装了但没载入的插件：目录和原因。 */
export function NotLoaded({ notLoaded }: { notLoaded: PluginsView["notLoaded"] }) {
  if (notLoaded.length === 0) return null;
  return (
    <section aria-label={t("plugins.page.notLoaded")} className="mt-8">
      <h2 className="m-0 text-section font-semibold text-text">{t("plugins.page.notLoaded")}</h2>
      <ul className="m-0 mt-2 flex list-none flex-col gap-2 p-0">
        {notLoaded.map((n) => (
          <li key={n.dir} className="rounded-card border border-border bg-card px-4 py-2.5">
            <div className="mono truncate text-tag text-label" title={n.dir}>
              {n.dir}
            </div>
            <div className="text-small text-warn">{n.reason}</div>
          </li>
        ))}
      </ul>
    </section>
  );
}
