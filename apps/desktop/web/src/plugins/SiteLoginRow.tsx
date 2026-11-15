import { useRef, useState } from "react";
import type { SiteLogin } from "../api/endpoints";
import { useForgetLogin, useImportLoginFile } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { dayText } from "../item/format";
import { clearLoginData, openLogin } from "../shell";
import { Button } from "../ui/Button";
import { CardRow } from "../ui/Card";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import { LOGIN_STATE } from "./words";

/** 开壳的登录窗口：在 usuyo 给这个站点单独开的窗口里，你自己登录，和你日常用的浏览器分开。 */
function useLogin(site: SiteLogin) {
  const [say, setSay] = useState<string | null>(null);
  async function login() {
    setSay(null);
    try {
      const opened = await openLogin(site.site);
      setSay(opened ? t("plugins.accounts.loginOpened") : t("plugins.accounts.loginAppOnly"));
    } catch (error) {
      setSay(typeof error === "string" ? error : t("plugins.accounts.loginFailed"));
    }
  }
  return { login, say, setSay };
}

function Say({ text }: { text: string | null }) {
  return text ? (
    <p role="status" className="m-0 mt-1.5 text-small text-sub">
      {text}
    </p>
  ) : null;
}

/** 缺登录时那一步的按钮。 */
export function SiteLoginButton({ site }: { site: SiteLogin }) {
  const { login, say } = useLogin(site);
  return (
    <div className="min-w-0">
      <Button variant="soft" size="sm" onPress={() => void login()} isDisabled={!site.loginUrl}>
        {site.login === "present" ? t("plugins.accounts.relogin") : t("plugins.accounts.login")}
      </Button>
      <Say text={say} />
    </div>
  );
}

/** cookie 文件至多多大，和内核的上限一样。 */
const COOKIE_FILE_MAX = 256 << 10;

/**
 * 「账号和访客身份」里的一个账号站点：登录着没有，「登录」开壳的登录窗口，「从文件导入登录状态…」读你选的 cookies.txt
 * 交给内核（只留这个站点的，存进钥匙串，文件本身不留），「退出登录」让内核忘掉、再清掉那份网页数据。
 */
export function SiteLoginRow({ site, users }: { site: SiteLogin; users?: string[] }) {
  const forget = useForgetLogin();
  const importFile = useImportLoginFile();
  const picker = useRef<HTMLInputElement>(null);
  const { login, say, setSay } = useLogin(site);
  const logged = site.login === "present";
  const fromFile = site.source === "cookie-file";

  async function picked(file: File | undefined) {
    setSay(null);
    importFile.reset();
    if (!file) return;
    if (file.size > COOKIE_FILE_MAX) {
      setSay(t("plugins.accounts.fileFailed"));
      return;
    }
    let text: string;
    try {
      text = await file.text();
    } catch {
      setSay(t("plugins.accounts.fileFailed"));
      return;
    }
    importFile.mutate({ site: site.site, text });
  }

  function logout() {
    setSay(null);
    forget.mutate({ site: site.site }, {
      onSuccess: () => {
        clearLoginData(site.site).catch((error: unknown) => setSay(typeof error === "string" ? error : t("plugins.accounts.loginClearFailed")));
      },
    });
  }

  return (
    <CardRow
      title={t("plugins.accounts.siteTitle", { name: site.name })}
      sub={
        <>
          <span className={site.login === "expired" ? "text-warn" : logged ? "text-label" : "text-sub"}>{LOGIN_STATE[site.login]}</span>
          {site.capturedAt ? ` · ${t(fromFile ? "plugins.accounts.fileSaved" : "plugins.accounts.siteSaved", { day: dayText(site.capturedAt) })}` : ""}
          {users && users.length > 0 ? ` · ${t("plugins.accounts.users", { names: formatList(users) })}` : ""}
          <span className="mono ml-2 text-tag text-faint">{site.site}</span>
          {!site.loginUrl ? <span className="block">{t("plugins.accounts.siteNoLoginUrl")}</span> : null}
          <span className="block">{t("plugins.accounts.fileNote", { name: site.name })}</span>
        </>
      }
      control={
        <>
          <Button variant={logged ? "outline" : "soft"} size="sm" onPress={() => void login()} isDisabled={!site.loginUrl}>
            {logged ? t("plugins.accounts.relogin") : t("plugins.accounts.login")}
          </Button>
          <Button variant="outline" size="sm" onPress={() => picker.current?.click()} isDisabled={importFile.isPending}>
            {t("plugins.accounts.importFile")}
          </Button>
          <input
            ref={picker}
            type="file"
            accept=".txt,text/plain"
            hidden
            aria-hidden="true"
            tabIndex={-1}
            onChange={(e) => {
              const input = e.currentTarget;
              void picked(input.files?.[0]).finally(() => {
                input.value = "";
              });
            }}
          />
          {site.login !== "missing" ? (
            <Button variant="quiet" size="sm" onPress={logout} isDisabled={forget.isPending}>
              {t("plugins.accounts.logout")}
            </Button>
          ) : null}
        </>
      }
    >
      {say || forget.error || importFile.error ? (
        <>
          <Say text={say} />
          {forget.error ? <ErrorLine error={forget.error} className="mt-1.5" /> : null}
          {importFile.error ? <ErrorLine error={importFile.error} className="mt-1.5" /> : null}
        </>
      ) : null}
    </CardRow>
  );
}
