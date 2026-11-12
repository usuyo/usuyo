import { Copy } from "lucide-react";
import { useState } from "react";
import { Input, Label, ListBox, ListBoxItem, Popover, Select, SelectValue, TextField, Button as AriaButton } from "react-aria-components";
import type { DeliverClientKind, NewDeliverToken } from "../api/endpoints";
import { useCreateDeliverToken, useDeliverTokens, useInbox, useRevokeDeliverToken } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { dayText } from "../item/format";
import { Button } from "../ui/Button";
import { FlashLine, useFlash } from "../ui/Flash";

const KIND_TEXT: Record<DeliverClientKind, string> = {
  extension: "浏览器扩展或脚本",
  mobile: "手机",
  api: "别的程序",
};
const KINDS: DeliverClientKind[] = ["extension", "mobile", "api"];

/** 发给脚本、扩展、手机的投递令牌：只能往 usuyo 交，读不了资料库；交已取到内容的文件只认扩展和脚本的。新的只显示一次。 */
export function DeliverTokens() {
  const tokens = useDeliverTokens();
  const inbox = useInbox();
  const create = useCreateDeliverToken();
  const revoke = useRevokeDeliverToken();
  const [name, setName] = useState("");
  const [kind, setKind] = useState<DeliverClientKind>("extension");
  // 令牌只在发的这一次回来：放在这里，不进缓存，离开这一页就没了。
  const [fresh, setFresh] = useState<NewDeliverToken | null>(null);
  const flash = useFlash();

  function issue() {
    const n = name.trim();
    if (n === "") return;
    create.mutate(
      { name: n, kind },
      {
        onSuccess: (t) => {
          setFresh(t);
          setName("");
          // 回的令牌也别留在请求的记录里。
          create.reset();
        },
      },
    );
  }

  return (
    <section aria-label="交进来的清单" className="mt-10">
      <h2 className="m-0 text-section font-semibold text-text">交进来的清单</h2>
      <p className="m-0 mt-1 text-small text-sub">
        浏览器里的脚本或扩展、手机，可以把一批链接交给 usuyo；扩展和脚本还能交一个装着已经读到的内容的文件。每一批在右边单独一条，一样先给你看再收。每个用自己的令牌，只能交，读不了你的资料库；撤销马上生效。
      </p>

      <div className="mt-3.5 flex flex-wrap items-end gap-2">
        <TextField value={name} onChange={setName} aria-label="给它起个名字" className="w-[260px] max-w-full">
          <Label className="mb-1 block text-small text-faint">给它起个名字</Label>
          <Input
            placeholder="例如 收藏页的脚本"
            onKeyDown={(e) => {
              if (e.key === "Enter") issue();
            }}
            className="h-ctl w-full rounded-input border border-border2 bg-transparent px-3 text-body text-text outline-none focus:border-label/70"
          />
        </TextField>
        <Select selectedKey={kind} onSelectionChange={(k) => setKind(KINDS.find((x) => x === k) ?? "extension")} aria-label="用在哪" className="w-[180px]">
          <Label className="mb-1 block text-small text-faint">用在哪</Label>
          <AriaButton className="press flex h-ctl w-full cursor-pointer items-center rounded-input border border-border2 bg-transparent px-3 text-left text-body text-text outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent">
            <SelectValue />
          </AriaButton>
          <Popover className="w-[180px] rounded-btn border border-border bg-card p-1 shadow-panel">
            <ListBox className="outline-none">
              {KINDS.map((k) => (
                <ListBoxItem
                  key={k}
                  id={k}
                  className="cursor-pointer rounded-sm px-2.5 py-1.5 text-note text-label outline-none data-[focused]:bg-chip data-[selected]:text-text"
                >
                  {KIND_TEXT[k]}
                </ListBoxItem>
              ))}
            </ListBox>
          </Popover>
        </Select>
        <Button variant="soft" size="md" onPress={issue} isDisabled={create.isPending || name.trim() === ""}>
          发一个令牌
        </Button>
      </div>
      {create.error ? <ErrorLine error={create.error} className="mt-2" /> : null}

      {fresh ? (
        <div className="mt-3 animate-rise rounded-card border border-border2 bg-card px-4 py-3.5">
          <div className="text-note text-label">给「{fresh.name}」的令牌，只显示这一次：抄到脚本里。关掉以后就看不到了，丢了就撤销再发一个。</div>
          <div className="mt-2 flex items-center gap-2">
            <code className="mono min-w-0 flex-1 truncate rounded-sm bg-chip px-3 py-2 text-tag text-text select-all">{fresh.token}</code>
            <Button
              variant="outline"
              size="sm"
              square
              aria-label="复制令牌"
              onPress={() => {
                navigator.clipboard.writeText(fresh.token).then(
                  () => flash.show("复制好了"),
                  () => flash.show("没能复制，手动选中抄一下"),
                );
              }}
            >
              <Copy size={15} aria-hidden="true" />
            </Button>
            <Button variant="quiet" size="sm" onPress={() => setFresh(null)}>
              抄好了
            </Button>
          </div>
          <FlashLine flash={flash} className="mt-1 block text-small text-sub" />
        </div>
      ) : null}

      {tokens.error ? <ErrorLine error={tokens.error} className="mt-3" /> : null}
      {tokens.data && tokens.data.length > 0 ? (
        <ul className="m-0 mt-4 flex list-none flex-col border-t border-border p-0">
          {tokens.data.map((t) => (
            <li key={t.id} className="flex items-center gap-3 border-b border-border py-2">
              <span className="min-w-0 flex-1 truncate text-body text-text">{t.name}</span>
              <span className="text-small text-sub">{KIND_TEXT[t.kind]}</span>
              <span className="mono text-tag text-faint">{dayText(t.createdAt)}发的</span>
              <Button variant="quiet" size="xs" isDisabled={revoke.isPending} onPress={() => revoke.mutate(t.id)}>
                撤销
              </Button>
            </li>
          ))}
        </ul>
      ) : null}
      {revoke.error ? <ErrorLine error={revoke.error} className="mt-2" /> : null}

      {inbox.data ? (
        <p className="m-0 mt-4 text-small text-sub">
          手机上分享出来的链接：存成 .txt 放进这个文件夹（经云盘同步也行），一行一条，收完会挪到里面的 .usuyo-done。
          <span className="mono mt-1 block text-tag text-label select-text">{inbox.data.path}</span>
        </p>
      ) : null}
    </section>
  );
}
