import { create } from "zustand";

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

interface AppState {
  currentView: View;
  theme: Theme;
  isUnlocked: boolean;
  isOnboarded: boolean;
  setView: (view: View) => void;
  toggleTheme: () => void;
  setUnlocked: (unlocked: boolean) => void;
  setOnboarded: (onboarded: boolean) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  currentView: "timeline",
  theme: (localStorage.getItem("mp-theme") as Theme) || "dark",
  isUnlocked: false,
  isOnboarded: true, // assume onboarded until check completes
  setView: (view) => set({ currentView: view }),
  setUnlocked: (unlocked) => set({ isUnlocked: unlocked }),
  setOnboarded: (onboarded) => set({ isOnboarded: onboarded }),
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
