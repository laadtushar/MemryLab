import { useEffect, useState, useRef } from "react";
import { commands, type MemoryFactResponse } from "@/lib/tauri";
import { Brain, Trash2, Filter, Loader2, Download, Shield, ShieldAlert, Search } from "lucide-react";

const CATEGORIES = [
  { value: "", label: "All" },
  { value: "insight", label: "Insights" },
  { value: "belief", label: "Beliefs" },
  { value: "preference", label: "Preferences" },
  { value: "fact", label: "Facts" },
  { value: "self_description", label: "Self-Descriptions" },
];

const categoryColors: Record<string, string> = {
  insight: "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
  belief: "bg-blue-500/10 text-blue-400 border-blue-500/20",
  preference: "bg-purple-500/10 text-purple-400 border-purple-500/20",
  fact: "bg-green-500/10 text-green-400 border-green-500/20",
  self_description: "bg-amber-500/10 text-amber-400 border-amber-500/20",
};

export function MemoryBrowser() {
  const [facts, setFacts] = useState<MemoryFactResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState("");
  const [deleting, setDeleting] = useState<string | null>(null);
  const [piiFlags, setPiiFlags] = useState<Record<string, string[]>>({});
  const [scanning, setScanning] = useState(false);
  const [scanMsg, setScanMsg] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const searchTimer = useRef<ReturnType<typeof setTimeout>>(undefined);

  const loadFacts = async () => {
    setLoading(true);
    try {
      if (searchQuery.trim().length >= 2) {
        const data = await commands.searchMemoryFacts(searchQuery, filter || undefined);
        setFacts(data);
      } else {
        const data = await commands.getMemoryFacts(filter || undefined);
        setFacts(data);
      }
    } catch {
      setFacts([]);
    }
    setLoading(false);
  };

  const loadPiiFlags = async () => {
    try {
      const flags = await commands.getPiiFlags();
      const map: Record<string, string[]> = {};
      for (const f of flags) {
        map[f.fact_id] = f.pii_types;
      }
      setPiiFlags(map);
    } catch {
      // ignore - table may not have data yet
    }
  };

  useEffect(() => {
    loadFacts();
    loadPiiFlags();
  }, [filter]);

  // Debounced search
  useEffect(() => {
    clearTimeout(searchTimer.current);
    searchTimer.current = setTimeout(() => loadFacts(), 300);
    return () => clearTimeout(searchTimer.current);
  }, [searchQuery]);

  const handleDelete = async (id: string) => {
    setDeleting(id);
    try {
      await commands.deleteMemoryFact(id);
      setFacts((prev) => prev.filter((f) => f.id !== id));
    } catch {
      // ignore
    }
    setDeleting(null);
  };

  const handleScanPii = async () => {
    setScanning(true);
    setScanMsg(null);
    try {
      const result = await commands.scanPii();
      setScanMsg(
        `Scanned ${result.total_scanned} facts, ${result.total_flagged} flagged with PII.`
      );
      await loadPiiFlags();
    } catch {
      setScanMsg("PII scan failed.");
    }
    setScanning(false);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h1 className="text-2xl font-semibold flex items-center gap-2">
            <Brain size={24} className="text-primary" /> Memory Store
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            {facts.length} extracted facts about you, from your own writing.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleScanPii}
            disabled={scanning}
            className="flex items-center gap-1.5 rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
            title="Scan for PII (emails, SSNs, phone numbers, etc.)"
          >
            {scanning ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <ShieldAlert size={14} />
            )}
            Scan for PII
          </button>
          <div className="relative">
            <button
              onClick={async () => {
                try {
                  const md = await commands.exportMemoryMarkdown();
                  const blob = new Blob([md], { type: "text/markdown" });
                  const url = URL.createObjectURL(blob);
                  const a = document.createElement("a");
                  a.href = url;
                  a.download = "memory-palace-export.md";
                  a.click();
                  URL.revokeObjectURL(url);
                } catch { /* ignore */ }
              }}
              className="rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
              title="Export as Markdown"
            >
              <Download size={14} />
            </button>
          </div>
          <div className="relative">
            <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Filter..."
              className="rounded-md border border-input bg-background pl-8 pr-3 py-1.5 text-sm w-40 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>
          <Filter size={14} className="text-muted-foreground" />
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="rounded-md border border-input bg-background px-3 py-1.5 text-sm"
          >
            {CATEGORIES.map((c) => (
              <option key={c.value} value={c.value}>
                {c.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* PII scan message */}
      {scanMsg && (
        <div className="px-6 py-2 text-xs text-muted-foreground bg-secondary/30 border-b border-border">
          {scanMsg}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {loading ? (
          <div className="flex items-center justify-center h-40 text-muted-foreground">
            <Loader2 size={20} className="animate-spin mr-2" /> Loading...
          </div>
        ) : facts.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <Brain size={48} className="text-muted-foreground/30 mb-3" />
            <p className="text-muted-foreground">
              No memory facts yet. Import documents and run analysis to extract beliefs, preferences, and facts.
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {facts.map((fact) => {
              const factPii = piiFlags[fact.id];
              return (
                <div
                  key={fact.id}
                  className={`group flex items-start gap-3 rounded-lg border bg-card px-4 py-3 hover:border-border/80 transition-colors ${
                    factPii
                      ? "border-red-500/40"
                      : "border-border"
                  }`}
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span
                        className={`inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-medium ${
                          categoryColors[fact.category] ?? "bg-secondary text-muted-foreground border-border"
                        }`}
                      >
                        {fact.category.replace("_", " ")}
                      </span>
                      {factPii && (
                        <span
                          className="inline-flex items-center gap-1 rounded-full border border-red-500/30 bg-red-500/10 px-2 py-0.5 text-[10px] font-medium text-red-400"
                          title={`PII detected: ${factPii.join(", ")}`}
                        >
                          <Shield size={10} />
                          PII: {factPii.join(", ")}
                        </span>
                      )}
                      <span className="text-[10px] text-muted-foreground tabular-nums">
                        {new Date(fact.first_seen).toLocaleDateString()}
                      </span>
                      <span className="text-[10px] text-muted-foreground">
                        conf: {(fact.confidence * 100).toFixed(0)}%
                      </span>
                    </div>
                    <p className="text-sm">{fact.fact_text}</p>
                  </div>
                  <button
                    onClick={() => handleDelete(fact.id)}
                    disabled={deleting === fact.id}
                    className="opacity-0 group-hover:opacity-100 rounded p-1 text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-all"
                    title="Delete this memory"
                  >
                    {deleting === fact.id ? (
                      <Loader2 size={14} className="animate-spin" />
                    ) : (
                      <Trash2 size={14} />
                    )}
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
