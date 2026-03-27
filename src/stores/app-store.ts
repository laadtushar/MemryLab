import { create } from "zustand";
import type { ImportProgress, ImportSummary } from "@/lib/tauri";

export type View =
  | "timeline"
  | "activity"
  | "search"
  | "ask"
  | "insights"
  | "import"
  | "memory"
  | "entities"
  | "graph"
  | "evolution"
  | "logs"
  | "settings";

export type Theme = "dark" | "light";

export interface BackgroundImport {
  id: string;
  sourceName: string;
  progress: ImportProgress | null;
  summary: ImportSummary | null;
  error: string | null;
  running: boolean;
}

interface AppState {
  currentView: View;
  theme: Theme;
  isUnlocked: boolean;
  isOnboarded: boolean;
  backgroundImports: BackgroundImport[];
  setView: (view: View) => void;
  toggleTheme: () => void;
  setUnlocked: (unlocked: boolean) => void;
  setOnboarded: (onboarded: boolean) => void;
  addBackgroundImport: (bg: BackgroundImport) => void;
  updateBackgroundImport: (update: Partial<BackgroundImport>) => void;
  removeBackgroundImport: (id: string) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  currentView: "timeline",
  theme: (localStorage.getItem("mp-theme") as Theme) || "dark",
  isUnlocked: false,
  isOnboarded: true, // assume onboarded until check completes
  backgroundImports: [],
  setView: (view) => set({ currentView: view }),
  setUnlocked: (unlocked) => set({ isUnlocked: unlocked }),
  setOnboarded: (onboarded) => set({ isOnboarded: onboarded }),
  addBackgroundImport: (bg) => set((s) => ({ backgroundImports: [...s.backgroundImports, bg] })),
  updateBackgroundImport: (update) => {
    // Update the most recent running import (progress events don't carry an ID)
    set((s) => {
      const imports = [...s.backgroundImports];
      let idx = -1;
      for (let j = imports.length - 1; j >= 0; j--) {
        if (imports[j].running) { idx = j; break; }
      }
      if (idx >= 0) imports[idx] = { ...imports[idx], ...update };
      return { backgroundImports: imports };
    });
  },
  removeBackgroundImport: (id) => set((s) => ({
    backgroundImports: s.backgroundImports.filter((i) => i.id !== id),
  })),
  toggleTheme: () => {
    const next = get().theme === "dark" ? "light" : "dark";
    localStorage.setItem("mp-theme", next);
    document.documentElement.classList.toggle("light", next === "light");
    set({ theme: next });
  },
}));

// Apply saved theme on load
const saved = localStorage.getItem("mp-theme");
if (saved === "light") {
  document.documentElement.classList.add("light");
}
