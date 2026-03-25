use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct YouTubeAdapter;

impl SourceAdapter for YouTubeAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "youtube".into(),
            display_name: "YouTube".into(),
            icon: "youtube".into(),
            takeout_url: Some("https://takeout.google.com/".into()),
            instructions: "Download your YouTube data from Google Takeout (select YouTube only). Upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::YouTube,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_watch = file_listing.iter().any(|f| f.contains("watch-history.json"));
        if has_watch { 0.85 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "youtube"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Find relevant JSON files
        let json_files: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    e.path().extension().is_some_and(|ext| ext == "json")
                        && (name.contains("watch-history")
                            || name.contains("search-history")
                            || name.contains("my-comments")
                            || name.contains("subscriptions"))
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            vec![path.to_path_buf()]
        };

        for json_path in json_files {
            let content = match std::fs::read_to_string(&json_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let file_name = json_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            let category = if file_name.contains("watch") {
                "watch"
            } else if file_name.contains("search") {
                "search"
            } else if file_name.contains("comment") {
                "comment"
            } else {
                "other"
            };

            if let Some(arr) = value.as_array() {
                for item in arr {
                    let title = item
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if title.trim().is_empty() {
                        continue;
                    }

                    let timestamp = item
                        .get("time")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now);

                    let mut meta = serde_json::Map::new();
                    meta.insert("category".into(), serde_json::Value::String(category.into()));
                    if let Some(url) = item.get("titleUrl").and_then(|v| v.as_str()) {
                        meta.insert("url".into(), serde_json::Value::String(url.into()));
                    }
                    if let Some(channel) = item.get("subtitles")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|s| s.get("name"))
                        .and_then(|v| v.as_str())
                    {
                        meta.insert("channel".into(), serde_json::Value::String(channel.into()));
                    }

                    documents.push(parse_utils::build_document(
                        title.to_string(),
                        SourcePlatform::YouTube,
                        timestamp,
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
    fn test_detect_youtube() {
        let adapter = YouTubeAdapter;
        let files = vec!["YouTube/history/watch-history.json"];
        assert!(adapter.detect(&files) >= 0.85);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = YouTubeAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = YouTubeAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "youtube");
        assert!(meta.takeout_url.is_some());
    }
}
