import { useEffect } from "react";
import { useAppStore, type BackgroundImport } from "@/stores/app-store";
import { events } from "@/lib/tauri";
import { CheckCircle, Loader2, X, AlertCircle } from "lucide-react";

const STAGE_LABELS: Record<string, string> = {
  scanning: "Scanning",
  parsing: "Parsing",
  dedup: "Deduplication",
  normalize: "Normalizing",
  storing: "Storing",
  embedding: "Embedding",
  sweep: "Sweep",
  analysis: "Analyzing",
  "analysis-complete": "Analysis Complete",
  complete: "Complete",
};

function ImportRow({ bg }: { bg: BackgroundImport }) {
  const remove = useAppStore((s) => s.removeBackgroundImport);

  // Completed
  if (!bg.running && bg.summary && !bg.error) {
    return (
      <div className="flex items-center justify-between text-sm py-1">
        <div className="flex items-center gap-2 text-green-400">
          <CheckCircle size={14} />
          <span>
            {bg.sourceName}: {bg.summary.documents_imported} docs,{" "}
            {bg.summary.chunks_created} chunks
            {bg.summary.duration_ms > 0 &&
              ` (${(bg.summary.duration_ms / 1000).toFixed(1)}s)`}
          </span>
        </div>
        <button
          onClick={() => remove(bg.id)}
          className="text-muted-foreground hover:text-foreground"
        >
          <X size={14} />
        </button>
      </div>
    );
  }

  // Error
  if (!bg.running && bg.error) {
    return (
      <div className="flex items-center justify-between text-sm py-1">
        <div className="flex items-center gap-2 text-destructive">
          <AlertCircle size={14} />
          <span>{bg.sourceName}: {bg.error}</span>
        </div>
        <button
          onClick={() => remove(bg.id)}
          className="text-muted-foreground hover:text-foreground"
        >
          <X size={14} />
        </button>
      </div>
    );
  }

  // Running
  const progress = bg.progress;
  const pct =
    progress && progress.total > 0
      ? Math.round((progress.current / progress.total) * 100)
      : null;

  return (
    <div className="flex items-center gap-3 text-sm py-1">
      <Loader2 size={14} className="animate-spin text-primary shrink-0" />
      <span className="text-muted-foreground shrink-0">
        {bg.sourceName}
      </span>
      {progress && (
        <>
          <span className="text-xs text-muted-foreground shrink-0">
            {STAGE_LABELS[progress.stage] ?? progress.stage}
          </span>
          <div className="flex-1 h-1.5 rounded-full bg-secondary overflow-hidden max-w-xs">
            {pct !== null ? (
              <div
                className="h-full bg-primary rounded-full transition-all duration-300"
                style={{ width: `${pct}%` }}
              />
            ) : (
              <div className="h-full w-1/3 bg-primary/60 rounded-full animate-pulse" />
            )}
          </div>
          {progress.total > 0 && (
            <span className="text-xs font-mono text-muted-foreground shrink-0">
              {progress.current}/{progress.total}
            </span>
          )}
        </>
      )}
    </div>
  );
}

export function ImportProgressBanner() {
  const imports = useAppStore((s) => s.backgroundImports);

  // Global progress listener — always mounted in AppShell, never unmounts
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    events.onImportProgress((p) => {
      useAppStore.getState().updateBackgroundImport({ progress: p });
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  if (imports.length === 0) return null;

  return (
    <div className="border-t border-border bg-card px-4 py-1.5 space-y-0.5">
      {imports.map((bg) => (
        <ImportRow key={bg.id} bg={bg} />
      ))}
    </div>
  );
}
