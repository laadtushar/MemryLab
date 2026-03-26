import { useState, useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type ProviderPreset,
  type LlmConfig,
  type ImportSummary,
} from "@/lib/tauri";
import { useAppStore } from "@/stores/app-store";
import {
  ExternalLink,
  Key,
  Upload,
  CheckCircle,
  ArrowRight,
  Sparkles,
  Shield,
  FolderOpen,
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

export function OnboardingWizard() {
  const [step, setStep] = useState<Step>(0);
  const [presets, setPresets] = useState<ProviderPreset[]>([]);
  const [selectedProvider, setSelectedProvider] =
    useState<ProviderPreset | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [providerSaved, setProviderSaved] = useState(false);
  const [importResult, setImportResult] = useState<ImportSummary | null>(null);
  const [importing, setImporting] = useState(false);
  const setOnboarded = useAppStore((s) => s.setOnboarded);
  const setView = useAppStore((s) => s.setView);

  useEffect(() => {
    commands
      .listProviderPresets()
      .then(setPresets)
      .catch(() => {});
  }, []);

  const filteredPresets = presets.filter((p) =>
    ONBOARDING_PROVIDERS.includes(p.id),
  );

  const handleSaveProvider = useCallback(async () => {
    if (!selectedProvider) return;
    try {
      const currentConfig = await commands.getLlmConfig();
      const config: LlmConfig = {
        ...currentConfig,
        active_provider:
          selectedProvider.id === "ollama"
            ? "ollama"
            : "openai_compat",
        openai_compat_base_url:
          selectedProvider.id !== "ollama"
            ? selectedProvider.base_url
            : currentConfig.openai_compat_base_url,
        openai_compat_api_key:
          selectedProvider.id !== "ollama" && apiKey
            ? apiKey
            : currentConfig.openai_compat_api_key,
        openai_compat_model:
          selectedProvider.id !== "ollama"
            ? selectedProvider.default_model
            : currentConfig.openai_compat_model,
        openai_compat_embedding_model:
          selectedProvider.id !== "ollama"
            ? selectedProvider.embedding_model ?? null
            : currentConfig.openai_compat_embedding_model,
        openai_compat_provider_id:
          selectedProvider.id !== "ollama"
            ? selectedProvider.id
            : currentConfig.openai_compat_provider_id,
      };
      await commands.saveLlmConfig(config);
      setProviderSaved(true);
    } catch {
      // silently continue — user can configure later
    }
  }, [selectedProvider, apiKey]);

  const handleImportSource = useCallback(
    async (sourceId: string) => {
      setImporting(true);
      try {
        const isDir = sourceId === "obsidian" || sourceId === "markdown";
        let path: string | null = null;

        if (isDir) {
          path = (await open({ directory: true })) as string | null;
        } else {
          const result = await open({
            filters: [
              {
                name: "Data Export",
                extensions: [
                  "zip",
                  "json",
                  "csv",
                  "txt",
                  "html",
                  "xml",
                  "enex",
                ],
              },
            ],
            multiple: false,
          });
          path = result as string | null;
        }

        if (!path) {
          setImporting(false);
          return;
        }

        const result = await commands.importSource(path, sourceId);
        setImportResult(result);
        setImporting(false);
      } catch {
        setImporting(false);
      }
    },
    [],
  );

  const handleAutoImport = useCallback(async () => {
    setImporting(true);
    try {
      const result = await open({
        filters: [
          {
            name: "Data Export",
            extensions: [
              "zip",
              "json",
              "csv",
              "txt",
              "html",
              "xml",
              "enex",
            ],
          },
        ],
        multiple: false,
      });
      const path = result as string | null;
      if (!path) {
        setImporting(false);
        return;
      }
      const importResult = await commands.importSource(path);
      setImportResult(importResult);
      setImporting(false);
    } catch {
      setImporting(false);
    }
  }, []);

  const handleFinish = useCallback(async () => {
    try {
      await commands.completeOnboarding();
    } catch {
      // best effort
    }
    setOnboarded(true);
    setView("timeline");
  }, [setOnboarded, setView]);

  const nextStep = () => setStep((s) => Math.min(s + 1, 3) as Step);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/95 backdrop-blur-sm">
      <div className="w-full max-w-lg mx-4">
        <div className="rounded-xl border border-border bg-card shadow-2xl overflow-hidden">
          {/* Content area with transition */}
          <div className="p-8 min-h-[420px] flex flex-col">
            {step === 0 && <StepWelcome onNext={nextStep} />}
            {step === 1 && (
              <StepProvider
                presets={filteredPresets}
                selectedProvider={selectedProvider}
                apiKey={apiKey}
                providerSaved={providerSaved}
                onSelectProvider={(p) => {
                  setSelectedProvider(p);
                  setProviderSaved(false);
                  setApiKey("");
                }}
                onApiKeyChange={setApiKey}
                onSave={handleSaveProvider}
                onNext={nextStep}
                onSkip={nextStep}
              />
            )}
            {step === 2 && (
              <StepImport
                importing={importing}
                importResult={importResult}
                onImportSource={handleImportSource}
                onAutoImport={handleAutoImport}
                onNext={nextStep}
                onSkip={nextStep}
              />
            )}
            {step === 3 && (
              <StepReady
                providerSaved={providerSaved}
                providerName={selectedProvider?.name ?? null}
                importResult={importResult}
                onFinish={handleFinish}
              />
            )}
          </div>

          {/* Step indicator */}
          <div className="flex items-center justify-center gap-2 pb-6">
            {[0, 1, 2, 3].map((i) => (
              <button
                key={i}
                onClick={() => setStep(i as Step)}
                className={`h-2 rounded-full transition-all duration-300 ${
                  i === step
                    ? "w-6 bg-primary"
                    : i < step
                      ? "w-2 bg-primary/50"
                      : "w-2 bg-muted-foreground/30"
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
      {/* Logo badge */}
      <div className="w-16 h-16 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center">
        <span className="text-3xl font-bold text-primary">M</span>
      </div>

      <div className="space-y-2">
        <h1 className="text-2xl font-bold">Welcome to Memory Palace</h1>
        <p className="text-muted-foreground leading-relaxed max-w-sm">
          A searchable, visual timeline of how your thinking evolved.
        </p>
      </div>

      <div className="flex items-center gap-2 text-sm text-muted-foreground bg-muted/50 rounded-lg px-4 py-2.5">
        <Shield size={16} className="text-green-400 shrink-0" />
        <span>Your data stays on your device. Always.</span>
      </div>

      <button
        onClick={onNext}
        className="flex items-center gap-2 rounded-lg bg-primary px-6 py-3 text-primary-foreground font-medium hover:bg-primary/90 transition-colors"
      >
        Get Started
        <ArrowRight size={16} />
      </button>
    </div>
  );
}

/* ── Step 2: Choose AI Provider ── */

function StepProvider({
  presets,
  selectedProvider,
  apiKey,
  providerSaved,
  onSelectProvider,
  onApiKeyChange,
  onSave,
  onNext,
  onSkip,
}: {
  presets: ProviderPreset[];
  selectedProvider: ProviderPreset | null;
  apiKey: string;
  providerSaved: boolean;
  onSelectProvider: (p: ProviderPreset) => void;
  onApiKeyChange: (key: string) => void;
  onSave: () => Promise<void>;
  onNext: () => void;
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
    <div className="flex flex-col flex-1 space-y-5">
      <div className="text-center space-y-1">
        <h2 className="text-xl font-semibold">Set up your AI provider</h2>
        <p className="text-sm text-muted-foreground">
          Choose a provider for analysis and search. All offer free tiers.
        </p>
      </div>

      {/* Provider cards */}
      <div className="grid grid-cols-2 gap-2">
        {presets.map((p) => (
          <button
            key={p.id}
            onClick={() => onSelectProvider(p)}
            className={`flex flex-col gap-1 rounded-lg border p-3 text-left transition-colors ${
              selectedProvider?.id === p.id
                ? "border-primary bg-primary/5"
                : "border-border hover:border-primary/40 bg-card"
            }`}
          >
            <div className="flex items-center gap-2">
              <span className="font-medium text-sm">{p.name}</span>
              {p.free_tier && (
                <span className="text-[10px] font-semibold bg-green-500/15 text-green-400 px-1.5 py-0.5 rounded">
                  FREE
                </span>
              )}
            </div>
            <p className="text-xs text-muted-foreground line-clamp-2">
              {p.description}
            </p>
          </button>
        ))}
      </div>

      {/* API key input */}
      {selectedProvider && selectedProvider.id !== "ollama" && (
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium">API Key</label>
            <a
              href={selectedProvider.signup_url}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1 text-xs text-primary hover:underline"
            >
              Get API Key <ExternalLink size={10} />
            </a>
          </div>
          <div className="relative">
            <Key
              size={14}
              className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
            />
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
        <button
          onClick={onSkip}
          className="text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          Skip for now
        </button>
        {selectedProvider && (
          <button
            onClick={handleSaveAndContinue}
            disabled={
              saving ||
              (selectedProvider.id !== "ollama" && !apiKey.trim())
            }
            className="flex items-center gap-2 rounded-lg bg-primary px-5 py-2.5 text-sm text-primary-foreground font-medium hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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
  importing,
  importResult,
  onImportSource,
  onAutoImport,
  onNext,
  onSkip,
}: {
  importing: boolean;
  importResult: ImportSummary | null;
  onImportSource: (id: string) => void;
  onAutoImport: () => void;
  onNext: () => void;
  onSkip: () => void;
}) {
  return (
    <div className="flex flex-col flex-1 space-y-5">
      <div className="text-center space-y-1">
        <h2 className="text-xl font-semibold">Import your data</h2>
        <p className="text-sm text-muted-foreground">
          Memory Palace supports 30+ platforms. Select one to begin.
        </p>
      </div>

      {importResult ? (
        <div className="flex flex-col items-center justify-center flex-1 space-y-4">
          <CheckCircle size={40} className="text-green-400" />
          <div className="text-center">
            <p className="font-medium">
              {importResult.documents_imported} documents imported
            </p>
            <p className="text-sm text-muted-foreground">
              {importResult.chunks_created} chunks,{" "}
              {importResult.embeddings_generated} embeddings
            </p>
          </div>
          <button
            onClick={onNext}
            className="flex items-center gap-2 rounded-lg bg-primary px-5 py-2.5 text-sm text-primary-foreground font-medium hover:bg-primary/90 transition-colors"
          >
            Continue
            <ArrowRight size={14} />
          </button>
        </div>
      ) : importing ? (
        <div className="flex flex-col items-center justify-center flex-1 space-y-3">
          <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin" />
          <p className="text-sm text-muted-foreground">Importing...</p>
        </div>
      ) : (
        <>
          {/* Quick source grid */}
          <div className="grid grid-cols-4 gap-2">
            {QUICK_SOURCES.map((src) => (
              <button
                key={src.id}
                onClick={() => onImportSource(src.id)}
                className="flex flex-col items-center gap-1.5 rounded-lg border border-border bg-card p-3 text-center hover:border-primary/50 hover:bg-accent transition-colors group"
              >
                <SourceIcon
                  icon={src.id}
                  size={24}
                  className="text-muted-foreground group-hover:text-primary transition-colors"
                />
                <span className="text-xs font-medium text-muted-foreground group-hover:text-foreground transition-colors">
                  {src.label}
                </span>
              </button>
            ))}
          </div>

          {/* Drop zone */}
          <button
            onClick={onAutoImport}
            className="flex items-center justify-center gap-3 rounded-lg border-2 border-dashed border-muted-foreground/25 py-4 text-sm text-muted-foreground hover:border-primary/40 hover:text-foreground transition-colors"
          >
            <Upload size={16} />
            Or drop any file/folder
          </button>

          <div className="flex-1" />

          <div className="flex justify-start">
            <button
              onClick={onSkip}
              className="text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              Skip for now
            </button>
          </div>
        </>
      )}
    </div>
  );
}

/* ── Step 4: Ready ── */

function StepReady({
  providerSaved,
  providerName,
  importResult,
  onFinish,
}: {
  providerSaved: boolean;
  providerName: string | null;
  importResult: ImportSummary | null;
  onFinish: () => void;
}) {
  return (
    <div className="flex flex-col items-center justify-center flex-1 text-center space-y-6">
      <div className="w-16 h-16 rounded-full bg-green-500/10 border border-green-500/20 flex items-center justify-center">
        <Sparkles size={28} className="text-green-400" />
      </div>

      <div className="space-y-2">
        <h1 className="text-2xl font-bold">You're all set!</h1>
        <p className="text-muted-foreground">
          Memory Palace is ready to explore your data.
        </p>
      </div>

      {/* Summary */}
      <div className="space-y-2 w-full max-w-xs">
        <div className="flex items-center gap-3 rounded-lg bg-muted/50 px-4 py-2.5 text-sm">
          <CheckCircle
            size={16}
            className={
              providerSaved ? "text-green-400" : "text-muted-foreground/40"
            }
          />
          <span className={providerSaved ? "" : "text-muted-foreground"}>
            {providerSaved && providerName
              ? `AI Provider: ${providerName}`
              : "AI Provider: not configured"}
          </span>
        </div>
        <div className="flex items-center gap-3 rounded-lg bg-muted/50 px-4 py-2.5 text-sm">
          <FolderOpen
            size={16}
            className={
              importResult ? "text-green-400" : "text-muted-foreground/40"
            }
          />
          <span className={importResult ? "" : "text-muted-foreground"}>
            {importResult
              ? `${importResult.documents_imported} documents imported`
              : "No data imported yet"}
          </span>
        </div>
      </div>

      <button
        onClick={onFinish}
        className="flex items-center gap-2 rounded-lg bg-primary px-6 py-3 text-primary-foreground font-medium hover:bg-primary/90 transition-colors"
      >
        Start Exploring
        <ArrowRight size={16} />
      </button>
    </div>
  );
}
