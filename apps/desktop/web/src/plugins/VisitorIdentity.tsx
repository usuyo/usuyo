import { useState } from "react";
import type { VisitorIdentity as Visitor } from "../api/endpoints";
import { useForgetVisitor } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { openVisitor } from "../shell";
import { Button } from "../ui/Button";

const STATE_TEXT: Record<Visitor["state"], string> = {
  missing: "还没有访客身份",
  present: "有一份访客身份",
  expired: "访客身份过期了",
};

/** 插件清单里声明的访客身份：不登录也要一份才取得稳。开一次那个站点的页面，页面自己生成的访客 cookie 由壳交给内核。 */
export function VisitorIdentity({ visitor }: { visitor: Visitor }) {
  const forget = useForgetVisitor();
  const [say, setSay] = useState<string | null>(null);
  const stored = visitor.state !== "missing";

  async function take() {
    setSay(null);
    try {
      const opened = await openVisitor(visitor.site);
      setSay(opened ? "窗口开着：页面自己打开，取到访客身份就关上。" : "这个窗口只在 usuyo 的应用里开得了，浏览器标签页里不行。");
    } catch (error) {
      setSay(typeof error === "string" ? error : "没能打开窗口");
    }
  }

  return (
    <>
      <h3 className="m-0 mt-8 text-section font-semibold text-text">访客身份</h3>
      <p className="m-0 mt-1.5 max-w-[700px] text-note text-sub">
        不登录也能取公开的内容，但平台常要一个访客身份。usuyo 开一次{visitor.name}首页，不登录，把页面自己生成的访客身份存下来。它不是你的账号。
      </p>
      <div className="mt-3 flex items-center gap-3 rounded-card border border-border bg-card px-4 py-3.5">
        <div className={`min-w-0 flex-1 text-body font-medium ${visitor.state === "expired" ? "text-warn" : "text-text"}`}>{STATE_TEXT[visitor.state]}</div>
        <Button variant={stored ? "outline" : "soft"} size="sm" onPress={() => void take()}>
          {stored ? "换一份" : "取访客身份"}
        </Button>
        {stored ? (
          <Button variant="quiet" size="sm" onPress={() => forget.mutate(visitor.site)} isDisabled={forget.isPending}>
            去掉
          </Button>
        ) : null}
      </div>
      {say ? (
        <p role="status" className="m-0 mt-2 text-small text-sub">
          {say}
        </p>
      ) : null}
      {forget.error ? <ErrorLine error={forget.error} className="mt-2" /> : null}
    </>
  );
}
