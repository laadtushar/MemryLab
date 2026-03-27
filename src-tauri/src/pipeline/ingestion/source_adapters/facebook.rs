use std::path::Path;

use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct FacebookAdapter;

impl SourceAdapter for FacebookAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "facebook".into(),
            display_name: "Facebook".into(),
            icon: "facebook".into(),
            takeout_url: Some("https://www.facebook.com/dyi/?referrer=yfi_settings".into()),
            instructions: "Download your Facebook data (JSON format recommended). Go to Settings > Your Facebook Information.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Facebook,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_messages = file_listing.iter().any(|f| f.contains("messages/inbox/"));
        let has_posts = file_listing.iter().any(|f| f.contains("posts/your_posts"));
        if has_messages || has_posts { 0.9 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "facebook"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Parse messages
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let p = e.path().to_string_lossy().replace('\\', "/");
                e.path().extension().is_some_and(|ext| ext == "json")
                    && (p.contains("messages/inbox/") || p.contains("posts/"))
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

            let rel_path = entry.path().to_string_lossy().replace('\\', "/");

            if rel_path.contains("messages/inbox/") {
                parse_fb_messages(&value, &mut documents);
            } else if rel_path.contains("posts/") {
                parse_fb_posts(&value, &mut documents);
            }
        }

        Ok(documents)
    }
}

fn parse_fb_messages(value: &serde_json::Value, docs: &mut Vec<Document>) {
    let thread_name = value
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| parse_utils::fix_facebook_encoding(s))
        .unwrap_or_default();

    let messages = match value.get("messages").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return,
    };

    for msg in messages {
        let content = msg
            .get("content")
            .and_then(|v| v.as_str())
            .map(|s| parse_utils::fix_facebook_encoding(s))
            .unwrap_or_default();

        if content.trim().is_empty() {
            continue;
        }

        let sender = msg
            .get("sender_name")
            .and_then(|v| v.as_str())
            .map(|s| parse_utils::fix_facebook_encoding(s))
            .unwrap_or_default();

        let timestamp = msg
            .get("timestamp_ms")
            .and_then(|v| v.as_i64())
            .and_then(|ms| {
                chrono::DateTime::from_timestamp(ms / 1000, ((ms % 1000) * 1_000_000) as u32)
            })
            ;

        let mut meta = serde_json::Map::new();
        meta.insert("thread".into(), serde_json::Value::String(thread_name.clone()));
        meta.insert("type".into(), serde_json::Value::String("message".into()));

        docs.push(parse_utils::build_document(
            content,
            SourcePlatform::Facebook,
            timestamp,
            vec![sender],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_fb_posts(value: &serde_json::Value, docs: &mut Vec<Document>) {
    // Posts can be an array at top level or nested
    let posts = if let Some(arr) = value.as_array() {
        arr.clone()
    } else if let Some(arr) = value.get("status_updates").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        return;
    };

    for post in &posts {
        let text = post
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("post"))
            .and_then(|v| v.as_str())
            .map(|s| parse_utils::fix_facebook_encoding(s))
            .unwrap_or_default();

        if text.trim().is_empty() {
            continue;
        }

        let timestamp = post
            .get("timestamp")
            .and_then(|v| v.as_i64())
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
            ;

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("post".into()));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Facebook,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_facebook() {
        let adapter = FacebookAdapter;
        let files = vec!["messages/inbox/alice/message_1.json", "posts/your_posts_1.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = FacebookAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_parse_fb_messages() {
        let value = serde_json::json!({
            "title": "Alice",
            "messages": [
                {
                    "sender_name": "Alice",
                    "content": "Hello there!",
                    "timestamp_ms": 1710504600000_i64
                },
                {
                    "sender_name": "Bob",
                    "content": "Hi Alice!",
                    "timestamp_ms": 1710504660000_i64
                }
            ]
        });

        let mut docs = Vec::new();
        parse_fb_messages(&value, &mut docs);
        assert_eq!(docs.len(), 2);
        assert!(docs[0].raw_text.contains("Hello"));
    }
}
