import { Quote } from "lucide-react";
import type { Thread } from "../api/endpoints";
import { Button } from "../ui/Button";
import { MachineTag } from "../ui/Chip";
import { SegmentSlip } from "../writing/Quote";

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
  if (threads.length === 0) return <p className="m-0 text-note text-sub">现在没有机器挑出来的线头。</p>;
  return (
    <div className="flex flex-col gap-3">
      {threads.map((t) => (
        <section key={t.id} aria-label={`机器挑的 ${t.segments.length} 段`} className="rounded-card border border-dashed border-border2 px-3.5 pt-3 pb-2.5">
          <div className="flex items-center gap-2">
            <MachineTag>机器挑的</MachineTag>
            <span className="text-small text-sub">{t.segments.length} 段相近的原话</span>
          </div>
          <div className="mt-2.5 flex flex-col gap-1.5">
            {t.segments.map((s) => (
              <SegmentSlip
                key={`${s.itemId}:${s.layer}:${s.text}`}
                segment={s}
                action={
                  <Button variant="ghost" size="xs" isDisabled={busy} aria-label={`引用「${s.text}」到这一页`} onPress={() => onCite(s)}>
                    <Quote size={12} aria-hidden="true" />
                    引用
                  </Button>
                }
              />
            ))}
          </div>
          <div className="mt-2 flex justify-end">
            <Button variant="quiet" size="xs" isDisabled={busy} onPress={() => onDismiss(t.id)}>
              不要这一组
            </Button>
          </div>
        </section>
      ))}
    </div>
  );
}
