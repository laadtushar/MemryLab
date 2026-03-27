use std::path::Path;

use chrono::Utc;
use regex::Regex;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct WhatsAppAdapter;

impl SourceAdapter for WhatsAppAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "whatsapp".into(),
            display_name: "WhatsApp".into(),
            icon: "message-circle".into(),
            takeout_url: None,
            instructions: "Export a chat from WhatsApp (Settings > Chats > Export Chat) and upload the ZIP or TXT file.".into(),
            accepted_extensions: vec!["zip".into(), "txt".into()],
            handles_zip: true,
            platform: SourcePlatform::WhatsApp,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_chat = file_listing.iter().any(|f| {
            f.contains("_chat.txt") || f.ends_with("WhatsApp Chat.txt")
        });
        if has_chat { 0.9 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "whatsapp"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        let txt_files: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "txt"))
                .map(|e| e.path().to_path_buf())
                .collect()
        } else if path.extension().is_some_and(|ext| ext == "txt") {
            vec![path.to_path_buf()]
        } else {
            vec![]
        };

        // WhatsApp message pattern: [dd/mm/yy, HH:MM:SS] Sender: message
        // or variants: [M/D/YY, H:MM:SS AM/PM] Sender: message
        let msg_re = Regex::new(
            r"\[(\d{1,2}/\d{1,2}/\d{2,4}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\s*(?:AM|PM|am|pm)?\]\s*([^:]+):\s*(.*)"
        ).unwrap();

        for txt_path in txt_files {
            let content = match std::fs::read_to_string(&txt_path) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Skipping {}: {}", txt_path.display(), e);
                    continue;
                }
            };

            let chat_name = txt_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("WhatsApp Chat")
                .to_string();

            let mut current_sender: Option<String> = None;
            let mut current_messages: Vec<String> = Vec::new();
            let mut current_ts = Utc::now();

            let flush = |sender: &Option<String>,
                         messages: &[String],
                         ts: chrono::DateTime<Utc>,
                         chat: &str,
                         docs: &mut Vec<Document>| {
                if let Some(ref s) = sender {
                    let text = messages.join("\n");
                    if !text.trim().is_empty() {
                        let mut meta = serde_json::Map::new();
                        meta.insert("chat".into(), serde_json::Value::String(chat.into()));
                        docs.push(parse_utils::build_document(
                            text,
                            SourcePlatform::WhatsApp,
                            Some(ts),
                            vec![s.clone()],
                            serde_json::Value::Object(meta),
                        ));
                    }
                }
            };

            for line in content.lines() {
                if let Some(caps) = msg_re.captures(line) {
                    // Flush previous message group
                    flush(&current_sender, &current_messages, current_ts, &chat_name, &mut documents);

                    let date_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let time_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                    let sender = caps.get(3).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                    let message = caps.get(4).map(|m| m.as_str().to_string()).unwrap_or_default();

                    // Try to parse timestamp
                    let dt_str = format!("{} {}", date_str, time_str);
                    let ts = chrono::NaiveDateTime::parse_from_str(&dt_str, "%d/%m/%y %H:%M:%S")
                        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&dt_str, "%d/%m/%Y %H:%M:%S"))
                        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&dt_str, "%m/%d/%y %H:%M:%S"))
                        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&dt_str, "%d/%m/%y %H:%M"))
                        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&dt_str, "%m/%d/%y %H:%M"))
                        .map(|ndt| ndt.and_utc())
                        .unwrap_or_else(|_| Utc::now());

                    current_sender = Some(sender);
                    current_messages = vec![message];
                    current_ts = ts;
                } else if current_sender.is_some() {
                    // Continuation line
                    current_messages.push(line.to_string());
                }
            }

            // Flush last message
            flush(&current_sender, &current_messages, current_ts, &chat_name, &mut documents);
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_whatsapp() {
        let adapter = WhatsAppAdapter;
        let files = vec!["WhatsApp Chat with Alice/_chat.txt"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = WhatsAppAdapter;
        assert!(adapter.detect(&["random.txt"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = WhatsAppAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "whatsapp");
        assert!(meta.handles_zip);
    }
}
