import { ApiError, DaemonUnreachableError } from "../api/client";

/** 一行平静的提示。内核没在运行时不报错，只说一句。 */
export function ErrorLine({ error }: { error: Error }) {
  const text =
    error instanceof DaemonUnreachableError
      ? "内核没在运行"
      : error instanceof ApiError
        ? `内核回了 ${error.status}`
        : "没能取到";
  return (
    <p role="status" className="muted">
      {text}
    </p>
  );
}
