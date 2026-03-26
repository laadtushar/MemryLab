import { useState } from "react";
import { commands, type EvolutionDiffResponse } from "@/lib/tauri";
import { Loader2, ArrowRightLeft } from "lucide-react";

export function DiffView() {
  const [periodAStart, setPeriodAStart] = useState("");
  const [periodAEnd, setPeriodAEnd] = useState("");
  const [periodBStart, setPeriodBStart] = useState("");
  const [periodBEnd, setPeriodBEnd] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<EvolutionDiffResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleCompare = async () => {
    if (!periodAStart || !periodAEnd || !periodBStart || !periodBEnd) {
      setError("Please fill in all date fields.");
      return;
    }
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const diff = await commands.getEvolutionDiff(
        periodAStart,
        periodAEnd,
        periodBStart,
        periodBEnd,
      );
      setResult(diff);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const sentimentColor = (s: string) => {
    switch (s.toLowerCase()) {
      case "positive":
        return "text-green-400";
      case "negative":
        return "text-red-400";
      default:
        return "text-zinc-400";
    }
  };

  return (
    <div className="flex flex-col gap-6 p-6 overflow-y-auto h-full">
      {/* Date pickers */}
      <div className="grid grid-cols-2 gap-6">
        <div className="space-y-2">
          <h3 className="text-sm font-medium text-muted-foreground">
            Period A
          </h3>
          <div className="flex gap-2 items-center">
            <input
              type="date"
              value={periodAStart}
              onChange={(e) => setPeriodAStart(e.target.value)}
              className="rounded-md border border-border bg-background px-3 py-1.5 text-sm"
            />
            <span className="text-muted-foreground">to</span>
            <input
              type="date"
              value={periodAEnd}
              onChange={(e) => setPeriodAEnd(e.target.value)}
              className="rounded-md border border-border bg-background px-3 py-1.5 text-sm"
            />
          </div>
        </div>
        <div className="space-y-2">
          <h3 className="text-sm font-medium text-muted-foreground">
            Period B
          </h3>
          <div className="flex gap-2 items-center">
            <input
              type="date"
              value={periodBStart}
              onChange={(e) => setPeriodBStart(e.target.value)}
              className="rounded-md border border-border bg-background px-3 py-1.5 text-sm"
            />
            <span className="text-muted-foreground">to</span>
            <input
              type="date"
              value={periodBEnd}
              onChange={(e) => setPeriodBEnd(e.target.value)}
              className="rounded-md border border-border bg-background px-3 py-1.5 text-sm"
            />
          </div>
        </div>
      </div>

      <button
        onClick={handleCompare}
        disabled={loading}
        className="self-start flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
      >
        {loading ? (
          <Loader2 size={16} className="animate-spin" />
        ) : (
          <ArrowRightLeft size={16} />
        )}
        Compare
      </button>

      {error && (
        <p className="text-sm text-red-400">{error}</p>
      )}

      {result && (
        <div className="space-y-6">
          {/* Summary */}
          <div className="rounded-lg border border-border bg-card p-4">
            <h3 className="text-sm font-semibold mb-2">Summary</h3>
            <p className="text-sm text-muted-foreground">{result.summary}</p>
          </div>

          {/* Key shift */}
          <div className="rounded-lg border border-border bg-card p-4">
            <h3 className="text-sm font-semibold mb-2">Key Shift</h3>
            <p className="text-sm text-muted-foreground">{result.key_shift}</p>
          </div>

          {/* Side-by-side */}
          <div className="grid grid-cols-2 gap-4">
            {/* Period A */}
            <div className="rounded-lg border border-border bg-card p-4 space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-semibold">{result.period_a_label}</h3>
                <span className="text-xs text-muted-foreground">
                  {result.period_a_doc_count} docs
                </span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">Sentiment:</span>
                <span className={`text-sm font-medium ${sentimentColor(result.sentiment_a)}`}>
                  {result.sentiment_a}
                </span>
              </div>
              <blockquote className="border-l-2 border-primary/50 pl-3 text-sm italic text-muted-foreground">
                {result.quote_a}
              </blockquote>
            </div>

            {/* Period B */}
            <div className="rounded-lg border border-border bg-card p-4 space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-semibold">{result.period_b_label}</h3>
                <span className="text-xs text-muted-foreground">
                  {result.period_b_doc_count} docs
                </span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">Sentiment:</span>
                <span className={`text-sm font-medium ${sentimentColor(result.sentiment_b)}`}>
                  {result.sentiment_b}
                </span>
              </div>
              <blockquote className="border-l-2 border-primary/50 pl-3 text-sm italic text-muted-foreground">
                {result.quote_b}
              </blockquote>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
