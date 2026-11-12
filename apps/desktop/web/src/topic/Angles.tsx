import { Quote, RotateCw } from "lucide-react";
import { useState } from "react";
import { Input, Label, TextField } from "react-aria-components";
import type { Angle, SegmentRef } from "../api/endpoints";
import { Button } from "../ui/Button";
import { Choice } from "../ui/Choice";
import { MachineTag } from "../ui/Chip";
import { SegmentSlip } from "../writing/Quote";
import { runText } from "./words";

/**
 * 几个角度并排：同一些段，换一个问题让模型读一遍。模型写的标「机器写的」，它引的每段原话另外核对过；
 * 每读一次留一份，能换着看，也能再读一次。
 */
export function Angles({
  angles,
  busy,
  running,
  onCite,
  onRun,
  onAdd,
}: {
  angles: Angle[];
  busy: boolean;
  /** 正在读的那个角度。 */
  running: string | null;
  onCite: (segment: SegmentRef) => void;
  onRun: (angleId: string) => void;
  onAdd: (question: string) => void;
}) {
  const [question, setQuestion] = useState("");
  const add = () => {
    if (question.trim() === "") return;
    onAdd(question);
    setQuestion("");
  };
  return (
    <div className="flex flex-col gap-4">
      {angles.length > 0 ? (
        <div className="grid grid-cols-[repeat(auto-fit,minmax(min(100%,300px),1fr))] items-start gap-4">
          {angles.map((a) => (
            <AngleColumn key={a.id} angle={a} busy={busy} running={running === a.id} onCite={onCite} onRun={() => onRun(a.id)} />
          ))}
        </div>
      ) : (
        <p className="m-0 text-note text-sub">还没有角度。写一个问题，让模型照它把这一主题的段读一遍；读出来的标着，引用都指回原话。</p>
      )}
      <TextField value={question} onChange={setQuestion} className="flex max-w-[560px] items-center gap-2" aria-label="加一个角度">
        <Label className="sr-only-text">加一个角度</Label>
        <Input
          placeholder="换个问题读这些段，例如：哪些说法互相对不上？"
          onKeyDown={(e) => {
            if (e.key === "Enter") add();
          }}
          className="h-ctl-sm min-w-0 flex-1 rounded-input border border-border2 bg-transparent px-3 font-sans text-note text-text outline-none focus:border-label"
        />
        <Button variant="outline" size="sm" isDisabled={busy || question.trim() === ""} onPress={add}>
          加上这个角度
        </Button>
      </TextField>
    </div>
  );
}

function AngleColumn({ angle, busy, running, onCite, onRun }: { angle: Angle; busy: boolean; running: boolean; onCite: (s: SegmentRef) => void; onRun: () => void }) {
  const [shown, setShown] = useState<string | undefined>(undefined);
  const run = angle.runs.find((r) => r.id === shown) ?? angle.runs[0];
  return (
    <section aria-label={`角度：${angle.question}`} className="flex min-w-0 flex-col rounded-card border border-dashed border-border2 px-4 pt-3.5 pb-3">
      <h3 className="m-0 text-section font-semibold text-text">{angle.question}</h3>
      {angle.runs.length > 1 ? (
        <div className="mt-2">
          <Choice label="看哪一次读的" options={angle.runs.map((r) => ({ id: r.id, text: runText(r, false) }))} value={run?.id} onChange={setShown} />
        </div>
      ) : null}
      {run ? (
        <>
          <div className="mt-2.5 flex flex-wrap items-center gap-2">
            <MachineTag>机器写的</MachineTag>
            <span className="text-small text-sub">{runText(run, true)}</span>
          </div>
          <ol className="m-0 mt-2 flex list-none flex-col gap-3 p-0">
            {run.points.map((p) => (
              <li key={p.text} className="flex flex-col gap-1.5">
                <p className="m-0 text-body text-label">{p.text}</p>
                {p.cites.map((c) => (
                  <SegmentSlip
                    key={`${c.segment.itemId}:${c.segment.text}`}
                    segment={c.segment}
                    note={c.checked ? undefined : "没核对上：原文里找不到模型引的这句"}
                    action={
                      <Button variant="ghost" size="xs" square isDisabled={busy} aria-label={`引用「${c.segment.text}」到这一页`} onPress={() => onCite(c.segment)}>
                        <Quote size={12} aria-hidden="true" />
                      </Button>
                    }
                  />
                ))}
              </li>
            ))}
          </ol>
        </>
      ) : (
        <p className="m-0 mt-2 text-note text-sub">还没读过。</p>
      )}
      <div className="mt-3 flex items-center gap-2 border-t border-border pt-2.5">
        <Button variant="ghost" size="xs" isDisabled={busy || running} onPress={onRun}>
          <RotateCw size={12} aria-hidden="true" />
          {running ? "在读" : "再读一次"}
        </Button>
        <span className="text-small text-faint">新读的和旧的都留着</span>
      </div>
    </section>
  );
}
