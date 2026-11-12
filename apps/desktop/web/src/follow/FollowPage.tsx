import { Link } from "@tanstack/react-router";
import { Plus } from "lucide-react";
import { useState } from "react";
import { isNotServed } from "../api/client";
import type { FollowsView, Topic } from "../api/endpoints";
import { useFollowChange, useFollows, useTopics } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { timeText } from "../plugins/words";
import { Button } from "../ui/Button";
import { MachineTag, PlatformChip } from "../ui/Chip";
import { Toggle } from "../ui/Toggle";
import { FollowRow } from "./FollowRow";
import { NewFollow } from "./NewFollow";

/** 跟着收：左边是跟着的那几个和新加一个，右边是最近收下的。没有计数，也不催。 */
export function FollowPage() {
  const follows = useFollows();
  const topics = useTopics();
  const missing = isNotServed(follows.error);
  const [adding, setAdding] = useState(false);
  const topicName = (id: string) => topics.data?.find((t) => t.id === id)?.name ?? id;
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航" />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-7 pb-16 lg:px-gutter">
        <header className="flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0 max-w-[640px]">
            <h1 className="m-0 text-title font-semibold text-text">跟着收</h1>
            <p className="m-0 mt-1 text-note text-sub">你在平台上的收藏夹、想跟着的创作者、订阅源，有新出现的就收下来。只收新的；已经有的，用「导入」一次收下。</p>
          </div>
          {follows.data && !adding ? (
            <Button variant="soft" size="md" onPress={() => setAdding(true)}>
              <Plus size={15} aria-hidden="true" />
              跟着收一个新的
            </Button>
          ) : null}
        </header>
        {missing ? <NotServedLine className="mt-6">这台电脑上的 usuyo 内核还不会跟着收；现在用「导入」一次收下存过的，以后新出现的在这里跟着收。</NotServedLine> : null}
        {follows.error && !missing ? <ErrorLine error={follows.error} className="mt-6" /> : null}
        {follows.data ? <FollowBody view={follows.data} adding={adding} topicName={topicName} topics={topics.data ?? []} onClose={() => setAdding(false)} /> : null}
      </main>
    </div>
  );
}

function FollowBody({
  view,
  adding,
  topicName,
  topics,
  onClose,
}: {
  view: FollowsView;
  adding: boolean;
  topicName: (id: string) => string;
  topics: Topic[];
  onClose: () => void;
}) {
  const change = useFollowChange();
  const titleOf = (id: string) => view.follows.find((f) => f.id === id)?.title;
  return (
    <div className="mt-6 grid items-start gap-x-12 gap-y-10 @[900px]:grid-cols-[minmax(0,1fr)_minmax(0,340px)]">
      <div className="flex min-w-0 flex-col gap-5">
        {adding ? <NewFollow options={view.options} topics={topics} onClose={onClose} /> : null}
        <section aria-labelledby="follows-h">
          <h2 id="follows-h" className="sr-only-text">
            跟着的
          </h2>
          {view.follows.length === 0 && !adding ? <p className="m-0 text-body text-sub">还没有跟着收的。点「跟着收一个新的」，先试列一次看会收下哪些。</p> : null}
          <ul className="m-0 flex list-none flex-col border-t border-border p-0">
            {view.follows.map((f) => (
              <FollowRow key={f.id} follow={f} topicName={topicName} />
            ))}
          </ul>
        </section>
        <div>
          <Toggle isSelected={view.powerOnly} isDisabled={change.isPending} onChange={(on) => change.mutate({ kind: "power", powerOnly: on })} label="只在接着电源时跑">
            {view.powerOnly ? "只在接着电源时跑" : "用电池时也跑"}
          </Toggle>
          <p className="m-0 mt-1 text-small text-sub">关掉主窗口以后，usuyo 留在菜单栏里照常跟着收。</p>
        </div>
      </div>
      <section aria-labelledby="follow-recent-h" className="min-w-0">
        <h2 id="follow-recent-h" className="m-0 text-section font-semibold text-text">
          最近收下的
        </h2>
        <p className="m-0 mt-0.5 text-small text-sub">按词命中收下的标「机器挑的」，一直留着这个来历。</p>
        {view.recent.length === 0 ? <p className="m-0 mt-3 text-note text-sub">还没有。</p> : null}
        <ul className="m-0 mt-3 flex list-none flex-col border-t border-border p-0">
          {view.recent.map((r) => (
            <li key={`${r.followId}:${r.itemId}`} className="border-b border-border">
              <Link to="/items/$id" params={{ id: r.itemId }} className="press -mx-2 my-1 block rounded-btn px-2 py-2 no-underline hover:bg-chip/60">
                <span className="flex items-center gap-2">
                  <PlatformChip>{r.platform.name}</PlatformChip>
                  <span className="min-w-0 truncate text-body text-text">{r.title}</span>
                </span>
                <span className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-small text-sub">
                  {r.picked ? (
                    <>
                      <MachineTag>机器挑的</MachineTag>
                      <span>命中{r.picked.words.map((w) => `「${w}」`).join("")}</span>
                    </>
                  ) : null}
                  <span>
                    从「{titleOf(r.followId) ?? "不再跟着的"}」　{timeText(r.takenAt)}
                  </span>
                </span>
              </Link>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
