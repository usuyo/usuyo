import type { SettingsView } from "../api/endpoints";
import { useChangeSettings } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { HeldNotice } from "../trash/HeldNotice";
import { Choice } from "../ui/Choice";
import { t } from "../i18n";
import { Row } from "./Row";
import { TRASH_DAYS, trashChoiceOf, trashChoices, trashText } from "./words";

/** 存储与废纸篓：资料库在哪、废纸篓保留多久；时钟跳了被拦下时，只给「按时间删除」这一个动作。 */
export function Storage({ view }: { view: SettingsView }) {
  const change = useChangeSettings();
  return (
    <div className="flex flex-col gap-5">
      <Row label={t("settings.storage.library")} sub={t("settings.storage.librarySub")}>
        <code className="mono block truncate rounded-sm bg-chip px-3 py-2 text-tag text-text select-text" title={view.library}>
          {view.library}
        </code>
      </Row>
      <Row label={t("settings.storage.keep")} sub={trashText(view.trashDays)}>
        <Choice
          label={t("settings.storage.keepLabel")}
          options={trashChoices()}
          value={trashChoiceOf(view.trashDays)}
          isDisabled={change.isPending}
          onChange={(id) => change.mutate({ trashDays: TRASH_DAYS.find((c) => c.id === id)?.days ?? null })}
        />
      </Row>
      <HeldNotice view={view} />
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
