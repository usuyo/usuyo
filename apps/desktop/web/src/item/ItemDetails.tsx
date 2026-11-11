import { FileText, Plus, RefreshCw, RotateCcw, Trash2 } from "lucide-react";
import { useState, type ReactNode } from "react";
import { Button as AriaButton, Dialog, DialogTrigger, Popover, ToggleButton } from "react-aria-components";
import type { Item, Topic } from "../api/endpoints";
import { useItemChange } from "../api/hooks";
import { openExternal } from "../shell";
import { TopicPill } from "../ui/Chip";
import { FlashLine, useFlash } from "../ui/Flash";
import { dayText, savedText } from "./format";

function Row({ label, top, children }: { label: string; top?: boolean; children: ReactNode }) {
  return (
    <div className={`flex gap-3 ${top ? "items-start" : "items-baseline"}`}>
      <span className={`w-[52px] shrink-0 text-small text-faint ${top ? "leading-[36px]" : "leading-5"}`}>{label}</span>
      <div className="min-w-0 flex-1 text-note text-label">{children}</div>
    </div>
  );
}

/** 条目页左下：来自、收下、存下了、主题。 */
export function ItemMeta({ item, topics }: { item: Item; topics: Topic[] }) {
  const src = useFlash();
  const topicFlash = useFlash();
  const change = useItemChange(item.id);
  const followed = new Set(topics.filter((t) => t.followed).map((t) => t.id));
  const names = new Map(topics.map((t) => [t.id, t.name]));
  return (
    <div className="flex min-w-0 flex-1 flex-col gap-2.5">
      <div className="flex min-h-7 items-center gap-3">
        <span className="w-[52px] shrink-0 text-small text-faint">来自</span>
        <span className="text-note text-label">
          {item.platform.name}
          {item.author ? ` · ${item.author}` : ""} ·
        </span>
        <AriaButton
          isDisabled={item.url === ""}
          onPress={() => {
            openExternal(item.url);
            src.show("在浏览器里打开了");
          }}
          className="-my-1 -ml-1.5 h-ctl-sm cursor-pointer rounded-sm border-0 bg-transparent px-1.5 font-sans text-note text-text underline decoration-border2 underline-offset-4 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
        >
          原帖链接
        </AriaButton>
        <FlashLine flash={src} className="text-small whitespace-nowrap text-sub" />
      </div>
      <Row label="收下">{dayText(item.takenAt)}</Row>
      <Row label="存下了">
        {savedText(item)}
        {item.missing ? <span className="text-warn">{item.missing}</span> : null}
      </Row>
      <Row label="主题" top>
        <div className="flex min-h-ctl-sm flex-wrap items-center gap-1.5">
          {item.topicIds.map((t) => (
            <TopicPill key={t} followed={followed.has(t)} size="md">
              {names.get(t)}
            </TopicPill>
          ))}
          <DialogTrigger>
            <AriaButton className="press flex h-ctl-sm cursor-pointer items-center gap-1.5 rounded-full border border-dashed border-border2 bg-transparent px-3 font-sans text-small font-medium text-label outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent">
              <Plus size={12} aria-hidden="true" />
              <span>放进主题</span>
            </AriaButton>
            <Popover placement="top start" offset={8} className="w-[300px] animate-rise rounded-card border border-border2 bg-card p-3 shadow-panel">
              <Dialog aria-label="放进哪个主题" className="outline-none">
                <div className="px-0.5 pb-2 text-tag text-sub">点一下放进去，再点一下拿出来</div>
                <div className="flex flex-wrap gap-1.5">
                  {topics.map((t) => {
                    const on = item.topicIds.includes(t.id);
                    const fo = followed.has(t.id);
                    return (
                      <ToggleButton
                        key={t.id}
                        isSelected={on}
                        onChange={(member) => {
                          change.mutate({ kind: "topic", topicId: t.id, member });
                          topicFlash.show(member ? `放进了「${t.name}」` : `从「${t.name}」拿出来了`);
                        }}
                        className={`press h-ctl-sm cursor-pointer rounded-full border px-3 font-sans text-small font-medium outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent ${on ? `bg-chip ${fo ? "border-accent text-accent" : "border-label text-text"}` : "border-border bg-transparent text-sub"}`}
                      >
                        {t.name}
                      </ToggleButton>
                    );
                  })}
                </div>
              </Dialog>
            </Popover>
          </DialogTrigger>
        </div>
        <FlashLine flash={topicFlash} className="mt-1 block min-h-[18px] text-small text-sub" />
      </Row>
    </div>
  );
}

function ActionCard({
  icon,
  label,
  sub,
  armed,
  off,
  onPress,
  onCancel,
  cancelText,
}: {
  icon: ReactNode;
  label: string;
  sub: string;
  armed?: boolean;
  off?: boolean;
  onPress?: () => void;
  onCancel?: () => void;
  cancelText?: string;
}) {
  return (
    <div className="relative">
      <AriaButton
        isDisabled={off}
        onPress={onPress}
        className={`press block min-h-[78px] w-full cursor-pointer rounded-input border px-3 py-2.5 text-left font-sans outline-none data-[disabled]:cursor-default data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent ${armed ? "border-text bg-chip" : off ? "border-border bg-transparent" : "border-border2 bg-transparent"}`}
      >
        {armed ? <span className="float-right h-[30px] w-[74px]" /> : null}
        <span className={`flex items-center gap-2 text-note font-medium ${off ? "text-sub" : "text-text"}`}>
          {icon}
          <span>{label}</span>
        </span>
        <span className={`mt-0.5 block pl-[22px] text-small ${off ? "text-faint" : "text-sub"}`}>{sub}</span>
      </AriaButton>
      {armed && onCancel ? (
        <AriaButton
          onPress={onCancel}
          className="press absolute top-1.5 right-1.5 h-ctl-sm animate-rise cursor-pointer rounded-[9px] border border-border2 bg-bg px-3 font-sans text-note text-label outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
        >
          {cancelText}
        </AriaButton>
      ) : null}
    </div>
  );
}

/** 条目页右下的三件事：只留文字、删掉、重新获取。动原样的两件要点两次。 */
export function ItemActions({ item }: { item: Item }) {
  const change = useItemChange(item.id);
  const [armed, setArmed] = useState<"text" | "del" | null>(null);
  const [msg, setMsg] = useState("");
  const saved = item.stored !== "link";
  return (
    <div className="flex w-[264px] shrink-0 flex-col gap-2">
      {item.stored === "text-only" ? (
        <ActionCard
          icon={<RotateCcw size={14} aria-hidden="true" />}
          label="恢复视频和封面"
          sub="从回收站拿回来，回到都存好了"
          onPress={() =>
            change.mutate({ kind: "restore-media" }, { onSuccess: () => setMsg("视频和封面从回收站拿回来了") })
          }
        />
      ) : (
        <ActionCard
          icon={<FileText size={14} aria-hidden="true" />}
          label={armed === "text" ? "再点一次，只留文字" : "只留文字"}
          sub={saved ? "视频和图片移进回收站，30 天内能恢复，文字留着" : "只有链接，还没有视频和图片可以移走"}
          off={!saved}
          armed={armed === "text"}
          cancelText="不改了"
          onCancel={() => setArmed(null)}
          onPress={() => {
            if (armed !== "text") return setArmed("text");
            setArmed(null);
            change.mutate({ kind: "text-only" }, { onSuccess: () => setMsg("视频和封面移进了回收站，口播文字和图里的字留着") });
          }}
        />
      )}
      <ActionCard
        icon={<Trash2 size={14} aria-hidden="true" />}
        label={armed === "del" ? "再点一次，删掉这条" : "删掉这条"}
        sub="整条移进回收站，30 天内能恢复"
        armed={armed === "del"}
        cancelText="不删了"
        onCancel={() => setArmed(null)}
        onPress={() => {
          if (armed !== "del") return setArmed("del");
          setArmed(null);
          setMsg("");
          change.mutate({ kind: "trash" });
        }}
      />
      {saved ? (
        <ActionCard
          icon={<RefreshCw size={14} aria-hidden="true" />}
          label={change.isPending && change.variables.kind === "refetch" ? "正在重新获取" : "重新获取"}
          sub={item.stored === "text-only" ? "从原帖再取视频和封面，原帖还在才行" : "从原帖再取一次，存着的不动"}
          off={change.isPending}
          onPress={() =>
            change.mutate(
              { kind: "refetch" },
              {
                onSuccess: (next) =>
                  setMsg(item.stored === "text-only" && next.stored === "complete" ? "从原帖重新取到了视频和封面" : `补了一次快照（${dayText(new Date().toISOString())}），原来存的都留着`),
              },
            )
          }
        />
      ) : null}
      <div role="status" className="min-h-[18px] px-1 pt-0.5 text-small text-label">
        {msg}
      </div>
    </div>
  );
}
