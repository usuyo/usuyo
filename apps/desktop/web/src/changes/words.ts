// 改动记录的几句话：谁、在哪台电脑、做了什么、这一批现在怎样。只说发生过的事，不计数。
import type { ChangeActor, ChangeBatch, ChangeOp } from "../api/endpoints";
import { dayText } from "../item/format";

export function actorText(by: ChangeActor): string {
  switch (by.kind) {
    case "you":
      return "你";
    case "agent":
      return by.name;
    case "readback":
      return `从 ${by.target} 读回`;
    default:
      return "同步来的";
  }
}

const OP_TEXT: Record<ChangeOp["op"], string> = {
  "add-item": "收下",
  "set-note": "记一句",
  "edit-page": "改一段",
  "add-citation": "加一处引用",
  "add-to-topic": "放进主题",
  "keep-text": "只留文字",
  restore: "拿回来",
};

/** 一处做了什么：「收下「这周的豆子」」「把「Portra 400」放进「胶片摄影」」「在「咖啡」里改一段」。 */
export function opText(o: ChangeOp): string {
  const t = `「${o.target.title}」`;
  switch (o.op) {
    case "add-to-topic":
      return o.after ? `把${t}放进「${o.after}」` : `把${t}放进主题`;
    case "edit-page":
    case "add-citation":
      return `在${t}里${OP_TEXT[o.op]}`;
    case "set-note":
      return `给${t}${OP_TEXT[o.op]}`;
    default:
      return `${OP_TEXT[o.op]}${t}`;
  }
}

export function stateText(b: Pick<ChangeBatch, "state">): string {
  switch (b.state) {
    case "proposed":
      return "改动提议：你接受了才生效";
    case "declined":
      return "都不要了，资料库没变";
    case "undone":
      return "撤销了";
    default:
      return "生效了";
  }
}

/** 「今天」「昨天」「11 月 9 日」。 */
export function dayHeading(iso: string, now = new Date()): string {
  const d = new Date(iso);
  const start = (x: Date) => new Date(x.getFullYear(), x.getMonth(), x.getDate()).getTime();
  const days = Math.round((start(now) - start(d)) / 86_400_000);
  return days === 0 ? "今天" : days === 1 ? "昨天" : dayText(iso);
}

/** 按「谁」挑着看。 */
export type Who = "all" | "you" | "agent" | "other";

export function byWho(b: ChangeBatch, who: Who): boolean {
  if (who === "all") return true;
  if (who === "other") return b.by.kind === "readback" || b.by.kind === "sync";
  return b.by.kind === who;
}

/** 「14:20」：日子写在上面那一行。 */
export function hm(iso: string): string {
  const d = new Date(iso);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}
