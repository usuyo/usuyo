import { useState } from "react";
import type { SiteLogin } from "../api/endpoints";
import { useForgetLogin } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { dayText } from "../item/format";
import { clearLoginData, openLogin } from "../shell";
import { Button } from "../ui/Button";

const STATE_TEXT: Record<SiteLogin["login"], string> = {
  missing: "还没登录",
  present: "登录着",
  expired: "登录过期了",
};

/** 一个账号站点：登录着没有，「登录」开壳的登录窗口，「退出登录」让内核忘掉、再清掉那份网页数据。 */
export function SiteLoginRow({ site }: { site: SiteLogin }) {
  const forget = useForgetLogin();
  const [say, setSay] = useState<string | null>(null);
  const logged = site.login === "present";

  async function login() {
    setSay(null);
    try {
      const opened = await openLogin(site.site);
      setSay(opened ? "登录窗口开着：在那里登录好了，它会自己关上。" : "登录窗口只在 usuyo 的应用里开得了，浏览器标签页里不行。");
    } catch (error) {
      setSay(typeof error === "string" ? error : "没能打开登录窗口");
    }
  }

  function logout() {
    setSay(null);
    forget.mutate(site.site, {
      onSuccess: () => {
        clearLoginData(site.site).catch((error: unknown) => setSay(typeof error === "string" ? error : "没能清掉这个站点的网页数据"));
      },
    });
  }

  return (
    <div className="rounded-card border border-border bg-card px-4 py-3.5">
      <div className="flex items-center gap-3">
        <div className="min-w-0 flex-1">
          <div className="text-body font-medium text-text">会用到你的{site.name}账号</div>
          <div className="mt-0.5 text-small text-sub">
            <span className={site.login === "expired" ? "text-warn" : logged ? "text-label" : "text-sub"}>{STATE_TEXT[site.login]}</span>
            {site.capturedAt ? ` · ${dayText(site.capturedAt)}存下` : ""}
            <span className="mono ml-2 text-tag text-faint">{site.site}</span>
          </div>
        </div>
        <Button variant={logged ? "outline" : "soft"} size="sm" onPress={() => void login()} isDisabled={!site.loginUrl}>
          {logged ? "重新登录" : "登录"}
        </Button>
        {site.login !== "missing" ? (
          <Button variant="quiet" size="sm" onPress={logout} isDisabled={forget.isPending}>
            退出登录
          </Button>
        ) : null}
      </div>
      {!site.loginUrl ? <p className="m-0 mt-2 text-small text-sub">插件没写登录页在哪，这里开不了登录窗口。</p> : null}
      {say ? (
        <p role="status" className="m-0 mt-2 text-small text-sub">
          {say}
        </p>
      ) : null}
      {forget.error ? <ErrorLine error={forget.error} className="mt-2" /> : null}
    </div>
  );
}
