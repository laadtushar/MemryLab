import { useEffect, useRef } from "react";
import { useAppStore, type BackgroundTask } from "@/stores/app-store";
import { commands, events } from "@/lib/tauri";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { CheckCircle, Loader2, X, AlertCircle, Sparkles, StopCircle, RotateCcw } from "lucide-react";

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
  // Analysis pipeline stages
  themes: "Extracting themes",
  sampling: "Sampling documents",
  sentiment: "Classifying sentiment",
  beliefs: "Extracting beliefs",
  entities: "Extracting entities",
  insights: "Generating insights",
  contradictions: "Detecting contradictions",
  narratives: "Generating narratives",
};

function TaskRow({ task }: { task: BackgroundTask }) {
  const remove = useAppStore((s) => s.removeTask);
  const updateTask = useAppStore((s) => s.updateTask);

  const handleCancel = async () => {
    try {
      await commands.cancelTask(task.id);
      updateTask(task.id, { running: false, error: "Cancelled" });
    } catch {
      updateTask(task.id, { running: false, error: "Cancelled" });
    }
  };

  // Completed
  if (!task.running && task.result && !task.error) {
    const isImport = task.type === "import";
    return (
      <div className="flex items-center justify-between text-sm py-1.5 gap-3">
        <div className="flex items-center gap-2 text-green-400 min-w-0">
          <CheckCircle size={14} className="shrink-0" />
          <span className="truncate">{task.label}: {task.result}</span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          {isImport && (
            <button
              onClick={() => {
                remove(task.id);
                useAppStore.getState().setView("insights");
                const taskId = `analysis-${Date.now()}`;
                useAppStore.getState().addTask({
                  id: taskId,
                  type: "analysis",
                  label: "Running analysis pipeline",
                  progress: null,
                  result: null,
                  error: null,
                  running: true,
                });
                commands.runAnalysis(undefined, taskId)
                  .then((r) => {
                    useAppStore.getState().updateTask(taskId, {
                      running: false,
                      result: `${r.themes_extracted} themes, ${r.beliefs_extracted} beliefs, ${r.entities_extracted} entities`,
                    });
                  })
                  .catch((e) => {
                    useAppStore.getState().updateTask(taskId, { running: false, error: String(e) });
                  });
              }}
              className="flex items-center gap-1.5 rounded-md bg-primary px-3 py-1 text-xs font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
            >
              <Sparkles size={12} />
              Run Analysis
            </button>
          )}
          <button
            onClick={() => remove(task.id)}
            className="text-muted-foreground hover:text-foreground"
          >
            <X size={14} />
          </button>
        </div>
      </div>
    );
  }

  // Error / Cancelled / Interrupted
  if (!task.running && task.error) {
    const isInterrupted = task.error.includes("interrupted");
    return (
      <div className="flex items-center justify-between text-sm py-1.5">
        <div className={`flex items-center gap-2 min-w-0 ${isInterrupted ? "text-yellow-400" : "text-destructive"}`}>
          <AlertCircle size={14} className="shrink-0" />
          <span className="truncate">{task.label}: {task.error}</span>
        </div>
        <div className="flex items-center gap-1 shrink-0">
          {isInterrupted && (
            <button
              onClick={() => {
                remove(task.id);
                useAppStore.getState().setView("import");
              }}
              className="text-muted-foreground hover:text-primary shrink-0 transition-colors"
              title="Retry import"
            >
              <RotateCcw size={14} />
            </button>
          )}
          <button
            onClick={() => remove(task.id)}
            className="text-muted-foreground hover:text-foreground shrink-0"
          >
            <X size={14} />
          </button>
        </div>
      </div>
    );
  }

  // Running
  const progress = task.progress;
  const pct =
    progress && progress.total > 0
      ? Math.round((progress.current / progress.total) * 100)
      : null;

  return (
    <div className="flex items-center gap-3 text-sm py-1.5">
      <Loader2 size={14} className="animate-spin text-primary shrink-0" />
      <span className="text-muted-foreground shrink-0">{task.label}</span>
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
      {!progress && (
        <div className="flex-1 h-1.5 rounded-full bg-secondary overflow-hidden max-w-xs">
          <div className="h-full w-1/3 bg-primary/60 rounded-full animate-pulse" />
        </div>
      )}
      <button
        onClick={handleCancel}
        className="text-muted-foreground hover:text-destructive shrink-0 transition-colors"
        title="Cancel task"
      >
        <StopCircle size={14} />
      </button>
    </div>
  );
}

export function ImportProgressBanner() {
  const tasks = useAppStore((s) => s.backgroundTasks);
  const prevRunning = useRef(new Set<string>());

  // Global progress listener — always mounted in AppShell, never unmounts
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    events.onImportProgress((p) => {
      useAppStore.getState().updateTaskByProgress(p);
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  // Also listen for analysis progress events
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    events.onAnalysisProgress((p) => {
      const s = useAppStore.getState();
      const analysisTask = s.backgroundTasks.find((t) => t.type === "analysis" && t.running);
      if (analysisTask) {
        s.updateTask(analysisTask.id, {
          label: "Analyzing",
          progress: { import_id: analysisTask.id, stage: p.stage, current: 0, total: 0, message: p.message },
        });
      }
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  // Show interrupted tasks from previous session on app start
  useEffect(() => {
    commands.getInterruptedTasks().then((interrupted) => {
      const store = useAppStore.getState();
      for (const task of interrupted) {
        store.addTask({
          id: task.id,
          type: task.task_type as "import" | "analysis" | "embeddings",
          label: task.label,
          progress: null,
          result: null,
          error: task.error ?? "Task interrupted",
          running: false,
        });
      }
    }).catch(() => {});
  }, []);

  // Desktop notification when a task finishes
  useEffect(() => {
    const runningNow = new Set(
      tasks.filter((t) => t.running).map((t) => t.id)
    );
    const prev = prevRunning.current;

    for (const id of prev) {
      if (!runningNow.has(id)) {
        const task = tasks.find((t) => t.id === id);
        if (task) {
          const title = task.error
            ? `${task.label} failed`
            : `${task.label} complete`;
          const body = task.error ?? task.result ?? "";
          try { sendNotification({ title, body }); } catch { /* ignore */ }
        }
      }
    }

    prevRunning.current = runningNow;
  }, [tasks]);

  if (tasks.length === 0) return null;

  return (
    <div className="border-t border-border bg-card px-4 py-1 space-y-0">
      {tasks.map((task) => (
        <TaskRow key={task.id} task={task} />
      ))}
    </div>
  );
}
