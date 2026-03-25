use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Category of a memory fact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FactCategory {
    Belief,
    Preference,
    Fact,
    SelfDescription,
    Insight,
}

/// A long-term memory fact extracted from documents (Mem0-style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFact {
    pub id: String,
    pub fact_text: String,
    pub source_chunks: Vec<String>,
    pub confidence: f64,
    pub category: FactCategory,
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub contradicted_by: Vec<String>,
    pub is_active: bool,
}
