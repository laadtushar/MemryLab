import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  commands,
  events,
  type ImportSummary,
  type ImportProgress,
} from "@/lib/tauri";
import {
  FolderOpen,
  FileText,
  BookOpen,
  CheckCircle,
  AlertCircle,
  Loader2,
} from "lucide-react";

type Source = "obsidian" | "markdown" | "dayone";
type Step = "select" | "pick" | "importing" | "done";

const sources: { id: Source; label: string; desc: string; icon: React.ReactNode }[] = [
  {
    id: "obsidian",
    label: "Obsidian Vault",
    desc: "Import .md files with frontmatter, tags, and wikilinks",
    icon: <BookOpen size={24} />,
  },
  {
    id: "markdown",
    label: "Markdown / Text",
    desc: "Import a folder of .md or .txt files",
    icon: <FileText size={24} />,
  },
  {
    id: "dayone",
    label: "Day One Export",
    desc: "Import a Day One JSON export file",
    icon: <FolderOpen size={24} />,
  },
];

export function ImportWizard() {
  const [step, setStep] = useState<Step>("select");
  const [progress, setProgress] = useState<ImportProgress | null>(null);
  const [summary, setSummary] = useState<ImportSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    events.onImportProgress((p) => setProgress(p)).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  const pickAndImport = async (src: Source) => {
    setError(null);
    setSummary(null);

    try {
      let path: string | null = null;

      if (src === "dayone") {
        const result = await open({
          filters: [{ name: "JSON", extensions: ["json"] }],
        });
        path = result as string | null;
      } else {
        const result = await open({ directory: true });
        path = result as string | null;
      }

      if (!path) {
        setStep("select");
        return;
      }

      setStep("importing");

      let result: ImportSummary;
      if (src === "obsidian") {
        result = await commands.importObsidian(path);
      } else if (src === "markdown") {
        result = await commands.importMarkdown(path);
      } else {
        result = await commands.importDayone(path);
      }

      setSummary(result);
      setStep("done");
    } catch (e) {
      setError(String(e));
      setStep("done");
    }
  };

  const reset = () => {
    setStep("select");
    setProgress(null);
    setSummary(null);
    setError(null);
  };

  return (
    <div className="p-6 max-w-2xl mx-auto space-y-6">
      <h1 className="text-2xl font-semibold">Import Data</h1>

      {step === "select" && (
        <div className="space-y-3">
          <p className="text-sm text-muted-foreground">
            Choose a data source to import from:
          </p>
          {sources.map((s) => (
            <button
              key={s.id}
              onClick={() => pickAndImport(s.id)}
              className="w-full flex items-center gap-4 rounded-lg border border-border bg-card p-4 text-left hover:border-primary/50 transition-colors"
            >
              <div className="text-primary">{s.icon}</div>
              <div>
                <p className="font-medium">{s.label}</p>
                <p className="text-sm text-muted-foreground">{s.desc}</p>
              </div>
            </button>
          ))}
        </div>
      )}

      {step === "importing" && (
        <div className="rounded-lg border border-border bg-card p-6 space-y-4">
          <div className="flex items-center gap-3">
            <Loader2 size={20} className="animate-spin text-primary" />
            <p className="font-medium">Importing...</p>
          </div>
          {progress && (
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground capitalize">
                  {progress.stage}
                </span>
                <span>
                  {progress.current} / {progress.total || "?"}
                </span>
              </div>
              <div className="h-2 rounded-full bg-secondary overflow-hidden">
                <div
                  className="h-full bg-primary rounded-full transition-all duration-300"
                  style={{
                    width: `${progress.total > 0 ? (progress.current / progress.total) * 100 : 0}%`,
                  }}
                />
              </div>
              <p className="text-xs text-muted-foreground">
                {progress.message}
              </p>
            </div>
          )}
        </div>
      )}

      {step === "done" && (
        <div className="rounded-lg border border-border bg-card p-6 space-y-4">
          {error ? (
            <div className="flex items-start gap-3">
              <AlertCircle size={20} className="text-destructive mt-0.5" />
              <div>
                <p className="font-medium text-destructive">Import failed</p>
                <p className="text-sm text-muted-foreground mt-1">{error}</p>
              </div>
            </div>
          ) : summary ? (
            <>
              <div className="flex items-center gap-3">
                <CheckCircle size={20} className="text-green-400" />
                <p className="font-medium">Import complete</p>
              </div>
              <div className="grid grid-cols-4 gap-4 text-center">
                <div>
                  <p className="text-2xl font-bold">
                    {summary.documents_imported}
                  </p>
                  <p className="text-xs text-muted-foreground">Documents</p>
                </div>
                <div>
                  <p className="text-2xl font-bold">{summary.chunks_created}</p>
                  <p className="text-xs text-muted-foreground">Chunks</p>
                </div>
                <div>
                  <p className="text-2xl font-bold">{summary.embeddings_generated}</p>
                  <p className="text-xs text-muted-foreground">Embeddings</p>
                </div>
                <div>
                  <p className="text-2xl font-bold">
                    {summary.duplicates_skipped}
                  </p>
                  <p className="text-xs text-muted-foreground">Duplicates</p>
                </div>
              </div>
              <p className="text-xs text-muted-foreground">
                Completed in {(summary.duration_ms / 1000).toFixed(1)}s
              </p>
              {summary.errors.length > 0 && (
                <div className="text-xs text-muted-foreground">
                  <p>{summary.errors.length} warnings:</p>
                  {summary.errors.slice(0, 3).map((e, i) => (
                    <p key={i} className="truncate">
                      {e}
                    </p>
                  ))}
                </div>
              )}
            </>
          ) : null}
          <button
            onClick={reset}
            className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:bg-primary/90"
          >
            Import more
          </button>
        </div>
      )}
    </div>
  );
}
