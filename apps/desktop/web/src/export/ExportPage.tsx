import { Link } from "@tanstack/react-router";
import { FolderOutput } from "lucide-react";
import { useState } from "react";
import { Button as AriaButton, Input, Label, TextField } from "react-aria-components";
import { isNotServed } from "../api/client";
import type { ExportView } from "../api/endpoints";
import { useExportFolder, useExportNow, useExportView } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { rich, t } from "../i18n";
import { dayText } from "../item/format";
import { hm } from "../changes/words";
import { Button } from "../ui/Button";
import { NoteShape } from "./NoteShape";
import { RunResult } from "./RunResult";
import { runText, willText } from "./words";

/** 导出到 Obsidian：只在你点「现在导出」时做，先读回你在那边改的，再写出。左边是这一次和设置，右边是记录。 */
export function ExportPage() {
  const view = useExportView();
  const missing = isNotServed(view.error);
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label={t("common.nav.main")} />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-7 pb-16 lg:px-gutter">
        {missing ? (
          <>
            <h1 className="m-0 text-title font-semibold text-text">{t("export.title")}</h1>
            <NotServedLine className="mt-4">{t("export.notServed")}</NotServedLine>
          </>
        ) : view.error ? (
          <ErrorLine error={view.error} />
        ) : view.data ? (
          <ExportBody view={view.data} />
        ) : null}
      </main>
    </div>
  );
}

function ExportBody({ view }: { view: ExportView }) {
  const run = useExportNow();
  const folderChange = useExportFolder();
  const [shown, setShown] = useState<string | null>(null);
  const [editing, setEditing] = useState(view.folder === null);
  const [folder, setFolder] = useState(view.folder ?? "");
  const current = view.runs.find((r) => r.id === shown) ?? view.runs[0];
  const name = view.target.name;
  return (
    <>
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0 max-w-[640px]">
          <h1 className="m-0 text-title font-semibold text-text">{t("export.titleTo", { name })}</h1>
          <p className="m-0 mt-1 text-note text-sub">{t("export.intro", { name })}</p>
        </div>
        <Button variant="soft" size="md" isDisabled={view.folder === null || run.isPending} onPress={() => run.mutate(undefined, { onSuccess: (r) => setShown(r.id) })}>
          <FolderOutput size={15} aria-hidden="true" />
          {run.isPending ? t("export.exporting") : view.runs.length > 0 ? t("export.exportAgain") : t("export.exportNow")}
        </Button>
      </header>
      {run.error ? <ErrorLine error={run.error} className="mt-3" /> : null}

      <div className="mt-7 grid items-start gap-x-12 gap-y-10 @[900px]:grid-cols-[minmax(0,1fr)_minmax(0,300px)]">
        <div className="flex min-w-0 flex-col gap-9">
          {current ? <RunResult run={current} name={name} /> : null}

          <section aria-labelledby="export-where-h">
            <h2 id="export-where-h" className="m-0 text-section font-semibold text-text">
              {t("export.whereTitle")}
            </h2>
            {view.folder !== null && !editing ? (
              <div className="mt-2.5 flex flex-wrap items-center gap-2">
                <code className="mono min-w-0 flex-1 truncate rounded-sm bg-chip px-3 py-2 text-tag text-text select-text" title={view.folder}>
                  {view.folder}
                </code>
                <Button variant="outline" size="sm" onPress={() => setEditing(true)}>
                  {t("export.changeFolder")}
                </Button>
              </div>
            ) : (
              <TextField value={folder} onChange={setFolder} aria-label={t("export.folderLabel")} className="mt-2.5 flex flex-wrap items-center gap-2">
                <Label className="sr-only-text">{t("export.folderLabel")}</Label>
                <Input
                  placeholder={t("export.folderPlaceholder", { name })}
                  className="mono h-ctl-sm min-w-0 flex-1 rounded-input border border-border2 bg-transparent px-3 text-tag text-text outline-none focus:border-label"
                />
                <Button
                  variant="soft"
                  size="sm"
                  isDisabled={folder.trim() === "" || folderChange.isPending}
                  onPress={() => folderChange.mutate(folder, { onSuccess: () => setEditing(false) })}
                >
                  {t("export.useFolder")}
                </Button>
                {view.folder !== null ? (
                  <Button variant="quiet" size="sm" onPress={() => setEditing(false)}>
                    {t("common.cancel")}
                  </Button>
                ) : null}
              </TextField>
            )}
            {folderChange.error ? <ErrorLine error={folderChange.error} className="mt-2" /> : null}
            <p className="m-0 mt-2 text-small text-sub">
              {t("export.whereNote")}
            </p>
          </section>

          <section aria-labelledby="export-what-h">
            <h2 id="export-what-h" className="m-0 text-section font-semibold text-text">
              {t("export.whatTitle")}
            </h2>
            <p className="m-0 mt-1.5 text-note text-label">{willText(view.will)}</p>
            <p className="m-0 mt-1 text-small text-sub">{t("export.readBackNote", { name })}</p>
            <div className="mt-4">
              <NoteShape sample={view.sample} name={name} />
            </div>
          </section>
        </div>

        <aside aria-labelledby="export-runs-h" className="min-w-0">
          <h2 id="export-runs-h" className="m-0 text-section font-semibold text-text">
            {t("export.runsTitle")}
          </h2>
          <p className="m-0 mt-0.5 text-small text-sub">
            {rich("export.runsNote", {
              link: <Link to="/changes" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text" />,
            })}
          </p>
          {view.runs.length === 0 ? <p className="m-0 mt-3 text-note text-sub">{t("export.noRuns")}</p> : null}
          <ul className="m-0 mt-3 flex list-none flex-col border-t border-border p-0">
            {view.runs.map((r) => (
              <li key={r.id} className="border-b border-border">
                <AriaButton
                  aria-current={r.id === current?.id ? "true" : undefined}
                  onPress={() => setShown(r.id)}
                  className={`press -mx-2 my-1 block w-[calc(100%+1rem)] cursor-pointer rounded-btn border-0 px-2 py-2 text-left font-sans outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60 ${r.id === current?.id ? "bg-chip" : "bg-transparent"}`}
                >
                  <span className="block text-tag text-faint">
                    {dayText(r.at)} {hm(r.at)}
                  </span>
                  <span className={`block text-note ${r.state === "failed" ? "text-warn" : "text-label"}`}>{runText(r)}</span>
                </AriaButton>
              </li>
            ))}
          </ul>
        </aside>
      </div>
    </>
  );
}
