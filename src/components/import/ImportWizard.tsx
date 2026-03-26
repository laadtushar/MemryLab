import { useState, useEffect, useMemo } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  commands,
  events,
  type ImportSummary,
  type ImportProgress,
  type SourceAdapterMeta,
} from "@/lib/tauri";
import {
  CheckCircle,
  AlertCircle,
  Loader2,
  ExternalLink,
  Upload,
  Search,
  ArrowLeft,
  FolderOpen,
} from "lucide-react";
import { SourceIcon } from "./SourceIcon";

type Step = "select" | "instructions" | "importing" | "done";

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

interface SourceCategory {
  label: string;
  sources: string[];
}

const CATEGORIES: SourceCategory[] = [
  {
    label: "Social Media",
    sources: [
      "facebook",
      "instagram",
      "twitter",
      "reddit",
      "bluesky",
      "mastodon",
      "threads",
      "tiktok",
      "snapchat",
      "pinterest",
      "tumblr",
    ],
  },
  {
    label: "Messaging",
    sources: ["whatsapp", "telegram", "discord", "slack", "signal"],
  },
  {
    label: "Notes & Writing",
    sources: [
      "obsidian",
      "notion",
      "evernote",
      "markdown",
      "dayone",
      "substack",
      "medium",
    ],
  },
  {
    label: "Media & Entertainment",
    sources: ["spotify", "youtube", "netflix"],
  },
  {
    label: "Productivity & Cloud",
    sources: ["google_takeout", "linkedin", "apple", "amazon", "microsoft"],
  },
];

export function ImportWizard() {
  const [step, setStep] = useState<Step>("select");
  const [sources, setSources] = useState<SourceAdapterMeta[]>([]);
  const [selectedSource, setSelectedSource] =
    useState<SourceAdapterMeta | null>(null);
  const [progress, setProgress] = useState<ImportProgress | null>(null);
  const [summary, setSummary] = useState<ImportSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    commands
      .listSources()
      .then(setSources)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    events.onImportProgress((p) => setProgress(p)).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  const sourceMap = useMemo(() => {
    const map = new Map<string, SourceAdapterMeta>();
    for (const s of sources) map.set(s.id, s);
    return map;
  }, [sources]);

  const filteredCategories = useMemo(() => {
    const q = searchQuery.toLowerCase();
    if (!q) return CATEGORIES;
    return CATEGORIES.map((cat) => ({
      ...cat,
      sources: cat.sources.filter((id) => {
        const src = sourceMap.get(id);
        return (
          src &&
          (src.display_name.toLowerCase().includes(q) ||
            src.id.includes(q) ||
            src.platform.toLowerCase().includes(q))
        );
      }),
    })).filter((cat) => cat.sources.length > 0);
  }, [searchQuery, sourceMap]);

  const handleSelectSource = (source: SourceAdapterMeta) => {
    setSelectedSource(source);
    setStep("instructions");
  };

  const handleImport = async () => {
    if (!selectedSource) return;
    setError(null);
    setSummary(null);

    try {
      const isZip = selectedSource.handles_zip;
      const isDir =
        !isZip &&
        (selectedSource.id === "obsidian" || selectedSource.id === "markdown");

      let path: string | null = null;

      if (isDir) {
        path = (await open({ directory: true })) as string | null;
      } else {
        const extensions = selectedSource.accepted_extensions.length > 0
          ? selectedSource.accepted_extensions
          : ["zip", "json", "csv", "txt", "html", "xml", "enex"];
        const result = await open({
          filters: [
            {
              name: selectedSource.display_name,
              extensions,
            },
          ],
          multiple: false,
        });
        path = result as string | null;
      }

      if (!path) return;

      setStep("importing");
      const result = await commands.importSource(path, selectedSource.id);
      setSummary(result);
      setStep("done");
    } catch (e) {
      setError(String(e));
      setStep("done");
    }
  };

  const handleAutoImport = async () => {
    setError(null);
    setSummary(null);
    setSelectedSource(null);

    try {
      const result = await open({
        filters: [
          {
            name: "Data Export",
            extensions: [
              "zip",
              "json",
              "csv",
              "txt",
              "html",
              "xml",
              "enex",
              "car",
              "mbox",
            ],
          },
        ],
        multiple: false,
      });
      const path = result as string | null;
      if (!path) return;

      setStep("importing");
      const importResult = await commands.importSource(path);
      setSummary(importResult);
      setStep("done");
    } catch (e) {
      setError(String(e));
      setStep("done");
    }
  };

  const reset = () => {
    setStep("select");
    setSelectedSource(null);
    setProgress(null);
    setSummary(null);
    setError(null);
    setSearchQuery("");
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        <Loader2 size={20} className="animate-spin mr-2" /> Loading
        sources...
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="border-b border-border px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-semibold">Import Data</h1>
            <p className="text-sm text-muted-foreground mt-0.5">
              {sources.length} sources supported. Upload a ZIP, JSON, CSV, or
              folder from any platform.
            </p>
          </div>
          {step !== "select" && (
            <button
              onClick={reset}
              className="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
            >
              <ArrowLeft size={14} /> Back
            </button>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Step: Select Source */}
        {step === "select" && (
          <div className="p-6 space-y-6">
            {/* Auto-detect button */}
            <button
              onClick={handleAutoImport}
              className="w-full flex items-center gap-4 rounded-lg border-2 border-dashed border-primary/30 bg-primary/5 p-5 text-left hover:border-primary/60 hover:bg-primary/10 transition-colors"
            >
              <div className="rounded-full bg-primary/10 p-3">
                <Upload size={24} className="text-primary" />
              </div>
              <div>
                <p className="font-semibold text-lg">
                  Auto-detect &amp; Import
                </p>
                <p className="text-sm text-muted-foreground">
                  Upload any export file or ZIP — we'll figure out the format
                  automatically
                </p>
              </div>
            </button>

            {/* Search */}
            <div className="relative">
              <Search
                size={16}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
              />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search sources..."
                className="w-full rounded-md border border-input bg-background pl-9 pr-4 py-2.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
              />
            </div>

            {/* Categorized source grid */}
            {filteredCategories.map((cat) => (
              <div key={cat.label}>
                <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                  {cat.label}
                </h3>
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
                  {cat.sources.map((id) => {
                    const src = sourceMap.get(id);
                    if (!src) return null;
                    return (
                      <button
                        key={id}
                        onClick={() => handleSelectSource(src)}
                        className="flex items-center gap-3 rounded-lg border border-border bg-card px-3 py-2.5 text-left hover:border-primary/50 hover:bg-accent transition-colors group"
                      >
                        <SourceIcon
                          icon={src.icon}
                          className="text-muted-foreground group-hover:text-primary transition-colors"
                        />
                        <span className="text-sm font-medium truncate">
                          {src.display_name}
                        </span>
                      </button>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Step: Instructions */}
        {step === "instructions" && selectedSource && (
          <div className="p-6 max-w-xl mx-auto space-y-6">
            <div className="flex items-center gap-4">
              <div className="rounded-xl bg-card border border-border p-4">
                <SourceIcon icon={selectedSource.icon} size={40} />
              </div>
              <div>
                <h2 className="text-xl font-semibold">
                  {selectedSource.display_name}
                </h2>
                <p className="text-sm text-muted-foreground">
                  Accepts:{" "}
                  {selectedSource.accepted_extensions
                    .map((e) => `.${e}`)
                    .join(", ") || "any format"}
                </p>
              </div>
            </div>

            {/* Instructions */}
            <div className="rounded-lg border border-border bg-card p-5 space-y-3">
              <h3 className="font-medium">How to export your data</h3>
              <p className="text-sm text-muted-foreground leading-relaxed">
                {selectedSource.instructions}
              </p>

              {selectedSource.takeout_url && (
                <a
                  href={selectedSource.takeout_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 rounded-md bg-primary/10 px-3 py-2 text-sm font-medium text-primary hover:bg-primary/20 transition-colors"
                >
                  <ExternalLink size={14} />
                  Open {selectedSource.display_name} Data Export
                </a>
              )}
            </div>

            {/* Steps */}
            <div className="rounded-lg border border-border bg-card p-5 space-y-3">
              <h3 className="font-medium">Steps</h3>
              <ol className="list-decimal list-inside text-sm text-muted-foreground space-y-1.5">
                {selectedSource.takeout_url && (
                  <li>
                    Click the link above to open{" "}
                    {selectedSource.display_name}'s data export page
                  </li>
                )}
                <li>
                  Request and download your data export (
                  {selectedSource.handles_zip ? "ZIP file" : "folder/file"})
                </li>
                <li>
                  Click "Choose File" below and select the downloaded export
                </li>
                <li>
                  Memory Palace will automatically parse and import your data
                </li>
              </ol>
            </div>

            {/* Import button */}
            <button
              onClick={handleImport}
              className="w-full flex items-center justify-center gap-2 rounded-md bg-primary px-4 py-3 text-primary-foreground font-medium hover:bg-primary/90 transition-colors"
            >
              <FolderOpen size={18} />
              Choose{" "}
              {selectedSource.handles_zip ||
              selectedSource.accepted_extensions.includes("json")
                ? "File"
                : "Folder"}{" "}
              to Import
            </button>
          </div>
        )}

        {/* Step: Importing */}
        {step === "importing" && (
          <div className="p-6 max-w-xl mx-auto">
            <div className="rounded-lg border border-border bg-card p-6 space-y-4">
              <div className="flex items-center gap-3">
                <Loader2 size={20} className="animate-spin text-primary" />
                <p className="font-medium">
                  Importing
                  {selectedSource
                    ? ` from ${selectedSource.display_name}`
                    : " (auto-detecting)"}
                  ...
                </p>
              </div>
              {progress && (
                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">
                      {STAGE_LABELS[progress.stage] ?? progress.stage}
                    </span>
                    {progress.total > 0 && (
                      <span>
                        {progress.current} / {progress.total}
                      </span>
                    )}
                  </div>
                  <div className="h-2 rounded-full bg-secondary overflow-hidden">
                    {progress.total > 0 ? (
                      <div
                        className="h-full bg-primary rounded-full transition-all duration-300"
                        style={{
                          width: `${(progress.current / progress.total) * 100}%`,
                        }}
                      />
                    ) : (
                      <div className="h-full w-1/3 bg-primary/60 rounded-full animate-pulse" />
                    )}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {progress.message}
                  </p>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Step: Done */}
        {step === "done" && (
          <div className="p-6 max-w-xl mx-auto">
            <div className="rounded-lg border border-border bg-card p-6 space-y-4">
              {error ? (
                <div className="flex items-start gap-3">
                  <AlertCircle
                    size={20}
                    className="text-destructive mt-0.5"
                  />
                  <div>
                    <p className="font-medium text-destructive">
                      Import failed
                    </p>
                    <p className="text-sm text-muted-foreground mt-1">
                      {error}
                    </p>
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
                      <p className="text-xs text-muted-foreground">
                        Documents
                      </p>
                    </div>
                    <div>
                      <p className="text-2xl font-bold">
                        {summary.chunks_created}
                      </p>
                      <p className="text-xs text-muted-foreground">Chunks</p>
                    </div>
                    <div>
                      <p className="text-2xl font-bold">
                        {summary.embeddings_generated}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        Embeddings
                      </p>
                    </div>
                    <div>
                      <p className="text-2xl font-bold">
                        {summary.duplicates_skipped}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        Duplicates
                      </p>
                    </div>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Completed in {(summary.duration_ms / 1000).toFixed(1)}s
                  </p>
                  {summary.errors.length > 0 && (
                    <div className="text-xs text-muted-foreground">
                      <p>{summary.errors.length} warnings:</p>
                      {summary.errors.slice(0, 5).map((e, i) => (
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
          </div>
        )}
      </div>
    </div>
  );
}
