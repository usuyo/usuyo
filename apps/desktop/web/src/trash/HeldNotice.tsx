import { useState } from "react";
import type { SettingsView } from "../api/endpoints";
import { useReleaseHeld } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { heldText } from "../settings/words";
import { Button } from "../ui/Button";

/** 时钟跳了、到期检查被拦下时那一块：说清楚为什么没清，只给「照时间清掉」一个动作。回收站页和设置页都放它。 */
export function HeldNotice({ view }: { view: SettingsView }) {
  const release = useReleaseHeld();
  const [asking, setAsking] = useState(false);
  if (!view.trashHeld) {
    return release.data ? (
      <p role="status" className="m-0 text-small text-sub">
        照时间清掉了 {release.data.removed} 项。
      </p>
    ) : null;
  }
  return (
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
      {release.error ? <ErrorLine error={release.error} className="mt-2" /> : null}
    </div>
  );
}
