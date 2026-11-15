import { ArrowDown, ArrowUp, Plus, Trash2, X } from "lucide-react";
import type { ReactNode } from "react";
import { Button as AriaButton } from "react-aria-components";
import type { FetchPlugin, StrategyRule, StrategyStep, StrategyView } from "../api/endpoints";
import { usePlugins, useSetStrategy, useStrategy } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { Select, type SelectOption } from "../ui/Select";
import { FUNCTIONS, functionName, GROUPS, presetName, presetSub, PRESETS, stepKey, stepName, wayUse, wayKey, wayName, withPreset, withRules } from "../plugins/strategy";
import { t } from "../i18n";
import { Row } from "./Row";

const ANY = "any";

/**
 * 获取策略：选一个总策略；选「自定义」时下面是规则表，一条一张，从上到下第一条匹配的生效，最后一条是兜底。
 * 改了马上交给内核，没有「保存」。
 */
export function StrategySection() {
  const strategy = useStrategy();
  const plugins = usePlugins();
  const set = useSetStrategy();
  const v = strategy.data;
  if (!v) return strategy.error ? <ErrorLine error={strategy.error} /> : null;
  return (
    <div className="flex flex-col gap-4">
      <Row label={t("settings.strategy.preset")} sub={presetSub(v.preset)}>
        <Select
          label={t("settings.strategy.preset")}
          options={PRESETS.map((p) => ({ id: p, text: presetName(p) }))}
          value={v.preset}
          isDisabled={set.isPending}
          onChange={(id) => {
            const p = PRESETS.find((x) => x === id);
            if (p) set.mutate(withPreset(v, p));
          }}
        />
      </Row>
      <p className="m-0 text-small text-sub">{t("plugins.strategy.tiersNote")}</p>
      {v.preset === "custom" ? <Rules view={v} plugins={plugins.data?.plugins ?? []} pending={set.isPending} onChange={(rules) => set.mutate(withRules(v, rules))} /> : null}
      {set.error ? <ErrorLine error={set.error} /> : null}
    </div>
  );
}

function Rules({ view, plugins, pending, onChange }: { view: StrategyView; plugins: FetchPlugin[]; pending: boolean; onChange: (rules: StrategyRule[]) => void }) {
  const rules = view.rules ?? [];
  const put = (i: number, rule: StrategyRule) => onChange(rules.map((r, j) => (j === i ? rule : r)));
  const move = (i: number, by: -1 | 1) => {
    const next = [...rules];
    const [r] = next.splice(i, 1);
    if (r) next.splice(i + by, 0, r);
    onChange(next);
  };
  const fallback = rules[rules.length - 1];
  return (
    <div className="flex flex-col gap-3">
      <div>
        <div className="text-note font-medium text-text">{t("settings.strategy.rules")}</div>
        <p className="m-0 mt-0.5 text-small text-sub">{t("settings.strategy.rulesSub")}</p>
      </div>
      <ol className="m-0 flex list-none flex-col gap-2.5 p-0">
        {rules.map((rule, i) => {
          const last = i === rules.length - 1;
          return (
            <li key={i} aria-label={t("settings.strategy.rule", { n: i + 1 })} className="min-w-0 rounded-card border border-border px-3.5 py-3">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <span className="text-small text-faint">{last ? t("settings.strategy.fallback") : t("settings.strategy.rule", { n: i + 1 })}</span>
                {last ? null : (
                  <div className="flex items-center gap-0.5">
                    <IconButton label={t("settings.strategy.moveUp")} isDisabled={pending || i === 0} onPress={() => move(i, -1)}>
                      <ArrowUp size={14} aria-hidden="true" />
                    </IconButton>
                    <IconButton label={t("settings.strategy.moveDown")} isDisabled={pending || i >= rules.length - 2} onPress={() => move(i, 1)}>
                      <ArrowDown size={14} aria-hidden="true" />
                    </IconButton>
                    <IconButton label={t("settings.strategy.removeRule")} isDisabled={pending} onPress={() => onChange(rules.filter((_, j) => j !== i))}>
                      <Trash2 size={14} aria-hidden="true" />
                    </IconButton>
                  </div>
                )}
              </div>
              {last ? null : <Match rule={rule} view={view} pending={pending} onChange={(r) => put(i, r)} />}
              <Steps rule={rule} view={view} plugins={plugins} pending={pending} onChange={(r) => put(i, r)} />
            </li>
          );
        })}
      </ol>
      <div>
        <Button variant="outline" size="sm" isDisabled={pending || !fallback} onPress={() => onChange([...rules.slice(0, -1), { use: [...(fallback?.use ?? [])] }, ...rules.slice(-1)])}>
          <Plus size={14} aria-hidden="true" />
          <span>{t("settings.strategy.addRule")}</span>
        </Button>
      </div>
    </div>
  );
}

/** 一条规则匹配什么：平台、内容，「任意」是不写。 */
function Match({ rule, view, pending, onChange }: { rule: StrategyRule; view: StrategyView; pending: boolean; onChange: (r: StrategyRule) => void }) {
  const platforms: SelectOption[] = [{ id: ANY, text: t("settings.strategy.anyPlatform") }, ...view.routes.map((r) => ({ id: r.platform.id, text: r.platform.name }))];
  // 存着的规则写了一个现在没装插件的平台，也照样列出来。
  if (rule.platform && !platforms.some((p) => p.id === rule.platform)) platforms.push({ id: rule.platform, text: rule.platform });
  const functions: SelectOption[] = [{ id: ANY, text: t("settings.strategy.anyFunction") }, ...FUNCTIONS.map((f) => ({ id: f, text: functionName(f) }))];
  const without = (key: "platform" | "function") => {
    const next = { ...rule };
    delete next[key];
    return next;
  };
  return (
    <div className="mt-2.5 flex flex-col gap-2">
      <Row label={t("settings.strategy.platform")}>
        <Select
          label={t("settings.strategy.platform")}
          options={platforms}
          value={rule.platform ?? ANY}
          isDisabled={pending}
          onChange={(id) => onChange(id === ANY ? without("platform") : { ...rule, platform: id })}
        />
      </Row>
      <Row label={t("settings.strategy.function")}>
        <Select
          label={t("settings.strategy.function")}
          options={functions}
          value={rule.function ?? ANY}
          isDisabled={pending}
          onChange={(id) => {
            const f = FUNCTIONS.find((x) => x === id);
            onChange(f ? { ...rule, function: f } : without("function"));
          }}
        />
      </Row>
    </div>
  );
}

/** 去向：几段，按顺序回退；一段也没有是「不获取」。加一段从组和装着的方式里选。 */
function Steps({ rule, view, plugins, pending, onChange }: { rule: StrategyRule; view: StrategyView; plugins: FetchPlugin[]; pending: boolean; onChange: (r: StrategyRule) => void }) {
  const have = new Set(rule.use.map(stepKey));
  const ways = new Map<string, string>();
  for (const r of view.routes) {
    for (const f of r.functions) {
      for (const w of f.choices) {
        const u = wayUse(plugins, w);
        if (u) ways.set(wayKey(w), plugins.length > 1 ? t("plugins.link.optionName", { method: wayName(u), plugin: u.plugin.name }) : wayName(u));
      }
    }
  }
  const choices: { step: StrategyStep; option: SelectOption }[] = [
    ...GROUPS.map((g) => ({ step: g.step, option: { id: `group:${g.id}`, text: g.name() } })),
    ...[...ways].map(([key, text]) => ({ step: { method: key }, option: { id: `method:${key}`, text } })),
  ].filter((c) => !have.has(stepKey(c.step)));
  return (
    <div className="mt-2.5">
      <Row label={t("settings.strategy.use")}>
        <div className="flex flex-col gap-2">
          {rule.use.length === 0 ? (
            <span className="pt-2 text-note text-label">{t("settings.strategy.skip")}</span>
          ) : (
            <ol className="m-0 flex list-none flex-col gap-1.5 p-0">
              {rule.use.map((s, i) => {
                const name = stepName(s, plugins);
                return (
                  <li key={stepKey(s)} className="flex min-w-0 items-center gap-2">
                    <span className="mono w-4 shrink-0 text-tag text-faint">{i + 1}</span>
                    <span className="min-w-0 flex-1 text-note text-label">{name}</span>
                    <IconButton label={t("settings.strategy.removeStep", { name })} isDisabled={pending} onPress={() => onChange({ ...rule, use: rule.use.filter((_, j) => j !== i) })}>
                      <X size={14} aria-hidden="true" />
                    </IconButton>
                  </li>
                );
              })}
            </ol>
          )}
          {choices.length > 0 ? (
            <Select
              label={t("settings.strategy.addStep")}
              options={choices.map((c) => c.option)}
              value={null}
              placeholder={t("settings.strategy.addStep")}
              isDisabled={pending}
              onChange={(id) => {
                const c = choices.find((x) => x.option.id === id);
                if (c) onChange({ ...rule, use: [...rule.use, c.step] });
              }}
            />
          ) : null}
        </div>
      </Row>
    </div>
  );
}

function IconButton({ label, isDisabled, onPress, children }: { label: string; isDisabled?: boolean; onPress: () => void; children: ReactNode }) {
  return (
    <AriaButton
      aria-label={label}
      isDisabled={isDisabled}
      onPress={onPress}
      className="press flex size-7 cursor-pointer items-center justify-center rounded-sm border-0 bg-transparent text-sub outline-none hover:bg-chip/60 hover:text-text data-[disabled]:cursor-default data-[disabled]:opacity-40 data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
    >
      {children}
    </AriaButton>
  );
}
