import { useState, useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type ProviderPreset,
  type LlmConfig,
} from "@/lib/tauri";
import { useAppStore } from "@/stores/app-store";
import {
  ExternalLink,
  Key,
  Upload,
  CheckCircle,
  ArrowRight,
  ArrowLeft,
  Sparkles,
  Shield,
  FolderOpen,
  Cpu,
  Cloud,
  Info,
} from "lucide-react";
import { SourceIcon } from "@/components/import/SourceIcon";

type Step = 0 | 1 | 2 | 3;

const ONBOARDING_PROVIDERS = ["gemini", "groq", "openrouter", "ollama"];

const QUICK_SOURCES = [
  { id: "google_takeout", label: "Google Takeout" },
  { id: "obsidian", label: "Obsidian" },
  { id: "whatsapp", label: "WhatsApp" },
  { id: "markdown", label: "Markdown" },
  { id: "twitter", label: "Twitter / X" },
  { id: "telegram", label: "Telegram" },
  { id: "notion", label: "Notion" },
  { id: "discord", label: "Discord" },
];

const OLLAMA_MODELS = [
  { vram: "4 GB", model: "llama3.2:3b", speed: "~40 tok/s" },
  { vram: "8 GB", model: "llama3.1:8b", speed: "~35 tok/s" },
  { vram: "12 GB", model: "qwen2.5:14b-instruct-q4_K_M", speed: "~25 tok/s", recommended: true },
  { vram: "16 GB+", model: "qwen2.5:32b-instruct-q4_K_M", speed: "~15 tok/s" },
];

export function OnboardingWizard() {
  const [step, setStep] = useState<Step>(0);
  const [presets, setPresets] = useState<ProviderPreset[]>([]);
  const [selectedProvider, setSelectedProvider] = useState<ProviderPreset | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [ollamaModel, setOllamaModel] = useState("llama3.1:8b");
  const [providerSaved, setProviderSaved] = useState(false);
  const [importStarted, setImportStarted] = useState(false);
  const setOnboarded = useAppStore((s) => s.setOnboarded);
  const setView = useAppStore((s) => s.setView);
  const addTask = useAppStore((s) => s.addTask);
  const updateTask = useAppStore((s) => s.updateTask);

  useEffect(() => {
    commands.listProviderPresets().then(setPresets).catch(() => {});
  }, []);

  const filteredPresets = presets.filter((p) => ONBOARDING_PROVIDERS.includes(p.id));

  const handleSaveProvider = useCallback(async () => {
    if (!selectedProvider) return;
    try {
      const currentConfig = await commands.getLlmConfig();
      const config: LlmConfig = {
        ...currentConfig,
        active_provider: selectedProvider.id === "ollama" ? "ollama" : "openai_compat",
        ollama_model: selectedProvider.id === "ollama" ? ollamaModel : currentConfig.ollama_model,
        openai_compat_base_url:
          selectedProvider.id !== "ollama" ? selectedProvider.base_url : currentConfig.openai_compat_base_url,
        openai_compat_api_key:
          selectedProvider.id !== "ollama" && apiKey ? apiKey : currentConfig.openai_compat_api_key,
        openai_compat_model:
          selectedProvider.id !== "ollama" ? selectedProvider.default_model : currentConfig.openai_compat_model,
        openai_compat_embedding_model:
          selectedProvider.id !== "ollama" ? selectedProvider.embedding_model ?? null : currentConfig.openai_compat_embedding_model,
        openai_compat_provider_id:
          selectedProvider.id !== "ollama" ? selectedProvider.id : currentConfig.openai_compat_provider_id,
      };
      await commands.saveLlmConfig(config);
      setProviderSaved(true);
    } catch {
      // silently continue
    }
  }, [selectedProvider, apiKey, ollamaModel]);

  const handleImportSource = useCallback(async (sourceId: string) => {
    try {
      const isDir = sourceId === "obsidian" || sourceId === "markdown";
      let path: string | null = null;
      if (isDir) {
        path = (await open({ directory: true })) as string | null;
      } else {
        const result = await open({
          filters: [{ name: "Data Export", extensions: ["zip", "json", "csv", "txt", "html", "xml", "enex"] }],
          multiple: false,
        });
        path = result as string | null;
      }
      if (!path) return;
      setImportStarted(true);
      const importId = `onboard-${Date.now()}`;
      addTask({ id: importId, type: "import", label: `Importing ${sourceId}`, progress: null, result: null, error: null, running: true });
      commands.importSource(path, sourceId, importId)
        .then((r) => updateTask(importId, { running: false, result: `${r.documents_imported} docs, ${r.chunks_created} chunks` }))
        .catch((e) => updateTask(importId, { running: false, error: String(e) }));
    } catch { /* */ }
  }, [addTask, updateTask]);

  const handleAutoImport = useCallback(async () => {
    try {
      const result = await open({
        filters: [{ name: "Data Export", extensions: ["zip", "json", "csv", "txt", "html", "xml", "enex"] }],
        multiple: false,
      });
      const path = result as string | null;
      if (!path) return;
      setImportStarted(true);
      const importId = `onboard-${Date.now()}`;
      addTask({ id: importId, type: "import", label: "Importing (auto-detect)", progress: null, result: null, error: null, running: true });
      commands.importSource(path, undefined, importId)
        .then((r) => updateTask(importId, { running: false, result: `${r.documents_imported} docs, ${r.chunks_created} chunks` }))
        .catch((e) => updateTask(importId, { running: false, error: String(e) }));
    } catch { /* */ }
  }, [addTask, updateTask]);

  const handleFinish = useCallback(async () => {
    try { await commands.completeOnboarding(); } catch { /* */ }
    setOnboarded(true);
    setView("insights");
  }, [setOnboarded, setView]);

  const nextStep = () => setStep((s) => Math.min(s + 1, 3) as Step);
  const prevStep = () => setStep((s) => Math.max(s - 1, 0) as Step);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/95 backdrop-blur-sm">
      <div className="w-full max-w-lg mx-4">
        <div className="rounded-xl border border-border bg-card shadow-2xl overflow-hidden">
          <div className="p-8 min-h-[460px] flex flex-col">
            {step === 0 && <StepWelcome onNext={nextStep} />}
            {step === 1 && (
              <StepProvider
                presets={filteredPresets}
                selectedProvider={selectedProvider}
                apiKey={apiKey}
                ollamaModel={ollamaModel}
                providerSaved={providerSaved}
                onSelectProvider={(p) => { setSelectedProvider(p); setProviderSaved(false); setApiKey(""); }}
                onApiKeyChange={setApiKey}
                onOllamaModelChange={setOllamaModel}
                onSave={handleSaveProvider}
                onNext={nextStep}
                onBack={prevStep}
                onSkip={nextStep}
              />
            )}
            {step === 2 && (
              <StepImport
                importStarted={importStarted}
                onImportSource={handleImportSource}
                onAutoImport={handleAutoImport}
                onNext={nextStep}
                onBack={prevStep}
                onSkip={nextStep}
              />
            )}
            {step === 3 && (
              <StepReady
                providerSaved={providerSaved}
                providerName={selectedProvider?.name ?? null}
                importStarted={importStarted}
                onFinish={handleFinish}
                onBack={prevStep}
              />
            )}
          </div>

          {/* Step indicator */}
          <div className="flex items-center justify-center gap-2 pb-6">
            {[0, 1, 2, 3].map((i) => (
              <div
                key={i}
                className={`h-2 rounded-full transition-all duration-300 ${
                  i === step ? "w-6 bg-primary" : i < step ? "w-2 bg-primary/50" : "w-2 bg-muted-foreground/30"
                }`}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

/* ── Step 1: Welcome ── */
function StepWelcome({ onNext }: { onNext: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center flex-1 text-center space-y-6">
      <div className="w-16 h-16 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center">
        <span className="text-3xl font-bold text-primary">M</span>
      </div>

      <div className="space-y-2">
        <h1 className="text-2xl font-bold">Welcome to MemryLab</h1>
        <p className="text-muted-foreground leading-relaxed max-w-sm">
          Import your digital life, discover how your thinking evolved, and explore your personal knowledge graph.
        </p>
      </div>

      <div className="space-y-2 w-full max-w-xs text-sm">
        <div className="flex items-center gap-2 text-muted-foreground bg-muted/50 rounded-lg px-4 py-2">
          <Shield size={14} className="text-green-400 shrink-0" />
          <span>All data stays on your device</span>
        </div>
        <div className="flex items-center gap-2 text-muted-foreground bg-muted/50 rounded-lg px-4 py-2">
          <Sparkles size={14} className="text-primary shrink-0" />
          <span>8 free AI providers supported</span>
        </div>
        <div className="flex items-center gap-2 text-muted-foreground bg-muted/50 rounded-lg px-4 py-2">
          <FolderOpen size={14} className="text-yellow-400 shrink-0" />
          <span>30+ platform imports (Google, WhatsApp, etc.)</span>
        </div>
      </div>

      <button
        onClick={onNext}
        className="flex items-center gap-2 rounded-lg bg-primary px-6 py-3 text-primary-foreground font-medium hover:bg-primary/90 transition-colors"
      >
        Get Started <ArrowRight size={16} />
      </button>
    </div>
  );
}

/* ── Step 2: Choose AI Provider ── */
function StepProvider({
  presets, selectedProvider, apiKey, ollamaModel, providerSaved,
  onSelectProvider, onApiKeyChange, onOllamaModelChange, onSave, onNext, onBack, onSkip,
}: {
  presets: ProviderPreset[];
  selectedProvider: ProviderPreset | null;
  apiKey: string;
  ollamaModel: string;
  providerSaved: boolean;
  onSelectProvider: (p: ProviderPreset) => void;
  onApiKeyChange: (key: string) => void;
  onOllamaModelChange: (model: string) => void;
  onSave: () => Promise<void>;
  onNext: () => void;
  onBack: () => void;
  onSkip: () => void;
}) {
  const [saving, setSaving] = useState(false);

  const handleSaveAndContinue = async () => {
    setSaving(true);
    await onSave();
    setSaving(false);
    onNext();
  };

  return (
    <div className="flex flex-col flex-1 space-y-4">
      <div className="text-center space-y-1">
        <h2 className="text-xl font-semibold">Set up AI Provider</h2>
        <p className="text-sm text-muted-foreground">
          MemryLab needs an LLM for analysis. Pick one — all have free tiers.
        </p>
      </div>

      {/* Provider cards */}
      <div className="grid grid-cols-2 gap-2">
        {presets.map((p) => (
          <button
            key={p.id}
            onClick={() => onSelectProvider(p)}
            className={`flex flex-col gap-1 rounded-lg border p-3 text-left transition-colors ${
              selectedProvider?.id === p.id ? "border-primary bg-primary/5" : "border-border hover:border-primary/40 bg-card"
            }`}
          >
            <div className="flex items-center gap-2">
              {p.id === "ollama" ? <Cpu size={14} className="text-primary" /> : <Cloud size={14} className="text-primary" />}
              <span className="font-medium text-sm">{p.name}</span>
              {p.free_tier && (
                <span className="text-[10px] font-semibold bg-green-500/15 text-green-400 px-1.5 py-0.5 rounded">FREE</span>
              )}
            </div>
            <p className="text-xs text-muted-foreground line-clamp-2">{p.description}</p>
          </button>
        ))}
      </div>

      {/* Ollama model selection */}
      {selectedProvider?.id === "ollama" && (
        <div className="space-y-2">
          <label className="text-sm font-medium flex items-center gap-1.5">
            <Info size={12} className="text-muted-foreground" /> Select model for your GPU
          </label>
          <div className="space-y-1">
            {OLLAMA_MODELS.map((m) => (
              <button
                key={m.model}
                onClick={() => onOllamaModelChange(m.model)}
                className={`w-full flex items-center justify-between rounded-md border px-3 py-2 text-sm transition-colors ${
                  ollamaModel === m.model ? "border-primary bg-primary/5" : "border-border hover:border-primary/40"
                }`}
              >
                <div className="flex items-center gap-2">
                  <span className="font-mono text-xs">{m.model}</span>
                  {m.recommended && (
                    <span className="text-[10px] font-semibold bg-primary/15 text-primary px-1.5 py-0.5 rounded">BEST</span>
                  )}
                </div>
                <span className="text-xs text-muted-foreground">{m.vram} VRAM · {m.speed}</span>
              </button>
            ))}
          </div>
          <p className="text-xs text-muted-foreground">
            Run <code className="bg-muted px-1 rounded">ollama pull {ollamaModel}</code> and <code className="bg-muted px-1 rounded">ollama pull nomic-embed-text</code> first.
          </p>
        </div>
      )}

      {/* API key input for cloud providers */}
      {selectedProvider && selectedProvider.id !== "ollama" && (
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium">API Key</label>
            <a href={selectedProvider.signup_url} target="_blank" rel="noopener noreferrer"
              className="flex items-center gap-1 text-xs text-primary hover:underline">
              Get free API Key <ExternalLink size={10} />
            </a>
          </div>
          <div className="relative">
            <Key size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <input
              type="password"
              value={apiKey}
              onChange={(e) => onApiKeyChange(e.target.value)}
              placeholder={`Paste your ${selectedProvider.name} API key`}
              className="w-full rounded-md border border-input bg-background pl-9 pr-4 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>
        </div>
      )}

      <div className="flex-1" />

      {/* Actions */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <button onClick={onBack} className="text-sm text-muted-foreground hover:text-foreground transition-colors">
            <ArrowLeft size={14} className="inline mr-1" />Back
          </button>
          <button onClick={onSkip} className="text-sm text-muted-foreground hover:text-foreground transition-colors">
            Skip for now
          </button>
        </div>
        {selectedProvider && (
          <button
            onClick={handleSaveAndContinue}
            disabled={saving || (selectedProvider.id !== "ollama" && !apiKey.trim())}
            className="flex items-center gap-2 rounded-lg bg-primary px-5 py-2.5 text-sm text-primary-foreground font-medium hover:bg-primary/90 transition-colors disabled:opacity-50"
          >
            {saving ? "Saving..." : providerSaved ? "Saved!" : "Save & Continue"}
            {!saving && <ArrowRight size={14} />}
          </button>
        )}
      </div>
    </div>
  );
}

/* ── Step 3: Import Data ── */
function StepImport({
  importStarted, onImportSource, onAutoImport, onNext, onBack, onSkip,
}: {
  importStarted: boolean;
  onImportSource: (id: string) => void;
  onAutoImport: () => void;
  onNext: () => void;
  onBack: () => void;
  onSkip: () => void;
}) {
  return (
    <div className="flex flex-col flex-1 space-y-4">
      <div className="text-center space-y-1">
        <h2 className="text-xl font-semibold">Import your data</h2>
        <p className="text-sm text-muted-foreground">
          Select a source below. Import runs in the background — you can continue setup.
        </p>
      </div>

      {importStarted && (
        <div className="flex items-center gap-2 rounded-lg bg-green-500/10 border border-green-500/20 px-4 py-2.5 text-sm text-green-400">
          <CheckCircle size={14} />
          <span>Import started! It's running in the background. You can import more or continue.</span>
        </div>
      )}

      {/* Quick source grid */}
      <div className="grid grid-cols-4 gap-2">
        {QUICK_SOURCES.map((src) => (
          <button
            key={src.id}
            onClick={() => onImportSource(src.id)}
            className="flex flex-col items-center gap-1.5 rounded-lg border border-border bg-card p-3 text-center hover:border-primary/50 hover:bg-accent transition-colors group"
          >
            <SourceIcon icon={src.id} size={24} className="text-muted-foreground group-hover:text-primary transition-colors" />
            <span className="text-xs font-medium text-muted-foreground group-hover:text-foreground transition-colors">
              {src.label}
            </span>
          </button>
        ))}
      </div>

      {/* Auto-detect */}
      <button
        onClick={onAutoImport}
        className="flex items-center justify-center gap-3 rounded-lg border-2 border-dashed border-muted-foreground/25 py-4 text-sm text-muted-foreground hover:border-primary/40 hover:text-foreground transition-colors"
      >
        <Upload size={16} />
        Or drop any file/folder (auto-detect)
      </button>

      <div className="flex-1" />

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <button onClick={onBack} className="text-sm text-muted-foreground hover:text-foreground transition-colors">
            <ArrowLeft size={14} className="inline mr-1" />Back
          </button>
          <button onClick={onSkip} className="text-sm text-muted-foreground hover:text-foreground transition-colors">
            Skip for now
          </button>
        </div>
        {importStarted && (
          <button onClick={onNext}
            className="flex items-center gap-2 rounded-lg bg-primary px-5 py-2.5 text-sm text-primary-foreground font-medium hover:bg-primary/90 transition-colors">
            Continue <ArrowRight size={14} />
          </button>
        )}
      </div>
    </div>
  );
}

/* ── Step 4: Ready ── */
function StepReady({
  providerSaved, providerName, importStarted, onFinish, onBack,
}: {
  providerSaved: boolean;
  providerName: string | null;
  importStarted: boolean;
  onFinish: () => void;
  onBack: () => void;
}) {
  return (
    <div className="flex flex-col items-center justify-center flex-1 text-center space-y-6">
      <div className="w-16 h-16 rounded-full bg-green-500/10 border border-green-500/20 flex items-center justify-center">
        <Sparkles size={28} className="text-green-400" />
      </div>

      <div className="space-y-2">
        <h1 className="text-2xl font-bold">You're all set!</h1>
        <p className="text-muted-foreground">MemryLab is ready.</p>
      </div>

      {/* Summary */}
      <div className="space-y-2 w-full max-w-xs">
        <div className="flex items-center gap-3 rounded-lg bg-muted/50 px-4 py-2.5 text-sm">
          <CheckCircle size={16} className={providerSaved ? "text-green-400" : "text-muted-foreground/40"} />
          <span className={providerSaved ? "" : "text-muted-foreground"}>
            {providerSaved && providerName ? `AI: ${providerName}` : "AI Provider: not configured"}
          </span>
        </div>
        <div className="flex items-center gap-3 rounded-lg bg-muted/50 px-4 py-2.5 text-sm">
          <FolderOpen size={16} className={importStarted ? "text-green-400" : "text-muted-foreground/40"} />
          <span className={importStarted ? "" : "text-muted-foreground"}>
            {importStarted ? "Data import running in background" : "No data imported yet"}
          </span>
        </div>
      </div>

      {/* Next steps */}
      <div className="rounded-lg bg-primary/5 border border-primary/20 px-4 py-3 text-sm text-left w-full max-w-xs space-y-1.5">
        <p className="font-medium text-primary">Next steps:</p>
        <ol className="list-decimal list-inside text-muted-foreground space-y-1">
          <li>Wait for import to finish (see progress bar below)</li>
          <li>Click <strong className="text-foreground">"Run Analysis"</strong> on the completion banner</li>
          <li>Explore Timeline, Search, Ask, and Graph views</li>
        </ol>
      </div>

      <div className="flex items-center gap-4">
        <button onClick={onBack} className="text-sm text-muted-foreground hover:text-foreground transition-colors">
          <ArrowLeft size={14} className="inline mr-1" />Back
        </button>
        <button onClick={onFinish}
          className="flex items-center gap-2 rounded-lg bg-primary px-6 py-3 text-primary-foreground font-medium hover:bg-primary/90 transition-colors">
          Start Exploring <ArrowRight size={16} />
        </button>
      </div>
    </div>
  );
}
