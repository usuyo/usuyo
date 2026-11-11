import { create } from "zustand";

interface UiState {
  /** 收下页输入框里还没提交的链接，切走再回来还在。 */
  takeDraft: string;
  setTakeDraft: (value: string) => void;
}

export const useUiStore = create<UiState>((set) => ({
  takeDraft: "",
  setTakeDraft: (takeDraft) => set({ takeDraft }),
}));
