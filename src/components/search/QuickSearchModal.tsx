import { useEffect, useRef, useState } from "react";
import { commands, type QuickSearchResult } from "@/lib/tauri";
import { useAppStore } from "@/stores/app-store";
import {
  Search, FileText, Brain, Users, MessageCircle, X,
} from "lucide-react";

const TYPE_ICONS: Record<string, React.ReactNode> = {
  document: <FileText size={14} className="text-blue-400" />,
  memory: <Brain size={14} className="text-yellow-400" />,
  entity: <Users size={14} className="text-green-400" />,
  chat: <MessageCircle size={14} className="text-purple-400" />,
};

const TYPE_LABELS: Record<string, string> = {
  document: "Document",
  memory: "Memory",
  entity: "Entity",
  chat: "Chat",
};

export function QuickSearchModal() {
  const open = useAppStore((s) => s.quickSearchOpen);
  const close = useAppStore((s) => s.setQuickSearchOpen);
  const setView = useAppStore((s) => s.setView);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<QuickSearchResult[]>([]);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  // Focus input when opened
  useEffect(() => {
    if (open) {
      setQuery("");
      setResults([]);
      setSuggestions([]);
      setSelected(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  // Debounced search
  useEffect(() => {
    if (!query || query.length < 2) {
      setResults([]);
      setSuggestions([]);
      return;
    }
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(async () => {
      const [searchResults, suggestResults] = await Promise.all([
        commands.quickSearch(query).catch(() => []),
        commands.searchSuggestions(query).catch(() => []),
      ]);
      setResults(searchResults);
      setSuggestions(suggestResults.slice(0, 3));
      setSelected(0);
    }, 150);
    return () => clearTimeout(timerRef.current);
  }, [query]);

  // Keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      close(false);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelected((s) => Math.min(s + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelected((s) => Math.max(s - 1, 0));
    } else if (e.key === "Enter" && results[selected]) {
      handleSelect(results[selected]);
    }
  };

  const handleSelect = (r: QuickSearchResult) => {
    close(false);
    switch (r.result_type) {
      case "document": setView("search"); break;
      case "memory": setView("memory"); break;
      case "entity": setView("entities"); break;
      case "chat": setView("ask"); break;
      default: setView("search");
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh]" onClick={() => close(false)}>
      <div className="absolute inset-0 bg-background/60 backdrop-blur-sm" />
      <div
        className="relative w-full max-w-lg mx-4 rounded-xl border border-border bg-card shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search input */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-border">
          <Search size={18} className="text-muted-foreground shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search documents, memories, entities, chats..."
            className="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
          />
          <kbd className="hidden sm:inline text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">ESC</kbd>
          <button onClick={() => close(false)} className="text-muted-foreground hover:text-foreground sm:hidden">
            <X size={16} />
          </button>
        </div>

        {/* Results */}
        <div className="max-h-[50vh] overflow-y-auto">
          {query.length < 2 ? (
            <div className="px-4 py-8 text-center text-sm text-muted-foreground">
              Type to search across all your data
            </div>
          ) : results.length === 0 && suggestions.length === 0 ? (
            <div className="px-4 py-8 text-center text-sm text-muted-foreground">
              No results for "{query}"
            </div>
          ) : (
            <div className="py-1">
              {/* Suggestions */}
              {suggestions.length > 0 && (
                <div className="px-3 py-1.5">
                  <p className="text-[10px] uppercase tracking-wider text-muted-foreground font-semibold mb-1">Suggestions</p>
                  <div className="flex flex-wrap gap-1.5">
                    {suggestions.map((s, i) => (
                      <button
                        key={i}
                        onClick={() => setQuery(s)}
                        className="text-xs bg-muted hover:bg-muted/80 rounded px-2 py-1 text-muted-foreground hover:text-foreground transition-colors truncate max-w-[200px]"
                      >
                        {s}
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {/* Results */}
              {results.map((r, i) => (
                <button
                  key={`${r.result_type}-${r.id}`}
                  onClick={() => handleSelect(r)}
                  onMouseEnter={() => setSelected(i)}
                  className={`w-full flex items-start gap-3 px-4 py-2.5 text-left transition-colors ${
                    i === selected ? "bg-accent" : "hover:bg-accent/50"
                  }`}
                >
                  <div className="mt-0.5 shrink-0">{TYPE_ICONS[r.result_type]}</div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium truncate">{r.title}</span>
                      <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded shrink-0">
                        {TYPE_LABELS[r.result_type] ?? r.result_type}
                      </span>
                    </div>
                    <p
                      className="text-xs text-muted-foreground line-clamp-2 mt-0.5"
                      dangerouslySetInnerHTML={{ __html: r.snippet }}
                    />
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-2 border-t border-border text-[10px] text-muted-foreground">
          <span>
            <kbd className="bg-muted px-1 py-0.5 rounded">↑↓</kbd> Navigate{" "}
            <kbd className="bg-muted px-1 py-0.5 rounded ml-1">↵</kbd> Open
          </span>
          <span>{results.length} results</span>
        </div>
      </div>
    </div>
  );
}
