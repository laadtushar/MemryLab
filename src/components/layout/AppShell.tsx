import { Sidebar } from "./Sidebar";
import { useAppStore } from "@/stores/app-store";
import { ErrorBoundary } from "@/components/shared/ErrorBoundary";
import { TimelineView } from "@/components/timeline/TimelineView";
import { SearchInterface } from "@/components/search/SearchInterface";
import { AskView } from "@/components/ask/AskView";
import { InsightFeed } from "@/components/insights/InsightFeed";
import { ImportWizard } from "@/components/import/ImportWizard";
import { MemoryBrowser } from "@/components/memory/MemoryBrowser";
import { EntityExplorer } from "@/components/entities/EntityExplorer";
import { GraphExplorer } from "@/components/graph/GraphExplorer";
import { EvolutionExplorer } from "@/components/evolution/EvolutionExplorer";
import { SettingsPage } from "@/components/settings/SettingsPage";

const viewComponents: Record<string, { component: React.ReactNode; label: string }> = {
  timeline: { component: <TimelineView />, label: "Timeline" },
  search: { component: <SearchInterface />, label: "Search" },
  ask: { component: <AskView />, label: "Ask" },
  insights: { component: <InsightFeed />, label: "Insights" },
  import: { component: <ImportWizard />, label: "Import" },
  memory: { component: <MemoryBrowser />, label: "Memory" },
  entities: { component: <EntityExplorer />, label: "Entities" },
  graph: { component: <GraphExplorer />, label: "Graph" },
  evolution: { component: <EvolutionExplorer />, label: "Evolution" },
  settings: { component: <SettingsPage />, label: "Settings" },
};

export function AppShell() {
  const currentView = useAppStore((s) => s.currentView);
  const view = viewComponents[currentView];

  return (
    <div className="flex h-screen w-screen bg-background">
      <Sidebar />
      <main className="flex-1 overflow-hidden">
        <ErrorBoundary fallbackTitle={`${view?.label ?? "View"} encountered an error`}>
          {view?.component}
        </ErrorBoundary>
      </main>
    </div>
  );
}
