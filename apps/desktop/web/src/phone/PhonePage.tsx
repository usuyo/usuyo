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
import { Button } from "../ui/Button";
import { FlashLine, useFlash } from "../ui/Flash";
import { clipboardText, DEVICE_TEXT, GRANT_TEXT, iosMissing, methodState, needsRegister, phoneMethods } from "./words";

const SECTIONS = [
  { id: "iphone", name: "iPhone" },
  { id: "android", name: "安卓" },
  { id: "methods", name: "用到手机的取法" },
] as const;

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
      <Rail label="主导航" />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto scroll-smooth px-6 pt-7 pb-24 motion-reduce:scroll-auto lg:px-gutter">
        <h1 className="m-0 text-title font-semibold text-text">手机</h1>
        <p className="m-0 mt-1 max-w-[720px] text-note text-sub">
          usuyo 看着镜像到这台 Mac 上的手机屏幕，在你自己存过的列表里只点「分享」和「复制链接」。链接经通用剪贴板回到这台 Mac，内容再用不登录的取法去取。
        </p>
        {missing ? (
          <NotServedLine className="mt-4">这台电脑上的 usuyo 内核还不管手机；以后在这里替你装上手机的驱动，看授权和手机连没连上，装好以后也在这里管。</NotServedLine>
        ) : null}
        <div className="mt-8 grid items-start gap-x-10 @3xl:grid-cols-[160px_minmax(0,1fr)]">
          <nav aria-label="手机的几节" className="sticky top-0 hidden flex-col gap-0.5 @3xl:flex">
            {SECTIONS.filter((s) => !missing || s.id === "methods").map((s) => (
              <a
                key={s.id}
                href={`#${s.id}`}
                className="press flex h-ctl-xs items-center rounded-sm px-2.5 text-note text-sub no-underline hover:bg-chip/60 hover:text-text"
              >
                {s.name}
              </a>
            ))}
          </nav>
          <div className="flex min-w-0 max-w-[760px] flex-col">
            {missing ? null : (
              <>
                <Section id="iphone" name="iPhone" sub="经这台 Mac 上的 iPhone 镜像。手机锁着、放在旁边就行，不用拿在手里。">
                  {gate((v) => (
                    <Ios ios={v.ios} />
                  ))}
                </Section>
                <Section id="android" name="安卓" sub="经 USB 和 adb。手机上不用装 App，也不开无障碍服务。">
                  {gate((v) => (
                    <Android android={v.android} />
                  ))}
                </Section>
              </>
            )}
            <Section id="methods" name="用到手机的取法" sub="账号留在你手机的 App 里，usuyo 不拿任何凭据。档位和记下的风险只是给你挑的，不拦什么。">
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
      <Row label="系统">
        <Fact>{ios.system.mirroring ? `macOS ${ios.system.macos}，有 iPhone 镜像。` : `macOS ${ios.system.macos}，没有 iPhone 镜像：要 macOS 15 或更新的。`}</Fact>
      </Row>
      <Driver ios={ios} />
      <Grant
        label="屏幕录制"
        state={ios.grants.screen}
        pane="screen-recording"
        present={present}
        unregistered={unregistered}
        to={ios.grants.to}
        why={`让「${ios.grants.to}」截下镜像窗口，认出上面的字。`}
      />
      <Grant
        label="辅助功能"
        state={ios.grants.accessibility}
        pane="accessibility"
        present={present}
        unregistered={unregistered}
        to={ios.grants.to}
        why={`让「${ios.grants.to}」替你点按和滑动。`}
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
  const from = offer ? (
    <>
      mirroir 是开源的手机驱动（Apache-2.0）。从它的
      <AriaButton
        onPress={() => void openExternal(offer.source)}
        className="mx-0.5 cursor-pointer border-0 bg-transparent p-0 font-sans text-small text-label underline decoration-border2 underline-offset-4 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text"
      >
        官方 release
      </AriaButton>
      取 v{offer.version}，约 {sizeText(offer.size)}，和官方的 SHA256SUMS 对上了才放进这台电脑自己的目录。装好以后，让它常驻的那一步要你在「终端」里做。
    </>
  ) : null;
  const serving = `它以「${ios.grants.to}」常驻在这台电脑上，只听本机。usuyo 没替它写权限文件，它只开看屏幕的工具，不点、不打字。`;
  const sub =
    d.state === "present"
      ? `${d.step ? "" : serving}卸掉只删 usuyo 放进这台电脑目录里的这一份：在跑的「${ios.grants.to}」不停，它的登录项和证书也留着。`
      : d.state === "installing"
        ? "装好以后，让它常驻的那一步要你在「终端」里做。"
        : from;
  const busy = action.isPending;
  // 登记那一步跑完以后这台 Mac 上多出来的东西；换新、停掉的那两步不加这一句。
  const registers = needsRegister(ios);
  return (
    <Row label="驱动" sub={sub}>
      {d.state === "absent" ? (
        <Fact
          action={
            <Button variant="soft" size="sm" isDisabled={busy} onPress={() => action.mutate({ kind: "install" })}>
              装上
            </Button>
          }
        >
          还没装 mirroir。
        </Fact>
      ) : d.state === "installing" ? (
        <Fact>在装 mirroir{offer ? ` v${offer.version}` : ""}。</Fact>
      ) : d.state === "failed" ? (
        <Fact
          warn
          action={
            <Button variant="soft" size="sm" isDisabled={busy} onPress={() => action.mutate({ kind: "install" })}>
              再装一次
            </Button>
          }
        >
          没装上：{d.hint ?? "原因没写"}
        </Fact>
      ) : (
        <Fact
          action={
            <>
              {offer ? (
                <Button variant="soft" size="sm" isDisabled={busy} onPress={() => action.mutate({ kind: "update" })}>
                  更新
                </Button>
              ) : null}
              <Button variant="quiet" size="sm" isDisabled={busy} onPress={() => action.mutate({ kind: "remove" })}>
                卸掉
              </Button>
            </>
          }
        >
          装着 mirroir v{d.version}
          {offer ? `，有新的 v${offer.version}` : ""}。
        </Fact>
      )}
      {d.state === "present" && d.hint ? <p className="m-0 mt-1 text-note text-warn">没换成新的，装着的这一份还在：{d.hint}</p> : null}
      {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
      {d.step ? (
        <StepBox
          step={d.step}
          label="要你在终端里做的一步"
          after={registers ? "跑完以后，这台 Mac 上多两样：登录钥匙串里一张 mirroir 自己生成的签名证书（钥匙串会问你，选「始终允许」），和一个登录时启动的后台项。" : undefined}
        />
      ) : null}
    </Row>
  );
}

/** 内核给的一步：它的那句话，一条命令一行，各带「复制」。usuyo 不替你跑。 */
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
              aria-label={`复制第 ${i + 1} 条命令`}
              onPress={() => {
                navigator.clipboard.writeText(c).then(
                  () => flash.show("复制好了"),
                  () => flash.show("没能复制，手动选中抄一下"),
                );
              }}
            >
              <Copy size={14} aria-hidden="true" />
              复制
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
  why,
}: {
  label: string;
  state: GrantState;
  pane: SettingsPane;
  present: boolean;
  unregistered: boolean;
  to: string;
  why: string;
}) {
  const action = usePhoneAction();
  return (
    <Row label={label} sub={`${why}授权给的是「${to}」，不是 usuyo；这一下只有你自己能在系统设置里打开。`}>
      {unregistered ? (
        <Fact>先做完「驱动」那一行要你在终端里做的那一步，系统设置里才有「{to}」。</Fact>
      ) : present ? (
        <Fact
          action={
            state === "given" ? null : (
              <Button variant="outline" size="sm" isDisabled={action.isPending} onPress={() => action.mutate({ kind: "open", pane })}>
                打开系统设置
              </Button>
            )
          }
        >
          {GRANT_TEXT[state]}
          {state === "given" ? "" : `在系统设置里找到「${to}」，把开关打开。`}
        </Fact>
      ) : (
        <Fact>装上 mirroir 以后再给。</Fact>
      )}
      {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
    </Row>
  );
}

function Mirroring({ ios }: { ios: PhoneIos }) {
  const action = usePhoneAction();
  if (!ios.system.mirroring) {
    return (
      <Row label="iPhone 镜像">
        <Fact>这台 Mac 用不了 iPhone 镜像。</Fact>
      </Row>
    );
  }
  const sub = {
    "not-set-up": "打开它；第一次打开时照它说的做：iPhone 是 iOS 18 或更新的、设了密码，两边登同一个 Apple 账户，蓝牙和 Wi-Fi 都开着。",
    away: "把 iPhone 锁上，放在这台 Mac 旁边就会连上。你拿起手机解锁，镜像就断开。",
    connected: "你拿起手机解锁，镜像就断开。usuyo 只在你让它取的时候用。",
  }[ios.mirroring];
  return (
    <Row label="iPhone 镜像" sub={sub}>
      {ios.mirroring === "not-set-up" ? (
        <Fact
          action={
            <Button variant="outline" size="sm" isDisabled={action.isPending} onPress={() => action.mutate({ kind: "open", pane: "iphone-mirroring" })}>
              打开 iPhone 镜像
            </Button>
          }
        >
          iPhone 镜像没开着，或者这台 Mac 上还没设置过它。
        </Fact>
      ) : (
        <Fact>{ios.mirroring === "away" ? "iPhone 没连上：手机解锁着，或者不在旁边。" : "iPhone 连着。"}</Fact>
      )}
      {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
    </Row>
  );
}

function Clipboard({ ios }: { ios: PhoneIos }) {
  const action = usePhoneAction();
  const t = ios.clipboard;
  const sub =
    t?.state === "unchanged"
      ? "看一下这几样：iPhone 和这台 Mac 都开着「接力」，登的是同一个 Apple 账户，蓝牙和 Wi-Fi 都开着。"
      : "点「试一下」以后，在 iPhone 上随便拷贝一段字。usuyo 只看这台 Mac 的剪贴板变没变，拷的字不显示、不留下。";
  return (
    <Row label="通用剪贴板" sub={sub}>
      <Fact
        action={
          t?.state === "waiting" ? null : (
            <Button variant="outline" size="sm" isDisabled={action.isPending} onPress={() => action.mutate({ kind: "clipboard" })}>
              {t ? "再试一下" : "试一下"}
            </Button>
          )
        }
      >
        {clipboardText(t)}
      </Fact>
      {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
    </Row>
  );
}

function DryRun({ ios }: { ios: PhoneIos }) {
  const run = useDryRun();
  const missing = iosMissing(ios);
  return (
    <Row label="看一眼" sub="usuyo 读这一屏，说它认出这是哪个流程的哪一步、该有的字对上了几段、要点的字找没找到，什么都不点。读到的字和截下的图都不存。">
      {missing.length > 0 ? (
        <Fact>先要{missing.join("、")}。</Fact>
      ) : (
        <Fact
          action={
            <Button variant="soft" size="sm" isDisabled={run.isPending} onPress={() => run.mutate()}>
              看一眼，不点
            </Button>
          }
        >
          {run.isPending ? "在读这一屏。" : "把 iPhone 上的 App 翻到要看的那一屏，再点这里。"}
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
        `像是「${m.flow}」的第 ${m.step} 步。`,
        m.expect.matched === m.expect.total ? `这一步该有的 ${m.expect.total} 段字都对上了。` : `这一步该有的 ${m.expect.total} 段字，对上了 ${m.expect.matched} 段。`,
        ...m.targets.map((t) => `要点的「${t.label}」${t.found ? "找到了" : "没找到"}。`),
      ]
    : [`认不出这一屏：读到 ${report.read} 段字，对不上哪个流程的哪一步。`];
  return (
    <ul aria-label="看一眼的结果" className="m-0 mt-2 flex list-none flex-col gap-1 rounded-btn bg-card px-3 py-2.5">
      {lines.map((l) => (
        <li key={l} className="text-note text-label">
          {l}
        </li>
      ))}
      <li className="text-small text-sub">什么都没点。</li>
    </ul>
  );
}

function Android({ android }: { android: PhoneAndroid }) {
  const action = usePhoneAction();
  const adb = android.adb;
  return (
    <>
      <Row
        label="adb"
        sub={
          adb ? (
            <span className="mono break-all text-tag">{adb.path}</span>
          ) : (
            <>
              装上 Android 的平台工具就有：
              <code className="mono mt-1.5 block rounded-sm bg-chip px-3 py-2 text-tag text-text select-text">brew install android-platform-tools</code>
            </>
          )
        }
      >
        <Fact>{adb ? `找到了 adb ${adb.version}。` : "没找到 adb。"}</Fact>
      </Row>
      <Row
        label="手机"
        sub={
          adb && !adb.running
            ? "要找的时候 adb 会起它自己的服务，之后留在这台电脑的后台，直到你停掉它。usuyo 只在你点这里时起它。"
            : adb && android.devices.length === 0
              ? "用 USB 连上，在手机的「开发者选项」里打开「USB 调试」。"
              : undefined
        }
      >
        {adb === null ? (
          <Fact>装上 adb 以后，这里列出连着的安卓手机。</Fact>
        ) : !adb.running ? (
          <Fact
            action={
              <Button variant="outline" size="sm" isDisabled={action.isPending} onPress={() => action.mutate({ kind: "android" })}>
                找一下连着的手机
              </Button>
            }
          >
            {action.isPending ? "在找连着的安卓手机。" : "还没找过连着的安卓手机。"}
          </Fact>
        ) : android.devices.length === 0 ? (
          <Fact>没有连着的安卓手机。</Fact>
        ) : (
          <ul aria-label="连着的安卓手机" className="m-0 flex list-none flex-col gap-1.5 p-0 pt-1">
            {android.devices.map((d, i) => (
              // 序列号是内核缩短过的，两台可能一样：按位置排。
              <li key={i} className="flex min-w-0 flex-wrap items-baseline gap-x-2.5 rounded-btn bg-card px-3 py-2">
                <span className="text-note font-medium text-text">{d.model ?? "安卓手机"}</span>
                <span className="mono text-tag text-faint">{d.serial}</span>
                <span className="text-small text-label">{DEVICE_TEXT[d.state]}</span>
              </li>
            ))}
          </ul>
        )}
        {action.error ? <ErrorLine error={action.error} className="mt-1" /> : null}
        {android.step ? <StepBox step={android.step} label="怎么停掉 adb 的服务" /> : null}
      </Row>
    </>
  );
}

function Methods({ phone }: { phone: PhoneView | null }) {
  const plugins = usePlugins();
  if (plugins.error) return <ErrorLine error={plugins.error} />;
  if (!plugins.data) return null;
  const methods = phoneMethods(plugins.data.plugins);
  if (methods.length === 0) return <p className="m-0 text-note text-sub">装着的获取插件里还没有用到手机的取法。</p>;
  return (
    <ul aria-label="用到手机的取法" className="m-0 flex list-none flex-col border-t border-border p-0">
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
                {m.name}
                {m.name.includes(m.phone === "android" ? "安卓" : "iPhone") ? "" : ` · ${m.phone === "android" ? "安卓" : "iPhone 镜像"}`}
              </span>
            </span>
            <span className="min-w-0">
              <span className="block text-note text-label">
                {ACCESS_TEXT[m.access]} <span className="mono text-tag text-faint">{m.access}</span>
              </span>
              <span className="block text-small text-sub">{methodState(m, phone)}</span>
              {m.risk ? <span className="block text-small text-faint">记下的：{m.risk}</span> : null}
            </span>
          </Link>
        </li>
      ))}
    </ul>
  );
}
