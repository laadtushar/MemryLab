import { useState, useEffect, useCallback } from "react";
import {
  commands,
  type ActivityEntry,
} from "@/lib/tauri";
import {
  Upload,
  Brain,
  Sparkles,
  MessageCircle,
  Search,
  Settings,
  Clock,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  Activity,
} from "lucide-react";

const ACTION_ICONS: Record<string, React.ReactNode> = {
  import: <Upload size={16} />,
  analysis: <Brain size={16} />,
  embeddings: <Sparkles size={16} />,
  ask: <MessageCircle size={16} />,
  search: <Search size={16} />,
  config: <Settings size={16} />,
};

const STATUS_STYLES: Record<string, { icon: React.ReactNode; className: string }> = {
  success: {
    icon: <CheckCircle2 size={14} />,
    className: "text-green-400",
  },
  error: {
    icon: <XCircle size={14} />,
    className: "text-red-400",
  },
  warning: {
    icon: <AlertTriangle size={14} />,
    className: "text-yellow-400",
  },
};

const FILTER_OPTIONS = [
  { label: "All", value: undefined },
  { label: "Imports", value: "import" },
  { label: "Analysis", value: "analysis" },
  { label: "Queries", value: "ask" },
  { label: "Search", value: "search" },
  { label: "Config", value: "config" },
] as const;

function relativeTime(timestamp: string): string {
  const date = new Date(timestamp + "Z"); // SQLite stores UTC
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHr = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHr / 24);

  if (diffSec < 60) return "just now";
  if (diffMin < 60) return `${diffMin} min ago`;
  if (diffHr < 24) return `${diffHr} hr ago`;
  if (diffDay < 7) return `${diffDay}d ago`;
  return date.toLocaleDateString();
}

export function ActivityView() {
  const [entries, setEntries] = useState<ActivityEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<string | undefined>(undefined);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const loadActivity = useCallback(async () => {
    setLoading(true);
    try {
      const result = await commands.getActivityLog(200, filter);
      setEntries(result);
    } catch (e) {
      console.error("Failed to load activity:", e);
    }
    setLoading(false);
  }, [filter]);

  useEffect(() => {
    loadActivity();
  }, [loadActivity]);

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b border-border px-6 py-4">
        <h1 className="text-2xl font-semibold flex items-center gap-2">
          <Activity size={24} className="text-primary" /> Activity History
        </h1>
        <p className="text-sm text-muted-foreground mt-1">
          A log of every action and its results.
        </p>
      </div>

      {/* Filters */}
      <div className="border-b border-border px-6 py-3 flex gap-2 flex-wrap">
        {FILTER_OPTIONS.map((opt) => (
          <button
            key={opt.label}
            onClick={() => setFilter(opt.value)}
            className={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
              filter === opt.value
                ? "bg-primary text-primary-foreground"
                : "bg-card border border-border text-muted-foreground hover:text-foreground"
            }`}
          >
            {opt.label}
          </button>
        ))}
      </div>

      {/* Feed */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {loading && (
          <div className="flex items-center justify-center py-12 text-muted-foreground">
            <Clock size={20} className="animate-pulse mr-2" />
            Loading activity...
          </div>
        )}

        {!loading && entries.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-center space-y-3">
            <Activity size={48} className="text-muted-foreground/30" />
            <p className="text-lg font-medium text-muted-foreground">
              No activity yet
            </p>
            <p className="text-sm text-muted-foreground/70">
              Import some documents to get started.
            </p>
          </div>
        )}

        {!loading && entries.length > 0 && (
          <div className="space-y-2">
            {entries.map((entry) => {
              const statusStyle = STATUS_STYLES[entry.status] || STATUS_STYLES.success;
              const isExpanded = expandedId === entry.id;
              const icon = ACTION_ICONS[entry.action_type] || <Activity size={16} />;

              return (
                <div
                  key={entry.id}
                  className="rounded-lg border border-border bg-card hover:border-border/80 transition-colors"
                >
                  <button
                    className="w-full flex items-center gap-3 px-4 py-3 text-left"
                    onClick={() =>
                      setExpandedId(isExpanded ? null : entry.id)
                    }
                  >
                    {/* Icon */}
                    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
                      {icon}
                    </div>

                    {/* Content */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium truncate">
                          {entry.title}
                        </span>
                        <span className={statusStyle.className}>
                          {statusStyle.icon}
                        </span>
                      </div>
                      {entry.result_summary && (
                        <p className="text-xs text-muted-foreground truncate mt-0.5">
                          {entry.result_summary}
                        </p>
                      )}
                    </div>

                    {/* Meta */}
                    <div className="flex items-center gap-3 shrink-0 text-xs text-muted-foreground">
                      {entry.duration_ms > 0 && (
                        <span>{(entry.duration_ms / 1000).toFixed(1)}s</span>
                      )}
                      <span>{relativeTime(entry.timestamp)}</span>
                      {isExpanded ? (
                        <ChevronDown size={14} />
                      ) : (
                        <ChevronRight size={14} />
                      )}
                    </div>
                  </button>

                  {/* Expanded detail */}
                  {isExpanded && (
                    <div className="px-4 pb-3 pt-0 border-t border-border/50 mt-0">
                      <div className="mt-3 space-y-1.5 text-xs text-muted-foreground">
                        {entry.description && (
                          <div>
                            <span className="font-medium text-foreground">
                              Description:{" "}
                            </span>
                            {entry.description}
                          </div>
                        )}
                        <div>
                          <span className="font-medium text-foreground">
                            Type:{" "}
                          </span>
                          {entry.action_type}
                        </div>
                        <div>
                          <span className="font-medium text-foreground">
                            Time:{" "}
                          </span>
                          {new Date(entry.timestamp + "Z").toLocaleString()}
                        </div>
                        {entry.metadata &&
                          Object.keys(entry.metadata).length > 0 && (
                            <div>
                              <span className="font-medium text-foreground">
                                Metadata:{" "}
                              </span>
                              <pre className="mt-1 rounded bg-background/50 p-2 overflow-x-auto">
                                {JSON.stringify(entry.metadata, null, 2)}
                              </pre>
                            </div>
                          )}
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
