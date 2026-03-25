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
  insights_generated: number;
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
