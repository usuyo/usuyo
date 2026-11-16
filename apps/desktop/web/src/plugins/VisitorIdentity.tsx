import { useState } from "react";
import type { VisitorIdentity as Visitor } from "../api/endpoints";
import { useForgetVisitor } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { dayText } from "../item/format";
import { clearVisitorData, openVisitor } from "../shell";
import { Button } from "../ui/Button";
import { CardRow } from "../ui/Card";
import { Fold } from "../ui/Fold";
import { t } from "../i18n";
import { formatList } from "../i18n/format";
import { VISITOR_STATE } from "./words";

/** 取一份（或换一份）访客身份：开一次那个站点的页面，页面自己生成的访客 cookie 由壳交给内核。 */
function useTake(visitor: Visitor) {
  const [say, setSay] = useState<string | null>(null);
  const stored = visitor.state !== "missing";
  async function take() {
    setSay(null);
    try {
      const opened = await openVisitor(visitor.site, stored);
      setSay(opened ? t("plugins.accounts.visitorOpened") : t("plugins.accounts.visitorAppOnly"));
    } catch (error) {
      setSay(typeof error === "string" ? error : t("plugins.accounts.visitorFailed"));
    }
  }
  return { take, say, setSay, stored };
}

function Say({ text }: { text: string | null }) {
  return text ? (
    <p role="status" className="m-0 mt-1.5 text-small text-sub">
      {text}
    </p>
  ) : null;
}

/** 缺访客身份时那一步的按钮。 */
export function VisitorTake({ visitor }: { visitor: Visitor }) {
  const { take, say, stored } = useTake(visitor);
  return (
    <div className="min-w-0">
      <Button variant="soft" size="sm" onPress={() => void take()}>
        {stored ? t("plugins.accounts.replaceVisitor") : t("plugins.accounts.takeVisitor")}
      </Button>
      <Say text={say} />
    </div>
  );
}

/** 技术信息：存着的 cookie 名字（不含值）、取它时窗口的 UA、上一回没取到时缺了哪几个。真跑一回以后照它看缺什么。 */
function Technical({ visitor }: { visitor: Visitor }) {
  const none = t("plugins.accounts.visitorNone");
  const names = (list: string[]) => (list.length > 0 ? list.join(", ") : none);
  const miss = visitor.lastMissed;
  return (
    <Fold title={t("plugins.accounts.visitorDiagnostics")} className="mt-1.5">
      <dl className="m-0 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-small text-sub">
        <dt>{t("plugins.accounts.visitorCookies")}</dt>
        <dd className="m-0 min-w-0 break-words font-mono">{names(visitor.storedCookies)}</dd>
        <dt>{t("plugins.accounts.visitorAgent")}</dt>
        <dd className="m-0 min-w-0 break-words font-mono">{visitor.userAgent ?? none}</dd>
        {miss ? (
          <>
            <dt>{t("plugins.accounts.visitorMissed", { day: dayText(miss.at) })}</dt>
            <dd className="m-0 min-w-0 break-words">
              {t("plugins.accounts.visitorMissedNames", { seen: names(miss.seen), missing: names(miss.missing) })}
            </dd>
          </>
        ) : null}
      </dl>
    </Fold>
  );
}

/**
 * 「账号和访客身份」里的一行：插件清单里声明的访客身份，不登录也要一份才取得稳；有没有、取一份、去掉，和哪几种方法用它。
 */
export function VisitorIdentity({ visitor, users }: { visitor: Visitor; users?: string[] }) {
  const forget = useForgetVisitor();
  const { take, say, setSay, stored } = useTake(visitor);

  // 先清窗口的网页数据、再让内核删：前一步失败就什么都没去掉，可以再点；
  // 反过来的话内核删了而窗口还留着旧 cookie，再取会拿回上一份。内核那一步成败才算去掉了没有。
  async function remove() {
    setSay(null);
    try {
      await clearVisitorData(visitor.site);
    } catch (error) {
      setSay(typeof error === "string" ? error : t("plugins.accounts.visitorClearFailed"));
      return;
    }
    forget.mutate(visitor.site);
  }

  return (
    <CardRow
      title={t("plugins.accounts.visitorTitle", { name: visitor.name })}
      sub={
        <>
          <span className={visitor.state === "expired" ? "text-warn" : stored ? "text-label" : "text-sub"}>{VISITOR_STATE[visitor.state]}</span>
          {stored && visitor.capturedAt ? ` · ${t("plugins.accounts.visitorSaved", { day: dayText(visitor.capturedAt) })}` : ""}
          {users && users.length > 0 ? ` · ${t("plugins.accounts.users", { names: formatList(users) })}` : ""}
          <span className="block">{t("plugins.accounts.visitorBody", { name: visitor.name })}</span>
        </>
      }
      control={
        <>
          <Button variant={stored ? "outline" : "soft"} size="sm" onPress={() => void take()}>
            {stored ? t("plugins.accounts.replaceVisitor") : t("plugins.accounts.takeVisitor")}
          </Button>
          {stored ? (
            <Button variant="quiet" size="sm" onPress={() => void remove()} isDisabled={forget.isPending}>
              {t("plugins.accounts.removeVisitor")}
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
      <Technical visitor={visitor} />
    </CardRow>
  );
}
