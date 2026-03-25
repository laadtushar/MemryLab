import { X } from "lucide-react";
import type { EntityResponse } from "@/lib/tauri";

const typeColors: Record<string, string> = {
  person: "bg-blue-500/10 text-blue-400 border-blue-500/20",
  place: "bg-green-500/10 text-green-400 border-green-500/20",
  organization: "bg-purple-500/10 text-purple-400 border-purple-500/20",
  concept: "bg-amber-500/10 text-amber-400 border-amber-500/20",
  topic: "bg-amber-500/10 text-amber-400 border-amber-500/20",
};

interface NodeDetailPanelProps {
  entity: EntityResponse;
  onClose: () => void;
}

export function NodeDetailPanel({ entity, onClose }: NodeDetailPanelProps) {
  return (
    <div className="w-72 border-l border-border bg-card flex flex-col overflow-y-auto">
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <span className="text-sm font-medium text-muted-foreground">
          Node Details
        </span>
        <button
          onClick={onClose}
          className="h-6 w-6 flex items-center justify-center rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
        >
          <X size={14} />
        </button>
      </div>

      <div className="p-4 space-y-4">
        <div>
          <h2 className="text-lg font-bold">{entity.name}</h2>
          <span
            className={`mt-1 inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium ${
              typeColors[entity.entity_type] ?? "bg-secondary border-border"
            }`}
          >
            {entity.entity_type}
          </span>
        </div>

        <div className="space-y-3">
          <div>
            <span className="text-xs text-muted-foreground">Mentions</span>
            <p className="text-sm font-medium tabular-nums">
              {entity.mention_count}
            </p>
          </div>

          {entity.first_seen && (
            <div>
              <span className="text-xs text-muted-foreground">First seen</span>
              <p className="text-sm">
                {new Date(entity.first_seen).toLocaleDateString(undefined, {
                  year: "numeric",
                  month: "short",
                  day: "numeric",
                })}
              </p>
            </div>
          )}

          {entity.last_seen && (
            <div>
              <span className="text-xs text-muted-foreground">Last seen</span>
              <p className="text-sm">
                {new Date(entity.last_seen).toLocaleDateString(undefined, {
                  year: "numeric",
                  month: "short",
                  day: "numeric",
                })}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
