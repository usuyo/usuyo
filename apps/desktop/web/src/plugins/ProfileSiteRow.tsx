import { useState } from "react";
import type { SiteLogin } from "../api/endpoints";
import { useForgetLogin } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { dayText } from "../item/format";
import { clearLoginData } from "../shell";
import { Button } from "../ui/Button";
import { CardRow } from "../ui/Card";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import { LOGIN_STATE } from "./words";

/**
 * 「账号与访客身份」里在 usuyo 的站点窗口里登录的站点（`browser-profile`）。照实写：usuyo 在这台电脑上为它存了一份
 * 单独的网页数据（登录在里面，cookie 不拷到别处），和访客身份、你日常的浏览器分开；什么时候登录的，上一回哪种方法用它、
 * 找到几条；哪几种方法用它；登录后的页面里跑插件的脚本。「退出登录」先删掉那份网页数据，再让内核忘掉记下的时间。
 */
export function ProfileSiteRow({ site, users }: { site: SiteLogin; users?: string[] }) {
  const forget = useForgetLogin();
  const [say, setSay] = useState<string | null>(null);
  const signedIn = site.login === "present";

  // 先删网页数据、再让内核忘掉：前一步没成就什么都没去掉，可以再点
  async function logout() {
    setSay(null);
    try {
      await clearLoginData(site.site);
    } catch (error) {
      setSay(typeof error === "string" ? error : t("plugins.accounts.profileClearFailed"));
      return;
    }
    forget.mutate(site.site);
  }

  const lastUse = site.lastUse
    ? t("plugins.accounts.profileLastUse", { day: dayText(site.lastUse.at), method: site.lastUse.methodName, count: site.lastUse.noted })
    : t("plugins.accounts.profileNotUsed");

  return (
    <CardRow
      title={t("plugins.accounts.profileTitle", { name: site.name })}
      sub={
        <>
          <span className={signedIn ? "text-label" : "text-sub"}>{LOGIN_STATE[site.login]}</span>
          {signedIn && site.capturedAt ? ` · ${t("plugins.accounts.profileSignedIn", { day: dayText(site.capturedAt) })}` : ""}
          <span className="mono ml-2 text-tag text-faint">{site.site}</span>
          <span className="mt-1 block">{t("plugins.accounts.profileHeld", { name: site.name })}</span>
          <span className="block">{t("plugins.accounts.profileSeparate", { name: site.name })}</span>
          <span className="block">{t("plugins.accounts.profileScript")}</span>
          <span className="block">{lastUse}</span>
          {users && users.length > 0 ? <span className="block">{t("plugins.accounts.users", { names: formatList(users) })}</span> : null}
          {signedIn ? <span className="block">{t("plugins.accounts.profileLogoutNote")}</span> : null}
        </>
      }
      control={
        signedIn ? (
          <Button variant="quiet" size="sm" onPress={() => void logout()} isDisabled={forget.isPending}>
            {t("plugins.accounts.logout")}
          </Button>
        ) : null
      }
    >
      {say || forget.error ? (
        <>
          {say ? (
            <p role="status" className="m-0 mt-1.5 text-small text-sub">
              {say}
            </p>
          ) : null}
          {forget.error ? <ErrorLine error={forget.error} className="mt-1.5" /> : null}
        </>
      ) : null}
    </CardRow>
  );
}
