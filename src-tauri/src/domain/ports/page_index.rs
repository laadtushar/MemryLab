use crate::error::AppError;

/// A full-text search result with BM25 ranking
#[derive(Debug, Clone, serde::Serialize)]
pub struct FtsResult {
    pub chunk_id: String,
    pub snippet: String,
    pub rank_score: f64,
}

/// Port for full-text search with BM25 ranking
pub trait IPageIndex: Send + Sync {
    fn index_text(&self, chunk_id: &str, text: &str) -> Result<(), AppError>;
    fn index_batch(&self, items: &[(String, String)]) -> Result<(), AppError>;
    fn search(&self, query: &str, top_k: usize) -> Result<Vec<FtsResult>, AppError>;
    fn remove(&self, chunk_id: &str) -> Result<(), AppError>;

    /// Search with <mark> highlighted snippets for UI display
    fn search_highlighted(&self, query: &str, top_k: usize) -> Result<Vec<FtsResult>, AppError> {
        self.search(query, top_k) // default: same as search
    }

    /// Prefix-based suggestions for autocomplete
    fn suggest(&self, prefix: &str, top_k: usize) -> Result<Vec<String>, AppError> {
        let _ = (prefix, top_k);
        Ok(Vec::new())
    }

    /// Find related chunks to a given chunk using BM25 on extracted terms
    fn find_related(&self, chunk_id: &str, top_k: usize) -> Result<Vec<FtsResult>, AppError> {
        let _ = (chunk_id, top_k);
        Ok(Vec::new())
    }
}
