import { Link, useNavigate, useSearch } from "@tanstack/react-router";
import { Plus } from "lucide-react";
import type { ImportView } from "../api/endpoints";
import { useImports, usePlugins } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { t } from "../i18n";
import { dayText } from "../item/format";
import { Button } from "../ui/Button";
import { Choice } from "../ui/Choice";
import { DeliverTokens } from "./DeliverTokens";
import { FileImport } from "./FileImport";
import { ImportDetail } from "./ImportDetail";
import { SourcePicker } from "./SourcePicker";
import { stateLine } from "./words";

/** 导入：左边选来源、看预览、收；右边一栏是每一次导入的记录，交进来的清单也在这里。 */
export function ImportPage() {
  const { open, from, pick, reader } = useSearch({ from: "/import" });
  const navigate = useNavigate();
  const imports = useImports();
  const plugins = usePlugins();
  const methodOf = (pluginId: string | undefined, method: string | undefined) =>
    plugins.data?.plugins.find((p) => p.id === pluginId)?.methods.find((m) => m.id === method)?.name ?? method;
  const show = (id: string | undefined) => void navigate({ to: "/import", search: id ? { open: id } : {} });
  const half = (next: "platform" | "file") => void navigate({ to: "/import", search: next === "file" ? { from: "file" } : {}, replace: true });

  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label={t("common.nav.main")} />
      <main className="absolute inset-y-0 right-0 left-rail overflow-y-auto">
        {/* 窗口够宽时两栏，记录在右；窄了记录排到下面。 */}
        <div className="grid grid-cols-[minmax(0,752px)_320px] justify-between gap-x-8 gap-y-6 px-gutter pt-7 pb-10 max-[1040px]:grid-cols-1">
          <header className="col-span-full flex max-w-[752px] items-start justify-between gap-4">
            <div className="min-w-0">
              <h1 className="m-0 text-title font-semibold text-text">{t("imports.title")}</h1>
              <p className="m-0 mt-1 text-note text-sub">{t("imports.intro")}</p>
            </div>
            {open ? (
              <Button variant="outline" size="md" className="flex-none" onPress={() => show(undefined)}>
                <Plus size={15} aria-hidden="true" />
                {t("imports.newImport")}
              </Button>
            ) : null}
          </header>
          <div className="min-w-0">
            {open ? (
              <ImportDetail key={open} id={open} onGone={() => show(undefined)} />
            ) : (
              <>
                <div className="mb-5">
                  <Choice
                    label={t("imports.fromLabel")}
                    options={[
                      { id: "platform", text: t("imports.fromPlatform") },
                      { id: "file", text: t("imports.fromFile") },
                    ]}
                    value={from === "file" ? "file" : "platform"}
                    onChange={half}
                  />
                </div>
                {from === "file" ? (
                  <FileImport onStarted={show} {...(reader ? { initial: reader } : {})} />
                ) : (
                  <>
                    <SourcePicker onStarted={show} {...(pick ? { initial: pick } : {})} />
                    <DeliverTokens />
                  </>
                )}
              </>
            )}
          </div>
          <aside aria-label={t("imports.history.title")} className="min-w-0 px-2 max-[1040px]:mt-4">
            <h2 className="m-0 text-body font-semibold text-text">{t("imports.history.title")}</h2>
            <p className="m-0 mt-0.5 text-small text-sub">{t("imports.history.note")}</p>
            {imports.error ? <ErrorLine error={imports.error} className="mt-3" /> : null}
            <ul className="m-0 mt-3 flex list-none flex-col border-t border-border p-0">
              {(imports.data ?? []).map((v) => (
                <li key={v.id} className="border-b border-border">
                  <Link
                    to="/import"
                    search={{ open: v.id }}
                    className={`press -mx-2 my-1 block rounded-btn px-2 py-2 no-underline hover:bg-chip/60 ${v.id === open ? "bg-chip" : ""}`}
                  >
                    <div className="mono text-tag text-faint">{historyMeta(v, dayText(v.createdAt), methodOf(v.pluginId, v.method))}</div>
                    <div className="mt-0.5 truncate text-body font-medium text-text">{v.title}</div>
                    <div className="text-small text-sub">{stateLine(v)}</div>
                  </Link>
                </li>
              ))}
            </ul>
            {imports.data && imports.data.length === 0 ? <p className="m-0 mt-3 text-small text-faint">{t("imports.history.empty")}</p> : null}
          </aside>
        </div>
      </main>
    </div>
  );
}

/** 记录里那一行的日子和来处：发送来的写发送方，文件写「其他工具导出的文件」，插件列的写那种获取方式。 */
function historyMeta(v: ImportView, day: string, method: string | undefined): string {
  if (v.kind === "delivered") return v.client ? t("imports.history.delivered", { day, client: v.client }) : t("imports.history.deliveredAnon", { day });
  if (v.kind === "file") return t("imports.history.file", { day });
  return method ? t("imports.history.method", { day, method }) : day;
}
