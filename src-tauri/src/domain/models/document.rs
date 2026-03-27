use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::SourcePlatform;

/// A raw ingested document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub source_platform: SourcePlatform,
    pub raw_text: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub participants: Vec<String>,
    pub metadata: serde_json::Value,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A chunk of a document, used for embedding and retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub text: String,
    pub token_count: usize,
    pub position: usize,
    pub created_at: DateTime<Utc>,
}
