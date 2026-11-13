import { Link } from "@tanstack/react-router";
import { Check, Circle } from "lucide-react";
import { useState, type ReactNode } from "react";
import { Form, Input, TextField } from "react-aria-components";
import type { PluginSetting, SiteLogin, VisitorIdentity } from "../api/endpoints";
import { usePluginChange, useSetSecret } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import type { Use } from "./platforms";
import { SiteLoginButton } from "./SiteLoginRow";
import { VisitorTake } from "./VisitorIdentity";
import { t } from "../i18n";
import { LOGIN_STATE, VISITOR_STATE } from "./words";

interface Need {
  key: string;
  ok: boolean;
  text: string;
  action?: ReactNode;
}

/** 本机检查没过时内核写的那一句（缺的程序、没开的服务……），和插件没能自查。 */
const CHECKS = new Set(["check-failed", "health-failed"]);

/**
 * 这种方法在这台电脑上要的几样，一样一行：要的登录、访客身份、手机，插件本机检查没过的那几句（内核的原话），
 * 要填的密钥，停着的。每行写现在怎样，缺的给一个动手的地方。开关不在这里，由调用处摆。
 */
function needsOf(u: Use, sites: SiteLogin[], visitors: VisitorIdentity[]): Need[] {
  const m = u.method;
  const out: Need[] = [];
  if (m.account.kind === "in-usuyo") {
    for (const s of m.account.sites ?? []) {
      const site = sites.find((x) => x.site === s);
      if (!site) continue;
      out.push({
        key: `login:${s}`,
        ok: site.login === "present",
        text: t("plugins.needs.login", { name: site.name, state: LOGIN_STATE[site.login] }),
        ...(site.login === "present" ? {} : { action: <SiteLoginButton site={site} /> }),
      });
    }
  }
  if (m.account.kind === "visitor") {
    for (const s of m.account.sites ?? []) {
      const visitor = visitors.find((x) => x.site === s);
      if (!visitor) continue;
      out.push({
        key: `visitor:${s}`,
        ok: visitor.state === "present",
        text: t("plugins.needs.visitor", { name: visitor.name, state: VISITOR_STATE[visitor.state] }),
        ...(visitor.state === "present" ? {} : { action: <VisitorTake visitor={visitor} /> }),
      });
    }
  }
  if (m.account.kind === "own-phone") {
    out.push({
      key: "phone",
      ok: m.usable,
      text: t("plugins.needs.phone", { where: m.account.where ?? "" }),
      action: (
        <Link to="/phone" className="text-small text-label underline decoration-border2 underline-offset-4 hover:text-text">
          {t("plugins.needs.goPhone")}
        </Link>
      ),
    });
  }
  const checks = m.blockers.filter((b) => CHECKS.has(b.code));
  for (const b of checks) {
    out.push({ key: `check:${b.hint}`, ok: false, text: b.hint, action: <TryAgain pluginId={u.plugin.id} label={t("plugins.needs.recheck")} /> });
  }
  // 插件的密钥设置不分方法：用开发者凭据的、本机检查没过的，把还没填的列上。
  if (m.account.kind === "developer" || checks.length > 0) {
    for (const s of u.plugin.settings.filter((x) => x.secret && !x.filled)) {
      out.push({ key: `secret:${s.key}`, ok: false, text: t("plugins.needs.secret", { label: s.label }), action: <SecretField pluginId={u.plugin.id} setting={s} /> });
    }
  }
  for (const b of m.blockers.filter((x) => x.code === "paused")) {
    out.push({ key: `paused:${b.hint}`, ok: false, text: b.hint, action: <TryAgain pluginId={u.plugin.id} label={t("plugins.needs.retryNow")} /> });
  }
  return out;
}

/** 一种方法在这台电脑上要的几样，一样一行，带现在的样子和动手的地方；什么都不缺时说一句。 */
export function Needs({ use, sites, visitors }: { use: Use; sites: SiteLogin[]; visitors: VisitorIdentity[] }) {
  const needs = needsOf(use, sites, visitors);
  if (needs.length === 0) return <p className="m-0 text-small text-sub">{t("plugins.needs.nothing")}</p>;
  return (
    <ul aria-label={t("plugins.needs.label", { name: use.method.name })} className="m-0 flex list-none flex-col gap-2 p-0">
      {needs.map((n) => (
        <li key={n.key} className="flex items-start gap-2">
          <span className="mt-[3px] shrink-0 text-sub" aria-hidden="true">
            {n.ok ? <Check size={14} /> : <Circle size={14} />}
          </span>
          <div className="flex min-w-0 flex-1 flex-wrap items-start justify-between gap-x-3 gap-y-1.5">
            <span className={`min-w-0 flex-[1_1_220px] text-note ${n.ok ? "text-sub" : "text-label"}`}>
              <span className="sr-only-text">{n.ok ? t("plugins.needs.done") : t("plugins.needs.todo")}</span>
              {n.text}
            </span>
            {n.action ? <div className="min-w-0">{n.action}</div> : null}
          </div>
        </li>
      ))}
    </ul>
  );
}

function TryAgain({ pluginId, label }: { pluginId: string; label: string }) {
  const change = usePluginChange(pluginId);
  return (
    <div className="min-w-0">
      <Button variant="outline" size="sm" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "try-now" })}>
        {label}
      </Button>
      {change.error ? <ErrorLine error={change.error} className="mt-1" /> : null}
    </div>
  );
}

/** 填一个密钥设置：存进钥匙串，值不回出来。 */
export function SecretField({ pluginId, setting }: { pluginId: string; setting: PluginSetting }) {
  const set = useSetSecret(pluginId);
  const [value, setValue] = useState("");
  return (
    <Form
      onSubmit={(e) => {
        e.preventDefault();
        if (value.trim() === "") return;
        set.mutate({ key: setting.key, value }, { onSuccess: () => setValue("") });
      }}
      className="flex flex-wrap items-center gap-1.5"
    >
      <TextField value={value} onChange={setValue} aria-label={setting.label} type="password" className="w-[200px] max-w-full">
        <Input
          placeholder={setting.filled ? t("plugins.needs.secretFilled") : setting.label}
          className="h-ctl-sm w-full rounded-input border border-border2 bg-transparent px-3 font-sans text-note text-text outline-none focus:border-label"
        />
      </TextField>
      <Button type="submit" variant="soft" size="sm" isDisabled={set.isPending || value.trim() === ""}>
        {t("plugins.needs.secretSave")}
      </Button>
      {set.error ? <ErrorLine error={set.error} className="w-full" /> : null}
    </Form>
  );
}
