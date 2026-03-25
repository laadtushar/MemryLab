import { useEffect, useState } from "react";
import { commands, type MemoryFactResponse, type AnalysisResult } from "@/lib/tauri";
import { Brain, Trash2, Lightbulb, Play, Loader2, Sparkles } from "lucide-react";

const categories = ["insight", "belief", "preference", "fact", "self_description"];

const categoryColors: Record<string, string> = {
  insight: "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
  belief: "bg-blue-500/10 text-blue-400 border-blue-500/20",
  preference: "bg-purple-500/10 text-purple-400 border-purple-500/20",
  fact: "bg-green-500/10 text-green-400 border-green-500/20",
  self_description: "bg-amber-500/10 text-amber-400 border-amber-500/20",
};

export function InsightFeed() {
  const [facts, setFacts] = useState<MemoryFactResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<string | undefined>(undefined);
  const [analyzing, setAnalyzing] = useState(false);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResult | null>(null);
  const [analysisError, setAnalysisError] = useState<string | null>(null);

  const loadFacts = async () => {
    setLoading(true);
    try {
      const result = await commands.getMemoryFacts(filter);
      setFacts(result);
    } catch {
      setFacts([]);
    }
    setLoading(false);
  };

  useEffect(() => {
    loadFacts();
  }, [filter]);

  const deleteFact = async (id: string) => {
    try {
      await commands.deleteMemoryFact(id);
      setFacts((prev) => prev.filter((f) => f.id !== id));
    } catch {
      /* ignore */
    }
  };

  const runAnalysis = async () => {
    setAnalyzing(true);
    setAnalysisResult(null);
    setAnalysisError(null);
    try {
      const result = await commands.runAnalysis();
      setAnalysisResult(result);
      // Reload facts to show new insights
      await loadFacts();
    } catch (e) {
      setAnalysisError(String(e));
    }
    setAnalyzing(false);
  };

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="border-b border-border px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-semibold flex items-center gap-2">
              <Lightbulb size={24} className="text-primary" /> Insights
            </h1>
            <p className="text-sm text-muted-foreground mt-1">
              {facts.length} extracted insights and facts from your documents.
            </p>
          </div>
          <button
            onClick={runAnalysis}
            disabled={analyzing}
            className="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors"
          >
            {analyzing ? (
              <>
                <Loader2 size={16} className="animate-spin" />
                Analyzing...
              </>
            ) : (
              <>
                <Play size={16} />
                Run Analysis
              </>
            )}
          </button>
        </div>

        {/* Analysis result banner */}
        {analysisResult && (
          <div className="mt-3 rounded-md bg-green-500/10 border border-green-500/20 px-4 py-2 text-sm text-green-400">
            <Sparkles size={14} className="inline mr-1" />
            Analysis complete: {analysisResult.themes_extracted} themes, {analysisResult.beliefs_extracted} beliefs,{" "}
            {analysisResult.sentiments_classified} sentiments, {analysisResult.insights_generated} insights generated.
          </div>
        )}
        {analysisError && (
          <div className="mt-3 rounded-md bg-destructive/10 border border-destructive/20 px-4 py-2 text-sm text-destructive">
            {analysisError}
          </div>
        )}

        {/* Category filter */}
        <div className="flex gap-2 flex-wrap mt-3">
          <button
            onClick={() => setFilter(undefined)}
            className={`rounded-md px-3 py-1 text-sm transition-colors ${
              !filter
                ? "bg-primary text-primary-foreground"
                : "bg-secondary text-muted-foreground hover:text-foreground"
            }`}
          >
            All
          </button>
          {categories.map((cat) => (
            <button
              key={cat}
              onClick={() => setFilter(cat)}
              className={`rounded-md px-3 py-1 text-sm capitalize transition-colors ${
                filter === cat
                  ? "bg-primary text-primary-foreground"
                  : "bg-secondary text-muted-foreground hover:text-foreground"
              }`}
            >
              {cat.replace("_", " ")}
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
        ) : facts.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <Brain size={48} className="text-muted-foreground/30 mb-3" />
            <p className="text-lg font-medium text-muted-foreground">No insights yet</p>
            <p className="text-sm text-muted-foreground mt-1 max-w-sm">
              Import your documents first, then click <strong>Run Analysis</strong> to extract
              themes, beliefs, and surprising insights from your writing.
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {facts.map((fact) => (
              <div
                key={fact.id}
                className="group rounded-lg border border-border bg-card p-4 space-y-2 hover:border-border/80 transition-colors"
              >
                <div className="flex items-start justify-between gap-3">
                  <p className="text-sm">{fact.fact_text}</p>
                  <button
                    onClick={() => deleteFact(fact.id)}
                    className="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive shrink-0 transition-opacity"
                    title="Forget this fact"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
                <div className="flex items-center gap-3 text-xs text-muted-foreground">
                  <span
                    className={`inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-medium capitalize ${
                      categoryColors[fact.category] ?? "bg-secondary text-muted-foreground border-border"
                    }`}
                  >
                    {fact.category.replace("_", " ")}
                  </span>
                  <span>
                    conf: {(fact.confidence * 100).toFixed(0)}%
                  </span>
                  <span>
                    {new Date(fact.first_seen).toLocaleDateString()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
