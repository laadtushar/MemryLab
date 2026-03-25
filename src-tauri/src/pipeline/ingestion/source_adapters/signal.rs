use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct SignalAdapter;

impl SourceAdapter for SignalAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "signal".into(),
            display_name: "Signal".into(),
            icon: "message-circle".into(),
            takeout_url: None,
            instructions: "Export from Signal Desktop. The backup is a JSON file containing your message history.".into(),
            accepted_extensions: vec!["json".into()],
            handles_zip: false,
            platform: SourcePlatform::Signal,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_signal_json = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("signal") && lower.ends_with(".json")
        });
        if has_signal_json { 0.85 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "signal"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        let json_files: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "json")
                        && e.path().to_string_lossy().to_lowercase().contains("signal")
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        } else if path.extension().is_some_and(|ext| ext == "json") {
            vec![path.to_path_buf()]
        } else {
            vec![]
        };

        for json_path in json_files {
            let content = match std::fs::read_to_string(&json_path) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Skipping {}: {}", json_path.display(), e);
                    continue;
                }
            };

            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Skipping {}: {}", json_path.display(), e);
                    continue;
                }
            };

            // Signal Desktop backup exports conversations as an array
            if let Some(conversations) = value.get("conversations").and_then(|v| v.as_array()) {
                for conv in conversations {
                    parse_signal_conversation(conv, &mut documents);
                }
            }

            // Some exports have messages at top level
            if let Some(messages) = value.get("messages").and_then(|v| v.as_array()) {
                parse_signal_messages(messages, "", &mut documents);
            }

            // Array of messages at top level
            if let Some(arr) = value.as_array() {
                parse_signal_messages(arr, "", &mut documents);
            }
        }

        Ok(documents)
    }
}

fn parse_signal_conversation(conv: &serde_json::Value, docs: &mut Vec<Document>) {
    let contact_name = conv
        .get("name")
        .or_else(|| conv.get("profileName"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    if let Some(messages) = conv.get("messages").and_then(|v| v.as_array()) {
        parse_signal_messages(messages, contact_name, docs);
    }
}

fn parse_signal_messages(messages: &[serde_json::Value], contact: &str, docs: &mut Vec<Document>) {
    for msg in messages {
        let body = msg
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if body.trim().is_empty() {
            continue;
        }

        let sender = msg
            .get("source")
            .or_else(|| msg.get("sender"))
            .or_else(|| msg.get("from"))
            .and_then(|v| v.as_str())
            .unwrap_or(contact)
            .to_string();

        let timestamp = msg
            .get("timestamp")
            .or_else(|| msg.get("sent_at"))
            .and_then(|v| v.as_i64())
            .and_then(|ms| chrono::DateTime::from_timestamp(ms / 1000, ((ms % 1000) * 1_000_000) as u32))
            .unwrap_or_else(Utc::now);

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("message".into()));
        if !contact.is_empty() {
            meta.insert("conversation".into(), serde_json::Value::String(contact.into()));
        }
        if let Some(msg_type) = msg.get("type").and_then(|v| v.as_str()) {
            meta.insert("message_type".into(), serde_json::Value::String(msg_type.into()));
        }

        docs.push(parse_utils::build_document(
            body.to_string(),
            SourcePlatform::Signal,
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
    fn test_detect_signal() {
        let adapter = SignalAdapter;
        let files = vec!["signal_backup.json"];
        assert!(adapter.detect(&files) >= 0.85);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = SignalAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_parse_signal_messages() {
        let messages = vec![
            serde_json::json!({
                "body": "Hello from Signal!",
                "source": "Alice",
                "timestamp": 1710504600000_i64
            }),
            serde_json::json!({
                "body": "Hey there!",
                "source": "Bob",
                "timestamp": 1710504660000_i64
            }),
        ];
        let mut docs = Vec::new();
        parse_signal_messages(&messages, "Group Chat", &mut docs);
        assert_eq!(docs.len(), 2);
        assert!(docs[0].raw_text.contains("Hello"));
    }
}
