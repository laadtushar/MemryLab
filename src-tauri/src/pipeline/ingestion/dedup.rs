use crate::domain::models::document::Document;
use crate::domain::ports::document_store::IDocumentStore;

/// Result of deduplication: documents split into new and duplicate.
pub struct DedupResult {
    pub new_documents: Vec<Document>,
    pub duplicates_skipped: usize,
}

/// Check documents against the store, returning only those with new content hashes.
pub fn deduplicate(
    documents: Vec<Document>,
    store: &dyn IDocumentStore,
) -> DedupResult {
    let mut new_documents = Vec::new();
    let mut duplicates_skipped = 0;

    for doc in documents {
        match store.get_by_content_hash(&doc.content_hash) {
            Ok(Some(_)) => {
                duplicates_skipped += 1;
                log::debug!("Skipping duplicate: hash={}", &doc.content_hash[..16]);
            }
            Ok(None) => {
                new_documents.push(doc);
            }
            Err(e) => {
                log::warn!("Error checking hash {}: {}", &doc.content_hash[..16], e);
                // On error, include the document to avoid data loss
                new_documents.push(doc);
            }
        }
    }

    DedupResult {
        new_documents,
        duplicates_skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::connection::SqliteConnection;
    use crate::adapters::sqlite::document_store::SqliteDocumentStore;
    use crate::domain::models::common::SourcePlatform;
    use chrono::Utc;
    use std::sync::Arc;

    fn make_doc(id: &str, hash: &str) -> Document {
        Document {
            id: id.to_string(),
            source_platform: SourcePlatform::Markdown,
            raw_text: format!("content for {}", id),
            timestamp: Some(Utc::now()),
            participants: vec![],
            metadata: serde_json::json!({}),
            content_hash: hash.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_dedup_filters_existing() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteDocumentStore::new(db);

        // Pre-insert a document
        let existing = make_doc("existing", "hash_existing");
        store.save_document(&existing).unwrap();

        // Try to import with one existing and one new
        let docs = vec![
            make_doc("d1", "hash_existing"), // duplicate
            make_doc("d2", "hash_new"),       // new
        ];

        let result = deduplicate(docs, &store);
        assert_eq!(result.new_documents.len(), 1);
        assert_eq!(result.duplicates_skipped, 1);
        assert_eq!(result.new_documents[0].content_hash, "hash_new");
    }
}
