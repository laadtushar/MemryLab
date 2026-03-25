import {
  Clock,
  Search,
  MessageCircle,
  Lightbulb,
  Import,
  Brain,
  Network,
  TrendingUp,
  Settings,
  Sun,
  Moon,
} from "lucide-react";
import { useAppStore, type View } from "@/stores/app-store";

const navItems: { view: View; label: string; icon: React.ReactNode }[] = [
  { view: "timeline", label: "Timeline", icon: <Clock size={20} /> },
  { view: "search", label: "Search", icon: <Search size={20} /> },
  { view: "ask", label: "Ask", icon: <MessageCircle size={20} /> },
  { view: "insights", label: "Insights", icon: <Lightbulb size={20} /> },
  { view: "evolution", label: "Evolution", icon: <TrendingUp size={20} /> },
  { view: "import", label: "Import", icon: <Import size={20} /> },
  { view: "memory", label: "Memory", icon: <Brain size={20} /> },
  { view: "entities", label: "Entities", icon: <Network size={20} /> },
  { view: "settings", label: "Settings", icon: <Settings size={20} /> },
];

export function Sidebar() {
  const { currentView, setView, theme, toggleTheme } = useAppStore();

  return (
    <aside className="flex h-full w-16 flex-col items-center border-r border-border bg-sidebar py-4 gap-1">
      <div className="mb-6 flex h-10 w-10 items-center justify-center rounded-lg bg-primary text-primary-foreground font-bold text-lg">
        M
      </div>
      {navItems.map(({ view, label, icon }) => (
        <button
          key={view}
          onClick={() => setView(view)}
          className={`flex h-12 w-12 flex-col items-center justify-center rounded-lg text-xs transition-colors ${
            currentView === view
              ? "bg-accent text-accent-foreground"
              : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
          }`}
          title={label}
        >
          {icon}
          <span className="mt-0.5 text-[10px]">{label}</span>
        </button>
      ))}
      <div className="flex-1" />
      <button
        onClick={toggleTheme}
        className="flex h-10 w-10 items-center justify-center rounded-lg text-muted-foreground hover:bg-accent/50 hover:text-foreground transition-colors"
        title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
      >
        {theme === "dark" ? <Sun size={18} /> : <Moon size={18} />}
      </button>
    </aside>
  );
}
