import { useEffect, useState } from "react";
import {
  commands,
  type OllamaStatus,
  type AppStats,
  type LlmConfig,
  type ProviderPreset,
  type UsageLogEntry,
  type PromptVersionInfo,
  type LogEntry,
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
  const setView = useAppStore((s) => s.setView);

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
    testConnection();
    commands.getAppStats().then(setAppStats).catch(() => {});
    commands.getLlmConfig().then(setConfig).catch(() => {});
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
                <div className="pt-2 border-t border-border/50">
                  <button
                    onClick={async () => {
                      try {
                        setSaveMsg("Generating embeddings...");
                        const r = await commands.generateEmbeddings();
                        setSaveMsg(`Embedded ${r.embeddings_generated} chunks (${r.errors.length} errors)`);
                      } catch (e) {
                        setSaveErr(String(e));
                      }
                    }}
                    className="rounded-md bg-secondary px-3 py-1.5 text-sm hover:bg-secondary/80"
                  >
                    Generate Embeddings
                  </button>
                  <p className="text-[10px] text-muted-foreground mt-1">
                    Required for semantic search and RAG. Uses the configured embedding provider.
                  </p>
                </div>
              )}
            </div>
          ) : (
            <p className="text-muted-foreground text-sm">Loading...</p>
          )}
        </div>
      </section>

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

      {/* ── About ── */}
      <section className="text-center text-xs text-muted-foreground pb-8">
        <p>Memory Palace v0.1.0 — MVP</p>
        <p className="mt-1">
          Built with Rust, React, and the belief that your data should help you
          understand yourself.
        </p>
      </section>
    </div>
  );
}
