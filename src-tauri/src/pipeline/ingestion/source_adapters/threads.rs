use std::path::Path;

use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct ThreadsAdapter;

impl SourceAdapter for ThreadsAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "threads".into(),
            display_name: "Threads".into(),
            icon: "at-sign".into(),
            takeout_url: Some("https://www.instagram.com/download/request/".into()),
            instructions: "Threads data is bundled with your Instagram data export. Request a download from Instagram and look for the threads/ folder.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Threads,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_threads_dir = file_listing.iter().any(|f| {
            let normalized = f.replace('\\', "/");
            normalized.contains("threads/") || normalized.contains("threads_posts")
        });
        let has_threads_file = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("threads") && lower.ends_with(".json")
        });
        if has_threads_dir { 0.9 } else if has_threads_file { 0.6 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "threads"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let p = e.path().to_string_lossy().replace('\\', "/").to_lowercase();
                e.path().extension().is_some_and(|ext| ext == "json")
                    && (p.contains("threads/") || p.contains("threads_posts") || p.contains("threads_"))
            })
        {
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Skipping {}: {}", entry.path().display(), e);
                    continue;
                }
            };

            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Skipping {}: {}", entry.path().display(), e);
                    continue;
                }
            };

            let rel_path = entry.path().to_string_lossy().replace('\\', "/").to_lowercase();

            if rel_path.contains("posts") {
                parse_threads_posts(&value, &mut documents);
            } else {
                // Try generic threads content
                parse_threads_posts(&value, &mut documents);
            }
        }

        Ok(documents)
    }
}

fn parse_threads_posts(value: &serde_json::Value, docs: &mut Vec<Document>) {
    // Threads export format is similar to Instagram: array of post objects
    let items = if let Some(arr) = value.as_array() {
        arr.clone()
    } else if let Some(arr) = value.get("text_post_app_text_posts").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        return;
    };

    for item in &items {
        // Try multiple known field paths for the post text
        let text = item
            .get("post")
            .and_then(|p| p.as_array())
            .and_then(|arr| arr.first())
            .and_then(|p| p.get("text"))
            .and_then(|v| v.as_str())
            .or_else(|| item.get("text").and_then(|v| v.as_str()))
            .or_else(|| item.get("title").and_then(|v| v.as_str()))
            .or_else(|| {
                item.get("media")
                    .and_then(|m| m.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|m| m.get("title"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("");

        if text.trim().is_empty() {
            continue;
        }

        let timestamp = item
            .get("creation_timestamp")
            .or_else(|| item.get("timestamp"))
            .and_then(|v| v.as_i64())
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
            ;

        let author = item
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("thread_post".into()));
        if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
            meta.insert("url".into(), serde_json::Value::String(url.into()));
        }

        let participants = if author.is_empty() { vec![] } else { vec![author] };

        docs.push(parse_utils::build_document(
            text.to_string(),
            SourcePlatform::Threads,
            timestamp,
            participants,
            serde_json::Value::Object(meta),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_threads() {
        let adapter = ThreadsAdapter;
        let files = vec!["threads/threads_posts.json", "threads/profile.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = ThreadsAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_parse_threads_posts() {
        let value = serde_json::json!([
            {
                "text": "My first thread!",
                "creation_timestamp": 1710504600
            },
            {
                "text": "Another post here",
                "creation_timestamp": 1710504700
            }
        ]);
        let mut docs = Vec::new();
        parse_threads_posts(&value, &mut docs);
        assert_eq!(docs.len(), 2);
        assert!(docs[0].raw_text.contains("first thread"));
    }
}
