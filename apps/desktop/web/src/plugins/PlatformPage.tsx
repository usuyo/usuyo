import type { FetchPlugin, SiteLogin, VisitorIdentity as Visitor } from "../api/endpoints";
import { useFileReaders, usePluginChange } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { Card, CardRow } from "../ui/Card";
import { LinkCard } from "./LinkCard";
import { SecretField } from "./Needs";
import type { PlatformPage as Page, Use } from "./platforms";
import { SavedCard } from "./SavedCard";
import { ProfileSiteRow } from "./ProfileSiteRow";
import { SiteLoginRow } from "./SiteLoginRow";
import { VisitorIdentity } from "./VisitorIdentity";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import { pauseText } from "./words";

/**
 * 一个平台的一页：收一条链接、拿回存过的、账号和访客身份、节奏、关于，各一张卡片。
 * 档位和记下的风险只是给你挑的信息，不拦什么。
 */
export function PlatformPage({ page, sites, visitors, failureLog }: { page: Page; sites: SiteLogin[]; visitors: Visitor[]; failureLog: string }) {
  const readers = useFileReaders();
  const name = page.name;
  return (
    <div aria-label={name} role="region" className="flex min-w-0 animate-fade flex-col gap-4">
      <header>
        <h2 className="m-0 text-name font-semibold text-text">{name}</h2>
        <p className="m-0 mt-1 text-note text-sub">
          {t("plugins.page.header", { count: page.plugins.length, plugins: formatList(page.plugins.map((p) => p.name)) })}
        </p>
      </header>
      <LinkCard page={page} sites={sites} visitors={visitors} />
      <SavedCard page={page} sites={sites} visitors={visitors} readers={readers.data ?? []} />
      <AccountsCard page={page} sites={sites} visitors={visitors} />
      <PacingCard page={page} />
      <AboutCard page={page} failureLog={failureLog} />
    </div>
  );
}

const all = (page: Page): Use[] => [...page.link, ...page.parts, ...page.saved];

/** 这个平台的方法用到的登录和访客身份，各写哪几种方法用它。 */
function AccountsCard({ page, sites, visitors }: { page: Page; sites: SiteLogin[]; visitors: Visitor[] }) {
  const uses = all(page);
  const usersOf = (kind: "in-usuyo" | "visitor", site: string, plugin?: string) =>
    uses
      .filter((u) => u.method.account.kind === kind && (u.method.account.sites ?? []).includes(site) && (!plugin || u.plugin.id === plugin))
      .map((u) => u.method.name);
  // `browser-profile` 站点一个插件一项，只算那个插件自己的方法
  const loginUsers = (s: SiteLogin) => usersOf("in-usuyo", s.site, s.credential === "browser-profile" ? s.plugins[0] : undefined);
  const logins = sites.filter((s) => loginUsers(s).length > 0);
  const ids = visitors.filter((v) => usersOf("visitor", v.site).length > 0);
  return (
    <Card title={t("plugins.accounts.title")} sub={t("plugins.accounts.sub")}>
      {logins.length === 0 && ids.length === 0 ? <CardRow title={t("plugins.accounts.none")} /> : null}
      {logins.map((s) =>
        s.credential === "browser-profile" ? (
          <ProfileSiteRow key={`${s.plugins[0]}:${s.site}`} site={s} users={loginUsers(s)} />
        ) : (
          <SiteLoginRow key={s.site} site={s} users={loginUsers(s)} />
        ),
      )}
      {ids.map((v) => (
        <VisitorIdentity key={v.site} visitor={v} users={usersOf("visitor", v.site)} />
      ))}
    </Card>
  );
}

/** 节奏：两次之间隔多久、每天几次；停着的为什么、到什么时候；「现在再试」「先别试」。 */
function PacingCard({ page }: { page: Page }) {
  return (
    <Card title={t("plugins.pacing.title")} sub={t("plugins.pacing.sub")}>
      {page.plugins.map((p) => (
        <PacingRow key={p.id} plugin={p} platform={page.id} titled={page.plugins.length > 1} />
      ))}
    </Card>
  );
}

function PacingRow({ plugin, platform, titled }: { plugin: FetchPlugin; platform: string; titled: boolean }) {
  const change = usePluginChange(plugin.id);
  const pace = plugin.pacing.find((x) => x.platform === platform);
  const pauses = plugin.pauses.filter((x) => x.byYou || x.scope === platform || x.scope === "*");
  const held = plugin.pauses.some((x) => x.byYou);
  const words = !pace
    ? t("plugins.pacing.none")
    : pace.batch
      ? pace.daily
        ? t("plugins.pacing.batchDaily", { count: pace.intervalSec, daily: pace.daily })
        : t("plugins.pacing.batch", { count: pace.intervalSec })
      : t("plugins.pacing.noBatch", { count: pace.intervalSec });
  return (
    <CardRow
      title={titled ? plugin.name : t("plugins.pacing.row")}
      sub={words}
      control={
        <>
          {pauses.length > 0 ? (
            <Button variant="outline" size="sm" onPress={() => change.mutate({ kind: "try-now" })} isDisabled={change.isPending}>
              {t("plugins.pacing.retryNow")}
            </Button>
          ) : null}
          {!held ? (
            <Button variant="quiet" size="sm" onPress={() => change.mutate({ kind: "hold" })} isDisabled={change.isPending}>
              {t("plugins.pacing.pause")}
            </Button>
          ) : null}
        </>
      }
    >
      {pauses.length > 0 || change.error ? (
        <>
          {pauses.map((x) => (
            <p key={`${x.method}-${x.scope}-${x.byYou ? "you" : "platform"}`} className="m-0 text-small text-label">
              {pauseText(x, plugin.platforms)}
            </p>
          ))}
          {change.error ? <ErrorLine error={change.error} className="mt-1" /> : null}
        </>
      ) : null}
    </CardRow>
  );
}

/** 密钥与日志：插件要填的密钥（存进钥匙串）、取不到时的失败日志在哪。插件的代号、版本和参考的项目在「技术信息」里。 */
function AboutCard({ page, failureLog }: { page: Page; failureLog: string }) {
  const keyed = page.plugins.filter((p) => p.settings.some((s) => s.secret));
  return (
    <Card title={t("plugins.about.title")}>
      {keyed.map((p) => (
        <CardRow key={p.id} title={p.name}>
          <div className="flex flex-col gap-2">
            {p.settings
              .filter((s) => s.secret)
              .map((s) => (
                <div key={s.key} className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1.5">
                  <span className="text-note text-label">
                    {s.label}
                    <span className="ml-1.5 text-small text-sub">{s.filled === true ? t("plugins.about.secretFilled") : t("plugins.about.secretEmpty")}</span>
                  </span>
                  <SecretField pluginId={p.id} setting={s} />
                </div>
              ))}
          </div>
        </CardRow>
      ))}
      <CardRow title={t("plugins.about.failureLog")} sub={t("plugins.about.failureLogSub")}>
        <code className="mono block truncate rounded-sm bg-chip px-2.5 py-1.5 text-tag text-label select-text" title={failureLog}>
          {failureLog}
        </code>
      </CardRow>
    </Card>
  );
}
