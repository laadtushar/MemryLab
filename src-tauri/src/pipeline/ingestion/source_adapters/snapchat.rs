use std::path::Path;

use chrono::Utc;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct SnapchatAdapter;

impl SourceAdapter for SnapchatAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "snapchat".into(),
            display_name: "Snapchat".into(),
            icon: "ghost".into(),
            takeout_url: Some("https://accounts.snapchat.com/accounts/downloadmydata".into()),
            instructions: "Request your data from Snapchat (Settings > My Data > Submit Request). Upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Snapchat,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_chat = file_listing.iter().any(|f| f.contains("json/chat_history.json"));
        if has_chat { 0.9 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "snapchat"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Walk for JSON files
        let json_files: Vec<_> = if path.is_dir() {
            walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
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
                .unwrap_or("");

            if file_name.contains("chat_history") {
                parse_snap_chats(&value, &mut documents);
            } else if file_name.contains("memories") || file_name.contains("snap_history") {
                parse_snap_memories(&value, &mut documents);
            }
        }

        Ok(documents)
    }
}

fn parse_snap_chats(value: &serde_json::Value, docs: &mut Vec<Document>) {
    // Snapchat chat history: array of messages or nested under a key
    let items = if let Some(arr) = value.as_array() {
        arr.clone()
    } else if let Some(arr) = value.get("Received Chat History").and_then(|v| v.as_array()) {
        arr.clone()
    } else if let Some(arr) = value.get("Sent Chat History").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        return;
    };

    for item in &items {
        let text = item
            .get("Text")
            .or_else(|| item.get("text"))
            .or_else(|| item.get("Body"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if text.trim().is_empty() {
            continue;
        }

        let sender = item
            .get("From")
            .or_else(|| item.get("from"))
            .or_else(|| item.get("Sender"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let timestamp = item
            .get("Created")
            .or_else(|| item.get("created"))
            .or_else(|| item.get("Date"))
            .and_then(|v| v.as_str())
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| {
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %Z")
                            .ok()
                            .map(|dt| dt.and_utc())
                    })
            })
            ;

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("chat".into()));

        docs.push(parse_utils::build_document(
            text.to_string(),
            SourcePlatform::Snapchat,
            timestamp,
            if sender.is_empty() { vec![] } else { vec![sender] },
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_snap_memories(value: &serde_json::Value, docs: &mut Vec<Document>) {
    let text = parse_utils::flatten_json_to_text(value);
    if text.len() > 20 {
        let meta = serde_json::json!({"type": "memories"});
        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Snapchat,
            None,
            vec![],
            meta,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_snapchat() {
        let adapter = SnapchatAdapter;
        let files = vec!["json/chat_history.json", "json/memories_history.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = SnapchatAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_parse_snap_chats() {
        let value = serde_json::json!([
            {"From": "alice", "Text": "Hey there!", "Created": "2024-03-15T14:30:00Z"},
            {"From": "bob", "Text": "Hello!", "Created": "2024-03-15T14:31:00Z"}
        ]);
        let mut docs = Vec::new();
        parse_snap_chats(&value, &mut docs);
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn test_metadata() {
        let adapter = SnapchatAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "snapchat");
        assert!(meta.takeout_url.is_some());
    }
}
