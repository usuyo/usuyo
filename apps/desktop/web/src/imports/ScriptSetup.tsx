import { Copy } from "lucide-react";
import type { NewDeliverToken } from "../api/endpoints";
import { t } from "../i18n";
import type { Use } from "../plugins/platforms";
import { ScriptFile, ScriptPage } from "../plugins/SavedCard";
import { Button } from "../ui/Button";
import { FlashLine, useFlash } from "../ui/Flash";

/**
 * 浏览器脚本导入：令牌已经生成好（名字是平台名加「脚本」），这里给出要装的脚本、令牌和之后的两步。
 * 令牌只在这一次显示；点「完成」收起，以后在「高级」里撤销或另生成。
 */
export function ScriptSetup({ use, token, onDone }: { use: Use; token: NewDeliverToken; onDone: () => void }) {
  const flash = useFlash();
  const steps = [
    { key: "manager", title: t("plugins.script.stepManager"), body: <p className="m-0 text-small text-sub">{t("imports.script.managerBody")}</p> },
    { key: "script", title: t("plugins.script.stepScript"), body: <ScriptFile use={use} /> },
    {
      key: "token",
      title: t("imports.script.stepToken"),
      body: (
        <div className="min-w-0">
          <p className="m-0 text-small text-sub">{t("imports.script.tokenOnce")}</p>
          <div className="mt-1.5 flex items-center gap-1.5">
            <code className="mono min-w-0 flex-1 truncate rounded-sm bg-chip px-2.5 py-1.5 text-tag text-text select-all">{token.token}</code>
            <Button
              variant="outline"
              size="sm"
              square
              aria-label={t("plugins.script.copyToken")}
              onPress={() => {
                navigator.clipboard.writeText(token.token).then(
                  () => flash.show(t("common.copied")),
                  () => flash.show(t("common.copyFailed")),
                );
              }}
            >
              <Copy size={14} aria-hidden="true" />
            </Button>
          </div>
          <FlashLine flash={flash} className="mt-1 block text-small text-sub" />
        </div>
      ),
    },
    {
      key: "page",
      title: t("plugins.script.stepPage"),
      body: (
        <div className="flex flex-col gap-1">
          <ScriptPage use={use} />
          <p className="m-0 text-small text-sub">{t("imports.script.sendBody", { sendButton: t("plugins.quoted.sendButton") })}</p>
        </div>
      ),
    },
  ];
  return (
    <div className="mt-3 flex flex-col gap-3 border-t border-border pt-3">
      <ol aria-label={t("plugins.saved.steps", { name: use.method.name })} className="m-0 flex list-none flex-col gap-3.5 p-0">
        {steps.map((s, i) => (
          <li key={s.key} className="flex gap-2.5">
            <span className="mono flex size-5 shrink-0 items-center justify-center rounded-full border border-border2 text-tag text-label">{i + 1}</span>
            <div className="min-w-0 flex-1">
              <div className="text-note font-medium text-text">{s.title}</div>
              <div className="mt-1">{s.body}</div>
            </div>
          </li>
        ))}
      </ol>
      <div>
        <Button variant="outline" size="sm" onPress={onDone}>
          {t("common.done")}
        </Button>
      </div>
    </div>
  );
}
