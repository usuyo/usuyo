import { Link } from "@tanstack/react-router";
import { Plus, X } from "lucide-react";
import { useState, type ReactNode } from "react";
import { Input, Label, TextField } from "react-aria-components";
import type { ItemSummary, TopicPage } from "../api/endpoints";
import { Button } from "../ui/Button";
import { MachineTag, PlatformChip } from "../ui/Chip";
import { Toggle } from "../ui/Toggle";
import { hitText } from "./words";
import { t } from "../i18n";

/** 一条成员或命中的：平台、标题、作者，点进条目页。 */
function ItemLine({ item, sub, action }: { item: ItemSummary; sub?: string; action: ReactNode }) {
  return (
    <li className="flex items-center gap-2 border-b border-border py-1 last:border-b-0">
      <Link to="/items/$id" params={{ id: item.id }} className="press -mx-2 flex min-w-0 flex-1 items-center gap-2.5 rounded-btn px-2 py-1.5 no-underline hover:bg-chip/60">
        <PlatformChip>{item.platform.name}</PlatformChip>
        <span className="min-w-0">
          <span className="block truncate text-body text-text">{item.title}</span>
          {sub ? <span className="block truncate text-small text-sub">{sub}</span> : item.author ? <span className="block truncate text-small text-sub">{item.author}</span> : null}
        </span>
      </Link>
      {action}
    </li>
  );
}

/** 成员：你放进来的条目。拿出去只是不在这个主题里，条目照旧存着。 */
export function Members({ page, busy, onMember }: { page: TopicPage; busy: boolean; onMember: (itemId: string, member: boolean) => void }) {
  if (page.members.length === 0) return <p className="m-0 text-note text-sub">{t("topic.matching.noMembers")}</p>;
  return (
    <ul className="m-0 flex list-none flex-col p-0">
      {page.members.map((m) => (
        <ItemLine
          key={m.id}
          item={m}
          action={
            <Button variant="ghost" size="xs" square isDisabled={busy} aria-label={t("topic.matching.removeMember", { title: m.title })} onPress={() => onMember(m.id, false)}>
              <X size={13} aria-hidden="true" />
            </Button>
          }
        />
      ))}
    </ul>
  );
}

/**
 * 怎么算命中：关键词照字面看标题和内容里的字，相似度要你自己打开。命中的标「机器挑的」，不自动放进来。
 * 只有关注着的主题参与命中，跟着收也照这些词收。
 */
export function Matching({
  page,
  busy,
  onMatch,
  onMember,
}: {
  page: TopicPage;
  busy: boolean;
  onMatch: (match: TopicPage["match"]) => void;
  onMember: (itemId: string, member: boolean) => void;
}) {
  const [word, setWord] = useState("");
  const add = () => {
    const w = word.trim();
    if (w === "" || page.match.keywords.includes(w)) return;
    onMatch({ ...page.match, keywords: [...page.match.keywords, w] });
    setWord("");
  };
  return (
    <div className="flex flex-col gap-4">
      <div>
        <div className="flex flex-wrap items-center gap-1.5">
          {page.match.keywords.map((k) => (
            <span key={k} className="flex h-ctl-xs items-center gap-1 rounded-full border border-border2 pr-1 pl-3 text-note text-text">
              {k}
              <Button
                variant="ghost"
                size="none"
                isDisabled={busy}
                aria-label={t("topic.matching.removeKeyword", { word: k })}
                className="size-5 rounded-full"
                onPress={() => onMatch({ ...page.match, keywords: page.match.keywords.filter((x) => x !== k) })}
              >
                <X size={11} aria-hidden="true" />
              </Button>
            </span>
          ))}
          <TextField value={word} onChange={setWord} aria-label={t("topic.matching.addKeyword")} className="flex items-center">
            <Label className="sr-only-text">{t("topic.matching.addKeyword")}</Label>
            <Input
              placeholder={t("topic.matching.addKeyword")}
              onKeyDown={(e) => {
                if (e.key === "Enter") add();
              }}
              className="h-ctl-xs w-32 rounded-full border border-dashed border-border2 bg-transparent px-3 font-sans text-note text-text outline-none focus:border-label"
            />
          </TextField>
          {word.trim() !== "" ? (
            <Button variant="ghost" size="xs" square aria-label={t("topic.matching.add")} isDisabled={busy} onPress={add}>
              <Plus size={13} aria-hidden="true" />
            </Button>
          ) : null}
        </div>
        <p className="m-0 mt-2 text-small text-sub">
          {page.followed ? t("topic.matching.followedHint") : t("topic.matching.unfollowedHint")}
        </p>
      </div>
      <div>
        <Toggle isSelected={page.match.similarity} isDisabled={busy} onChange={(on) => onMatch({ ...page.match, similarity: on })} label={t("topic.matching.similarity")}>
          {page.match.similarity ? t("topic.matching.similarityOn") : t("topic.matching.similarityOff")}
        </Toggle>
        <p className="m-0 mt-1 text-small text-sub">{t("topic.matching.similaritySub")}</p>
      </div>
      {page.hits.length > 0 ? (
        <div>
          <div className="flex items-center gap-2">
            <MachineTag>{t("common.tag.autoSelected")}</MachineTag>
            <span className="text-small text-sub">{t("topic.matching.unadded")}</span>
          </div>
          <ul className="m-0 mt-1.5 flex list-none flex-col p-0">
            {page.hits.map((h) => (
              <ItemLine
                key={h.item.id}
                item={h.item}
                sub={hitText(h.words)}
                action={
                  <Button variant="outline" size="xs" isDisabled={busy} onPress={() => onMember(h.item.id, true)}>
                    {t("topic.matching.add")}
                  </Button>
                }
              />
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
