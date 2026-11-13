import type { SettingsView } from "../api/endpoints";
import { useChangeSettings } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { setLanguageChoice, t, type LanguageChoice } from "../i18n";
import { Choice } from "../ui/Choice";
import { Row } from "./Row";

/** 通用：界面用哪种语言。换了马上生效，存在这台电脑的设置里，内核给的话也跟着换。 */
export function General({ view }: { view: SettingsView }) {
  const change = useChangeSettings();
  const options: { id: LanguageChoice; text: string }[] = [
    { id: "system", text: t("settings.general.system") },
    { id: "zh-Hans", text: t("settings.general.zhHans") },
    { id: "en", text: t("settings.general.en") },
  ];
  return (
    <div className="flex flex-col gap-5">
      <Row label={t("settings.general.language")} sub={t("settings.general.languageSub")}>
        <Choice
          label={t("settings.general.language")}
          options={options}
          value={view.language}
          isDisabled={change.isPending}
          onChange={(language) => {
            setLanguageChoice(language);
            change.mutate({ language });
          }}
        />
      </Row>
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
