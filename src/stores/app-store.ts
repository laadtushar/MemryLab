import { create } from "zustand";

export type View =
  | "timeline"
  | "search"
  | "ask"
  | "insights"
  | "import"
  | "memory"
  | "entities"
  | "evolution"
  | "settings";

export type Theme = "dark" | "light";

interface AppState {
  currentView: View;
  theme: Theme;
  setView: (view: View) => void;
  toggleTheme: () => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  currentView: "timeline",
  theme: (localStorage.getItem("mp-theme") as Theme) || "dark",
  setView: (view) => set({ currentView: view }),
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
