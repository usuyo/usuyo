import { getRouteApi, Link } from "@tanstack/react-router";
import { ChevronLeft } from "lucide-react";
import { useState } from "react";
import type { Access, CatalogMethod, CatalogPlugin } from "../api/endpoints";
import { useCatalog, useCatalogChange } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { Card, CardRow } from "../ui/Card";
import { Choice } from "../ui/Choice";
import { t } from "../i18n";
import { formatBytes, formatDay } from "../i18n/format";
import { statusText } from "./Browser";
import { sharedName } from "./catalog";
import { PlatformCards } from "./PluginsPage";
import { TransformsPanel } from "./TransformsPanel";
import { ACCESS_TEXT } from "./words";

const route = getRouteApi("/plugins/$owner/$repo");

/** `/plugins/$owner/$repo`：一个插件占满右边：做什么、获取方式、版本和订阅源、安装更新删除；装着的再带它的设置。 */
export function PluginDetailRoute() {
  const { owner, repo } = route.useParams();
  const search = route.useSearch();
  const catalog = useCatalog();
  const id = `${owner}/${repo}`;
  const plugin = catalog.data?.plugins.find((p) => p.id === id);
  return (
    <div className="flex min-w-0 max-w-[760px] animate-fade flex-col gap-4">
      <Link to="/plugins" className="press -ml-1.5 inline-flex w-fit items-center gap-1 rounded-btn px-1.5 py-1 text-note text-sub no-underline outline-none hover:text-text focus-visible:outline-2 focus-visible:outline-accent">
        <ChevronLeft size={16} aria-hidden="true" />
        {t("plugins.catalog.back")}
      </Link>
      {catalog.error ? <ErrorLine error={catalog.error} /> : null}
      {catalog.data && !plugin ? <p className="m-0 text-note text-sub">{t("plugins.catalog.notFound")}</p> : null}
      {plugin ? <Detail plugin={plugin} target={catalog.data?.target} platform={search.platform} /> : null}
    </div>
  );
}

function Detail({ plugin: p, target, platform }: { plugin: CatalogPlugin; target?: string; platform?: string }) {
  return (
    <>
      <header style={{ viewTransitionName: sharedName(p.id) }}>
        <h1 className="m-0 text-title font-semibold text-text">{p.name}</h1>
        {p.description ? <p className="m-0 mt-1 text-note text-label">{p.description}</p> : null}
        <p className="m-0 mt-1 text-small text-sub">{statusText(p)}</p>
      </header>
      <Actions plugin={p} />
      <DetailsCard plugin={p} target={target} />
      {p.methods.length > 0 ? <MethodsCard methods={p.methods} /> : null}
      {p.installed ? <Installed plugin={p} platform={platform} /> : p.kind === "fetch" ? <p className="m-0 text-small text-sub">{t("plugins.catalog.notInstalledNote")}</p> : null}
    </>
  );
}

/** 安装、更新、删除；在做的时候说在哪一步，没成的说为什么。 */
function Actions({ plugin: p }: { plugin: CatalogPlugin }) {
  const change = useCatalogChange();
  const [asking, setAsking] = useState(false);
  const busy = change.isPending || p.action !== undefined;
  const canInstall = !p.installed && !p.blocked && p.latest?.asset !== undefined;
  const canRemove = p.installed?.origin === "source" && p.blocked?.code !== "builtin";
  return (
    <section aria-label={t("plugins.catalog.details")} className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-2">
        {canInstall ? (
          <Button variant="soft" size="sm" isDisabled={busy} onPress={() => change.mutate({ kind: "install", id: p.id })}>
            {t("plugins.catalog.install")}
          </Button>
        ) : null}
        {p.update && p.latest ? (
          <Button variant="soft" size="sm" isDisabled={busy} onPress={() => change.mutate({ kind: "update", id: p.id })}>
            {t("plugins.catalog.update", { version: p.latest.version })}
          </Button>
        ) : null}
        {canRemove ? (
          asking ? (
            <>
              <Button
                variant="strong"
                size="sm"
                isDisabled={busy}
                onPress={() => {
                  setAsking(false);
                  change.mutate({ kind: "remove", id: p.id });
                }}
              >
                {t("plugins.catalog.confirmRemove")}
              </Button>
              <Button variant="quiet" size="sm" onPress={() => setAsking(false)}>
                {t("common.cancel")}
              </Button>
            </>
          ) : (
            <Button variant="quiet" size="sm" isDisabled={busy} onPress={() => setAsking(true)}>
              {t("plugins.catalog.remove")}
            </Button>
          )
        ) : null}
      </div>
      {asking ? <p className="m-0 text-small text-sub">{t("plugins.catalog.removeNote")}</p> : null}
      {p.action ? (
        <p role="status" className="m-0 text-small text-label">
          {p.action.phase === "downloading" && p.action.received !== undefined && p.action.total !== undefined
            ? t("plugins.catalog.phase.downloadingBytes", { received: formatBytes(p.action.received), total: formatBytes(p.action.total) })
            : t(`plugins.catalog.phase.${p.action.phase}`)}
        </p>
      ) : null}
      {!p.action && p.failure ? <p className="m-0 text-small text-warn">{p.failure}</p> : null}
      {p.blocked && p.blocked.code !== "no-asset" ? <p className="m-0 text-small text-sub">{p.blocked.hint}</p> : null}
      {change.error ? <ErrorLine error={change.error} /> : null}
    </section>
  );
}

function DetailsCard({ plugin: p, target }: { plugin: CatalogPlugin; target?: string }) {
  const version = [p.installed ? t("plugins.catalog.versionInstalled", { version: p.installed.version }) : null, p.latest ? t("plugins.catalog.versionLatest", { version: p.latest.version }) : p.source ? t("plugins.catalog.versionNone") : null].filter((x): x is string => x !== null);
  return (
    <Card title={t("plugins.catalog.details")}>
      <CardRow title={t("plugins.catalog.version")} sub={version.join(" · ")} />
      <CardRow
        title={t("plugins.catalog.source")}
        sub={
          <>
            {p.source ? <span className="mono">{p.source}</span> : t("plugins.catalog.noSource")}
            {p.installed ? <span className="block">{p.installed.origin === "source" ? t("plugins.catalog.origin.source", { source: p.installed.source ?? "" }) : t("plugins.catalog.origin.development")}</span> : null}
          </>
        }
      />
      {p.latest ? (
        <CardRow
          title={t("plugins.catalog.package")}
          sub={
            <>
              {p.latest.asset ? (
                <span className="mono break-all">
                  {p.latest.asset.name} · {formatBytes(p.latest.asset.size)}
                </span>
              ) : (
                t("plugins.catalog.packageNone", { target: target ?? "" })
              )}
              {p.latest.publishedAt ? <span className="block">{`${t("plugins.catalog.published")} ${formatDay(p.latest.publishedAt)}`}</span> : null}
            </>
          }
        />
      ) : null}
      {p.repository ? <CardRow title={t("plugins.catalog.repository")} sub={<span className="mono break-all select-text">{p.repository}</span>} /> : null}
      {p.license ? <CardRow title={t("plugins.catalog.license")} sub={p.license} /> : null}
      {p.installed ? <CardRow title={t("plugins.catalog.location")} sub={<span className="mono break-all select-text">{p.installed.dir}</span>} /> : null}
    </Card>
  );
}

const ACCOUNT_KEY = {
  none: "plugins.catalog.account.none",
  visitor: "plugins.catalog.account.visitor",
  "own-browser": "plugins.catalog.account.ownBrowser",
  "own-phone": "plugins.catalog.account.ownPhone",
  "own-app": "plugins.catalog.account.ownApp",
  developer: "plugins.catalog.account.developer",
  "in-usuyo": "plugins.catalog.account.inUsuyo",
} as const;

function isAccess(v: string): v is Access {
  return v in ACCESS_TEXT;
}

function isAccount(v: string): v is keyof typeof ACCOUNT_KEY {
  return v in ACCOUNT_KEY;
}

function MethodsCard({ methods }: { methods: CatalogMethod[] }) {
  return (
    <Card title={t("plugins.catalog.methods")} sub={t("plugins.catalog.methodsSub")}>
      {methods.map((m) => {
        const account = isAccount(m.account) ? t(ACCOUNT_KEY[m.account]) : m.account;
        const tier = isAccess(m.access) ? ACCESS_TEXT[m.access] : m.access;
        return <CardRow key={m.id} title={m.built ? m.name : `${m.name} · ${t("plugins.catalog.notBuilt")}`} sub={`${tier} · ${account}`} />;
      })}
    </Card>
  );
}

/** 装着的插件的设置：获取插件是它每个平台的那几张卡片（几个平台时先选平台），转写与识别插件是它的模型。 */
function Installed({ plugin: p, platform }: { plugin: CatalogPlugin; platform?: string }) {
  const navigate = route.useNavigate();
  if (p.kind === "transform") return <TransformsPanel only={p.id} />;
  if (p.kind !== "fetch" || p.platforms.length === 0) return null;
  const current = p.platforms.find((x) => x.id === platform)?.id ?? p.platforms[0]?.id ?? "";
  return (
    <section aria-label={t("plugins.catalog.settingsTitle")} className="mt-4 flex flex-col gap-4">
      {p.platforms.length > 1 ? (
        <Choice
          label={t("plugins.catalog.platformPick")}
          value={current}
          onChange={(id) => void navigate({ search: { platform: id }, replace: true })}
          options={p.platforms.map((x) => ({ id: x.id, text: x.name }))}
        />
      ) : null}
      <PlatformCards id={current} />
    </section>
  );
}
