import { Link } from "@tanstack/react-router";
import { FileText, Plus, RefreshCw, RotateCcw, Trash2 } from "lucide-react";
import { useState, type ReactNode } from "react";
import { Button as AriaButton, Dialog, DialogTrigger, Input, Popover, TextField, ToggleButton } from "react-aria-components";
import type { Item, Topic } from "../api/endpoints";
import { useItemChange } from "../api/hooks";
import { openExternal, webUrl } from "../shell";
import { TopicPill } from "../ui/Chip";
import { FlashLine, useFlash } from "../ui/Flash";
import { t } from "../i18n";
import { dayText, keepText, savedText, sentences, type MediaKeep } from "./format";

function Row({ label, top, children }: { label: string; top?: boolean; children: ReactNode }) {
  return (
    <div className={`flex gap-3 ${top ? "items-start" : "items-baseline"}`}>
      <span className={`w-[52px] shrink-0 text-small text-faint ${top ? "leading-[36px]" : "leading-5"}`}>{label}</span>
      <div className="min-w-0 flex-1 text-note text-label">{children}</div>
    </div>
  );
}

/** 条目页左下：来自、收下、存下了、主题。 */
export function ItemMeta({ item, topics, keep }: { item: Item; topics: Topic[]; keep: MediaKeep }) {
  const src = useFlash();
  const topicFlash = useFlash();
  const change = useItemChange(item.id);
  const followed = new Set(topics.filter((x) => x.followed).map((x) => x.id));
  const names = new Map(topics.map((x) => [x.id, x.name]));
  return (
    <div className="flex min-w-0 flex-[1_1_280px] flex-col gap-2.5">
      <div className="flex min-h-7 items-center gap-3">
        <span className="w-[52px] shrink-0 text-small text-faint">{t("item.meta.source")}</span>
        <span className="text-note text-label">
          {item.platform.name}
          {item.author ? ` · ${item.author}` : ""} ·
        </span>
        <AriaButton
          isDisabled={webUrl(item.url) === null}
          onPress={() => {
            void openExternal(item.url).then((opened) => {
              if (opened) src.show(t("item.page.opened"));
            });
          }}
          className="-my-1 -ml-1.5 h-ctl-sm cursor-pointer rounded-sm border-0 bg-transparent px-1.5 font-sans text-note text-text underline decoration-border2 underline-offset-4 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
        >
          {t("item.meta.originalLink")}
        </AriaButton>
        <FlashLine flash={src} className="text-small whitespace-nowrap text-sub" />
      </div>
      <Row label={t("item.meta.added")}>{dayText(item.takenAt)}</Row>
      <Row label={t("item.meta.saved")}>
        {savedText(item, keep)}
        {item.missing ? <span className="text-warn">{item.missing}</span> : null}
      </Row>
      <Row label={t("item.meta.topics")} top>
        <div className="flex min-h-ctl-sm flex-wrap items-center gap-1.5">
          {item.topicIds.map((id) => (
            <Link key={id} to="/topics/$id" params={{ id }} aria-label={t("item.meta.openTopic", { name: names.get(id) ?? id })} className="press rounded-full no-underline">
              <TopicPill followed={followed.has(id)} size="md">
                {names.get(id)}
              </TopicPill>
            </Link>
          ))}
          <DialogTrigger>
            <AriaButton className="press flex h-ctl-sm cursor-pointer items-center gap-1.5 rounded-full border border-dashed border-border2 bg-transparent px-3 font-sans text-small font-medium text-label outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent">
              <Plus size={12} aria-hidden="true" />
              <span>{t("item.meta.addToTopic")}</span>
            </AriaButton>
            <Popover placement="top start" offset={8} className="w-[300px] animate-rise rounded-card border border-border2 bg-card p-3 shadow-panel">
              <Dialog aria-label={t("item.meta.chooseTopic")} className="outline-none">
                <div className="px-0.5 pb-2 text-tag text-sub">{t("item.meta.chooseHint")}</div>
                <div className="flex flex-wrap gap-1.5">
                  {topics.map((topic) => {
                    const on = item.topicIds.includes(topic.id);
                    const fo = followed.has(topic.id);
                    return (
                      <ToggleButton
                        key={topic.id}
                        isSelected={on}
                        onChange={(member) => {
                          change.mutate({ kind: "topic", topicId: topic.id, member });
                          topicFlash.show(member ? t("item.meta.addedTo", { name: topic.name }) : t("item.meta.removedFrom", { name: topic.name }));
                        }}
                        className={`press h-ctl-sm cursor-pointer rounded-full border px-3 font-sans text-small font-medium outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent ${on ? `bg-chip ${fo ? "border-accent text-accent" : "border-label text-text"}` : "border-border bg-transparent text-sub"}`}
                      >
                        {topic.name}
                      </ToggleButton>
                    );
                  })}
                </div>
                <NewTopicField
                  onSubmit={(name) => {
                    const existing = topics.find((x) => x.name === name);
                    if (existing) {
                      if (!item.topicIds.includes(existing.id)) change.mutate({ kind: "topic", topicId: existing.id, member: true });
                      topicFlash.show(t("item.meta.addedTo", { name: existing.name }));
                      return;
                    }
                    change.mutate({ kind: "new-topic", name }, { onSuccess: () => topicFlash.show(t("item.meta.createdAndAdded", { name })) });
                  }}
                />
              </Dialog>
            </Popover>
          </DialogTrigger>
        </div>
        <FlashLine flash={topicFlash} className="mt-1 block min-h-[18px] text-small text-sub" />
      </Row>
    </div>
  );
}

/** 放进主题浮层底下：写个名字回车，没有这个主题就建一个再放进去。 */
function NewTopicField({ onSubmit }: { onSubmit: (name: string) => void }) {
  const [name, setName] = useState("");
  return (
    <TextField value={name} onChange={setName} aria-label={t("item.meta.newTopicLabel")} className="mt-2.5">
      <Input
        placeholder={t("item.meta.newTopicPlaceholder")}
        maxLength={80}
        className="h-ctl-sm w-full rounded-input border border-border2 bg-transparent px-3 font-sans text-small text-text outline-none data-[focused]:border-label"
        onKeyDown={(e) => {
          if (e.key !== "Enter") return;
          const trimmed = name.trim();
          if (trimmed === "") return;
          onSubmit(trimmed);
          setName("");
        }}
      />
    </TextField>
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
export function ItemActions({ item, keep }: { item: Item; keep: MediaKeep }) {
  const change = useItemChange(item.id);
  const [armed, setArmed] = useState<"text" | "del" | null>(null);
  const [msg, setMsg] = useState("");
  const saved = item.stored !== "link";
  return (
    <div className="flex min-w-0 flex-[1_1_240px] flex-col gap-2 @[840px]:max-w-[264px]">
      {item.stored === "text-only" && keep.gone ? null : item.stored === "text-only" ? (
        <ActionCard
          icon={<RotateCcw size={14} aria-hidden="true" />}
          label={t("item.actions.restoreMedia")}
          sub={t("item.actions.restoreMediaSub")}
          onPress={() => change.mutate({ kind: "restore-media" }, { onSuccess: () => setMsg(t("item.actions.restoredMedia")) })}
        />
      ) : (
        <ActionCard
          icon={<FileText size={14} aria-hidden="true" />}
          label={armed === "text" ? t("item.actions.keepTextArmed") : t("item.actions.keepText")}
          sub={saved ? sentences(t("item.actions.keepTextSub"), keepText(keep.days)) : t("item.actions.keepTextLinkOnly")}
          off={!saved}
          armed={armed === "text"}
          cancelText={t("common.cancel")}
          onCancel={() => setArmed(null)}
          onPress={() => {
            if (armed !== "text") return setArmed("text");
            setArmed(null);
            change.mutate({ kind: "text-only" }, { onSuccess: () => setMsg(t("item.actions.keptText")) });
          }}
        />
      )}
      <ActionCard
        icon={<Trash2 size={14} aria-hidden="true" />}
        label={armed === "del" ? t("item.actions.deleteArmed") : t("item.actions.delete")}
        sub={sentences(t("item.actions.deleteSub"), keepText(keep.days))}
        armed={armed === "del"}
        cancelText={t("common.cancel")}
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
          label={change.isPending && change.variables.kind === "refetch" ? t("item.actions.refetching") : t("item.actions.refetch")}
          sub={
            item.stored === "text-only"
              ? keep.gone
                ? t("item.actions.refetchSubTextOnlyGone")
                : t("item.actions.refetchSubTextOnly")
              : t("item.actions.refetchSub")
          }
          off={change.isPending}
          onPress={() =>
            change.mutate(
              { kind: "refetch" },
              {
                // 内核只是排上了；取到以后事件流会让这一条自己刷新。
                onSuccess: () =>
                  setMsg(item.stored === "text-only" ? t("item.actions.refetchQueuedTextOnly") : t("item.actions.refetchQueued")),
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
