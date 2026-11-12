import { useState } from "react";
import { isNotServed } from "../api/client";
import type { ChangeBatch } from "../api/endpoints";
import { useChanges } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { Choice } from "../ui/Choice";
import { BatchCard } from "./BatchCard";
import { byWho, dayHeading, type Who } from "./words";

const WHO: { id: Who; text: string }[] = [
  { id: "all", text: "全部" },
  { id: "you", text: "你" },
  { id: "agent", text: "Agent" },
  { id: "other", text: "读回和同步" },
];

/** 改动记录与改动提议：按批、按天排，新的在前。提议就在它那一批里定，不另列、不计数。 */
export function ChangesPage() {
  const changes = useChanges();
  const [who, setWho] = useState<Who>("all");
  const missing = isNotServed(changes.error);
  const shown = (changes.data?.batches ?? []).filter((b) => byWho(b, who));
  const days: { day: string; batches: ChangeBatch[] }[] = [];
  for (const b of shown) {
    const day = dayHeading(b.at);
    const last = days.at(-1);
    if (last?.day === day) last.batches.push(b);
    else days.push({ day, batches: [b] });
  }
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航" />
      <main className="absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-7 pb-16 lg:px-gutter">
        <div className="max-w-[820px]">
          <h1 className="m-0 text-title font-semibold text-text">改动记录</h1>
          <p className="m-0 mt-1 text-note text-sub">Agent 改已有的内容先出改动提议，你接受了才生效；收下新条目直接生效。每一批都能整批撤销，撤销了还能重做。</p>
          {missing ? <NotServedLine className="mt-6">这台电脑上的 usuyo 内核还没有改动记录；以后你、Agent、读回和同步做的每一批改动都记在这里，能整批撤销。</NotServedLine> : null}
          {changes.error && !missing ? <ErrorLine error={changes.error} className="mt-6" /> : null}
          {changes.data ? (
            <>
              <div className="mt-5">
                <Choice label="看谁做的" options={WHO} value={who} onChange={setWho} />
              </div>
              {days.length === 0 ? <p className="m-0 mt-6 text-body text-sub">{who === "all" ? "还没有改动。" : "这里还没有。"}</p> : null}
              {days.map((d) => (
                <section key={d.day} aria-label={d.day} className="mt-7">
                  <h2 className="m-0 text-note font-semibold text-label">{d.day}</h2>
                  <div className="mt-2.5 flex flex-col gap-3">
                    {d.batches.map((b) => (
                      <BatchCard key={b.id} batch={b} />
                    ))}
                  </div>
                </section>
              ))}
            </>
          ) : null}
        </div>
      </main>
    </div>
  );
}
