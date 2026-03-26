import { useEffect, useState } from "react";
import { commands, type MemoryFactResponse } from "@/lib/tauri";
import { Loader2, BookOpen } from "lucide-react";

interface NarrativeItem {
  id: string;
  theme: string;
  body: string;
  date: string;
}

export function NarrativeView() {
  const [narratives, setNarratives] = useState<NarrativeItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    commands
      .getMemoryFacts("insight")
      .then((facts: MemoryFactResponse[]) => {
        const parsed: NarrativeItem[] = facts
          .filter((f) => f.fact_text.startsWith("Narrative:"))
          .map((f) => {
            // Format: "Narrative: Theme\n\nBody text..."
            const withoutPrefix = f.fact_text.slice("Narrative:".length).trim();
            const newlineIdx = withoutPrefix.indexOf("\n");
            const theme =
              newlineIdx > -1
                ? withoutPrefix.slice(0, newlineIdx).trim()
                : withoutPrefix.slice(0, 60);
            const body =
              newlineIdx > -1
                ? withoutPrefix.slice(newlineIdx).trim()
                : "";
            return {
              id: f.id,
              theme,
              body,
              date: new Date(f.first_seen).toLocaleDateString(),
            };
          });
        setNarratives(parsed);
      })
      .catch(() => setNarratives([]))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        <Loader2 size={20} className="animate-spin mr-2" /> Loading...
      </div>
    );
  }

  if (narratives.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center space-y-3">
          <BookOpen size={48} className="mx-auto text-muted-foreground/50" />
          <h2 className="text-xl font-semibold">No narratives yet</h2>
          <p className="text-sm text-muted-foreground max-w-sm">
            Run Analysis to generate reflective narratives about your personal
            themes and evolution.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4 p-6 overflow-y-auto h-full">
      {narratives.map((n) => (
        <div
          key={n.id}
          className="rounded-lg border border-border bg-card p-5 space-y-3"
        >
          <div className="flex items-center justify-between">
            <h3 className="text-base font-semibold">{n.theme}</h3>
            <span className="text-xs text-muted-foreground">{n.date}</span>
          </div>
          <div className="text-sm text-muted-foreground leading-relaxed whitespace-pre-line">
            {n.body}
          </div>
        </div>
      ))}
    </div>
  );
}
