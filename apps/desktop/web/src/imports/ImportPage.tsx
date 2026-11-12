import { Link, useNavigate, useSearch } from "@tanstack/react-router";
import { Plus } from "lucide-react";
import { useImports, usePlugins } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
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
  const { open, from } = useSearch({ from: "/import" });
  const navigate = useNavigate();
  const imports = useImports();
  const plugins = usePlugins();
  const methodOf = (pluginId: string | undefined, method: string | undefined) =>
    plugins.data?.plugins.find((p) => p.id === pluginId)?.methods.find((m) => m.id === method)?.name ?? method;
  const show = (id: string | undefined) => void navigate({ to: "/import", search: id ? { open: id } : {} });
  const half = (next: "platform" | "file") => void navigate({ to: "/import", search: next === "file" ? { from: "file" } : {}, replace: true });

  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航" />
      <main className="absolute inset-y-0 right-0 left-rail overflow-y-auto">
        {/* 窗口够宽时两栏，记录在右；窄了记录排到下面。 */}
        <div className="grid grid-cols-[minmax(0,752px)_320px] justify-between gap-x-8 gap-y-6 px-gutter pt-7 pb-10 max-[1040px]:grid-cols-1">
          <header className="col-span-full flex max-w-[752px] items-start justify-between gap-4">
            <div className="min-w-0">
              <h1 className="m-0 text-title font-semibold text-text">导入</h1>
              <p className="m-0 mt-1 text-note text-sub">把你在平台上存过的、别的工具导出的列出来，先看一眼，再慢慢收下。</p>
            </div>
            {open ? (
              <Button variant="outline" size="md" className="flex-none" onPress={() => show(undefined)}>
                <Plus size={15} aria-hidden="true" />
                新的导入
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
                    label="从哪儿导入"
                    options={[
                      { id: "platform", text: "平台上存过的" },
                      { id: "file", text: "别的工具导出的文件" },
                    ]}
                    value={from === "file" ? "file" : "platform"}
                    onChange={half}
                  />
                </div>
                {from === "file" ? (
                  <FileImport onStarted={show} />
                ) : (
                  <>
                    <SourcePicker onStarted={show} />
                    <DeliverTokens />
                  </>
                )}
              </>
            )}
          </div>
          <aside aria-label="导入记录" className="min-w-0 px-2 max-[1040px]:mt-4">
            <h2 className="m-0 text-body font-semibold text-text">导入记录</h2>
            <p className="m-0 mt-0.5 text-small text-sub">每次导入留一条记录；交进来的清单也在这里。</p>
            {imports.error ? <ErrorLine error={imports.error} className="mt-3" /> : null}
            <ul className="m-0 mt-3 flex list-none flex-col border-t border-border p-0">
              {(imports.data ?? []).map((v) => (
                <li key={v.id} className="border-b border-border">
                  <Link
                    to="/import"
                    search={{ open: v.id }}
                    className={`press -mx-2 my-1 block rounded-btn px-2 py-2 no-underline hover:bg-chip/60 ${v.id === open ? "bg-chip" : ""}`}
                  >
                    <div className="mono text-tag text-faint">
                      {dayText(v.createdAt)}
                      {v.kind === "delivered"
                        ? ` · 交进来的${v.client ? ` · ${v.client}` : ""}`
                        : v.kind === "file"
                          ? " · 别的工具导出的文件"
                          : v.method
                            ? ` · ${methodOf(v.pluginId, v.method)}`
                            : ""}
                    </div>
                    <div className="mt-0.5 truncate text-body font-medium text-text">{v.title}</div>
                    <div className="text-small text-sub">{stateLine(v)}</div>
                  </Link>
                </li>
              ))}
            </ul>
            {imports.data && imports.data.length === 0 ? <p className="m-0 mt-3 text-small text-faint">还没有导入过。</p> : null}
          </aside>
        </div>
      </main>
    </div>
  );
}
