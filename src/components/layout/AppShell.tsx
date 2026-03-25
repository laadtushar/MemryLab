import { Sidebar } from "./Sidebar";
import { useAppStore } from "@/stores/app-store";
import { TimelineView } from "@/components/timeline/TimelineView";
import { SearchInterface } from "@/components/search/SearchInterface";
import { AskView } from "@/components/ask/AskView";
import { InsightFeed } from "@/components/insights/InsightFeed";
import { ImportWizard } from "@/components/import/ImportWizard";
import { MemoryBrowser } from "@/components/memory/MemoryBrowser";
import { SettingsPage } from "@/components/settings/SettingsPage";

const viewComponents: Record<string, React.ReactNode> = {
  timeline: <TimelineView />,
  search: <SearchInterface />,
  ask: <AskView />,
  insights: <InsightFeed />,
  import: <ImportWizard />,
  memory: <MemoryBrowser />,
  settings: <SettingsPage />,
};

export function AppShell() {
  const currentView = useAppStore((s) => s.currentView);

  return (
    <div className="flex h-screen w-screen bg-background">
      <Sidebar />
      <main className="flex-1 overflow-hidden">
        {viewComponents[currentView]}
      </main>
    </div>
  );
}
