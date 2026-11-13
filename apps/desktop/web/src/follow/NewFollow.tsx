import { Link } from "@tanstack/react-router";
import { useState, type ReactNode } from "react";
import { Input, Label, TextField, ToggleButton } from "react-aria-components";
import type { FollowKind, FollowOption, FollowTake, FollowTrialRequest, Topic } from "../api/endpoints";
import { useFollowChange, useFollowTrial } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { rich, t } from "../i18n";
import { formatList } from "../i18n/format";
import { Button } from "../ui/Button";
import { Choice } from "../ui/Choice";
import { MachineTag } from "../ui/Chip";
import { hourChoices, kindText, quotedList, trialText } from "./words";

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
  if (of.length > 0) return t("follow.form.unavailableBuilt", { list: formatList(of.map((o) => t("follow.form.methodOf", { plugin: o.pluginName, method: o.methodName }))) });
  switch (kind) {
    case "keywords":
      return t("follow.form.unavailableKeywords");
    case "list":
      return t("follow.form.unavailableList");
    case "creator":
      return t("follow.form.unavailableCreator");
    default:
      return t("follow.form.unavailableFeed");
  }
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
  const followed = topics.filter((x) => x.followed);
  const [chosen, setChosen] = useState<string[]>(followed.map((x) => x.id));
  const [words, setWords] = useState("");
  const [hours, setHours] = useState<string | undefined>(undefined);
  const trial = useFollowTrial();
  const create = useFollowChange();

  const sources = built.flatMap((o) => (o.sources ?? []).map((s) => ({ id: `${o.pluginId}|${s.source}`, text: options.length > 1 ? t("follow.form.sourceWithPlugin", { title: s.title, plugin: o.pluginName }) : s.title, option: o, source: s.source })));
  const source = kind === "list" ? (sources.find((s) => s.id === pick) ?? sources[0]) : undefined;
  const option = kind === "list" ? source?.option : built[0];
  const minHours = option?.minHours ?? 6;
  const every = hours ?? hourChoices(minHours)[kind === "list" ? 0 : 2]?.id ?? String(minHours);
  const take: FollowTake = kind === "list" || all === "all" ? { all: true } : { all: false, topics: chosen, words: words.split(/[,\uFF0C\u3001\s]+/).filter((w) => w !== "") };
  const request: FollowTrialRequest | null = option
    ? { kind, pluginId: option.pluginId, ...(source ? { source: source.source } : {}), ...(kind !== "list" ? { url: url.trim() } : {}), take }
    : null;
  const ready = request !== null && (kind === "list" || url.trim() !== "");
  const unavailable = unavailableText(kind, options);
  const reset = () => trial.reset();

  return (
    <section aria-labelledby="new-follow-h" className="@container rounded-card border border-border2 bg-card px-5 pt-4 pb-4">
      <h2 id="new-follow-h" className="m-0 text-section font-semibold text-text">
        {t("follow.new")}
      </h2>
      <p className="m-0 mt-0.5 text-small text-sub">
        {rich("follow.form.note", {
          link: <Link to="/import" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text" />,
        })}
      </p>
      <div className="mt-4 flex flex-col gap-4">
        <Step label={t("follow.form.kind")}>
          <Choice
            label={t("follow.form.kindLabel")}
            options={KINDS.map((k) => ({ id: k, text: kindText(k) }))}
            value={kind}
            onChange={(k) => {
              setKind(k);
              reset();
            }}
          />
          {unavailable ? <p className="m-0 mt-2 text-small text-sub">{unavailable}</p> : null}
        </Step>
        {unavailable ? null : kind === "list" ? (
          <Step label={t("follow.form.which")}>
            <Choice
              label={t("follow.form.whichLabel")}
              options={sources.map(({ id, text }) => ({ id, text }))}
              value={source?.id}
              onChange={(id) => {
                setPick(id);
                reset();
              }}
            />
          </Step>
        ) : (
          <Step label={t("follow.form.address")}>
            <TextField value={url} onChange={(v) => { setUrl(v); reset(); }} aria-label={kind === "feed" ? t("follow.form.feedUrl") : t("follow.form.creatorUrl")}>
              <Label className="sr-only-text">{kind === "feed" ? t("follow.form.feedUrl") : t("follow.form.creatorUrl")}</Label>
              <Input
                placeholder={kind === "feed" ? t("follow.form.feedPlaceholder") : t("follow.form.creatorUrl")}
                className="h-ctl-sm w-full rounded-input border border-border2 bg-transparent px-3 font-sans text-note text-text outline-none focus:border-label"
              />
            </TextField>
          </Step>
        )}
        {unavailable ? null : (
          <Step label={t("follow.form.take")}>
            {kind === "list" ? (
              <p className="m-0 pt-2 text-note text-label">{t("follow.form.listNote")}</p>
            ) : (
              <>
                <Choice
                  label={t("follow.form.takeLabel")}
                  options={[
                    { id: "matching", text: t("follow.form.takeMatching") },
                    { id: "all", text: t("follow.form.takeAll") },
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
                      {followed.map((topic) => (
                        <ToggleButton
                          key={topic.id}
                          isSelected={chosen.includes(topic.id)}
                          onChange={(on) => {
                            setChosen((c) => (on ? [...c, topic.id] : c.filter((x) => x !== topic.id)));
                            reset();
                          }}
                          className="press flex h-ctl-xs cursor-pointer items-center gap-2 rounded-full border border-border2 bg-transparent px-3 font-sans text-note text-label outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[selected]:border-accent data-[selected]:text-accent"
                        >
                          <span className="size-[7px] rounded-full bg-accent" aria-hidden="true" />
                          {t("follow.form.topicKeywords", { topic: topic.name })}
                        </ToggleButton>
                      ))}
                    </div>
                    <TextField value={words} onChange={(v) => { setWords(v); reset(); }} aria-label={t("follow.form.extraWords")}>
                      <Label className="sr-only-text">{t("follow.form.extraWords")}</Label>
                      <Input placeholder={t("follow.form.extraPlaceholder")} className="h-ctl-sm w-full max-w-[360px] rounded-input border border-border2 bg-transparent px-3 font-sans text-note text-text outline-none focus:border-label" />
                    </TextField>
                    <p className="m-0 text-small text-sub">{t("follow.form.matchNote")}</p>
                  </div>
                ) : null}
              </>
            )}
          </Step>
        )}
        {unavailable ? null : (
          <Step label={t("follow.form.every")}>
            <Choice label={t("follow.form.everyLabel")} options={hourChoices(minHours)} value={every} onChange={setHours} />
            <p className="m-0 mt-2 text-small text-sub">{t("follow.form.everyNote", { count: minHours })}</p>
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
                      <MachineTag>{t("common.tag.autoSelected")}</MachineTag>
                      <span className="text-small text-sub">{t("follow.matched", { words: quotedList(w.picked) })}</span>
                    </>
                  ) : null}
                </li>
              ))}
            </ul>
          ) : (
            <p className="m-0 mt-1 text-small text-sub">{t("follow.trial.nothing")}</p>
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
            {t("follow.form.subscribe")}
          </Button>
        ) : (
          <Button variant="soft" size="sm" isDisabled={!ready || trial.isPending || unavailable !== null} onPress={() => request && trial.mutate(request)}>
            {trial.isPending ? t("follow.form.trying") : t("follow.form.tryOnce")}
          </Button>
        )}
        <Button variant="quiet" size="sm" onPress={onClose}>
          {t("common.cancel")}
        </Button>
        {trial.data ? <span className="text-small text-faint">{t("follow.form.changed")}</span> : null}
      </div>
    </section>
  );
}
