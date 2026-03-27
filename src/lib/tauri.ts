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
  import_id: string;
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

export interface QuickSearchResult {
  result_type: string;
  id: string;
  title: string;
  snippet: string;
  score: number;
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

export interface RagSource {
  chunk_id: string;
  document_id: string;
  text_snippet: string;
  timestamp: string;
  score: number;
}

export interface RagResponse {
  answer: string;
  sources: RagSource[];
  conversation_id: string | null;
}

export interface AnalysisResult {
  themes_extracted: number;
  beliefs_extracted: number;
  sentiments_classified: number;
  entities_extracted: number;
  insights_generated: number;
  contradictions_found: number;
  narratives_generated: number;
}

export interface EvolutionDiffResponse {
  summary: string;
  sentiment_a: string;
  sentiment_b: string;
  key_shift: string;
  quote_a: string;
  quote_b: string;
  period_a_label: string;
  period_b_label: string;
  period_a_doc_count: number;
  period_b_doc_count: number;
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

export interface TimelineBucket {
  period: string;
  count: number;
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
  active_embedding_provider: string;
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

export interface UsageLogEntry {
  id: string;
  timestamp: string;
  provider: string;
  model: string;
  prompt_tokens: number;
  completion_tokens: number;
  purpose: string;
  duration_ms: number;
}

export interface TimeBoundary {
  id: string;
  name: string;
  date: string;
  end_date: string | null;
  color: string | null;
}

export interface PiiScanResult {
  total_scanned: number;
  total_flagged: number;
  flagged_facts: { fact_id: string; pii_types: string[] }[];
}

export interface PiiFlaggedFact {
  fact_id: string;
  pii_types: string[];
}

export interface PromptVersionInfo {
  id: string;
  name: string;
  version: string;
  template: string;
  is_active: boolean;
  created_at: string;
}

export interface LogEntry {
  timestamp: string;
  level: string;
  target: string;
  message: string;
}

export interface ActivityEntry {
  id: string;
  timestamp: string;
  action_type: string;
  title: string;
  description: string;
  result_summary: string;
  metadata: Record<string, unknown>;
  duration_ms: number;
  status: string;
}

export interface ChatConversation {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  message_count: number;
  last_message_preview: string;
}

export interface ChatMessage {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  sources: RagSource[];
  created_at: string;
}

// ── Commands ──

export const commands = {
  // Auth
  isFirstRun: () => invoke<boolean>("is_first_run"),
  isDatabaseLocked: () => invoke<boolean>("is_database_locked"),
  unlockDatabase: (passphrase: string) =>
    invoke<void>("unlock_database", { passphrase }),
  setPassphrase: (passphrase: string) =>
    invoke<void>("set_passphrase", { passphrase }),

  // Import
  importObsidian: (vaultPath: string) =>
    invoke<ImportSummary>("import_obsidian", { vaultPath }),
  importMarkdown: (dirPath: string) =>
    invoke<ImportSummary>("import_markdown", { dirPath }),
  importDayone: (filePath: string) =>
    invoke<ImportSummary>("import_dayone", { filePath }),
  importSource: (path: string, adapterId?: string, importId?: string) =>
    invoke<ImportSummary>("import_source", { path, adapterId, importId }),
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
  searchSuggestions: (prefix: string) =>
    invoke<string[]>("search_suggestions", { prefix }),
  relatedDocuments: (documentId: string, topK?: number) =>
    invoke<SearchResult[]>("related_documents", { documentId, topK }),
  searchMemoryFacts: (query: string, category?: string, topK?: number) =>
    invoke<MemoryFactResponse[]>("search_memory_facts", { query, category, topK }),
  searchEntities: (query: string, entityType?: string, topK?: number) =>
    invoke<EntityResponse[]>("search_entities", { query, entityType, topK }),
  quickSearch: (query: string) =>
    invoke<QuickSearchResult[]>("quick_search", { query }),

  // RAG
  ask: (query: string, conversationId?: string, mode?: string) =>
    invoke<RagResponse>("ask", { query, conversationId, mode }),

  // Timeline + Memory
  getTimelineData: () => invoke<TimelineDataResponse>("get_timeline_data"),
  getDetailedTimeline: (granularity: string) =>
    invoke<TimelineBucket[]>("get_detailed_timeline", { granularity }),
  getMemoryFacts: (category?: string) =>
    invoke<MemoryFactResponse[]>("get_memory_facts", { category }),
  deleteMemoryFact: (id: string) =>
    invoke<void>("delete_memory_fact", { id }),

  // Analysis
  runAnalysis: (granularity?: string) =>
    invoke<AnalysisResult>("run_analysis", { granularity }),

  // Entities
  listEntities: (entityType?: string) =>
    invoke<EntityResponse[]>("list_entities", { entityType }),
  getEntityGraph: (entityId: string, depth?: number) =>
    invoke<EntityGraphResponse>("get_entity_graph", { entityId, depth }),
  getFullGraph: (limit?: number, entityType?: string) =>
    invoke<EntityGraphResponse>("get_full_graph", { limit, entityType }),

  // Provider config
  getLlmConfig: () => invoke<LlmConfig>("get_llm_config"),
  saveLlmConfig: (config: LlmConfig) =>
    invoke<void>("save_llm_config", { config }),
  listProviderPresets: () =>
    invoke<ProviderPreset[]>("list_provider_presets"),

  // Evolution
  getEvolutionData: () => invoke<EvolutionData>("get_evolution_data"),
  getEvolutionDiff: (periodAStart: string, periodAEnd: string, periodBStart: string, periodBEnd: string) =>
    invoke<EvolutionDiffResponse>("get_evolution_diff", { periodAStart, periodAEnd, periodBStart, periodBEnd }),

  // Embeddings
  generateEmbeddings: () => invoke<EmbeddingResult>("generate_embeddings"),

  // Export
  exportMemoryJson: () => invoke<string>("export_memory_json"),
  exportMemoryMarkdown: () => invoke<string>("export_memory_markdown"),

  // Settings
  testOllamaConnection: () => invoke<OllamaStatus>("test_ollama_connection"),
  getAppStats: () => invoke<AppStats>("get_app_stats"),
  getUsageLog: (limit?: number) =>
    invoke<UsageLogEntry[]>("get_usage_log", { limit }),
  isOnboardingComplete: () => invoke<boolean>("is_onboarding_complete"),
  completeOnboarding: () => invoke<void>("complete_onboarding"),

  // Logs
  getAppLogs: (limit?: number, levelFilter?: string) =>
    invoke<LogEntry[]>("get_app_logs", { limit, levelFilter }),
  getLogPath: () => invoke<string>("get_log_path"),

  // Boundaries
  listBoundaries: () => invoke<TimeBoundary[]>("list_boundaries"),
  addBoundary: (name: string, date: string, endDate?: string, color?: string) =>
    invoke<TimeBoundary>("add_boundary", { name, date, endDate, color }),
  deleteBoundary: (id: string) => invoke<void>("delete_boundary", { id }),

  // PII
  scanPii: () => invoke<PiiScanResult>("scan_pii"),
  getPiiFlags: () => invoke<PiiFlaggedFact[]>("get_pii_flags"),

  // Prompts
  listPrompts: () => invoke<PromptVersionInfo[]>("list_prompts"),
  updatePrompt: (name: string, version: string, template: string) =>
    invoke<void>("update_prompt", { name, version, template }),
  setActivePrompt: (name: string, version: string) =>
    invoke<void>("set_active_prompt", { name, version }),

  // Activity log
  getActivityLog: (limit?: number, actionType?: string) =>
    invoke<ActivityEntry[]>("get_activity_log", { limit, actionType }),

  // Chat history
  listConversations: (limit?: number) =>
    invoke<ChatConversation[]>("list_conversations", { limit }),
  getConversationMessages: (conversationId: string) =>
    invoke<ChatMessage[]>("get_conversation_messages", { conversationId }),
  createConversation: () =>
    invoke<ChatConversation>("create_conversation"),
  deleteConversation: (id: string) =>
    invoke<void>("delete_conversation", { id }),

  // Folder watching
  addWatchFolder: (path: string, adapterId?: string) =>
    invoke<void>("add_watch_folder", { path, adapterId }),
  removeWatchFolder: (path: string) =>
    invoke<void>("remove_watch_folder", { path }),
  listWatchFolders: () =>
    invoke<WatchedFolder[]>("list_watch_folders"),
};

export interface WatchedFolder {
  path: string;
  adapter_id: string | null;
  enabled: boolean;
}

// ── Events ──

export interface EmbeddingProgress {
  stage: string;
  current: number;
  total: number;
  message: string;
}

export interface AnalysisProgress {
  stage: string;
  message: string;
}

export const events = {
  onImportProgress: (
    cb: (progress: ImportProgress) => void,
  ): Promise<UnlistenFn> =>
    listen<ImportProgress>("import-progress", (e) => cb(e.payload)),
  onEmbeddingProgress: (
    cb: (progress: EmbeddingProgress) => void,
  ): Promise<UnlistenFn> =>
    listen<EmbeddingProgress>("embedding-progress", (e) => cb(e.payload)),
  onAnalysisProgress: (
    cb: (progress: AnalysisProgress) => void,
  ): Promise<UnlistenFn> =>
    listen<AnalysisProgress>("analysis-progress", (e) => cb(e.payload)),
};
