import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ThemePreference = "system" | "dark" | "light";

/** 强调色可选的四种，默认第一种。 */
export const ACCENTS = ["#F2B866", "#7FD1BE", "#F08C6C", "#A9B4FF"] as const;
export type Accent = (typeof ACCENTS)[number];

interface UiState {
  theme: ThemePreference;
  accent: Accent;
  setTheme: (theme: ThemePreference) => void;
  setAccent: (accent: Accent) => void;
  /** 刚收下、还没在地图上落下的条目；落下的动画放完就拿掉。 */
  landing: string[];
  markLanding: (id: string) => void;
  clearLanding: (id: string) => void;
}

export const useUiStore = create<UiState>()(
  persist(
    (set) => ({
      theme: "system",
      accent: "#F2B866",
      setTheme: (theme) => set({ theme }),
      setAccent: (accent) => set({ accent }),
      landing: [],
      markLanding: (id) => set((s) => ({ landing: [...s.landing, id] })),
      clearLanding: (id) => set((s) => ({ landing: s.landing.filter((x) => x !== id) })),
    }),
    { name: "usuyo-ui", partialize: (s) => ({ theme: s.theme, accent: s.accent }) },
  ),
);
