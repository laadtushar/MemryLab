import { useEffect, useState, useRef } from "react";
import { commands, type EntityResponse } from "@/lib/tauri";
import { Users, MapPin, Building, Hash, Loader2, Network, Search } from "lucide-react";

const typeIcons: Record<string, React.ReactNode> = {
  person: <Users size={14} className="text-blue-400" />,
  place: <MapPin size={14} className="text-green-400" />,
  organization: <Building size={14} className="text-purple-400" />,
  concept: <Hash size={14} className="text-amber-400" />,
  topic: <Hash size={14} className="text-amber-400" />,
};

const typeColors: Record<string, string> = {
  person: "bg-blue-500/10 text-blue-400 border-blue-500/20",
  place: "bg-green-500/10 text-green-400 border-green-500/20",
  organization: "bg-purple-500/10 text-purple-400 border-purple-500/20",
  concept: "bg-amber-500/10 text-amber-400 border-amber-500/20",
  topic: "bg-amber-500/10 text-amber-400 border-amber-500/20",
};

const TYPES = [
  { value: "", label: "All" },
  { value: "person", label: "People" },
  { value: "place", label: "Places" },
  { value: "organization", label: "Organizations" },
  { value: "concept", label: "Concepts" },
];

export function EntityExplorer() {
  const [entities, setEntities] = useState<EntityResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const searchTimer = useRef<ReturnType<typeof setTimeout>>(undefined);

  const loadEntities = async () => {
    setLoading(true);
    try {
      if (searchQuery.trim().length >= 2) {
        const data = await commands.searchEntities(searchQuery, filter || undefined);
        setEntities(data);
      } else {
        const data = await commands.listEntities(filter || undefined);
        setEntities(data);
      }
    } catch {
      setEntities([]);
    }
    setLoading(false);
  };

  useEffect(() => {
    loadEntities();
  }, [filter]);

  useEffect(() => {
    clearTimeout(searchTimer.current);
    searchTimer.current = setTimeout(() => loadEntities(), 300);
    return () => clearTimeout(searchTimer.current);
  }, [searchQuery]);

  const maxMentions = Math.max(...entities.map((e) => e.mention_count), 1);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="border-b border-border px-6 py-4">
        <div>
          <h1 className="text-2xl font-semibold flex items-center gap-2">
            <Network size={24} className="text-primary" /> Entity Explorer
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            {entities.length} entities extracted from your documents.
            Run analysis to discover people, places, and concepts in your writing.
          </p>
        </div>

        {/* Search + Type filter */}
        <div className="flex items-center gap-3 mt-3">
          <div className="relative">
            <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search entities..."
              className="rounded-md border border-input bg-background pl-8 pr-3 py-1.5 text-sm w-48 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>
        </div>
        <div className="flex gap-2 flex-wrap mt-2">
          {TYPES.map((t) => (
            <button
              key={t.value}
              onClick={() => setFilter(t.value)}
              className={`rounded-md px-3 py-1 text-sm transition-colors ${
                filter === t.value
                  ? "bg-primary text-primary-foreground"
                  : "bg-secondary text-muted-foreground hover:text-foreground"
              }`}
            >
              {t.label}
            </button>
          ))}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {loading ? (
          <div className="flex items-center justify-center h-40 text-muted-foreground">
            <Loader2 size={20} className="animate-spin mr-2" /> Loading...
          </div>
        ) : entities.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <Network size={48} className="text-muted-foreground/30 mb-3" />
            <p className="text-lg font-medium text-muted-foreground">
              No entities yet
            </p>
            <p className="text-sm text-muted-foreground mt-1 max-w-sm">
              Import documents and run analysis to extract people, places, organizations,
              and concepts from your writing.
            </p>
          </div>
        ) : (
          <div className="space-y-1.5">
            {entities.map((entity) => {
              const barWidth = (entity.mention_count / maxMentions) * 100;
              return (
                <div
                  key={entity.id}
                  className="group flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-2.5 hover:border-border/80 transition-colors"
                >
                  <div className="shrink-0">
                    {typeIcons[entity.entity_type] ?? <Hash size={14} />}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-sm truncate">
                        {entity.name}
                      </span>
                      <span
                        className={`inline-flex items-center rounded-full border px-1.5 py-0 text-[9px] font-medium ${
                          typeColors[entity.entity_type] ?? "bg-secondary border-border"
                        }`}
                      >
                        {entity.entity_type}
                      </span>
                    </div>
                    {/* Mention bar */}
                    <div className="flex items-center gap-2 mt-1">
                      <div className="h-1.5 flex-1 max-w-32 rounded-full bg-secondary overflow-hidden">
                        <div
                          className="h-full bg-primary/60 rounded-full"
                          style={{ width: `${barWidth}%` }}
                        />
                      </div>
                      <span className="text-[10px] text-muted-foreground tabular-nums">
                        {entity.mention_count}x
                      </span>
                    </div>
                  </div>
                  {entity.first_seen && (
                    <span className="text-[10px] text-muted-foreground shrink-0">
                      {new Date(entity.first_seen).toLocaleDateString()}
                    </span>
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
