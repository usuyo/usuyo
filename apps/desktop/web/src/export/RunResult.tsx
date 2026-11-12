import { Link } from "@tanstack/react-router";
import type { ExportRun } from "../api/endpoints";
import { dayHeading, hm } from "../changes/words";
import { runText } from "./words";

/**
 * 一次导出做了什么：先读回的、两边都改过的（留下 usuyo 这份，那边那段进回收站）、这次没读也没写的和为什么。
 */
export function RunResult({ run, name }: { run: ExportRun; name: string }) {
  return (
    <section aria-labelledby="export-run-h" className="flex flex-col gap-4">
      <div>
        <h2 id="export-run-h" className="m-0 text-section font-semibold text-text">
          {dayHeading(run.at)} {hm(run.at)} 这一次
        </h2>
        <p className={`m-0 mt-0.5 text-note ${run.state === "failed" ? "text-warn" : "text-label"}`}>{runText(run)}。先读回，再写出：读回的改动在写出之前就合进了 usuyo。</p>
      </div>
      {run.readBack.length > 0 ? (
        <div>
          <h3 className="m-0 text-note font-semibold text-text">读回来的</h3>
          <ul className="m-0 mt-1.5 flex list-none flex-col p-0">
            {run.readBack.map((r) => (
              <li key={`${r.file}:${r.what}`} className="flex flex-wrap items-baseline gap-x-3 border-b border-border py-1.5 last:border-b-0">
                <span className="min-w-0 flex-1 truncate text-body text-text">{r.title}</span>
                <span className="text-small text-sub">{r.what}</span>
              </li>
            ))}
          </ul>
          <Link to="/changes" className="mt-1.5 inline-block text-small text-label underline decoration-border2 underline-offset-4 hover:text-text">
            在改动记录里看，能整批撤销
          </Link>
        </div>
      ) : null}
      {run.both.length > 0 ? (
        <div>
          <h3 className="m-0 text-note font-semibold text-text">两边都改过的</h3>
          <p className="m-0 mt-0.5 text-small text-sub">上次导出以后，usuyo 和 {name} 都改了同一处。留下的是 usuyo 这份；{name} 那段放进了回收站，想用它，在那里点「换上这段」。</p>
          {run.both.map((b) => (
            <div key={b.file} className="mt-2.5 rounded-card border border-border px-4 py-3">
              <div className="text-body font-medium text-text">{b.title}</div>
              <dl className="m-0 mt-2 grid gap-x-3 gap-y-1.5 text-note @lg:grid-cols-[7rem_minmax(0,1fr)]">
                <dt className="text-small text-faint">留下的：usuyo</dt>
                <dd className="m-0 text-text select-text">{b.kept}</dd>
                <dt className="text-small text-faint">进了回收站：{name}</dt>
                <dd className="m-0 text-sub select-text">{b.theirs}</dd>
              </dl>
            </div>
          ))}
          <Link to="/trash" className="mt-2 inline-block text-small text-label underline decoration-border2 underline-offset-4 hover:text-text">
            去回收站
          </Link>
        </div>
      ) : null}
      {run.left.length > 0 ? (
        <div>
          <h3 className="m-0 text-note font-semibold text-text">这次没读也没写的</h3>
          <p className="m-0 mt-0.5 text-small text-sub">在 {name} 里改好，下次导出照常读回。</p>
          <ul className="m-0 mt-1.5 flex list-none flex-col p-0">
            {run.left.map((l) => (
              <li key={l.file} className="border-b border-border py-1.5 last:border-b-0">
                <span className="mono block truncate text-tag text-label">{l.file}</span>
                <span className="block text-small text-warn">{l.reason}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </section>
  );
}
