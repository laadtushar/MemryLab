import { useEffect, useState } from "react";
import {
  commands,
  events,
  type OllamaStatus,
  type AppStats,
  type LlmConfig,
  type ProviderPreset,
  type UsageLogEntry,
  type PromptVersionInfo,
  type LogEntry,
  type EmbeddingProgress,
  type WatchedFolder,
} from "@/lib/tauri";
import {
  Cpu,
  Database,
  CheckCircle,
  XCircle,
  RefreshCw,
  Shield,
  Terminal,
  HardDrive,
  Save,
  Loader2,
  Key,
  ExternalLink,
  Sparkles,
  Zap,
  ChevronDown,
  ChevronUp,
  BarChart3,
  FileText,
  Edit3,
  Check,
  ScrollText,
  FolderOpen,
  Eye,
  Trash2,
  Plus,
} from "lucide-react";
import { useAppStore } from "@/stores/app-store";

export function SettingsPage() {
  const [ollamaStatus, setOllamaStatus] = useState<OllamaStatus | null>(null);
  const [appStats, setAppStats] = useState<AppStats | null>(null);
  const [testing, setTesting] = useState(false);
  const [config, setConfig] = useState<LlmConfig | null>(null);
  const [presets, setPresets] = useState<ProviderPreset[]>([]);
  const [saving, setSaving] = useState(false);
  const [saveMsg, setSaveMsg] = useState<string | null>(null);
  const [saveErr, setSaveErr] = useState<string | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [showUsageLog, setShowUsageLog] = useState(false);
  const [usageLog, setUsageLog] = useState<UsageLogEntry[]>([]);
  const [usageLoading, setUsageLoading] = useState(false);
  const [showPrompts, setShowPrompts] = useState(false);
  const [prompts, setPrompts] = useState<PromptVersionInfo[]>([]);
  const [promptsLoading, setPromptsLoading] = useState(false);
  const [expandedPrompt, setExpandedPrompt] = useState<string | null>(null);
  const [editingPrompt, setEditingPrompt] = useState<string | null>(null);
  const [editTemplate, setEditTemplate] = useState("");
  const [promptSaving, setPromptSaving] = useState(false);
  const [promptMsg, setPromptMsg] = useState<string | null>(null);
  const [showRecentLogs, setShowRecentLogs] = useState(false);
  const [recentLogs, setRecentLogs] = useState<LogEntry[]>([]);
  const [recentLogsLoading, setRecentLogsLoading] = useState(false);
  const [embedProgress, setEmbedProgress] = useState<EmbeddingProgress | null>(null);
  const [embedRunning, setEmbedRunning] = useState(false);
  const setView = useAppStore((s) => s.setView);

  // Listen for embedding progress events
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    events.onEmbeddingProgress((p) => setEmbedProgress(p)).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  const testConnection = async () => {
    setTesting(true);
    try {
      const status = await commands.testOllamaConnection();
      setOllamaStatus(status);
    } catch {
      setOllamaStatus({ connected: false, models: [] });
    }
    setTesting(false);
  };

  useEffect(() => {
    // Load config and stats immediately (fast, local DB queries)
    commands.getLlmConfig().then((c) => {
      setConfig(c);
      // Only test Ollama connection if it's the active provider
      if (c.active_provider === "ollama") {
        testConnection();
      }
    }).catch(() => {});
    commands.getAppStats().then(setAppStats).catch(() => {});
    commands.listProviderPresets().then(setPresets).catch(() => {});
  }, []);

  const saveConfig = async () => {
    if (!config) return;
    setSaving(true);
    setSaveMsg(null);
    setSaveErr(null);
    try {
      await commands.saveLlmConfig(config);
      setSaveMsg("Configuration saved. Provider switched.");
      testConnection();
    } catch (e) {
      setSaveErr(String(e));
    }
    setSaving(false);
  };

  const updateConfig = (patch: Partial<LlmConfig>) => {
    if (config) {
      setConfig({ ...config, ...patch });
      setSaveMsg(null);
      setSaveErr(null);
    }
  };

  const selectPreset = (preset: ProviderPreset) => {
    if (!config) return;
    if (preset.id === "ollama") {
      updateConfig({
        active_provider: "ollama",
        ollama_model: preset.default_model,
      });
    } else if (preset.id === "claude") {
      updateConfig({
        active_provider: "claude",
        claude_model: preset.default_model,
      });
    } else {
      updateConfig({
        active_provider: "openai_compat",
        openai_compat_base_url: preset.base_url,
        openai_compat_model: preset.default_model,
        openai_compat_embedding_model: preset.embedding_model,
        openai_compat_provider_id: preset.id,
      });
    }
  };

  const activePresetId =
    config?.active_provider === "openai_compat"
      ? config.openai_compat_provider_id
      : config?.active_provider;

  const activePreset = presets.find((p) => p.id === activePresetId);

  return (
    <div className="p-6 max-w-2xl mx-auto space-y-8 h-full overflow-y-auto">
      <h1 className="text-2xl font-semibold">Settings</h1>

      {/* ── Provider Selection ── */}
      <section className="space-y-4">
        <h2 className="text-lg font-medium flex items-center gap-2">
          <Cpu size={20} className="text-primary" /> AI Provider
        </h2>

        {/* Provider preset grid */}
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
          {presets.map((preset) => {
            const isActive = activePresetId === preset.id;
            return (
              <button
                key={preset.id}
                onClick={() => selectPreset(preset)}
                className={`relative rounded-lg border p-3 text-left transition-colors ${
                  isActive
                    ? "border-primary bg-primary/10"
                    : "border-border bg-card hover:border-border/80"
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="text-sm font-medium truncate">
                    {preset.name}
                  </span>
                  {preset.free_tier && (
                    <span className="rounded-full bg-green-500/10 text-green-400 border border-green-500/20 px-1.5 py-0 text-[9px] font-semibold shrink-0">
                      FREE
                    </span>
                  )}
                </div>
                <p className="text-[10px] text-muted-foreground line-clamp-2 leading-tight">
                  {preset.description}
                </p>
                {preset.supports_embeddings && (
                  <div className="mt-1.5 flex items-center gap-1 text-[9px] text-primary/70">
                    <Sparkles size={8} /> Chat + Embeddings
                  </div>
                )}
                {isActive && (
                  <div className="absolute top-1.5 right-1.5">
                    <CheckCircle size={14} className="text-primary" />
                  </div>
                )}
              </button>
            );
          })}
        </div>

        {/* Active provider config */}
        {config && activePreset && (
          <div className="rounded-lg border border-border bg-card p-4 space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Zap size={14} className="text-primary" />
                {activePreset.name} Configuration
              </h3>
              <a
                href={activePreset.signup_url}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 text-[10px] text-primary hover:underline"
              >
                {activePreset.id === "ollama"
                  ? "Download Ollama"
                  : "Get Free API Key"}
                <ExternalLink size={10} />
              </a>
            </div>

            {/* Ollama-specific */}
            {activePreset.id === "ollama" && (
              <div className="space-y-3">
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs text-muted-foreground">
                      Server URL
                    </label>
                    <input
                      type="text"
                      value={config.ollama_url}
                      onChange={(e) =>
                        updateConfig({ ollama_url: e.target.value })
                      }
                      className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono"
                    />
                  </div>
                  <div>
                    <label className="text-xs text-muted-foreground">
                      Model
                    </label>
                    <input
                      type="text"
                      value={config.ollama_model}
                      onChange={(e) =>
                        updateConfig({ ollama_model: e.target.value })
                      }
                      className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono"
                    />
                  </div>
                  <div className="col-span-2">
                    <label className="text-xs text-muted-foreground">
                      Embedding Model
                    </label>
                    <input
                      type="text"
                      value={config.embedding_model}
                      onChange={(e) =>
                        updateConfig({ embedding_model: e.target.value })
                      }
                      className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono"
                    />
                  </div>
                </div>

                {/* Ollama status */}
                <div className="flex items-center justify-between pt-2 border-t border-border/50">
                  <div className="flex items-center gap-2 text-sm">
                    {ollamaStatus?.connected ? (
                      <span className="flex items-center gap-1 text-green-400">
                        <CheckCircle size={14} /> Connected
                      </span>
                    ) : (
                      <span className="flex items-center gap-1 text-red-400">
                        <XCircle size={14} /> Not connected
                      </span>
                    )}
                    {ollamaStatus?.connected && (
                      <span className="text-xs text-muted-foreground">
                        ({ollamaStatus.models.length} models)
                      </span>
                    )}
                  </div>
                  <button
                    onClick={testConnection}
                    disabled={testing}
                    className="rounded-md bg-secondary px-3 py-1 text-xs hover:bg-secondary/80 disabled:opacity-50"
                  >
                    <RefreshCw
                      size={12}
                      className={testing ? "animate-spin" : ""}
                    />
                  </button>
                </div>

                {ollamaStatus?.connected &&
                  ollamaStatus.models.length > 0 && (
                    <div className="flex flex-wrap gap-1">
                      {ollamaStatus.models.map((m) => (
                        <span
                          key={m}
                          className="rounded bg-secondary px-1.5 py-0.5 text-[10px] font-mono"
                        >
                          {m}
                        </span>
                      ))}
                    </div>
                  )}

                {ollamaStatus && !ollamaStatus.connected && (
                  <div className="rounded bg-secondary/50 px-3 py-2 text-xs text-muted-foreground space-y-1">
                    <div className="flex items-center gap-1.5">
                      <Terminal size={10} /> ollama pull nomic-embed-text
                    </div>
                    <div className="flex items-center gap-1.5">
                      <Terminal size={10} /> ollama pull llama3.1:8b
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Claude-specific */}
            {activePreset.id === "claude" && (
              <div className="grid grid-cols-2 gap-3">
                <div className="col-span-2">
                  <label className="text-xs text-muted-foreground">
                    API Key
                  </label>
                  <input
                    type="password"
                    value={config.claude_api_key ?? ""}
                    onChange={(e) =>
                      updateConfig({
                        claude_api_key: e.target.value || null,
                      })
                    }
                    placeholder="sk-ant-..."
                    className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono"
                  />
                </div>
                <div>
                  <label className="text-xs text-muted-foreground">Model</label>
                  <select
                    value={config.claude_model}
                    onChange={(e) =>
                      updateConfig({ claude_model: e.target.value })
                    }
                    className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm"
                  >
                    {activePreset.models.map((m) => (
                      <option key={m.id} value={m.id}>
                        {m.name}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            )}

            {/* OpenAI-compatible providers */}
            {activePreset.id !== "ollama" &&
              activePreset.id !== "claude" && (
                <div className="space-y-3">
                  <div className="grid grid-cols-2 gap-3">
                    <div className="col-span-2">
                      <label className="text-xs text-muted-foreground">
                        API Key
                      </label>
                      <input
                        type="password"
                        value={config.openai_compat_api_key ?? ""}
                        onChange={(e) =>
                          updateConfig({
                            openai_compat_api_key: e.target.value || null,
                          })
                        }
                        placeholder="Paste your API key here..."
                        className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono"
                      />
                    </div>
                    <div>
                      <label className="text-xs text-muted-foreground">
                        Model
                      </label>
                      <select
                        value={config.openai_compat_model ?? ""}
                        onChange={(e) =>
                          updateConfig({
                            openai_compat_model: e.target.value,
                          })
                        }
                        className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm"
                      >
                        {activePreset.models.map((m) => (
                          <option key={m.id} value={m.id}>
                            {m.name}
                            {m.free ? " (free)" : ""}
                          </option>
                        ))}
                      </select>
                    </div>
                    {activePreset.supports_embeddings &&
                      activePreset.embedding_model && (
                        <div>
                          <label className="text-xs text-muted-foreground">
                            Embedding Model
                          </label>
                          <input
                            type="text"
                            value={
                              config.openai_compat_embedding_model ??
                              activePreset.embedding_model ??
                              ""
                            }
                            onChange={(e) =>
                              updateConfig({
                                openai_compat_embedding_model: e.target.value,
                              })
                            }
                            className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono"
                          />
                        </div>
                      )}
                  </div>

                  <div className="text-[10px] text-muted-foreground flex items-center justify-between">
                    <span>
                      Rate limits: {activePreset.rate_limits}
                    </span>
                    {!activePreset.supports_embeddings && (
                      <span className="text-yellow-500">
                        Embeddings via Ollama (local)
                      </span>
                    )}
                  </div>
                </div>
              )}

            {/* Save button */}
            <div className="flex items-center gap-3 pt-2 border-t border-border/50">
              <button
                onClick={saveConfig}
                disabled={saving}
                className="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              >
                {saving ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : (
                  <Save size={14} />
                )}
                Save & Activate
              </button>
              {saveMsg && (
                <span className="text-sm text-green-400">{saveMsg}</span>
              )}
              {saveErr && (
                <span className="text-sm text-destructive">{saveErr}</span>
              )}
            </div>
          </div>
        )}

        {/* Advanced: custom endpoint */}
        <button
          onClick={() => setShowAdvanced(!showAdvanced)}
          className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground"
        >
          {showAdvanced ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
          Custom OpenAI-compatible endpoint
        </button>

        {showAdvanced && config && (
          <div className="rounded-lg border border-border bg-card p-4 space-y-3">
            <p className="text-[10px] text-muted-foreground">
              Connect any OpenAI-compatible API by providing a base URL and key.
            </p>
            <div className="grid grid-cols-2 gap-3">
              <div className="col-span-2">
                <label className="text-xs text-muted-foreground">
                  Base URL
                </label>
                <input
                  type="text"
                  value={config.openai_compat_base_url ?? ""}
                  onChange={(e) =>
                    updateConfig({
                      active_provider: "openai_compat",
                      openai_compat_base_url: e.target.value,
                      openai_compat_provider_id: "custom",
                    })
                  }
                  placeholder="https://api.example.com/v1"
                  className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono"
                />
              </div>
              <div>
                <label className="text-xs text-muted-foreground">
                  API Key
                </label>
                <input
                  type="password"
                  value={config.openai_compat_api_key ?? ""}
                  onChange={(e) =>
                    updateConfig({
                      openai_compat_api_key: e.target.value || null,
                    })
                  }
                  className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono"
                />
              </div>
              <div>
                <label className="text-xs text-muted-foreground">
                  Model ID
                </label>
                <input
                  type="text"
                  value={config.openai_compat_model ?? ""}
                  onChange={(e) =>
                    updateConfig({ openai_compat_model: e.target.value })
                  }
                  placeholder="gpt-4o-mini"
                  className="mt-1 w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono"
                />
              </div>
            </div>
          </div>
        )}
      </section>

      {/* ── Data Overview ── */}
      <section className="space-y-4">
        <h2 className="text-lg font-medium flex items-center gap-2">
          <Database size={20} className="text-primary" /> Data Overview
        </h2>
        <div className="rounded-lg border border-border bg-card p-4">
          {appStats ? (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-2xl font-bold tabular-nums">
                    {appStats.total_documents}
                  </p>
                  <p className="text-sm text-muted-foreground">Documents</p>
                </div>
                <div>
                  <p className="text-2xl font-bold tabular-nums">
                    {appStats.total_memory_facts}
                  </p>
                  <p className="text-sm text-muted-foreground">Memory Facts</p>
                </div>
              </div>
              {appStats.date_range && (
                <div className="pt-2 border-t border-border/50">
                  <p className="text-sm text-muted-foreground">
                    Date range:{" "}
                    {new Date(appStats.date_range[0]).toLocaleDateString()} —{" "}
                    {new Date(appStats.date_range[1]).toLocaleDateString()}
                  </p>
                </div>
              )}
              {appStats.total_documents > 0 && (
                <div className="pt-2 border-t border-border/50 space-y-2">
                  <button
                    onClick={() => {
                      if (embedRunning) return;
                      setEmbedRunning(true);
                      setEmbedProgress(null);
                      setSaveErr(null);
                      commands.generateEmbeddings().then((r) => {
                        setEmbedRunning(false);
                        setEmbedProgress({ stage: "complete", current: r.embeddings_generated, total: r.chunks_processed, message: `Done! ${r.embeddings_generated} embeddings generated` });
                      }).catch((e) => {
                        setEmbedRunning(false);
                        setSaveErr(String(e));
                      });
                    }}
                    disabled={embedRunning}
                    className="rounded-md bg-secondary px-3 py-1.5 text-sm hover:bg-secondary/80 disabled:opacity-50"
                  >
                    {embedRunning ? "Generating..." : "Generate Embeddings"}
                  </button>
                  {embedProgress && (
                    <div className="space-y-1.5">
                      <div className="flex justify-between text-xs text-muted-foreground">
                        <span>{embedProgress.message}</span>
                        {embedProgress.total > 0 && (
                          <span className="tabular-nums">{embedProgress.current}/{embedProgress.total}</span>
                        )}
                      </div>
                      {embedProgress.total > 0 && (
                        <div className="h-1.5 rounded-full bg-secondary overflow-hidden">
                          <div
                            className="h-full bg-primary rounded-full transition-all duration-300"
                            style={{ width: `${(embedProgress.current / embedProgress.total) * 100}%` }}
                          />
                        </div>
                      )}
                    </div>
                  )}
                  {!embedProgress && !embedRunning && (
                    <p className="text-[10px] text-muted-foreground">
                      Required for semantic search and RAG. Uses the configured embedding provider.
                    </p>
                  )}
                </div>
              )}
            </div>
          ) : (
            <p className="text-muted-foreground text-sm">Loading...</p>
          )}
        </div>
      </section>

      {/* ── Watched Folders ── */}
      <WatchedFoldersSection />

      {/* ── Storage ── */}
      <section className="space-y-4">
        <h2 className="text-lg font-medium flex items-center gap-2">
          <HardDrive size={20} className="text-primary" /> Storage
        </h2>
        <div className="rounded-lg border border-border bg-card p-4 space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Database</span>
            <span className="font-mono">SQLite (WAL mode)</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Full-text search</span>
            <span className="font-mono">FTS5 (BM25)</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Vector store</span>
            <span className="font-mono">SQLite (cosine sim)</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Graph store</span>
            <span className="font-mono">SQLite (adjacency + CTE)</span>
          </div>
        </div>
      </section>

      {/* ── Embedding Provider ── */}
      <section className="space-y-4">
        <h2 className="text-lg font-medium flex items-center gap-2">
          <Database size={20} className="text-primary" /> Embedding Provider
        </h2>
        <p className="text-xs text-muted-foreground">
          Choose which provider generates embeddings for search. You can use a different provider than your LLM.
          Ollama (local) is recommended for privacy; cloud providers may be faster.
        </p>
        {config && (
          <div className="rounded-lg border border-border bg-card p-4 space-y-3">
            <select
              value={config.active_embedding_provider ?? "same"}
              onChange={(e) => updateConfig({ active_embedding_provider: e.target.value })}
              className="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm"
            >
              <option value="same">Same as LLM provider ({config.active_provider === "openai_compat" ? (config.openai_compat_provider_id ?? "cloud") : config.active_provider})</option>
              <option value="ollama">Ollama (local, private)</option>
              {config.active_provider !== "ollama" && config.openai_compat_provider_id && (
                <option value="openai_compat">{config.openai_compat_provider_id} (cloud)</option>
              )}
            </select>
            <div className="text-xs text-muted-foreground space-y-1">
              <p><strong>Tip:</strong> Use a cloud LLM (Gemini, Groq) for analysis + Ollama for embeddings to keep search data private.</p>
            </div>
          </div>
        )}
      </section>

      {/* ── Recommended Models ── */}
      <section className="space-y-4">
        <h2 className="text-lg font-medium flex items-center gap-2">
          <Sparkles size={20} className="text-primary" /> Recommended Models
        </h2>
        <div className="rounded-lg border border-border bg-card p-4 space-y-2 text-xs">
          <table className="w-full">
            <thead>
              <tr className="text-muted-foreground text-left">
                <th className="pb-1">VRAM</th>
                <th className="pb-1">LLM Model</th>
                <th className="pb-1">Embedding</th>
                <th className="pb-1">Speed</th>
              </tr>
            </thead>
            <tbody className="font-mono">
              <tr><td>4 GB</td><td>llama3.2:3b</td><td>nomic-embed-text</td><td>~40 t/s</td></tr>
              <tr><td>8 GB</td><td>llama3.1:8b</td><td>nomic-embed-text</td><td>~35 t/s</td></tr>
              <tr><td>12 GB</td><td>qwen2.5:14b-instruct-q5_K_M</td><td>nomic-embed-text</td><td>~25 t/s</td></tr>
              <tr><td>16 GB+</td><td>qwen2.5:32b-instruct-q4_K_M</td><td>nomic-embed-text</td><td>~15 t/s</td></tr>
            </tbody>
          </table>
          <p className="text-muted-foreground pt-2">
            Larger models produce better belief extraction and contradiction detection. For cloud (no GPU), use Gemini Flash (free) or Groq (free).
          </p>
        </div>
      </section>

      {/* ── Privacy ── */}
      <section className="space-y-4">
        <h2 className="text-lg font-medium flex items-center gap-2">
          <Shield size={20} className="text-primary" /> Privacy
        </h2>
        <div className="rounded-lg border border-border bg-card p-4 space-y-2">
          <div className="flex items-center gap-2 text-sm">
            <CheckCircle size={14} className="text-green-400 shrink-0" />
            <span>All data stored locally on your device</span>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <CheckCircle size={14} className="text-green-400 shrink-0" />
            <span>Zero telemetry — no analytics, no phone-home</span>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <CheckCircle size={14} className="text-green-400 shrink-0" />
            <span>
              {config?.active_provider === "ollama"
                ? "All processing runs locally via Ollama"
                : "Embeddings generated locally via Ollama"}
            </span>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <CheckCircle size={14} className="text-green-400 shrink-0" />
            <span>API keys stored in OS keychain (Windows Credential Manager / macOS Keychain)</span>
          </div>
          {config?.active_provider !== "ollama" && (
            <div className="flex items-center gap-2 text-sm text-yellow-400">
              <Key size={14} className="shrink-0" />
              <span>
                Cloud provider active — analysis prompts are sent to{" "}
                {activePreset?.name ?? "the cloud provider"}
              </span>
            </div>
          )}
        </div>
      </section>

      {/* ── Cloud Usage Log ── */}
      <section className="space-y-4">
        <h2 className="text-lg font-medium flex items-center gap-2">
          <BarChart3 size={20} className="text-primary" /> Cloud Usage Log
        </h2>
        <button
          onClick={async () => {
            const next = !showUsageLog;
            setShowUsageLog(next);
            if (next && usageLog.length === 0) {
              setUsageLoading(true);
              try {
                const log = await commands.getUsageLog(20);
                setUsageLog(log);
              } catch {
                /* empty */
              }
              setUsageLoading(false);
            }
          }}
          className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground"
        >
          {showUsageLog ? (
            <ChevronUp size={12} />
          ) : (
            <ChevronDown size={12} />
          )}
          {showUsageLog ? "Hide" : "Show"} recent LLM API calls
        </button>

        {showUsageLog && (
          <div className="rounded-lg border border-border bg-card p-4 space-y-3">
            {usageLoading ? (
              <p className="text-sm text-muted-foreground">Loading...</p>
            ) : usageLog.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                No API calls logged yet.
              </p>
            ) : (
              <>
                <div className="overflow-x-auto">
                  <table className="w-full text-xs">
                    <thead>
                      <tr className="border-b border-border/50 text-muted-foreground">
                        <th className="text-left py-1.5 pr-3 font-medium">
                          Time
                        </th>
                        <th className="text-left py-1.5 pr-3 font-medium">
                          Provider
                        </th>
                        <th className="text-left py-1.5 pr-3 font-medium">
                          Model
                        </th>
                        <th className="text-right py-1.5 pr-3 font-medium">
                          Tokens
                        </th>
                        <th className="text-left py-1.5 pr-3 font-medium">
                          Purpose
                        </th>
                        <th className="text-right py-1.5 font-medium">
                          Duration
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      {usageLog.map((entry) => (
                        <tr
                          key={entry.id}
                          className="border-b border-border/30"
                        >
                          <td className="py-1.5 pr-3 font-mono text-muted-foreground">
                            {new Date(entry.timestamp + "Z").toLocaleString(
                              undefined,
                              {
                                month: "short",
                                day: "numeric",
                                hour: "2-digit",
                                minute: "2-digit",
                              },
                            )}
                          </td>
                          <td className="py-1.5 pr-3">{entry.provider}</td>
                          <td className="py-1.5 pr-3 font-mono">
                            {entry.model}
                          </td>
                          <td className="py-1.5 pr-3 text-right tabular-nums">
                            {entry.prompt_tokens + entry.completion_tokens}
                          </td>
                          <td className="py-1.5 pr-3 text-muted-foreground">
                            {entry.purpose}
                          </td>
                          <td className="py-1.5 text-right tabular-nums font-mono">
                            {entry.duration_ms}ms
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <div className="pt-2 border-t border-border/50 text-xs text-muted-foreground flex justify-between">
                  <span>
                    Total tokens:{" "}
                    {usageLog
                      .reduce(
                        (sum, e) =>
                          sum + e.prompt_tokens + e.completion_tokens,
                        0,
                      )
                      .toLocaleString()}
                  </span>
                  <span>{usageLog.length} calls shown</span>
                </div>
              </>
            )}
          </div>
        )}
      </section>

      {/* ── Prompt Registry ── */}
      <section className="space-y-4">
        <h2 className="text-lg font-medium flex items-center gap-2">
          <FileText size={20} className="text-primary" /> Prompt Registry
        </h2>
        <button
          onClick={async () => {
            const next = !showPrompts;
            setShowPrompts(next);
            if (next && prompts.length === 0) {
              setPromptsLoading(true);
              try {
                const data = await commands.listPrompts();
                setPrompts(data);
              } catch {
                /* empty */
              }
              setPromptsLoading(false);
            }
          }}
          className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground"
        >
          {showPrompts ? (
            <ChevronUp size={12} />
          ) : (
            <ChevronDown size={12} />
          )}
          {showPrompts ? "Hide" : "Show"} versioned prompt templates
        </button>

        {showPrompts && (
          <div className="rounded-lg border border-border bg-card p-4 space-y-3">
            {promptsLoading ? (
              <p className="text-sm text-muted-foreground">Loading...</p>
            ) : prompts.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                No prompts registered yet.
              </p>
            ) : (
              <>
                {promptMsg && (
                  <div className="text-xs text-green-400 pb-2">{promptMsg}</div>
                )}
                <div className="space-y-2">
                  {(() => {
                    // Group prompts by name
                    const grouped: Record<string, PromptVersionInfo[]> = {};
                    for (const p of prompts) {
                      if (!grouped[p.name]) grouped[p.name] = [];
                      grouped[p.name].push(p);
                    }
                    return Object.entries(grouped).map(([name, versions]) => {
                      const active = versions.find((v) => v.is_active) ?? versions[0];
                      const isExpanded = expandedPrompt === name;
                      const isEditing = editingPrompt === name;
                      return (
                        <div
                          key={name}
                          className="rounded-md border border-border/50 bg-background"
                        >
                          <button
                            onClick={() => {
                              setExpandedPrompt(isExpanded ? null : name);
                              setEditingPrompt(null);
                            }}
                            className="w-full flex items-center justify-between px-3 py-2 text-sm hover:bg-secondary/30 transition-colors"
                          >
                            <div className="flex items-center gap-2">
                              <span className="font-medium">
                                {name.replace(/_/g, " ")}
                              </span>
                              <span className="rounded-full bg-primary/10 text-primary border border-primary/20 px-1.5 py-0 text-[9px] font-semibold">
                                {active.version}
                              </span>
                              {active.is_active && (
                                <span className="rounded-full bg-green-500/10 text-green-400 border border-green-500/20 px-1.5 py-0 text-[9px] font-semibold">
                                  ACTIVE
                                </span>
                              )}
                            </div>
                            <div className="flex items-center gap-2 text-muted-foreground">
                              <span className="text-[10px]">
                                {versions.length} version{versions.length !== 1 ? "s" : ""}
                              </span>
                              {isExpanded ? (
                                <ChevronUp size={12} />
                              ) : (
                                <ChevronDown size={12} />
                              )}
                            </div>
                          </button>

                          {isExpanded && (
                            <div className="px-3 pb-3 space-y-2">
                              {/* Version selector if multiple */}
                              {versions.length > 1 && (
                                <div className="flex flex-wrap gap-1">
                                  {versions.map((v) => (
                                    <button
                                      key={v.version}
                                      onClick={async () => {
                                        try {
                                          await commands.setActivePrompt(name, v.version);
                                          const data = await commands.listPrompts();
                                          setPrompts(data);
                                          setPromptMsg(`Activated ${name} ${v.version}`);
                                        } catch { /* ignore */ }
                                      }}
                                      className={`rounded px-2 py-0.5 text-[10px] border transition-colors ${
                                        v.is_active
                                          ? "border-primary bg-primary/10 text-primary"
                                          : "border-border bg-secondary text-muted-foreground hover:text-foreground"
                                      }`}
                                    >
                                      {v.version}
                                      {v.is_active ? " (active)" : ""}
                                    </button>
                                  ))}
                                </div>
                              )}

                              {/* Template display / edit */}
                              {isEditing ? (
                                <div className="space-y-2">
                                  <textarea
                                    value={editTemplate}
                                    onChange={(e) => setEditTemplate(e.target.value)}
                                    rows={12}
                                    className="w-full rounded-md border border-input bg-background px-3 py-2 text-xs font-mono leading-relaxed resize-y"
                                  />
                                  <div className="flex items-center gap-2">
                                    <button
                                      onClick={async () => {
                                        setPromptSaving(true);
                                        try {
                                          const nextVersion = `v${versions.length + 1}`;
                                          await commands.updatePrompt(name, nextVersion, editTemplate);
                                          const data = await commands.listPrompts();
                                          setPrompts(data);
                                          setEditingPrompt(null);
                                          setPromptMsg(`Saved ${name} ${nextVersion} and activated.`);
                                        } catch {
                                          setPromptMsg("Failed to save prompt.");
                                        }
                                        setPromptSaving(false);
                                      }}
                                      disabled={promptSaving}
                                      className="flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                                    >
                                      {promptSaving ? (
                                        <Loader2 size={12} className="animate-spin" />
                                      ) : (
                                        <Check size={12} />
                                      )}
                                      Save as new version
                                    </button>
                                    <button
                                      onClick={() => setEditingPrompt(null)}
                                      className="rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground"
                                    >
                                      Cancel
                                    </button>
                                  </div>
                                </div>
                              ) : (
                                <div className="space-y-2">
                                  <pre className="rounded-md bg-secondary/50 px-3 py-2 text-[11px] font-mono leading-relaxed whitespace-pre-wrap max-h-60 overflow-y-auto">
                                    {active.template}
                                  </pre>
                                  <button
                                    onClick={() => {
                                      setEditingPrompt(name);
                                      setEditTemplate(active.template);
                                    }}
                                    className="flex items-center gap-1.5 rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
                                  >
                                    <Edit3 size={12} /> Edit template
                                  </button>
                                </div>
                              )}
                            </div>
                          )}
                        </div>
                      );
                    });
                  })()}
                </div>
              </>
            )}
          </div>
        )}
      </section>

      {/* ── Recent Logs ── */}
      <section className="space-y-4">
        <h2 className="text-lg font-medium flex items-center gap-2">
          <ScrollText size={20} className="text-primary" /> Application Logs
        </h2>
        <button
          onClick={async () => {
            const next = !showRecentLogs;
            setShowRecentLogs(next);
            if (next && recentLogs.length === 0) {
              setRecentLogsLoading(true);
              try {
                const entries = await commands.getAppLogs(10);
                setRecentLogs(entries);
              } catch {
                /* empty */
              }
              setRecentLogsLoading(false);
            }
          }}
          className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground"
        >
          {showRecentLogs ? (
            <ChevronUp size={12} />
          ) : (
            <ChevronDown size={12} />
          )}
          {showRecentLogs ? "Hide" : "Show"} recent application logs
        </button>

        {showRecentLogs && (
          <div className="rounded-lg border border-border bg-card p-4 space-y-3">
            {recentLogsLoading ? (
              <p className="text-sm text-muted-foreground">Loading...</p>
            ) : recentLogs.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                No log entries yet.
              </p>
            ) : (
              <div className="space-y-1 font-mono text-[11px]">
                {recentLogs.map((entry, i) => (
                  <div key={i} className="flex gap-2 py-0.5">
                    <span
                      className={`shrink-0 font-semibold ${
                        entry.level === "ERROR"
                          ? "text-red-400"
                          : entry.level === "WARN"
                            ? "text-yellow-400"
                            : entry.level === "INFO"
                              ? "text-blue-400"
                              : "text-zinc-400"
                      }`}
                    >
                      {entry.level.padEnd(5)}
                    </span>
                    <span className="text-foreground break-all">
                      {entry.message}
                    </span>
                  </div>
                ))}
              </div>
            )}
            <button
              onClick={() => setView("logs")}
              className="flex items-center gap-1.5 rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              <ScrollText size={12} /> View all logs
            </button>
          </div>
        )}
      </section>

      {/* ── About & Open Source ── */}
      <section className="space-y-4 pb-8">
        <div className="rounded-lg border border-border bg-card p-6 space-y-5">
          {/* Logo / Title */}
          <div className="text-center space-y-2">
            <div className="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-primary/10 text-primary text-xl font-bold">
              M
            </div>
            <h2 className="text-lg font-bold">Memory Palace</h2>
            <p className="text-sm text-muted-foreground">
              A searchable, visual timeline of how your thinking evolved.
            </p>
            <p className="text-xs text-muted-foreground">Version 0.1.0</p>
          </div>

          {/* Open Source */}
          <div className="rounded-lg bg-secondary/30 p-4 space-y-2">
            <h3 className="text-sm font-semibold flex items-center gap-2">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" className="text-primary">
                <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
              </svg>
              Open Source
            </h3>
            <p className="text-xs text-muted-foreground leading-relaxed">
              Memory Palace is open source and built in public. All data stays on your device.
              Contributions, bug reports, and feature requests are welcome.
            </p>
            <div className="flex items-center gap-3 pt-1">
              <a
                href="https://github.com/laadtushar/MemryLab"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 rounded-md bg-primary/10 border border-primary/20 px-3 py-1.5 text-xs font-medium text-primary hover:bg-primary/20 transition-colors"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
                </svg>
                Star on GitHub
              </a>
              <a
                href="https://github.com/laadtushar/MemryLab/issues"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                Report Issue
              </a>
              <a
                href="https://github.com/laadtushar/MemryLab/blob/master/CONTRIBUTING.md"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                Contribute
              </a>
            </div>
          </div>

          {/* Creator */}
          <div className="rounded-lg bg-secondary/30 p-4 space-y-3">
            <h3 className="text-sm font-semibold">Created by</h3>
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-primary/20 flex items-center justify-center text-primary font-bold text-sm">
                TL
              </div>
              <div>
                <p className="text-sm font-medium">Tushar Laad</p>
                <p className="text-xs text-muted-foreground">Builder & Designer</p>
              </div>
            </div>
            <div className="flex items-center gap-3 pt-1">
              <a
                href="https://github.com/laadtushar"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
                </svg>
                @laadtushar
              </a>
              <a
                href="https://www.linkedin.com/in/tusharlaad2002/"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1.5 rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433a2.062 2.062 0 01-2.063-2.065 2.064 2.064 0 112.063 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z" />
                </svg>
                LinkedIn
              </a>
            </div>
          </div>

          {/* Tech stack */}
          <div className="text-center space-y-1 pt-2 border-t border-border/50">
            <p className="text-[10px] text-muted-foreground">
              Built with Rust, React, Tauri, SQLite, and D3.js
            </p>
            <p className="text-[10px] text-muted-foreground">
              Privacy-first. Local-first. Open source.
            </p>
          </div>
        </div>
      </section>
    </div>
  );
}

/* ── Watched Folders Section ── */
function WatchedFoldersSection() {
  const [folders, setFolders] = useState<WatchedFolder[]>([]);
  const [loading, setLoading] = useState(true);
  const { addTask } = useAppStore();

  const loadFolders = async () => {
    try {
      const data = await commands.listWatchFolders();
      setFolders(data);
    } catch { /* */ }
    setLoading(false);
  };

  useEffect(() => { loadFolders(); }, []);

  const handleAdd = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const path = await open({ directory: true }) as string | null;
      if (!path) return;

      const folderName = path.split(/[\\/]/).pop() ?? path;
      const importId = `watch-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
      addTask({
        id: importId,
        type: "import",
        label: `Importing ${folderName}`,
        progress: null,
        result: null,
        error: null,
        running: true,
      });

      await commands.addWatchFolder(path, undefined, importId);
      await loadFolders();
    } catch { /* */ }
  };

  const handleRemove = async (path: string) => {
    try {
      await commands.removeWatchFolder(path);
      await loadFolders();
    } catch { /* */ }
  };

  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium flex items-center gap-2">
        <Eye size={20} className="text-primary" /> Watched Folders
      </h2>
      <p className="text-xs text-muted-foreground">
        Add folders to watch for changes. New or modified files are automatically imported in the background.
      </p>
      <div className="rounded-lg border border-border bg-card p-4 space-y-3">
        {loading ? (
          <p className="text-sm text-muted-foreground">Loading...</p>
        ) : folders.length === 0 ? (
          <p className="text-sm text-muted-foreground">No watched folders. Add one to enable auto-import.</p>
        ) : (
          <div className="space-y-2">
            {folders.map((f) => (
              <div key={f.path} className="flex items-center justify-between rounded-md bg-muted/50 px-3 py-2">
                <div className="flex items-center gap-2 min-w-0">
                  <FolderOpen size={14} className="text-primary shrink-0" />
                  <span className="text-sm font-mono truncate">{f.path}</span>
                  {f.enabled ? (
                    <span className="text-[10px] bg-green-500/15 text-green-400 px-1.5 py-0.5 rounded shrink-0">Active</span>
                  ) : (
                    <span className="text-[10px] bg-muted text-muted-foreground px-1.5 py-0.5 rounded shrink-0">Paused</span>
                  )}
                </div>
                <button onClick={() => handleRemove(f.path)} className="text-muted-foreground hover:text-destructive shrink-0">
                  <Trash2 size={14} />
                </button>
              </div>
            ))}
          </div>
        )}
        <button
          onClick={handleAdd}
          className="flex items-center gap-1.5 rounded-md bg-secondary px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
        >
          <Plus size={14} /> Add Folder
        </button>
      </div>
    </section>
  );
}
