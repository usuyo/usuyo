import { useState } from "react";
import { Input, TextField } from "react-aria-components";
import type { NetworkSetting, SettingsView } from "../api/endpoints";
import { useChangeSettings } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { Choice } from "../ui/Choice";
import { Row } from "./Row";
import { NETWORK_TEXT, proxyProblem } from "./words";

const MODES = (["system", "direct", "custom"] as const).map((id) => ({ id, text: NETWORK_TEXT[id].name }));

/** 网络：插件出门走哪个出口。默认跟系统代理，和浏览器同一个出口。 */
export function Network({ view }: { view: SettingsView }) {
  const change = useChangeSettings();
  const [mode, setMode] = useState<NetworkSetting["mode"]>(view.network.mode);
  const [proxy, setProxy] = useState(view.network.proxy ?? "");
  const problem = mode === "custom" ? proxyProblem(proxy) : null;
  const saved = view.network.mode === "custom" && (view.network.proxy ?? "") === proxy.trim();
  // 读回来的地址不带密码；原样交回去，内核留着存着的那个，所以不用再填。
  const withUser = saved && URL.canParse(proxy.trim()) && new URL(proxy.trim()).username !== "";
  return (
    <div className="flex flex-col gap-4">
      <Row label="插件出门" sub={NETWORK_TEXT[mode].sub}>
        <Choice
          label="插件出门走哪个出口"
          options={MODES}
          value={mode}
          onChange={(m) => {
            setMode(m);
            // 自己填的要等填好了才生效。
            if (m !== "custom") change.mutate({ network: { mode: m } });
          }}
        />
      </Row>
      {mode === "custom" ? (
        <Row label="代理地址">
          <div className="flex flex-wrap items-center gap-2">
            <TextField value={proxy} onChange={setProxy} aria-label="代理地址" className="min-w-0 flex-[1_1_240px]">
              <Input
                placeholder="http://127.0.0.1:7890"
                className="mono h-ctl-sm w-full rounded-input border border-border2 bg-transparent px-3 text-note text-text outline-none data-[focused]:border-label"
              />
            </TextField>
            <Button
              variant="soft"
              size="sm"
              isDisabled={problem !== null || saved || change.isPending}
              onPress={() => change.mutate({ network: { mode: "custom", proxy: proxy.trim() } })}
            >
              {saved ? "在用这个" : "用这个"}
            </Button>
          </div>
          {problem && proxy !== "" ? <p className="m-0 mt-1.5 text-small text-sub">{problem}</p> : null}
          {withUser ? <p className="m-0 mt-1.5 text-small text-sub">这里不显示密码。地址不动，存着的密码一直用着；改了地址，连密码一起写上。</p> : null}
        </Row>
      ) : null}
      {view.network.fakeIp ? (
        <p className="m-0 text-small text-sub">这台电脑开着假 IP 模式的代理（Clash、Surge 一类），usuyo 已经照它调好了，插件照常能连。</p>
      ) : null}
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
