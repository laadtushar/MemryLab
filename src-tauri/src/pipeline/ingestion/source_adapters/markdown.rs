use std::path::Path;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};

/// Parses a directory of plain .md and .txt files.
pub struct MarkdownAdapter;

impl SourceAdapter for MarkdownAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "markdown".into(),
            display_name: "Markdown / Text".into(),
            icon: "file-text".into(),
            takeout_url: None,
            instructions: "Select a folder of .md or .txt files.".into(),
            accepted_extensions: vec!["md".into(), "txt".into()],
            handles_zip: false,
            platform: SourcePlatform::Markdown,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let count = file_listing
            .iter()
            .filter(|f| f.ends_with(".md") || f.ends_with(".txt"))
            .count();
        if count > 3 { 0.4 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "markdown"
    }

    fn parse(&self, dir_path: &Path) -> Result<Vec<Document>, AppError> {
        if !dir_path.is_dir() {
            return Err(AppError::Import(format!(
                "Path is not a directory: {}",
                dir_path.display()
            )));
        }

        let mut documents = Vec::new();

        for entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "md" || ext == "txt")
                    && !e
                        .path()
                        .components()
                        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
            })
        {
            match parse_text_file(entry.path()) {
                Ok(doc) => documents.push(doc),
                Err(e) => {
                    log::warn!("Skipping file {}: {}", entry.path().display(), e);
                }
            }
        }

        Ok(documents)
    }
}

fn parse_text_file(path: &Path) -> Result<Document, AppError> {
    let content = std::fs::read_to_string(path)?;

    if content.trim().is_empty() {
        return Err(AppError::Import("Empty file".to_string()));
    }

    let metadata = std::fs::metadata(path)?;
    let modified = metadata
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let timestamp = DateTime::<Utc>::from(modified);

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let content_hash = format!("{:x}", hasher.finalize());

    let mut meta = serde_json::Map::new();
    if let Some(title) = path.file_stem().and_then(|s| s.to_str()) {
        meta.insert("title".to_string(), serde_json::Value::String(title.to_string()));
    }
    meta.insert(
        "source_path".to_string(),
        serde_json::Value::String(path.display().to_string()),
    );

    Ok(Document {
        id: Uuid::new_v4().to_string(),
        source_platform: SourcePlatform::Markdown,
        raw_text: content,
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
    fn test_detect_markdown() {
        let adapter = MarkdownAdapter;
        let files = vec!["notes/a.md", "notes/b.md", "notes/c.txt", "notes/d.md"];
        assert!(adapter.detect(&files) > 0.3);
    }

    #[test]
    fn test_detect_too_few() {
        let adapter = MarkdownAdapter;
        let files = vec!["a.md", "b.txt"];
        assert!(adapter.detect(&files) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = MarkdownAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "markdown");
        assert_eq!(meta.platform, SourcePlatform::Markdown);
        assert!(!meta.handles_zip);
    }
}
