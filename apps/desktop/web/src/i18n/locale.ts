// 界面用哪种语言：设置里选的（存在内核按机器的设置里），「跟随系统」时看系统的首选语言。
// 内核的设置到达以前，先用这台电脑上次记下的选择，免得一打开先闪一下另一种语言；内核没在运行时也照它显示。
import { create } from "zustand";

export const LOCALES = ["zh-Hans", "en"] as const;
export type Locale = (typeof LOCALES)[number];
export type LanguageChoice = "system" | Locale;

const REMEMBERED = "usuyo.language";

/** 系统首选的语言里第一个是中文就用简体中文，别的都用英文。 */
export function systemLocale(languages: readonly string[] = navigator.languages): Locale {
  const first = languages[0] ?? navigator.language;
  return first.toLowerCase().startsWith("zh") ? "zh-Hans" : "en";
}

export function resolve(choice: LanguageChoice): Locale {
  return choice === "system" ? systemLocale() : choice;
}

function remembered(): LanguageChoice {
  try {
    const v = localStorage.getItem(REMEMBERED);
    return v === "zh-Hans" || v === "en" || v === "system" ? v : "system";
  } catch {
    // 存不了（无痕窗口、关了存储）：就跟随系统。
    return "system";
  }
}

interface LocaleState {
  choice: LanguageChoice;
  locale: Locale;
}

export const useLocaleStore = create<LocaleState>(() => {
  const choice = remembered();
  return { choice, locale: resolve(choice) };
});

/** 换语言：马上生效，记在这台电脑上，下次打开先用它。 */
export function setLanguageChoice(choice: LanguageChoice): void {
  try {
    localStorage.setItem(REMEMBERED, choice);
  } catch {
    // 记不下只影响下次打开的第一眼，内核的设置一到就对了。
  }
  useLocaleStore.setState({ choice, locale: resolve(choice) });
}

export function currentLocale(): Locale {
  return useLocaleStore.getState().locale;
}

export function useLocale(): Locale {
  return useLocaleStore((s) => s.locale);
}
