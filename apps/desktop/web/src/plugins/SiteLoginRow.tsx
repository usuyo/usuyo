import { useState } from "react";
import type { SiteLogin } from "../api/endpoints";
import { useForgetLogin } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { dayText } from "../item/format";
import { clearLoginData, openLogin } from "../shell";
import { Button } from "../ui/Button";
import { CardRow } from "../ui/Card";
import { LOGIN_STATE, PAGE } from "./words";


/** 开壳的登录窗口：在 usuyo 给这个站点单独开的窗口里，你自己登录，和你日常用的浏览器分开。 */
function useLogin(site: SiteLogin) {
  const [say, setSay] = useState<string | null>(null);
  async function login() {
    setSay(null);
    try {
      const opened = await openLogin(site.site);
      setSay(opened ? PAGE.loginOpened : PAGE.loginAppOnly);
    } catch (error) {
      setSay(typeof error === "string" ? error : PAGE.loginFailed);
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
        {site.login === "present" ? PAGE.relogin : PAGE.login}
      </Button>
      <Say text={say} />
    </div>
  );
}

/** 「账号和访客身份」里的一个账号站点：登录着没有，「登录」开壳的登录窗口，「退出登录」让内核忘掉、再清掉那份网页数据。 */
export function SiteLoginRow({ site, users }: { site: SiteLogin; users?: string[] }) {
  const forget = useForgetLogin();
  const { login, say, setSay } = useLogin(site);
  const logged = site.login === "present";

  function logout() {
    setSay(null);
    forget.mutate(site.site, {
      onSuccess: () => {
        clearLoginData(site.site).catch((error: unknown) => setSay(typeof error === "string" ? error : PAGE.loginClearFailed));
      },
    });
  }

  return (
    <CardRow
      title={PAGE.siteTitle(site.name)}
      sub={
        <>
          <span className={site.login === "expired" ? "text-warn" : logged ? "text-label" : "text-sub"}>{LOGIN_STATE[site.login]}</span>
          {site.capturedAt ? PAGE.siteSaved(dayText(site.capturedAt)) : ""}
          {users && users.length > 0 ? PAGE.users(users.join("、")) : ""}
          <span className="mono ml-2 text-tag text-faint">{site.site}</span>
          {!site.loginUrl ? <span className="block">{PAGE.siteNoLoginUrl}</span> : null}
        </>
      }
      control={
        <>
          <Button variant={logged ? "outline" : "soft"} size="sm" onPress={() => void login()} isDisabled={!site.loginUrl}>
            {logged ? PAGE.relogin : PAGE.login}
          </Button>
          {site.login !== "missing" ? (
            <Button variant="quiet" size="sm" onPress={logout} isDisabled={forget.isPending}>
              {PAGE.logout}
            </Button>
          ) : null}
        </>
      }
    >
      {say || forget.error ? (
        <>
          <Say text={say} />
          {forget.error ? <ErrorLine error={forget.error} className="mt-1.5" /> : null}
        </>
      ) : null}
    </CardRow>
  );
}
