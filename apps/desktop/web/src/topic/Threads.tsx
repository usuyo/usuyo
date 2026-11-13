import { Quote } from "lucide-react";
import type { Thread } from "../api/endpoints";
import { Button } from "../ui/Button";
import { MachineTag } from "../ui/Chip";
import { SegmentSlip } from "../writing/Quote";
import { t } from "../i18n";

/**
 * 机器挑的线头：几段相近的原话摆在一起，不起名、不写总结。每段能引用进这一页；不要的整组拿掉，不留记录。
 */
export function Threads({
  threads,
  busy,
  onCite,
  onDismiss,
}: {
  threads: Thread[];
  busy: boolean;
  onCite: (segment: Thread["segments"][number]) => void;
  onDismiss: (threadId: string) => void;
}) {
  if (threads.length === 0) return <p className="m-0 text-note text-sub">{t("topic.threads.empty")}</p>;
  return (
    <div className="flex flex-col gap-3">
      {threads.map((thread) => (
        <section key={thread.id} aria-label={t("topic.threads.landmark", { count: thread.segments.length })} className="rounded-card border border-dashed border-border2 px-3.5 pt-3 pb-2.5">
          <div className="flex items-center gap-2">
            <MachineTag>{t("common.tag.autoSelected")}</MachineTag>
            <span className="text-small text-sub">{t("topic.threads.similar", { count: thread.segments.length })}</span>
          </div>
          <div className="mt-2.5 flex flex-col gap-1.5">
            {thread.segments.map((s) => (
              <SegmentSlip
                key={`${s.itemId}:${s.layer}:${s.text}`}
                segment={s}
                action={
                  <Button variant="ghost" size="xs" isDisabled={busy} aria-label={t("topic.angles.cite", { text: s.text })} onPress={() => onCite(s)}>
                    <Quote size={12} aria-hidden="true" />
                    {t("topic.threads.quote")}
                  </Button>
                }
              />
            ))}
          </div>
          <div className="mt-2 flex justify-end">
            <Button variant="quiet" size="xs" isDisabled={busy} onPress={() => onDismiss(thread.id)}>
              {t("topic.threads.dismiss")}
            </Button>
          </div>
        </section>
      ))}
    </div>
  );
}
