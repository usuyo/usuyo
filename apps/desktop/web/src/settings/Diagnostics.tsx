import { useMap } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Row } from "./Row";

/** 诊断：读不了、被跳过的条目（地图里给）；日志在哪（设置里给）。 */
export function Diagnostics({ logs }: { logs: string | undefined }) {
  const map = useMap();
  const skipped = map.data?.unreadable ?? [];
  return (
    <div className="flex flex-col gap-5">
      <Row label="读不了的" sub="多半是在 usuyo 外面改坏了文件。usuyo 跳过它们，别的照常；改好了文件，下次打开就回来了。">
        {map.error ? (
          <ErrorLine error={map.error} className="pt-2" />
        ) : skipped.length === 0 ? (
          <p className="m-0 pt-2 text-note text-sub">{map.data ? "每一条都读得了。" : ""}</p>
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
      <Row label="日志" sub="内核记下的运行情况，出了问题时从这里查。">
        {logs ? (
          <code className="mono block truncate rounded-sm bg-chip px-3 py-2 text-tag text-text select-text" title={logs}>
            {logs}
          </code>
        ) : null}
      </Row>
    </div>
  );
}
