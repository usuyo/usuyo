import type { SettingsView } from "../api/endpoints";
import { useChangeSettings } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Toggle } from "../ui/Toggle";
import { t } from "../i18n";
import { Row } from "./Row";

/**
 * 窗口：用不用系统的标题栏。存在这台电脑自己的设置里，桌面壳开窗口时读；Tauri 换不干净开着的窗口的标题栏，所以下次打开才生效。
 */
export function WindowSection({ view }: { view: SettingsView }) {
  const change = useChangeSettings();
  const on = view.window.systemTitleBar;
  return (
    <div className="flex flex-col gap-5">
      <Row label={t("settings.window.titleBar")} sub={t("settings.window.titleBarSub")}>
        <Toggle isSelected={on} isDisabled={change.isPending} onChange={(v) => change.mutate({ window: { systemTitleBar: v } })} label={t("settings.window.titleBar")}>
          {on ? t("settings.window.on") : t("settings.window.off")}
        </Toggle>
      </Row>
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
