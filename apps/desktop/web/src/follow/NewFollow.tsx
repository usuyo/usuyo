import { Link } from "@tanstack/react-router";
import { useState, type ReactNode } from "react";
import { Input, Label, TextField, ToggleButton } from "react-aria-components";
import type { FollowKind, FollowOption, FollowTake, FollowTrialRequest, Topic } from "../api/endpoints";
import { useFollowChange, useFollowTrial } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { Choice } from "../ui/Choice";
import { MachineTag } from "../ui/Chip";
import { hourChoices, KIND_TEXT, trialText } from "./words";

const KINDS: FollowKind[] = ["list", "creator", "feed", "keywords"];

function Step({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="grid gap-x-6 gap-y-2 @xl:grid-cols-[96px_minmax(0,1fr)]">
      <div className="pt-2 text-note text-label">{label}</div>
      <div className="min-w-0">{children}</div>
    </div>
  );
}

/** 为什么这一种现在跟不了：没有插件能这样列，或者能列的还没做好。 */
function unavailableText(kind: FollowKind, options: FollowOption[]): string | null {
  const of = options.filter((o) => o.kind === kind);
  if (of.some((o) => o.built)) return null;
  if (of.length > 0) return `能这样列的取法还没做好：${of.map((o) => `${o.pluginName} · ${o.methodName}`).join("、")}。`;
  return kind === "keywords"
    ? "装着的插件里没有能按关键词找的。可以在订阅源上只收命中关键词的。"
    : `装着的插件里没有能列${KIND_TEXT[kind]}的。`;
}

/**
 * 跟着收一个新的：跟着什么、哪一个、收什么、多久一次。先试列一次看会收下哪些，再定下来。
 * 你自己的收藏夹新加进去的直接收；创作者、订阅源可以只收命中关注主题的，按词命中的标「机器挑的」。
 */
export function NewFollow({ options, topics, onClose }: { options: FollowOption[]; topics: Topic[]; onClose: () => void }) {
  const [kind, setKind] = useState<FollowKind>("list");
  const built = options.filter((o) => o.kind === kind && o.built);
  const [pick, setPick] = useState<string | undefined>(undefined);
  const [url, setUrl] = useState("");
  const [all, setAll] = useState<"all" | "matching">("matching");
  const followed = topics.filter((t) => t.followed);
  const [chosen, setChosen] = useState<string[]>(followed.map((t) => t.id));
  const [words, setWords] = useState("");
  const [hours, setHours] = useState<string | undefined>(undefined);
  const trial = useFollowTrial();
  const create = useFollowChange();

  const sources = built.flatMap((o) => (o.sources ?? []).map((s) => ({ id: `${o.pluginId}|${s.source}`, text: options.length > 1 ? `${s.title}（${o.pluginName}）` : s.title, option: o, source: s.source })));
  const source = kind === "list" ? (sources.find((s) => s.id === pick) ?? sources[0]) : undefined;
  const option = kind === "list" ? source?.option : built[0];
  const minHours = option?.minHours ?? 6;
  const every = hours ?? hourChoices(minHours)[kind === "list" ? 0 : 2]?.id ?? String(minHours);
  const take: FollowTake = kind === "list" || all === "all" ? { all: true } : { all: false, topics: chosen, words: words.split(/[,，、\s]+/).filter((w) => w !== "") };
  const request: FollowTrialRequest | null = option
    ? { kind, pluginId: option.pluginId, ...(source ? { source: source.source } : {}), ...(kind !== "list" ? { url: url.trim() } : {}), take }
    : null;
  const ready = request !== null && (kind === "list" || url.trim() !== "");
  const unavailable = unavailableText(kind, options);
  const reset = () => trial.reset();

  return (
    <section aria-labelledby="new-follow-h" className="@container rounded-card border border-border2 bg-card px-5 pt-4 pb-4">
      <h2 id="new-follow-h" className="m-0 text-section font-semibold text-text">
        跟着收一个新的
      </h2>
      <p className="m-0 mt-0.5 text-small text-sub">
        只收以后新出现的；已经有的，用
        <Link to="/import" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text">
          导入
        </Link>
        一次收下。
      </p>
      <div className="mt-4 flex flex-col gap-4">
        <Step label="跟着什么">
          <Choice
            label="跟着什么"
            options={KINDS.map((k) => ({ id: k, text: KIND_TEXT[k] }))}
            value={kind}
            onChange={(k) => {
              setKind(k);
              reset();
            }}
          />
          {unavailable ? <p className="m-0 mt-2 text-small text-sub">{unavailable}</p> : null}
        </Step>
        {unavailable ? null : kind === "list" ? (
          <Step label="哪一个">
            <Choice
              label="哪一个收藏夹"
              options={sources.map(({ id, text }) => ({ id, text }))}
              value={source?.id}
              onChange={(id) => {
                setPick(id);
                reset();
              }}
            />
          </Step>
        ) : (
          <Step label="地址">
            <TextField value={url} onChange={(v) => { setUrl(v); reset(); }} aria-label={kind === "feed" ? "订阅源的地址" : "创作者主页的链接"}>
              <Label className="sr-only-text">{kind === "feed" ? "订阅源的地址" : "创作者主页的链接"}</Label>
              <Input
                placeholder={kind === "feed" ? "RSS、Atom 或 JSON Feed 的地址" : "创作者主页的链接"}
                className="h-ctl-sm w-full rounded-input border border-border2 bg-transparent px-3 font-sans text-note text-text outline-none focus:border-label"
              />
            </TextField>
          </Step>
        )}
        {unavailable ? null : (
          <Step label="收什么">
            {kind === "list" ? (
              <p className="m-0 pt-2 text-note text-label">这是你自己的收藏夹：新加进去的直接收下，不逐条看，也不用挑。</p>
            ) : (
              <>
                <Choice
                  label="收什么"
                  options={[
                    { id: "matching", text: "只收命中的" },
                    { id: "all", text: "新出现的都收" },
                  ]}
                  value={all}
                  onChange={(v) => {
                    setAll(v);
                    reset();
                  }}
                />
                {all === "matching" ? (
                  <div className="mt-3 flex flex-col gap-2">
                    <div className="flex flex-wrap gap-1.5">
                      {followed.map((t) => (
                        <ToggleButton
                          key={t.id}
                          isSelected={chosen.includes(t.id)}
                          onChange={(on) => {
                            setChosen((c) => (on ? [...c, t.id] : c.filter((x) => x !== t.id)));
                            reset();
                          }}
                          className="press flex h-ctl-xs cursor-pointer items-center gap-2 rounded-full border border-border2 bg-transparent px-3 font-sans text-note text-label outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[selected]:border-accent data-[selected]:text-accent"
                        >
                          <span className="size-[7px] rounded-full bg-accent" aria-hidden="true" />
                          {t.name}的关键词
                        </ToggleButton>
                      ))}
                    </div>
                    <TextField value={words} onChange={(v) => { setWords(v); reset(); }} aria-label="另加的词">
                      <Label className="sr-only-text">另加的词</Label>
                      <Input placeholder="另加几个词，用逗号分开" className="h-ctl-sm w-full max-w-[360px] rounded-input border border-border2 bg-transparent px-3 font-sans text-note text-text outline-none focus:border-label" />
                    </TextField>
                    <p className="m-0 text-small text-sub">看标题和内容里的字；相似度和画面不算，要在主题页里打开。按词收下的标「机器挑的」。</p>
                  </div>
                ) : null}
              </>
            )}
          </Step>
        )}
        {unavailable ? null : (
          <Step label="多久一次">
            <Choice label="多久看一次" options={hourChoices(minHours)} value={every} onChange={setHours} />
            <p className="m-0 mt-2 text-small text-sub">最快每 {minHours} 小时，照插件声明的节奏。每次只翻到上次看过的地方；平台限频时只停这一个。</p>
          </Step>
        )}
      </div>

      {trial.data ? (
        <div role="status" className="mt-5 rounded-card border border-dashed border-border2 px-4 py-3">
          <p className="m-0 text-note text-label">{trialText(trial.data)}</p>
          {trial.data.would.length > 0 ? (
            <ul className="m-0 mt-2 flex list-none flex-col p-0">
              {trial.data.would.map((w) => (
                <li key={w.url} className="flex flex-wrap items-center gap-x-2.5 gap-y-1 border-b border-border py-1.5 last:border-b-0">
                  <span className="min-w-0 flex-1 truncate text-body text-text">{w.title}</span>
                  {w.picked ? (
                    <>
                      <MachineTag>机器挑的</MachineTag>
                      <span className="text-small text-sub">命中{w.picked.map((p) => `「${p}」`).join("")}</span>
                    </>
                  ) : null}
                </li>
              ))}
            </ul>
          ) : (
            <p className="m-0 mt-1 text-small text-sub">照这样，这一次一条也不会收。</p>
          )}
        </div>
      ) : null}
      {trial.error ? <ErrorLine error={trial.error} className="mt-3" /> : null}
      {create.error ? <ErrorLine error={create.error} className="mt-3" /> : null}

      <div className="mt-5 flex flex-wrap items-center gap-2">
        {trial.data && request ? (
          <Button
            variant="soft"
            size="sm"
            isDisabled={create.isPending}
            onPress={() => create.mutate({ kind: "new", body: { ...request, everyHours: Number(every), title: trial.data.title } }, { onSuccess: onClose })}
          >
            就这样跟着收
          </Button>
        ) : (
          <Button variant="soft" size="sm" isDisabled={!ready || trial.isPending || unavailable !== null} onPress={() => request && trial.mutate(request)}>
            {trial.isPending ? "在列……" : "试列一次"}
          </Button>
        )}
        <Button variant="quiet" size="sm" onPress={onClose}>
          算了
        </Button>
        {trial.data ? <span className="text-small text-faint">改了上面哪一样，要再试列一次</span> : null}
      </div>
    </section>
  );
}
