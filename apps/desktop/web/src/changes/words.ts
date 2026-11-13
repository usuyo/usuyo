// 更改记录的几句话：谁、在哪台电脑、做了什么、这一批现在怎样。只说发生过的事，不计数。
import type { ChangeActor, ChangeBatch, ChangeOp } from "../api/endpoints";
import { t } from "../i18n";
import { formatClockTime } from "../i18n/format";
import { dayText } from "../item/format";

export function actorText(by: ChangeActor): string {
  switch (by.kind) {
    case "you":
      return t("changes.actor.you");
    case "agent":
      return by.name;
    case "readback":
      return t("changes.actor.readback", { target: by.target });
    default:
      return t("changes.actor.sync");
  }
}

/** 一处做了什么：「添加了「这周的豆子」」「将「Portra 400」添加到「胶片摄影」」「编辑了「咖啡」中的一段」。 */
export function opText(o: ChangeOp): string {
  const title = o.target.title;
  switch (o.op) {
    case "add-to-topic":
      return o.after ? t("changes.op.addToTopic", { title, topic: o.after }) : t("changes.op.addToSomeTopic", { title });
    case "edit-page":
      return t("changes.op.editPage", { title });
    case "add-citation":
      return t("changes.op.addCitation", { title });
    case "set-note":
      return t("changes.op.setNote", { title });
    case "keep-text":
      return t("changes.op.keepText", { title });
    case "restore":
      return t("changes.op.restore", { title });
    default:
      return t("changes.op.addItem", { title });
  }
}

export function stateText(b: Pick<ChangeBatch, "state">): string {
  switch (b.state) {
    case "proposed":
      return t("changes.state.proposed");
    case "declined":
      return t("changes.state.declined");
    case "undone":
      return t("changes.state.undone");
    default:
      return t("changes.state.applied");
  }
}

/** 「今天」「昨天」「11月9日」。 */
export function dayHeading(iso: string, now = new Date()): string {
  const d = new Date(iso);
  const start = (x: Date) => new Date(x.getFullYear(), x.getMonth(), x.getDate()).getTime();
  const days = Math.round((start(now) - start(d)) / 86_400_000);
  return days === 0 ? t("common.today") : days === 1 ? t("common.yesterday") : dayText(iso);
}

/** 按「谁」挑着看。 */
export type Who = "all" | "you" | "agent" | "other";

export function byWho(b: ChangeBatch, who: Who): boolean {
  if (who === "all") return true;
  if (who === "other") return b.by.kind === "readback" || b.by.kind === "sync";
  return b.by.kind === who;
}

/** 时分，按当前语言（「14:20」「2:20 PM」）：日子写在上面那一行。 */
export function hm(iso: string): string {
  return formatClockTime(iso);
}
