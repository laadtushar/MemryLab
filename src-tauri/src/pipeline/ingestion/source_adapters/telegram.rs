use std::path::Path;

use chrono::Utc;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct TelegramAdapter;

impl SourceAdapter for TelegramAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "telegram".into(),
            display_name: "Telegram".into(),
            icon: "send".into(),
            takeout_url: None,
            instructions: "Export chat history from Telegram Desktop (Settings > Advanced > Export Telegram data). Choose JSON format.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Telegram,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_result = file_listing.iter().any(|f| f.ends_with("result.json"));
        if has_result { 0.9 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "telegram"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Find result.json
        let result_path = if path.is_dir() {
            let candidate = path.join("result.json");
            if candidate.is_file() {
                candidate
            } else {
                // Walk for it
                walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .find(|e| e.file_name() == "result.json")
                    .map(|e| e.path().to_path_buf())
                    .unwrap_or(candidate)
            }
        } else {
            path.to_path_buf()
        };

        let content = std::fs::read_to_string(&result_path)
            .map_err(|e| AppError::Import(format!("Cannot read Telegram export: {}", e)))?;

        let value: serde_json::Value = serde_json::from_str(&content)?;

        // Telegram export structure: { chats: { list: [ { messages: [...] } ] } }
        let chats = value
            .get("chats")
            .and_then(|c| c.get("list"))
            .and_then(|l| l.as_array());

        if let Some(chat_list) = chats {
            for chat in chat_list {
                let chat_name = chat
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown Chat")
                    .to_string();

                let messages = match chat.get("messages").and_then(|v| v.as_array()) {
                    Some(arr) => arr,
                    None => continue,
                };

                for msg in messages {
                    // Text can be a string or an array of text entities
                    let text = extract_telegram_text(msg);
                    if text.trim().is_empty() {
                        continue;
                    }

                    let sender = msg
                        .get("from")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let timestamp = msg
                        .get("date")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .or_else(|| {
                            msg.get("date_unixtime")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<i64>().ok())
                                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                        })
                        .unwrap_or_else(Utc::now);

                    let mut meta = serde_json::Map::new();
                    meta.insert("chat".into(), serde_json::Value::String(chat_name.clone()));
                    if let Some(msg_type) = msg.get("type").and_then(|v| v.as_str()) {
                        meta.insert("message_type".into(), serde_json::Value::String(msg_type.into()));
                    }

                    documents.push(parse_utils::build_document(
                        text,
                        SourcePlatform::Telegram,
                        timestamp,
                        if sender.is_empty() { vec![] } else { vec![sender] },
                        serde_json::Value::Object(meta),
                    ));
                }
            }
        }

        Ok(documents)
    }
}

/// Extract text from a Telegram message, handling both plain strings and text entity arrays.
fn extract_telegram_text(msg: &serde_json::Value) -> String {
    match msg.get("text") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => {
            arr.iter()
                .map(|item| match item {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Object(obj) => {
                        obj.get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    }
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("")
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_telegram() {
        let adapter = TelegramAdapter;
        let files = vec!["result.json", "photos/photo_1.jpg"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = TelegramAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_extract_text_string() {
        let msg = serde_json::json!({"text": "Hello world"});
        assert_eq!(extract_telegram_text(&msg), "Hello world");
    }

    #[test]
    fn test_extract_text_array() {
        let msg = serde_json::json!({
            "text": [
                "Hello ",
                {"type": "bold", "text": "world"}
            ]
        });
        assert_eq!(extract_telegram_text(&msg), "Hello world");
    }

    #[test]
    fn test_metadata() {
        let adapter = TelegramAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "telegram");
    }
}
