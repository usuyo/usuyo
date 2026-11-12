import { useQueryClient } from "@tanstack/react-query";
import { getRouteApi, Link } from "@tanstack/react-router";
import { ArrowLeft, Map as MapIcon } from "lucide-react";
import { useState, type ReactNode } from "react";
import { isNotServed } from "../api/client";
import type { SegmentRef, TopicPage as TopicView } from "../api/endpoints";
import { queryKeys, useDismissThread, useFollow, useTopicPage, useTopicPageChange, useTopics } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { Button } from "../ui/Button";
import { SectionLabel } from "../ui/Chip";
import { PageText } from "../writing/PageText";
import { Angles } from "./Angles";
import { Matching, Members } from "./Matching";
import { Threads } from "./Threads";

const route = getRouteApi("/topics/$id");

/**
 * 主题页：你自己写的那一页在左，机器挑的线头在旁边；下面是几个角度并排，再下面是成员和怎么算命中。
 * 你写的用实线，机器挑的、机器写的用虚线框着、标出来。
 */
export function TopicPage() {
  const { id } = route.useParams();
  const page = useTopicPage(id);
  const topics = useTopics();
  const followed = topics.data?.filter((t) => t.followed) ?? [];
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航">
        <Link
          to="/"
          className="press mt-[22px] flex h-ctl items-center gap-2.5 rounded-btn px-2.5 text-body text-label no-underline hover:bg-chip/60"
        >
          <MapIcon size={16} aria-hidden="true" />
          <span>地图</span>
        </Link>
        <SectionLabel className="mt-[22px]">关注的主题</SectionLabel>
        <div className="mt-1.5 flex flex-col gap-0.5">
          {followed.map((t) => (
            <Link
              key={t.id}
              to="/topics/$id"
              params={{ id: t.id }}
              className={`press flex h-ctl items-center gap-2.5 rounded-btn px-2.5 text-body no-underline hover:bg-chip ${t.id === id ? "bg-chip text-text" : "text-label"}`}
            >
              <span className="size-[7px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
              <span>{t.name}</span>
            </Link>
          ))}
        </div>
      </Rail>
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-5 pb-20 lg:px-gutter">
        <Link
          to="/"
          className="press -ml-2 inline-flex h-ctl-sm items-center gap-2 rounded-btn pr-3 pl-2 text-note text-sub no-underline hover:text-text"
        >
          <ArrowLeft size={16} aria-hidden="true" />
          <span>回到地图</span>
        </Link>
        {isNotServed(page.error) ? (
          <NotServedLine className="mt-4">这台电脑上的 usuyo 内核还没有主题页；主题的名字和关注在地图上照常用，你写的那一页以后在这里写。</NotServedLine>
        ) : page.error ? (
          <ErrorLine error={page.error} className="mt-4" />
        ) : null}
        {page.data ? <TopicBody key={page.data.id} page={page.data} /> : null}
      </main>
    </div>
  );
}

function Section({ id, title, sub, children }: { id: string; title: string; sub?: ReactNode; children: ReactNode }) {
  return (
    <section aria-labelledby={id} className="min-w-0">
      <h2 id={id} className="m-0 text-section font-semibold text-text">
        {title}
      </h2>
      {sub ? <p className="m-0 mt-0.5 text-note text-sub">{sub}</p> : null}
      <div className="mt-3.5">{children}</div>
    </section>
  );
}

function TopicBody({ page }: { page: TopicView }) {
  const change = useTopicPageChange(page.id);
  const dismiss = useDismissThread(page.id);
  const follow = useFollow();
  const queryClient = useQueryClient();
  const [running, setRunning] = useState<string | null>(null);
  const busy = change.isPending;
  const cite = (segment: SegmentRef) => change.mutate({ kind: "cite", body: { segment: { itemId: segment.itemId, layer: segment.layer, text: segment.text, ...(segment.position ? { position: segment.position } : {}) } } });
  const member = (itemId: string, on: boolean) => change.mutate({ kind: "member", itemId, member: on });

  return (
    <>
      <header className="mt-2.5 flex flex-wrap items-start justify-between gap-x-6 gap-y-3">
        <div className="min-w-0">
          <h1 className="m-0 text-title font-semibold text-text">{page.name}</h1>
          <p className="m-0 mt-1 text-note text-sub">
            这一页是普通 Markdown，存在 <span className="mono text-tag text-label select-text">{page.page.file}</span>
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          isDisabled={follow.isPending}
          aria-pressed={page.followed}
          onPress={() =>
            follow.mutate(
              { topicId: page.id, followed: !page.followed },
              { onSuccess: () => void queryClient.invalidateQueries({ queryKey: queryKeys.topicPage(page.id) }) },
            )
          }
          className={`rounded-full ${page.followed ? "border-accent text-accent" : ""}`}
        >
          {page.followed ? (
            <>
              <span className="size-[7px] rounded-full bg-accent" aria-hidden="true" />
              关注着，地图上这一片一直清楚
            </>
          ) : (
            "关注这个主题"
          )}
        </Button>
      </header>
      {change.error ? <ErrorLine error={change.error} className="mt-3" /> : null}

      <div className="mt-7 grid items-start gap-x-12 gap-y-10 @[880px]:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
        <Section id="topic-page-h" title="这一页" sub="你写的字，每段带着它引的原话；点原话回到那一秒、那张图。">
          <PageText
            page={page.page}
            busy={busy}
            empty="这一页还空着。从旁边机器挑的线头、下面的角度里引一段原话进来，或者先写一段。"
            onEdit={(patch) => change.mutate({ kind: "page", patch })}
          />
        </Section>
        <Section id="topic-threads-h" title="旁边的线头" sub="机器按相近拼的几组原话，不起名，也不替你总结。引进来的才是你的。">
          <Threads threads={page.threads} busy={busy || dismiss.isPending} onCite={cite} onDismiss={(t) => dismiss.mutate(t)} />
        </Section>
      </div>

      <div className="mt-12 border-t border-border pt-7">
        <Section id="topic-angles-h" title="几个角度" sub="同一些段，换个问题读一遍；不同的问题并排放，哪一个都不替换原话。">
          <Angles
            angles={page.angles}
            busy={busy}
            running={running}
            onCite={cite}
            onAdd={(question) => change.mutate({ kind: "angle", question })}
            onRun={(angleId) => {
              setRunning(angleId);
              change.mutate({ kind: "run", angleId }, { onSettled: () => setRunning(null) });
            }}
          />
        </Section>
      </div>

      <div className="mt-12 grid items-start gap-x-12 gap-y-10 border-t border-border pt-7 @[880px]:grid-cols-2">
        <Section id="topic-members-h" title="成员" sub="你放进来的。命中的不会自己变成成员。">
          <Members page={page} busy={busy} onMember={member} />
        </Section>
        <Section id="topic-match-h" title="怎么算命中" sub="先按关键词；相似度要你自己打开。">
          <Matching page={page} busy={busy} onMatch={(match) => change.mutate({ kind: "match", match })} onMember={member} />
        </Section>
      </div>
    </>
  );
}
