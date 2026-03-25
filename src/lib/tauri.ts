import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Response types ──

export interface ImportSummary {
  documents_imported: number;
  chunks_created: number;
  embeddings_generated: number;
  duplicates_skipped: number;
  errors: string[];
  duration_ms: number;
}

export interface ImportProgress {
  stage: string;
  current: number;
  total: number;
  message: string;
}

export interface SearchResult {
  chunk_id: string;
  document_id: string;
  text: string;
  score: number;
  timestamp: string;
  source_platform: string;
}

export interface TimelineDataResponse {
  months: { month: string; document_count: number }[];
  total_documents: number;
  date_range: { start: string; end: string } | null;
}

export interface MemoryFactResponse {
  id: string;
  fact_text: string;
  confidence: number;
  category: string;
  first_seen: string;
  last_updated: string;
  is_active: boolean;
}

export interface OllamaStatus {
  connected: boolean;
  models: string[];
}

export interface AppStats {
  total_documents: number;
  total_memory_facts: number;
  date_range: [string, string] | null;
}

export interface RagResponse {
  answer: string;
  sources: {
    chunk_id: string;
    document_id: string;
    text_snippet: string;
    timestamp: string;
    score: number;
  }[];
}

export interface AnalysisResult {
  themes_extracted: number;
  beliefs_extracted: number;
  sentiments_classified: number;
  entities_extracted: number;
  insights_generated: number;
}

export interface EntityResponse {
  id: string;
  name: string;
  entity_type: string;
  mention_count: number;
  first_seen: string | null;
  last_seen: string | null;
}

export interface EntityGraphResponse {
  entities: EntityResponse[];
  relationships: {
    id: string;
    source_entity_id: string;
    target_entity_id: string;
    rel_type: string;
    weight: number;
  }[];
}

export interface EvolutionData {
  months: { month: string; document_count: number; fact_count: number }[];
  total_facts: number;
  date_range: [string, string] | null;
}

export interface EmbeddingResult {
  chunks_processed: number;
  embeddings_generated: number;
  already_embedded: number;
  errors: string[];
}

export interface SourceAdapterMeta {
  id: string;
  display_name: string;
  icon: string;
  takeout_url: string | null;
  instructions: string;
  accepted_extensions: string[];
  handles_zip: boolean;
  platform: string;
}

export interface LlmConfig {
  active_provider: string;
  ollama_url: string;
  ollama_model: string;
  embedding_model: string;
  claude_api_key: string | null;
  claude_model: string;
  openai_compat_base_url: string | null;
  openai_compat_api_key: string | null;
  openai_compat_model: string | null;
  openai_compat_embedding_model: string | null;
  openai_compat_provider_id: string | null;
}

export interface ProviderPreset {
  id: string;
  name: string;
  description: string;
  base_url: string;
  signup_url: string;
  free_tier: boolean;
  default_model: string;
  embedding_model: string | null;
  embedding_dimensions: number | null;
  models: { id: string; name: string; free: boolean }[];
  rate_limits: string;
  supports_embeddings: boolean;
}

// ── Commands ──

export const commands = {
  // Import
  importObsidian: (vaultPath: string) =>
    invoke<ImportSummary>("import_obsidian", { vaultPath }),
  importMarkdown: (dirPath: string) =>
    invoke<ImportSummary>("import_markdown", { dirPath }),
  importDayone: (filePath: string) =>
    invoke<ImportSummary>("import_dayone", { filePath }),
  importSource: (path: string, adapterId?: string) =>
    invoke<ImportSummary>("import_source", { path, adapterId }),
  listSources: () =>
    invoke<SourceAdapterMeta[]>("list_sources"),

  // Search
  keywordSearch: (query: string, topK?: number) =>
    invoke<SearchResult[]>("keyword_search", { query, topK }),
  semanticSearch: (query: string, topK?: number) =>
    invoke<SearchResult[]>("semantic_search", { query, topK }),
  hybridSearch: (query: string, topK?: number) =>
    invoke<SearchResult[]>("hybrid_search", { query, topK }),
  getDocumentText: (documentId: string) =>
    invoke<string>("get_document_text", { documentId }),

  // RAG
  ask: (query: string) =>
    invoke<RagResponse>("ask", { query }),

  // Timeline + Memory
  getTimelineData: () => invoke<TimelineDataResponse>("get_timeline_data"),
  getMemoryFacts: (category?: string) =>
    invoke<MemoryFactResponse[]>("get_memory_facts", { category }),
  deleteMemoryFact: (id: string) =>
    invoke<void>("delete_memory_fact", { id }),

  // Analysis
  runAnalysis: () => invoke<AnalysisResult>("run_analysis"),

  // Entities
  listEntities: (entityType?: string) =>
    invoke<EntityResponse[]>("list_entities", { entityType }),
  getEntityGraph: (entityId: string, depth?: number) =>
    invoke<EntityGraphResponse>("get_entity_graph", { entityId, depth }),

  // Provider config
  getLlmConfig: () => invoke<LlmConfig>("get_llm_config"),
  saveLlmConfig: (config: LlmConfig) =>
    invoke<void>("save_llm_config", { config }),
  listProviderPresets: () =>
    invoke<ProviderPreset[]>("list_provider_presets"),

  // Evolution
  getEvolutionData: () => invoke<EvolutionData>("get_evolution_data"),

  // Embeddings
  generateEmbeddings: () => invoke<EmbeddingResult>("generate_embeddings"),

  // Export
  exportMemoryJson: () => invoke<string>("export_memory_json"),
  exportMemoryMarkdown: () => invoke<string>("export_memory_markdown"),

  // Settings
  testOllamaConnection: () => invoke<OllamaStatus>("test_ollama_connection"),
  getAppStats: () => invoke<AppStats>("get_app_stats"),
};

// ── Events ──

export const events = {
  onImportProgress: (
    cb: (progress: ImportProgress) => void,
  ): Promise<UnlistenFn> =>
    listen<ImportProgress>("import-progress", (e) => cb(e.payload)),
};
