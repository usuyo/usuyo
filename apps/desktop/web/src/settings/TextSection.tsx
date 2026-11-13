import { Link } from "@tanstack/react-router";
import { Switch } from "react-aria-components";
import type { SettingsView } from "../api/endpoints";
import { useChangeSettings, useTransforms } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { sizeText } from "../item/faces";
import { rich, t } from "../i18n";
import { modelStateText } from "../plugins/TransformsPanel";
import { Row } from "./Row";

/** 转写与识别：添加以后自动处理（只在接通电源时），开或关；模型一览，下载和删除在获取插件页。 */
export function TextSection({ view }: { view: SettingsView }) {
  const change = useChangeSettings();
  const transforms = useTransforms();
  const auto = view.transforms.auto;
  const models = transforms.data?.plugins.flatMap((p) => p.models.map((m) => ({ ...m, plugin: p.name }))) ?? [];
  return (
    <div className="flex flex-col gap-5">
      <Row
        label={t("settings.text.auto")}
        sub={auto ? t("settings.text.autoOnSub") : t("settings.text.autoOffSub")}
      >
        <Switch
          isSelected={auto}
          isDisabled={change.isPending}
          onChange={(on) => change.mutate({ transforms: { auto: on } })}
          className="group flex h-ctl-sm w-fit cursor-pointer items-center gap-3 font-sans text-note text-label outline-none"
        >
          <span className="flex h-6 w-10 items-center rounded-full border border-border2 bg-chip px-0.5 transition-colors duration-150 group-data-[focus-visible]:outline-2 group-data-[focus-visible]:outline-accent group-data-[selected]:border-label group-data-[selected]:bg-label">
            <span className="size-[18px] rounded-full bg-text transition-transform duration-150 ease-out group-data-[selected]:translate-x-4 group-data-[selected]:bg-bg motion-reduce:transition-none" />
          </span>
          <span>{auto ? t("settings.text.autoOn") : t("settings.text.autoOff")}</span>
        </Switch>
      </Row>
      <Row
        label={t("settings.text.models")}
        sub={rich("settings.text.modelsSub", {
          link: <Link to="/plugins/text" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text" />,
        })}
      >
        {models.length > 0 ? (
          <ul className="m-0 flex list-none flex-col gap-1 p-0">
            {models.map((m) => (
              <li key={m.name} className="flex flex-wrap items-baseline gap-x-2.5 py-1 text-note">
                <span className="break-all text-text">{m.name}</span>
                <span className="mono text-tag text-faint">{sizeText(m.size)}</span>
                <span className="text-small text-sub">{t("settings.text.modelLine", { plugin: m.plugin, state: modelStateText(m) })}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="m-0 pt-2 text-note text-sub">{transforms.data ? t("settings.text.noModels") : ""}</p>
        )}
      </Row>
      {transforms.error ? <ErrorLine error={transforms.error} /> : null}
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
