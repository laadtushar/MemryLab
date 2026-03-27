import { create } from "zustand";
import type { ImportProgress } from "@/lib/tauri";

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

export type BackgroundTaskType = "import" | "analysis" | "embeddings";

export interface BackgroundTask {
  id: string;
  type: BackgroundTaskType;
  label: string;
  progress: ImportProgress | null;
  result: string | null;
  error: string | null;
  running: boolean;
}

interface AppState {
  currentView: View;
  theme: Theme;
  isUnlocked: boolean;
  isOnboarded: boolean;
  backgroundTasks: BackgroundTask[];
  quickSearchOpen: boolean;
  setView: (view: View) => void;
  setQuickSearchOpen: (open: boolean) => void;
  toggleTheme: () => void;
  setUnlocked: (unlocked: boolean) => void;
  setOnboarded: (onboarded: boolean) => void;
  addTask: (task: BackgroundTask) => void;
  updateTask: (id: string, update: Partial<BackgroundTask>) => void;
  updateTaskByProgress: (progress: ImportProgress) => void;
  removeTask: (id: string) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  currentView: "timeline",
  theme: (localStorage.getItem("mp-theme") as Theme) || "dark",
  isUnlocked: false,
  isOnboarded: true, // assume onboarded until check completes
  backgroundTasks: [],
  quickSearchOpen: false,
  setView: (view) => set({ currentView: view }),
  setQuickSearchOpen: (open) => set({ quickSearchOpen: open }),
  setUnlocked: (unlocked) => set({ isUnlocked: unlocked }),
  setOnboarded: (onboarded) => set({ isOnboarded: onboarded }),
  addTask: (task) => set((s) => ({ backgroundTasks: [...s.backgroundTasks, task] })),
  updateTask: (id, update) => set((s) => ({
    backgroundTasks: s.backgroundTasks.map((t) => t.id === id ? { ...t, ...update } : t),
  })),
  updateTaskByProgress: (progress) => {
    set((s) => {
      const tasks = [...s.backgroundTasks];
      let idx = tasks.findIndex((t) => t.id === progress.import_id);
      if (idx < 0) {
        for (let j = tasks.length - 1; j >= 0; j--) {
          if (tasks[j].running) { idx = j; break; }
        }
      }
      if (idx >= 0) tasks[idx] = { ...tasks[idx], progress };
      return { backgroundTasks: tasks };
    });
  },
  removeTask: (id) => set((s) => ({
    backgroundTasks: s.backgroundTasks.filter((t) => t.id !== id),
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
