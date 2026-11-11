import { useState } from "react";
import type { FetchPlugin } from "../api/endpoints";
import { usePluginChange } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { openVisitor } from "../shell";
import { Button } from "../ui/Button";

/**
 * 访客身份存在插件的这个密钥设置里（协议还没有「不是账号的 cookie」一类，见 docs/research/15 §1 第 3 条）。
 * 哪个插件去哪个首页取，由壳那张表定（src-tauri/src/login.rs）。
 */
export const VISITOR_KEY = "visitor-cookie";

/** 小红书这类：不登录也要一份访客身份才取得稳。开一次首页，页面自己生成的访客 cookie 由壳存下。 */
export function VisitorIdentity({ plugin }: { plugin: FetchPlugin }) {
  const change = usePluginChange(plugin.id);
  const [say, setSay] = useState<string | null>(null);
  const filled = plugin.settings.some((s) => s.key === VISITOR_KEY && s.filled === true);

  async function take() {
    setSay(null);
    try {
      const opened = await openVisitor(plugin.id);
      setSay(opened ? "窗口开着：页面自己打开，取到访客身份就关上。" : "这个窗口只在 usuyo 的应用里开得了，浏览器标签页里不行。");
    } catch (error) {
      setSay(typeof error === "string" ? error : "没能打开窗口");
    }
  }

  return (
    <>
      <h3 className="m-0 mt-8 text-section font-semibold text-text">访客身份</h3>
      <p className="m-0 mt-1.5 max-w-[700px] text-note text-sub">
        不登录也能取公开的笔记，但平台常要一个访客身份。usuyo 开一次{plugin.platforms[0]?.name ?? plugin.name}首页，不登录，把页面自己生成的访客身份存下来。它不是你的账号。
      </p>
      <div className="mt-3 flex items-center gap-3 rounded-card border border-border bg-card px-4 py-3.5">
        <div className="min-w-0 flex-1 text-body font-medium text-text">{filled ? "有一份访客身份" : "还没有访客身份"}</div>
        <Button variant={filled ? "outline" : "soft"} size="sm" onPress={() => void take()}>
          {filled ? "换一份" : "取访客身份"}
        </Button>
        {filled ? (
          <Button variant="quiet" size="sm" onPress={() => change.mutate({ kind: "forget-secret", key: VISITOR_KEY })} isDisabled={change.isPending}>
            去掉
          </Button>
        ) : null}
      </div>
      {say ? (
        <p role="status" className="m-0 mt-2 text-small text-sub">
          {say}
        </p>
      ) : null}
      {change.error ? <ErrorLine error={change.error} className="mt-2" /> : null}
    </>
  );
}
