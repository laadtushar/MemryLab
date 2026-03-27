use std::collections::HashMap;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::domain::ports::document_store::IDocumentStore;
use crate::domain::ports::embedding_provider::IEmbeddingProvider;
use crate::domain::ports::page_index::IPageIndex;
use crate::domain::ports::timeline_store::ITimelineStore;
use crate::domain::ports::vector_store::IVectorStore;
use crate::error::AppError;

use super::chunker::{chunk_text, ChunkerConfig};
use super::dedup::deduplicate;
use super::normalizer::normalize_documents;
use super::source_adapters::SourceAdapter;

/// Summary of an import operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportSummary {
    pub documents_imported: usize,
    pub chunks_created: usize,
    pub embeddings_generated: usize,
    pub duplicates_skipped: usize,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

/// Progress callback type for reporting import progress.
pub type ProgressCallback = Box<dyn Fn(&str, usize, usize, &str) + Send>;

/// Orchestrates the full ingestion pipeline:
/// parse → dedup → normalize → chunk → store → FTS index → embed → vector store
pub struct IngestionOrchestrator<'a> {
    document_store: &'a dyn IDocumentStore,
    timeline_store: &'a dyn ITimelineStore,
    _page_index: &'a dyn IPageIndex,
    vector_store: Option<&'a dyn IVectorStore>,
    embedding_provider: Option<Arc<dyn IEmbeddingProvider>>,
    chunker_config: ChunkerConfig,
    embedding_batch_size: usize,
    cancel_token: Option<CancellationToken>,
}

impl<'a> IngestionOrchestrator<'a> {
    pub fn new(
        document_store: &'a dyn IDocumentStore,
        timeline_store: &'a dyn ITimelineStore,
        page_index: &'a dyn IPageIndex,
    ) -> Self {
        Self {
            document_store,
            timeline_store,
            _page_index: page_index,
            vector_store: None,
            embedding_provider: None,
            chunker_config: ChunkerConfig::default(),
            embedding_batch_size: 10,
            cancel_token: None,
        }
    }

    pub fn with_vector_store(mut self, store: &'a dyn IVectorStore) -> Self {
        self.vector_store = Some(store);
        self
    }

    pub fn with_embedding_provider(mut self, provider: Arc<dyn IEmbeddingProvider>) -> Self {
        self.embedding_provider = Some(provider);
        self
    }

    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancel_token = Some(token);
        self
    }

    #[allow(dead_code)]
    pub fn with_chunker_config(mut self, config: ChunkerConfig) -> Self {
        self.chunker_config = config;
        self
    }

    fn check_cancelled(&self) -> Result<(), AppError> {
        if let Some(ref token) = self.cancel_token {
            if token.is_cancelled() {
                return Err(AppError::Other("Task cancelled".to_string()));
            }
        }
        Ok(())
    }

    /// Run the full ingestion pipeline for a given source adapter and path.
    pub async fn ingest(
        &self,
        adapter: &dyn SourceAdapter,
        path: &std::path::Path,
        on_progress: Option<&ProgressCallback>,
    ) -> Result<ImportSummary, AppError> {
        // Stage 1: Parse
        report_progress(on_progress, "parsing", 0, 0, &format!("Parsing {} files...", adapter.name()));
        let documents = adapter.parse(path)?;
        let total_parsed = documents.len();
        report_progress(on_progress, "parsing", total_parsed, total_parsed, &format!("Parsed {} documents", total_parsed));

        self.ingest_documents(documents, on_progress).await
    }

    /// Run the ingestion pipeline on pre-parsed documents (dedup → normalize → chunk → store → embed).
    pub async fn ingest_documents(
        &self,
        documents: Vec<crate::domain::models::document::Document>,
        on_progress: Option<&ProgressCallback>,
    ) -> Result<ImportSummary, AppError> {
        let start = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut embeddings_generated = 0;
        let total_parsed = documents.len();
        let mut documents = documents;
        tracing::info!(total_documents = total_parsed, "Ingestion pipeline starting");

        self.check_cancelled()?;

        // Stage 2: Dedup
        report_progress(on_progress, "dedup", 0, total_parsed, "Checking for duplicates...");
        let dedup_result = deduplicate(documents, self.document_store);
        let duplicates_skipped = dedup_result.duplicates_skipped;
        documents = dedup_result.new_documents;
        report_progress(on_progress, "dedup", documents.len(), total_parsed,
            &format!("{} new, {} duplicates skipped", documents.len(), duplicates_skipped));

        self.check_cancelled()?;

        // Stage 3: Normalize
        report_progress(on_progress, "normalize", 0, documents.len(), "Normalizing text...");
        normalize_documents(&mut documents);

        self.check_cancelled()?;

        // Stage 4: Store documents + chunk + index
        let total_docs = documents.len();
        let mut chunks_created = 0;
        let mut all_chunk_texts: Vec<(String, String)> = Vec::new(); // (chunk_id, text) for embedding

        for (i, doc) in documents.iter().enumerate() {
            if i % 50 == 0 { self.check_cancelled()?; }
            report_progress(on_progress, "storing", i, total_docs,
                &format!("Processing document {}/{}", i + 1, total_docs));

            if let Err(e) = self.document_store.save_document(doc) {
                errors.push(format!("Failed to save document {}: {}", doc.id, e));
                continue;
            }

            if let Err(e) = self.timeline_store.index_document(doc) {
                errors.push(format!("Failed to index document {}: {}", doc.id, e));
            }

            let chunks = chunk_text(&doc.id, &doc.raw_text, &self.chunker_config);

            if let Err(e) = self.document_store.save_chunks(&chunks) {
                errors.push(format!("Failed to save chunks for {}: {}", doc.id, e));
                continue;
            }

            // Collect chunks for embedding
            for chunk in &chunks {
                all_chunk_texts.push((chunk.id.clone(), chunk.text.clone()));
            }
            chunks_created += chunks.len();
        }

        report_progress(on_progress, "storing", total_docs, total_docs,
            &format!("Stored {} documents, {} chunks", total_docs, chunks_created));
        tracing::info!(documents = total_docs, chunks = chunks_created, "Store stage complete");

        // Stage 5: Generate embeddings + store in vector store
        if let (Some(provider), Some(vector_store)) = (&self.embedding_provider, &self.vector_store) {
            let total_to_embed = all_chunk_texts.len();
            report_progress(on_progress, "embedding", 0, total_to_embed, "Generating embeddings...");

            for (batch_idx, batch) in all_chunk_texts.chunks(self.embedding_batch_size).enumerate() {
                self.check_cancelled()?;
                let texts: Vec<String> = batch.iter().map(|(_, t)| t.clone()).collect();
                let ids: Vec<&str> = batch.iter().map(|(id, _)| id.as_str()).collect();

                // Apply 120s timeout per embedding batch
                match tokio::time::timeout(
                    std::time::Duration::from_secs(120),
                    provider.embed_batch(&texts),
                ).await {
                    Ok(Ok(embeddings)) => {
                        let items: Vec<(String, Vec<f32>, HashMap<String, String>)> = ids
                            .iter()
                            .zip(embeddings.into_iter())
                            .map(|(id, vec)| (id.to_string(), vec, HashMap::new()))
                            .collect();

                        if let Err(e) = vector_store.upsert_batch(&items) {
                            errors.push(format!("Failed to store embeddings batch {}: {}", batch_idx, e));
                        } else {
                            embeddings_generated += items.len();
                        }
                    }
                    Ok(Err(e)) => {
                        errors.push(format!("Embedding batch {} failed: {}", batch_idx, e));
                    }
                    Err(_timeout) => {
                        errors.push(format!("Embedding batch {} timed out after 120s", batch_idx));
                        tracing::warn!(batch = batch_idx, "Embedding batch timed out");
                    }
                }

                let progress = std::cmp::min((batch_idx + 1) * self.embedding_batch_size, total_to_embed);
                report_progress(on_progress, "embedding", progress, total_to_embed,
                    &format!("Embedded {}/{} chunks", progress, total_to_embed));
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        report_progress(on_progress, "complete", total_docs, total_docs,
            &format!("Import complete in {}ms", duration_ms));

        Ok(ImportSummary {
            documents_imported: total_docs,
            chunks_created,
            embeddings_generated,
            duplicates_skipped,
            errors,
            duration_ms,
        })
    }
}

fn report_progress(
    on_progress: Option<&ProgressCallback>,
    stage: &str,
    current: usize,
    total: usize,
    message: &str,
) {
    if let Some(cb) = on_progress {
        cb(stage, current, total, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::SqliteConnection;
    use crate::adapters::sqlite::document_store::SqliteDocumentStore;
    use crate::adapters::sqlite::page_index::SqliteFts5Index;
    use crate::adapters::sqlite::timeline_store::SqliteTimelineStore;
    use crate::domain::models::common::SourcePlatform;
    use crate::domain::models::document::Document;
    use chrono::Utc;
    use sha2::{Digest, Sha256};

    struct MockAdapter {
        docs: Vec<Document>,
    }

    impl SourceAdapter for MockAdapter {
        fn metadata(&self) -> crate::pipeline::ingestion::source_adapters::SourceAdapterMeta {
            crate::pipeline::ingestion::source_adapters::SourceAdapterMeta {
                id: "mock".into(),
                display_name: "Mock".into(),
                icon: "file".into(),
                takeout_url: None,
                instructions: "".into(),
                accepted_extensions: vec![],
                handles_zip: false,
                platform: crate::domain::models::common::SourcePlatform::Custom,
            }
        }
        fn detect(&self, _file_listing: &[&str]) -> f32 {
            0.0
        }
        fn parse(&self, _path: &std::path::Path) -> Result<Vec<Document>, AppError> {
            Ok(self.docs.clone())
        }
        fn name(&self) -> &str {
            "mock"
        }
    }

    fn make_doc(text: &str) -> Document {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        Document {
            id: uuid::Uuid::new_v4().to_string(),
            source_platform: SourcePlatform::Markdown,
            raw_text: text.to_string(),
            timestamp: Some(Utc::now()),
            participants: vec![],
            metadata: serde_json::json!({}),
            content_hash: hash,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_full_pipeline() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let doc_store = SqliteDocumentStore::new(db.clone());
        let timeline_store = SqliteTimelineStore::new(db.clone());
        let page_index = SqliteFts5Index::new(db);

        let orchestrator = IngestionOrchestrator::new(&doc_store, &timeline_store, &page_index);

        let adapter = MockAdapter {
            docs: vec![
                make_doc("Today I thought about the meaning of life. It was a beautiful day for reflection."),
                make_doc("I went hiking in the mountains. The air was fresh and clear."),
                make_doc("Meeting with the team about the project roadmap. We decided to focus on quality."),
            ],
        };

        let summary = orchestrator
            .ingest(&adapter, std::path::Path::new("/fake"), None)
            .await
            .unwrap();

        assert_eq!(summary.documents_imported, 3);
        assert!(summary.chunks_created >= 3);
        assert_eq!(summary.duplicates_skipped, 0);
        assert!(summary.errors.is_empty());
    }

    #[tokio::test]
    async fn test_dedup_in_pipeline() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let doc_store = SqliteDocumentStore::new(db.clone());
        let timeline_store = SqliteTimelineStore::new(db.clone());
        let page_index = SqliteFts5Index::new(db);

        let orchestrator = IngestionOrchestrator::new(&doc_store, &timeline_store, &page_index);

        let doc = make_doc("Unique content for dedup test.");
        let adapter = MockAdapter { docs: vec![doc.clone()] };

        let s1 = orchestrator.ingest(&adapter, std::path::Path::new("/fake"), None).await.unwrap();
        assert_eq!(s1.documents_imported, 1);

        let adapter2 = MockAdapter { docs: vec![doc] };
        let s2 = orchestrator.ingest(&adapter2, std::path::Path::new("/fake"), None).await.unwrap();
        assert_eq!(s2.documents_imported, 0);
        assert_eq!(s2.duplicates_skipped, 1);
    }
}
