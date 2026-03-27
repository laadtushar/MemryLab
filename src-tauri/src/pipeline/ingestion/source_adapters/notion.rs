use std::path::Path;

use chrono::{DateTime, Utc};
use regex::Regex;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct NotionAdapter;

impl SourceAdapter for NotionAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "notion".into(),
            display_name: "Notion".into(),
            icon: "layout".into(),
            takeout_url: None,
            instructions: "Export from Notion (Settings > Workspace > Export all workspace content). Choose Markdown & CSV format.".into(),
            accepted_extensions: vec!["zip".into(), "md".into(), "csv".into()],
            handles_zip: true,
            platform: SourcePlatform::Notion,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        // Notion exports have files with 32-char hex UUIDs appended
        let uuid_re = Regex::new(r"[a-f0-9]{32}\.md$").unwrap();
        let notion_files = file_listing
            .iter()
            .filter(|f| uuid_re.is_match(f))
            .count();
        if notion_files > 2 { 0.7 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "notion"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();
        let uuid_suffix_re = Regex::new(r"\s+[a-f0-9]{32}$").unwrap();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "md" || ext == "csv")
            })
        {
            let file_path = entry.path();
            let ext = file_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            // Clean title: strip Notion UUID suffix from filename
            let raw_stem = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let title = uuid_suffix_re.replace(raw_stem, "").to_string();

            if ext == "md" {
                let content = match std::fs::read_to_string(file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        log::warn!("Skipping {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                if content.trim().is_empty() {
                    continue;
                }

                let full_text = if title.is_empty() {
                    content.clone()
                } else {
                    format!("# {}\n\n{}", title, content)
                };

                let file_meta = std::fs::metadata(file_path).ok();
                let timestamp = file_meta
                    .and_then(|m| m.modified().ok())
                    .map(|t| DateTime::<Utc>::from(t))
                    ;

                let mut hasher = Sha256::new();
                hasher.update(full_text.as_bytes());
                let content_hash = format!("{:x}", hasher.finalize());

                let mut meta = serde_json::Map::new();
                meta.insert("title".into(), serde_json::Value::String(title.clone()));
                meta.insert(
                    "source_path".into(),
                    serde_json::Value::String(file_path.display().to_string()),
                );

                documents.push(Document {
                    id: Uuid::new_v4().to_string(),
                    source_platform: SourcePlatform::Notion,
                    raw_text: full_text,
                    timestamp,
                    participants: vec![],
                    metadata: serde_json::Value::Object(meta),
                    content_hash,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            } else if ext == "csv" {
                let rows = match parse_utils::parse_csv_file(file_path) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                for row in rows {
                    let text = row.values().cloned().collect::<Vec<_>>().join(" | ");
                    if text.trim().is_empty() {
                        continue;
                    }

                    let mut meta = serde_json::Map::new();
                    meta.insert("title".into(), serde_json::Value::String(title.clone()));
                    meta.insert("type".into(), serde_json::Value::String("database_row".into()));

                    documents.push(parse_utils::build_document(
                        text,
                        SourcePlatform::Notion,
                        None,
                        vec![],
                        serde_json::Value::Object(meta),
                    ));
                }
            }
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_notion() {
        let adapter = NotionAdapter;
        let files = vec![
            "My Page abc123def456789012345678abcdef01.md",
            "Another Page 1234567890abcdef1234567890abcdef.md",
            "Third Page fedcba9876543210fedcba9876543210.md",
        ];
        assert!(adapter.detect(&files) >= 0.7);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = NotionAdapter;
        assert!(adapter.detect(&["notes.md", "readme.md"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = NotionAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "notion");
    }
}
