import { ApiError, DaemonUnreachableError, isNotServed } from "../api/client";
import { t } from "../i18n";

/** 一行平静的提示：内核给了 `hint` 就照它说；内核没在运行时不报错，只说一句。 */
export function ErrorLine({ error, className = "" }: { error: Error; className?: string }) {
  const text =
    error instanceof DaemonUnreachableError
      ? (error.reason ?? t("common.error.daemonDown"))
      : isNotServed(error)
        ? t("common.error.notServed")
        : error instanceof ApiError
          ? (error.hint ?? t("common.error.status", { status: error.status }))
          : t("common.error.loadFailed");
  return (
    <p role="status" className={`m-0 text-small text-sub ${className}`}>
      {text}
    </p>
  );
}

/** 页面要的接口内核还没有：一句话说清楚这里以后放什么，不报错，也不拿样例数据充数。 */
export function NotServedLine({ children, className = "" }: { children: string; className?: string }) {
  return (
    <p role="status" className={`m-0 max-w-[560px] text-note text-sub ${className}`}>
      {children}
    </p>
  );
}
