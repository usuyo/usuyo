import type { SettingsView } from "../api/endpoints";
import { useChangeSettings } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { HeldNotice } from "../trash/HeldNotice";
import { Choice } from "../ui/Choice";
import { Row } from "./Row";
import { TRASH_CHOICES, trashChoiceOf, trashText } from "./words";

/** 存储与回收站：资料库在哪、回收站留多久；时钟跳了被拦下时，只给「照时间清掉」这一个动作。 */
export function Storage({ view }: { view: SettingsView }) {
  const change = useChangeSettings();
  return (
    <div className="flex flex-col gap-5">
      <Row label="资料库" sub="原样、转出来的文字和你写的都在这里。">
        <code className="mono block truncate rounded-sm bg-chip px-3 py-2 text-tag text-text select-text" title={view.library}>
          {view.library}
        </code>
      </Row>
      <Row label="回收站留" sub={trashText(view.trashDays)}>
        <Choice
          label="回收站留多久"
          options={TRASH_CHOICES}
          value={trashChoiceOf(view.trashDays)}
          isDisabled={change.isPending}
          onChange={(id) => change.mutate({ trashDays: TRASH_CHOICES.find((c) => c.id === id)?.days ?? null })}
        />
      </Row>
      <HeldNotice view={view} />
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
