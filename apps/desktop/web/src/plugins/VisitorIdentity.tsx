import { useState } from "react";
import type { VisitorIdentity as Visitor } from "../api/endpoints";
import { useForgetVisitor } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { clearVisitorData, openVisitor } from "../shell";
import { Button } from "../ui/Button";
import { CardRow } from "../ui/Card";
import { PAGE, VISITOR_STATE } from "./words";


/** 取一份（或换一份）访客身份：开一次那个站点的页面，页面自己生成的访客 cookie 由壳交给内核。 */
function useTake(visitor: Visitor) {
  const [say, setSay] = useState<string | null>(null);
  const stored = visitor.state !== "missing";
  async function take() {
    setSay(null);
    try {
      const opened = await openVisitor(visitor.site, stored);
      setSay(opened ? PAGE.visitorOpened : PAGE.visitorAppOnly);
    } catch (error) {
      setSay(typeof error === "string" ? error : PAGE.visitorFailed);
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
        {stored ? PAGE.replaceVisitor : PAGE.takeVisitor}
      </Button>
      <Say text={say} />
    </div>
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
      setSay(typeof error === "string" ? error : PAGE.visitorClearFailed);
      return;
    }
    forget.mutate(visitor.site);
  }

  return (
    <CardRow
      title={PAGE.visitorTitle(visitor.name)}
      sub={
        <>
          <span className={visitor.state === "expired" ? "text-warn" : stored ? "text-label" : "text-sub"}>{VISITOR_STATE[visitor.state]}</span>
          {users && users.length > 0 ? PAGE.users(users.join("、")) : ""}
          <span className="block">{PAGE.visitorBody(visitor.name)}</span>
        </>
      }
      control={
        <>
          <Button variant={stored ? "outline" : "soft"} size="sm" onPress={() => void take()}>
            {stored ? PAGE.replaceVisitor : PAGE.takeVisitor}
          </Button>
          {stored ? (
            <Button variant="quiet" size="sm" onPress={() => void remove()} isDisabled={forget.isPending}>
              {PAGE.removeVisitor}
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
