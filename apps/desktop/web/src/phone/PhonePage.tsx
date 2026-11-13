import { Link } from "@tanstack/react-router";
import { Copy } from "lucide-react";
import type { ReactNode } from "react";
import { Button as AriaButton } from "react-aria-components";
import { isNotServed } from "../api/client";
import type { GrantState, PhoneAndroid, PhoneDryRun, PhoneIos, PhoneStep, PhoneView, SettingsPane } from "../api/endpoints";
import { useDryRun, usePhone, usePhoneAction, usePlugins } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { sizeText } from "../item/faces";
import { ACCESS_TEXT } from "../plugins/words";
import { Row } from "../settings/Row";
import { openExternal } from "../shell";
import { rich, t, type MessageKey } from "../i18n";
import { formatList } from "../i18n/format";
import { Button } from "../ui/Button";
import { FlashLine, useFlash } from "../ui/Flash";
import { clipboardText, deviceText, grantText, iosMissing, methodState, needsRegister, phoneMethods } from "./words";

const SECTIONS = [
  { id: "iphone", name: "phone.iphone.name" },
  { id: "android", name: "phone.android.name" },
  { id: "methods", name: "phone.methods.name" },
] as const satisfies { id: string; name: MessageKey }[];

/**
 * 手机：像设置一样一页从上到下。每一行说这台电脑上的一件事，最多给一个动作；
 * 只有你能点的那几下（系统授权、手机上的允许），给去那里的按钮。
 */
export function PhonePage() {
  const phone = usePhone();
  const missing = isNotServed(phone.error);
  const gate = (node: (v: PhoneView) => ReactNode) => (phone.data ? node(phone.data) : phone.error && !missing ? <ErrorLine error={phone.error} /> : null);

  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label={t("common.nav.main")} />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto scroll-smooth px-6 pt-7 pb-24 motion-reduce:scroll-auto lg:px-gutter">
        <h1 className="m-0 text-title font-semibold text-text">{t("phone.title")}</h1>
        <p className="m-0 mt-1 max-w-[720px] text-note text-sub">
          {t("phone.intro")}
        </p>
        {missing ? (
          <NotServedLine className="mt-4">{t("phone.notServed")}</NotServedLine>
        ) : null}
        <div className="mt-8 grid items-start gap-x-10 @3xl:grid-cols-[160px_minmax(0,1fr)]">
          <nav aria-label={t("phone.sectionsLabel")} className="sticky top-0 hidden flex-col gap-0.5 @3xl:flex">
            {SECTIONS.filter((s) => !missing || s.id === "methods").map((s) => (
              <a
                key={s.id}
                href={`#${s.id}`}
                className="press flex h-ctl-xs items-center rounded-sm px-2.5 text-note text-sub no-underline hover:bg-chip/60 hover:text-text"
              >
                {t(s.name)}
              </a>
            ))}
          </nav>
          <div className="flex min-w-0 max-w-[760px] flex-col">
            {missing ? null : (
              <>
                <Section id="iphone" name={t("phone.iphone.name")} sub={t("phone.iphone.sub")}>
                  {gate((v) => (
                    <Ios ios={v.ios} />
                  ))}
                </Section>
                <Section id="android" name={t("phone.android.name")} sub={t("phone.android.sub")}>
                  {gate((v) => (
                    <Android android={v.android} />
                  ))}
                </Section>
              </>
            )}
            <Section id="methods" name={t("phone.methods.name")} sub={t("phone.methods.sub")}>
              <Methods phone={phone.data ?? null} />
            </Section>
          </div>
        </div>
      </main>
    </div>
  );
}

function Section({ id, name, sub, children }: { id: string; name: string; sub: string; children: ReactNode }) {
  return (
    <section id={id} aria-labelledby={`${id}-h`} className="scroll-mt-6 border-t border-border pt-5 pb-9 first:border-t-0 first:pt-0">
      <h2 id={`${id}-h`} className="m-0 text-section font-semibold text-text">
        {name}
      </h2>
      <p className="m-0 mt-1 text-note text-sub">{sub}</p>
      <div className="mt-4 flex flex-col gap-5">{children}</div>
    </section>
  );
}

/** 一行里的事实和最多一个动作；窄的时候动作换到下一行。 */
function Fact({ children, action, warn }: { children: ReactNode; action?: ReactNode; warn?: boolean }) {
  return (
    <div className="flex min-h-ctl-sm flex-wrap items-center gap-x-3 gap-y-2">
      <p className={`m-0 min-w-0 flex-[1_1_220px] text-note ${warn ? "text-warn" : "text-text"}`}>{children}</p>
      {action ? <div className="flex items-center gap-1.5">{action}</div> : null}
    </div>
  );
}

function Ios({ ios }: { ios: PhoneIos }) {
  const present = ios.driver.state === "present";
  const unregistered = needsRegister(ios);
  return (
    <>
      <Row label={t("phone.system.label")}>
        <Fact>{t(ios.system.mirroring ? "phone.system.mirroring" : "phone.system.noMirroring", { version: ios.system.macos })}</Fact>
      </Row>
      <Driver ios={ios} />
      <Grant
        label={t("phone.grant.screen")}
        state={ios.grants.screen}
        pane="screen-recording"
        present={present}
        unregistered={unregistered}
        to={ios.grants.to}
        sub={t("phone.grant.screenSub", { to: ios.grants.to })}
      />
      <Grant
        label={t("phone.grant.accessibility")}
        state={ios.grants.accessibility}
        pane="accessibility"
        present={present}
        unregistered={unregistered}
        to={ios.grants.to}
        sub={t("phone.grant.accessibilitySub", { to: ios.grants.to })}
      />
      <Mirroring ios={ios} />
      <Clipboard ios={ios} />
      <DryRun ios={ios} />
    </>
  );
}

function Driver({ ios }: { ios: PhoneIos }) {
  const action = usePhoneAction();
  const d = ios.driver;
  const offer = d.available;
  const from = offer
    ? rich("phone.driver.source", {
        version: offer.version,
        size: sizeText(offer.size),
        link: <AriaButton
            onPress={() => void openExternal(offer.source)}
            className="mx-0.5 cursor-pointer border-0 bg-transparent p-0 font-sans text-small text-label underline decoration-border2 underline-offset-4 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text" />,
      })
    : null;
  const sub =
    d.state === "present"
      ? t(d.step ? "phone.driver.presentStepSub" : "phone.driver.presentSub", { to: ios.grants.to })
      : d.state === "installing"
        ? t("phone.driver.installingSub")
        : from;
  const busy = action.isPending;
  // 登记那一步跑完以后这台 Mac 上多出来的东西；换新、停掉的那两步不加这一句。
  const registers = needsRegister(ios);
  return (
    <Row label={t("phone.driver.label")} sub={sub}>
      {d.state === "absent" ? (
        <Fact
          action={
            <Button variant="soft" size="sm" isDisabled={busy} onPress={() => action.mutate({ kind: "install" })}>
              {t("phone.driver.install")}
            </Button>
          }
        >
          {t("phone.driver.absent")}
        </Fact>
      ) : d.state === "installing" ? (
        <Fact>{offer ? t("phone.driver.installingVersion", { version: offer.version }) : t("phone.driver.installing")}</Fact>
      ) : d.state === "failed" ? (
        <Fact
          warn
          action={
            <Button variant="soft" size="sm" isDisabled={busy} onPress={() => action.mutate({ kind: "install" })}>
              {t("phone.driver.installAgain")}
            </Button>
          }
        >
          {t("phone.driver.failed", { hint: d.hint ?? t("common.reasonUnknown") })}
        </Fact>
      ) : (
        <Fact
          action={
            <>
              {offer ? (
                <Button variant="soft" size="sm" isDisabled={busy} onPress={() => action.mutate({ kind: "update" })}>
                  {t("phone.driver.update")}
                </Button>
              ) : null}
              <Button variant="quiet" size="sm" isDisabled={busy} onPress={() => action.mutate({ kind: "remove" })}>
                {t("phone.driver.remove")}
              </Button>
            </>
          }
        >
          {offer ? t("phone.driver.presentUpdate", { version: d.version ?? "", next: offer.version }) : t("phone.driver.present", { version: d.version ?? "" })}
        </Fact>
      )}
      {d.state === "present" && d.hint ? <p className="m-0 mt-1 text-note text-warn">{t("phone.driver.updateFailed", { hint: d.hint })}</p> : null}
      {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
      {d.step ? (
        <StepBox
          step={d.step}
          label={t("phone.driver.stepLabel")}
          after={registers ? t("phone.driver.afterRegister") : undefined}
        />
      ) : null}
    </Row>
  );
}

/** 内核给的一步：它的那句话（原样显示），一条命令一行，各带「拷贝」。usuyo 不替你跑。 */
function StepBox({ step, label, after }: { step: PhoneStep; label: string; after?: string }) {
  const flash = useFlash();
  return (
    <div role="group" aria-label={label} className="mt-2 rounded-btn bg-card px-3 py-2.5">
      <p className="m-0 text-note text-label">{step.text}</p>
      <ul className="m-0 mt-2 flex list-none flex-col gap-1.5 p-0">
        {step.commands.map((c, i) => (
          <li key={i} className="flex min-w-0 items-center gap-2">
            <code className="mono min-w-0 flex-1 rounded-sm bg-chip px-3 py-1.5 text-tag break-all text-text select-text">{c}</code>
            <Button
              variant="quiet"
              size="sm"
              aria-label={t("phone.driver.copyCommand", { n: i + 1 })}
              onPress={() => {
                navigator.clipboard.writeText(c).then(
                  () => flash.show(t("common.copied")),
                  () => flash.show(t("common.copyFailed")),
                );
              }}
            >
              <Copy size={14} aria-hidden="true" />
              {t("common.copy")}
            </Button>
          </li>
        ))}
      </ul>
      {after ? <p className="m-0 mt-2 text-small text-sub">{after}</p> : null}
      <FlashLine flash={flash} className="mt-1 block text-small text-sub" />
    </div>
  );
}

function Grant({
  label,
  state,
  pane,
  present,
  unregistered,
  to,
  sub,
}: {
  label: string;
  state: GrantState;
  pane: SettingsPane;
  present: boolean;
  unregistered: boolean;
  to: string;
  sub: string;
}) {
  const action = usePhoneAction();
  return (
    <Row label={label} sub={sub}>
      {unregistered ? (
        <Fact>{t("phone.grant.afterStep", { to })}</Fact>
      ) : present ? (
        <Fact
          action={
            state === "given" ? null : (
              <Button variant="outline" size="sm" isDisabled={action.isPending} onPress={() => action.mutate({ kind: "open", pane })}>
                {t("phone.grant.open")}
              </Button>
            )
          }
        >
          {grantText(state, to)}
        </Fact>
      ) : (
        <Fact>{t("phone.grant.beforeDriver")}</Fact>
      )}
      {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
    </Row>
  );
}

function Mirroring({ ios }: { ios: PhoneIos }) {
  const action = usePhoneAction();
  if (!ios.system.mirroring) {
    return (
      <Row label={t("phone.mirroring.label")}>
        <Fact>{t("phone.mirroring.unsupported")}</Fact>
      </Row>
    );
  }
  const sub = t(
    ({ "not-set-up": "phone.mirroring.notSetUpSub", away: "phone.mirroring.awaySub", connected: "phone.mirroring.connectedSub" } as const)[ios.mirroring],
  );
  return (
    <Row label={t("phone.mirroring.label")} sub={sub}>
      {ios.mirroring === "not-set-up" ? (
        <Fact
          action={
            <Button variant="outline" size="sm" isDisabled={action.isPending} onPress={() => action.mutate({ kind: "open", pane: "iphone-mirroring" })}>
              {t("phone.mirroring.open")}
            </Button>
          }
        >
          {t("phone.mirroring.notOpen")}
        </Fact>
      ) : (
        <Fact>{ios.mirroring === "away" ? t("phone.mirroring.away") : t("phone.mirroring.connected")}</Fact>
      )}
      {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
    </Row>
  );
}

function Clipboard({ ios }: { ios: PhoneIos }) {
  const action = usePhoneAction();
  const test = ios.clipboard;
  const sub = test?.state === "unchanged" ? t("phone.clipboard.unchangedSub") : t("phone.clipboard.sub");
  return (
    <Row label={t("phone.clipboard.label")} sub={sub}>
      <Fact
        action={
          test?.state === "waiting" ? null : (
            <Button variant="outline" size="sm" isDisabled={action.isPending} onPress={() => action.mutate({ kind: "clipboard" })}>
              {test ? t("phone.clipboard.again") : t("phone.clipboard.test")}
            </Button>
          )
        }
      >
        {clipboardText(test)}
      </Fact>
      {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
    </Row>
  );
}

function DryRun({ ios }: { ios: PhoneIos }) {
  const run = useDryRun();
  const missing = iosMissing(ios);
  return (
    <Row label={t("phone.dryRun.label")} sub={t("phone.dryRun.sub")}>
      {missing.length > 0 ? (
        <Fact>{t("phone.dryRun.needs", { list: formatList(missing) })}</Fact>
      ) : (
        <Fact
          action={
            <Button variant="soft" size="sm" isDisabled={run.isPending} onPress={() => run.mutate()}>
              {t("phone.dryRun.run")}
            </Button>
          }
        >
          {run.isPending ? t("phone.dryRun.pending") : t("phone.dryRun.idle")}
        </Fact>
      )}
      {run.error ? <ErrorLine error={run.error} className="mt-1" /> : null}
      {run.data && !run.isPending ? <DryRunResult report={run.data} /> : null}
    </Row>
  );
}

function DryRunResult({ report }: { report: PhoneDryRun }) {
  const m = report.match;
  const lines = m
    ? [
        t("phone.dryRun.flow", { flow: m.flow, step: m.step }),
        m.expect.matched === m.expect.total
          ? t("phone.dryRun.allMatched", { count: m.expect.total })
          : t("phone.dryRun.someMatched", { total: m.expect.total, matched: m.expect.matched }),
        ...m.targets.map((target) => t(target.found ? "phone.dryRun.targetFound" : "phone.dryRun.targetMissing", { label: target.label })),
      ]
    : [t("phone.dryRun.unknown", { count: report.read })];
  return (
    <ul aria-label={t("phone.dryRun.resultLabel")} className="m-0 mt-2 flex list-none flex-col gap-1 rounded-btn bg-card px-3 py-2.5">
      {lines.map((l) => (
        <li key={l} className="text-note text-label">
          {l}
        </li>
      ))}
      <li className="text-small text-sub">{t("phone.dryRun.tappedNothing")}</li>
    </ul>
  );
}

function Android({ android }: { android: PhoneAndroid }) {
  const action = usePhoneAction();
  const adb = android.adb;
  return (
    <>
      <Row
        label={t("phone.adb.label")}
        sub={
          adb ? (
            <span className="mono break-all text-tag">{adb.path}</span>
          ) : (
            <>
              {t("phone.adb.install")}
              <code className="mono mt-1.5 block rounded-sm bg-chip px-3 py-2 text-tag text-text select-text">brew install android-platform-tools</code>
            </>
          )
        }
      >
        <Fact>{adb ? t("phone.adb.found", { version: adb.version }) : t("phone.adb.notFound")}</Fact>
      </Row>
      <Row
        label={t("phone.devices.label")}
        sub={adb && !adb.running ? t("phone.devices.notRunningSub") : adb && android.devices.length === 0 ? t("phone.devices.noneSub") : undefined}
      >
        {adb === null ? (
          <Fact>{t("phone.devices.noAdb")}</Fact>
        ) : !adb.running ? (
          <Fact
            action={
              <Button variant="outline" size="sm" isDisabled={action.isPending} onPress={() => action.mutate({ kind: "android" })}>
                {t("phone.devices.find")}
              </Button>
            }
          >
            {action.isPending ? t("phone.devices.finding") : t("phone.devices.notSearched")}
          </Fact>
        ) : android.devices.length === 0 ? (
          <Fact>{t("phone.devices.none")}</Fact>
        ) : (
          <ul aria-label={t("phone.devices.listLabel")} className="m-0 flex list-none flex-col gap-1.5 p-0 pt-1">
            {android.devices.map((d, i) => (
              // 序列号是内核缩短过的，两台可能一样：按位置排。
              <li key={i} className="flex min-w-0 flex-wrap items-baseline gap-x-2.5 rounded-btn bg-card px-3 py-2">
                <span className="text-note font-medium text-text">{d.model ?? t("phone.devices.unnamed")}</span>
                <span className="mono text-tag text-faint">{d.serial}</span>
                <span className="text-small text-label">{deviceText(d.state)}</span>
              </li>
            ))}
          </ul>
        )}
        {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
        {android.step ? <StepBox step={android.step} label={t("phone.devices.stopLabel")} /> : null}
      </Row>
    </>
  );
}

function Methods({ phone }: { phone: PhoneView | null }) {
  const plugins = usePlugins();
  if (plugins.error) return <ErrorLine error={plugins.error} />;
  if (!plugins.data) return null;
  const methods = phoneMethods(plugins.data.plugins);
  if (methods.length === 0) return <p className="m-0 text-note text-sub">{t("phone.methods.none")}</p>;
  return (
    <ul aria-label={t("phone.methods.name")} className="m-0 flex list-none flex-col border-t border-border p-0">
      {methods.map((m) => (
        <li key={`${m.pluginId}#${m.methodId}`} className="border-b border-border">
          <Link
            to="/plugins/$platform"
            params={{ platform: m.platformId }}
            className="press -mx-2.5 my-1 grid gap-x-5 gap-y-0.5 rounded-btn px-2.5 py-2 no-underline hover:bg-chip/60 @lg:grid-cols-[minmax(0,10rem)_minmax(0,1fr)]"
          >
            <span className="min-w-0">
              <span className="block text-body font-medium text-text">{m.platform}</span>
              <span className="block text-small text-sub">
                {namedPhone(m.name, m.phone) ? m.name : t(m.phone === "android" ? "phone.methods.viaAndroid" : "phone.methods.viaIphone", { name: m.name })}
              </span>
            </span>
            <span className="min-w-0">
              <span className="block text-note text-label">
                {ACCESS_TEXT[m.access]} <span className="mono text-tag text-faint">{m.access}</span>
              </span>
              <span className="block text-small text-sub">{methodState(m, phone)}</span>
              {m.risk ? <span className="block text-small text-faint">{t("phone.methods.risk", { risk: m.risk })}</span> : null}
            </span>
          </Link>
        </li>
      ))}
    </ul>
  );
}

/** 插件给的名字里已经说了是哪种手机（插件按界面语言起名，中文的写「安卓」），就不再补一遍。 */
function namedPhone(name: string, phone: "iphone" | "android"): boolean {
  return phone === "iphone" ? name.includes("iPhone") : name.includes("Android") || name.includes(t("phone.methods.androidWord"));
}
