import { useState } from "react";
import type { SettingsView } from "../api/endpoints";
import { useChangeSettings, useReleaseHeld } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { Choice } from "../ui/Choice";
import { Row } from "./Row";
import { heldText, TRASH_CHOICES, trashChoiceOf, trashText } from "./words";

/** 存储与回收站：资料库在哪、回收站留多久；时钟跳了被拦下时，只给「照时间清掉」这一个动作。 */
export function Storage({ view }: { view: SettingsView }) {
  const change = useChangeSettings();
  const release = useReleaseHeld();
  const [asking, setAsking] = useState(false);
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
      {view.trashHeld ? (
        <div role="status" className="rounded-card border border-border2 bg-card px-4 py-3.5">
          <p className="m-0 text-note text-label">{heldText(view.trashHeld, view.trashDays)}</p>
          <div className="mt-3 flex flex-wrap items-center gap-2">
            {asking ? (
              <>
                <Button
                  variant="strong"
                  size="sm"
                  isDisabled={release.isPending}
                  onPress={() => {
                    setAsking(false);
                    release.mutate();
                  }}
                >
                  清掉这 {view.trashHeld.count} 项，找不回来
                </Button>
                <Button variant="quiet" size="sm" onPress={() => setAsking(false)}>
                  先留着
                </Button>
              </>
            ) : (
              <Button variant="outline" size="sm" onPress={() => setAsking(true)}>
                照时间清掉
              </Button>
            )}
          </div>
        </div>
      ) : release.data ? (
        <p role="status" className="m-0 text-small text-sub">
          照时间清掉了 {release.data.removed} 项。
        </p>
      ) : null}
      {change.error ? <ErrorLine error={change.error} /> : null}
      {release.error ? <ErrorLine error={release.error} /> : null}
    </div>
  );
}
