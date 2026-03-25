use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct GenericAdapter;

impl SourceAdapter for GenericAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "generic".into(),
            display_name: "Generic / Other".into(),
            icon: "file-text".into(),
            takeout_url: None,
            instructions: "Import any text-based files (TXT, JSON, CSV, HTML, Markdown). The adapter will attempt to extract text from supported formats.".into(),
            accepted_extensions: vec![
                "txt".into(), "json".into(), "csv".into(),
                "html".into(), "htm".into(), "md".into(),
            ],
            handles_zip: true,
            platform: SourcePlatform::Custom,
        }
    }

    fn detect(&self, _file_listing: &[&str]) -> f32 {
        // Lowest priority fallback — always returns a tiny score
        0.1
    }

    fn name(&self) -> &str {
        "generic"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        let entries: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            vec![path.to_path_buf()]
        };

        for file_path in entries {
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let text = match ext.as_str() {
                "json" => {
                    let content = match std::fs::read_to_string(&file_path) {
                        Ok(c) => c,
                        Err(e) => {
                            log::warn!("Skipping {}: {}", file_path.display(), e);
                            continue;
                        }
                    };
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(v) => parse_utils::flatten_json_to_text(&v),
                        Err(_) => content,
                    }
                }
                "html" | "htm" => {
                    let content = match std::fs::read_to_string(&file_path) {
                        Ok(c) => c,
                        Err(e) => {
                            log::warn!("Skipping {}: {}", file_path.display(), e);
                            continue;
                        }
                    };
                    parse_utils::html_to_text(&content)
                }
                "csv" => {
                    match parse_utils::parse_csv_file(&file_path) {
                        Ok(rows) => {
                            let lines: Vec<String> = rows
                                .iter()
                                .map(|row| {
                                    row.values()
                                        .cloned()
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                })
                                .collect();
                            lines.join("\n")
                        }
                        Err(e) => {
                            log::warn!("Skipping CSV {}: {}", file_path.display(), e);
                            continue;
                        }
                    }
                }
                "txt" | "md" => {
                    match std::fs::read_to_string(&file_path) {
                        Ok(c) => c,
                        Err(e) => {
                            log::warn!("Skipping {}: {}", file_path.display(), e);
                            continue;
                        }
                    }
                }
                _ => continue,
            };

            if text.trim().is_empty() {
                continue;
            }

            let file_name = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let mut meta = serde_json::Map::new();
            meta.insert("source_file".into(), serde_json::Value::String(file_name));
            meta.insert("format".into(), serde_json::Value::String(ext.clone()));

            documents.push(parse_utils::build_document(
                text,
                SourcePlatform::Custom,
                Utc::now(),
                vec![],
                serde_json::Value::Object(meta),
            ));
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_generic() {
        let adapter = GenericAdapter;
        // Always returns 0.1 regardless of input
        assert!((adapter.detect(&["anything.txt"]) - 0.1).abs() < f32::EPSILON);
        assert!((adapter.detect(&[]) - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_metadata() {
        let adapter = GenericAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "generic");
        assert!(meta.takeout_url.is_none());
        assert_eq!(meta.platform, SourcePlatform::Custom);
    }
}
