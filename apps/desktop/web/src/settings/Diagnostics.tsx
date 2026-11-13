import { useMap } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { t } from "../i18n";
import { Row } from "./Row";

/** 诊断：读不了、被跳过的条目（地图里给，原因是内核的话，原样显示）；日志在哪（设置里给）。 */
export function Diagnostics({ logs }: { logs: string | undefined }) {
  const map = useMap();
  const skipped = map.data?.unreadable ?? [];
  return (
    <div className="flex flex-col gap-5">
      <Row label={t("settings.diagnostics.unreadable")} sub={t("settings.diagnostics.unreadableSub")}>
        {map.error ? (
          <ErrorLine error={map.error} className="pt-2" />
        ) : skipped.length === 0 ? (
          <p className="m-0 pt-2 text-note text-sub">{map.data ? t("settings.diagnostics.allReadable") : ""}</p>
        ) : (
          <ul className="m-0 flex list-none flex-col gap-1.5 p-0 pt-1">
            {skipped.map((u) => (
              <li key={u.id} className="min-w-0 rounded-btn bg-card px-3 py-2">
                <div className="mono truncate text-tag text-label" title={u.id}>
                  {u.id}
                </div>
                <div className="text-small text-sub">{u.reason}</div>
              </li>
            ))}
          </ul>
        )}
      </Row>
      <Row label={t("settings.diagnostics.log")} sub={t("settings.diagnostics.logSub")}>
        {logs ? (
          <code className="mono block truncate rounded-sm bg-chip px-3 py-2 text-tag text-text select-text" title={logs}>
            {logs}
          </code>
        ) : null}
      </Row>
    </div>
  );
}
