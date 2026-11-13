import { Link } from "@tanstack/react-router";
import type { ExportRun } from "../api/endpoints";
import { dayHeading, hm } from "../changes/words";
import { t } from "../i18n";
import { runText } from "./words";

/**
 * 一次导出做了什么：先读回的、两边都改过的（留下 usuyo 这份，那边那段进回收站）、这次没读也没写的和为什么。
 */
export function RunResult({ run, name }: { run: ExportRun; name: string }) {
  return (
    <section aria-labelledby="export-run-h" className="flex flex-col gap-4">
      <div>
        <h2 id="export-run-h" className="m-0 text-section font-semibold text-text">
          {t("export.run.heading", { day: dayHeading(run.at), time: hm(run.at) })}
        </h2>
        <p className={`m-0 mt-0.5 text-note ${run.state === "failed" ? "text-warn" : "text-label"}`}>{run.state === "failed" ? runText(run) : t("export.run.done", { summary: runText(run) })}</p>
      </div>
      {run.readBack.length > 0 ? (
        <div>
          <h3 className="m-0 text-note font-semibold text-text">{t("export.run.readBackTitle")}</h3>
          <ul className="m-0 mt-1.5 flex list-none flex-col p-0">
            {run.readBack.map((r) => (
              <li key={`${r.file}:${r.what}`} className="flex flex-wrap items-baseline gap-x-3 border-b border-border py-1.5 last:border-b-0">
                <span className="min-w-0 flex-1 truncate text-body text-text">{r.title}</span>
                <span className="text-small text-sub">{r.what}</span>
              </li>
            ))}
          </ul>
          <Link to="/changes" className="mt-1.5 inline-block text-small text-label underline decoration-border2 underline-offset-4 hover:text-text">
            {t("export.run.viewInChanges")}
          </Link>
        </div>
      ) : null}
      {run.both.length > 0 ? (
        <div>
          <h3 className="m-0 text-note font-semibold text-text">{t("export.run.bothTitle")}</h3>
          <p className="m-0 mt-0.5 text-small text-sub">{t("export.run.bothNote", { name })}</p>
          {run.both.map((b) => (
            <div key={b.file} className="mt-2.5 rounded-card border border-border px-4 py-3">
              <div className="text-body font-medium text-text">{b.title}</div>
              <dl className="m-0 mt-2 grid gap-x-3 gap-y-1.5 text-note @lg:grid-cols-[7rem_minmax(0,1fr)]">
                <dt className="text-small text-faint">{t("export.run.kept")}</dt>
                <dd className="m-0 text-text select-text">{b.kept}</dd>
                <dt className="text-small text-faint">{t("export.run.trashed", { name })}</dt>
                <dd className="m-0 text-sub select-text">{b.theirs}</dd>
              </dl>
            </div>
          ))}
          <Link to="/trash" className="mt-2 inline-block text-small text-label underline decoration-border2 underline-offset-4 hover:text-text">
            {t("export.run.goTrash")}
          </Link>
        </div>
      ) : null}
      {run.left.length > 0 ? (
        <div>
          <h3 className="m-0 text-note font-semibold text-text">{t("export.run.leftTitle")}</h3>
          <p className="m-0 mt-0.5 text-small text-sub">{t("export.run.leftNote", { name })}</p>
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
