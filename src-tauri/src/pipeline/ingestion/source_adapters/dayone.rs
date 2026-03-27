use std::path::Path;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};

/// Parses Day One JSON export files.
pub struct DayOneAdapter;

impl SourceAdapter for DayOneAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "dayone".into(),
            display_name: "Day One".into(),
            icon: "book-open".into(),
            takeout_url: None,
            instructions: "Select a Day One JSON export file.".into(),
            accepted_extensions: vec!["json".into()],
            handles_zip: true,
            platform: SourcePlatform::DayOne,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_entries_json = file_listing.iter().any(|f| f.ends_with("Journal.json"));
        let has_json_with_entries = file_listing.iter().any(|f| f.ends_with(".json"));
        if has_entries_json {
            0.8
        } else if has_json_with_entries {
            // Could be Day One but not certain without peeking at content
            0.0
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "dayone"
    }

    fn parse(&self, file_path: &Path) -> Result<Vec<Document>, AppError> {
        // If it's a directory, walk for JSON files
        if file_path.is_dir() {
            let mut all_docs = Vec::new();
            for entry in walkdir::WalkDir::new(file_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            {
                match parse_dayone_file(entry.path()) {
                    Ok(docs) => all_docs.extend(docs),
                    Err(e) => log::warn!("Skipping {}: {}", entry.path().display(), e),
                }
            }
            return Ok(all_docs);
        }

        parse_dayone_file(file_path)
    }
}

fn parse_dayone_file(file_path: &Path) -> Result<Vec<Document>, AppError> {
    let content = std::fs::read_to_string(file_path)?;
    let export: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| AppError::Import(format!("Invalid Day One JSON: {}", e)))?;

    let entries = export
        .get("entries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::Import("No 'entries' array in Day One export".to_string()))?;

    let mut documents = Vec::new();

    for entry in entries {
        match parse_entry(entry) {
            Ok(doc) => documents.push(doc),
            Err(e) => {
                log::warn!("Skipping Day One entry: {}", e);
            }
        }
    }

    Ok(documents)
}

fn parse_entry(entry: &serde_json::Value) -> Result<Document, AppError> {
    let text = entry
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if text.trim().is_empty() {
        return Err(AppError::Import("Empty entry".to_string()));
    }

    // Parse timestamp
    let timestamp = entry
        .get("creationDate")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
        .or_else(|| {
            entry
                .get("creationDate")
                .and_then(|v| v.as_str())
                .and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
                        .ok()
                        .map(|dt| dt.and_utc())
                })
        })
        ;

    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let content_hash = format!("{:x}", hasher.finalize());

    // Extract metadata
    let mut meta = serde_json::Map::new();
    if let Some(tags) = entry.get("tags") {
        meta.insert("tags".to_string(), tags.clone());
    }
    if let Some(weather) = entry.get("weather") {
        meta.insert("weather".to_string(), weather.clone());
    }
    if let Some(location) = entry.get("location") {
        meta.insert("location".to_string(), location.clone());
    }
    if let Some(uuid) = entry.get("uuid").and_then(|v| v.as_str()) {
        meta.insert("dayone_uuid".to_string(), serde_json::Value::String(uuid.to_string()));
    }

    Ok(Document {
        id: Uuid::new_v4().to_string(),
        source_platform: SourcePlatform::DayOne,
        raw_text: text.to_string(),
        timestamp,
        participants: vec![],
        metadata: serde_json::Value::Object(meta),
        content_hash,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_dayone() {
        let adapter = DayOneAdapter;
        let files = vec!["entries/Journal.json", "photos/abc.jpg"];
        assert!(adapter.detect(&files) >= 0.8);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = DayOneAdapter;
        let files = vec!["random.txt", "data.csv"];
        assert!(adapter.detect(&files) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = DayOneAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "dayone");
        assert_eq!(meta.platform, SourcePlatform::DayOne);
        assert!(meta.handles_zip);
    }

    #[test]
    fn test_parse_dayone_entry() {
        let entry = serde_json::json!({
            "text": "Today was a great day. I went for a walk.",
            "creationDate": "2024-03-15T14:30:00Z",
            "tags": ["journal", "daily"],
            "uuid": "ABC123"
        });

        let doc = parse_entry(&entry).unwrap();
        assert_eq!(doc.source_platform, SourcePlatform::DayOne);
        assert!(doc.raw_text.contains("great day"));
        assert!(!doc.content_hash.is_empty());
    }

    #[test]
    fn test_empty_entry_skipped() {
        let entry = serde_json::json!({
            "text": "",
            "creationDate": "2024-03-15T14:30:00Z"
        });
        assert!(parse_entry(&entry).is_err());
    }
}
