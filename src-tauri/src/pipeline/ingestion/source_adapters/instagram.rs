use std::path::Path;

use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct InstagramAdapter;

impl SourceAdapter for InstagramAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "instagram".into(),
            display_name: "Instagram".into(),
            icon: "instagram".into(),
            takeout_url: Some("https://www.instagram.com/download/request/".into()),
            instructions: "Request your data download from Instagram (Settings > Your Activity > Download Your Information). Choose JSON format.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Instagram,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_posts = file_listing.iter().any(|f| f.contains("content/posts_1.json"));
        let has_messages = file_listing.iter().any(|f| {
            f.contains("messages/inbox/") && f.contains("message_1.json")
        });
        if has_posts || has_messages { 0.85 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "instagram"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        {
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let rel_path = entry.path().to_string_lossy().replace('\\', "/");

            if rel_path.contains("content/") || rel_path.contains("posts") {
                parse_ig_posts(&value, &mut documents);
            } else if rel_path.contains("messages/") {
                parse_ig_messages(&value, &mut documents);
            }
        }

        Ok(documents)
    }
}

fn parse_ig_posts(value: &serde_json::Value, docs: &mut Vec<Document>) {
    // Posts can be an array at top level
    let items = if let Some(arr) = value.as_array() {
        arr.clone()
    } else {
        return;
    };

    for item in &items {
        // Instagram posts have nested media with title
        let text = item
            .get("media")
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|m| m.get("title"))
            .and_then(|v| v.as_str())
            .or_else(|| item.get("title").and_then(|v| v.as_str()))
            .unwrap_or("");

        if text.trim().is_empty() {
            continue;
        }

        let timestamp = item
            .get("media")
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|m| m.get("creation_timestamp"))
            .or_else(|| item.get("creation_timestamp"))
            .and_then(|v| v.as_i64())
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
            ;

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("post".into()));

        docs.push(parse_utils::build_document(
            text.to_string(),
            SourcePlatform::Instagram,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_ig_messages(value: &serde_json::Value, docs: &mut Vec<Document>) {
    let thread_name = value
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let messages = match value.get("messages").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return,
    };

    for msg in messages {
        let content = msg
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if content.trim().is_empty() {
            continue;
        }

        let sender = msg
            .get("sender_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let timestamp = msg
            .get("timestamp_ms")
            .and_then(|v| v.as_i64())
            .and_then(|ms| chrono::DateTime::from_timestamp(ms / 1000, ((ms % 1000) * 1_000_000) as u32))
            ;

        let mut meta = serde_json::Map::new();
        meta.insert("thread".into(), serde_json::Value::String(thread_name.into()));
        meta.insert("type".into(), serde_json::Value::String("message".into()));

        docs.push(parse_utils::build_document(
            content.to_string(),
            SourcePlatform::Instagram,
            timestamp,
            vec![sender],
            serde_json::Value::Object(meta),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_instagram() {
        let adapter = InstagramAdapter;
        let files = vec!["content/posts_1.json", "profile/profile.json"];
        assert!(adapter.detect(&files) >= 0.85);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = InstagramAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_parse_ig_messages() {
        let value = serde_json::json!({
            "title": "Alice",
            "messages": [
                {
                    "sender_name": "Alice",
                    "content": "Hey!",
                    "timestamp_ms": 1710504600000_i64
                }
            ]
        });
        let mut docs = Vec::new();
        parse_ig_messages(&value, &mut docs);
        assert_eq!(docs.len(), 1);
        assert!(docs[0].raw_text.contains("Hey"));
    }
}
