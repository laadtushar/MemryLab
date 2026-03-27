use std::path::Path;

use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct SlackAdapter;

impl SourceAdapter for SlackAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "slack".into(),
            display_name: "Slack".into(),
            icon: "hash".into(),
            takeout_url: None,
            instructions: "Ask your workspace admin to export data. Upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Slack,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_channels = file_listing.iter().any(|f| f.ends_with("channels.json"));
        let has_users = file_listing.iter().any(|f| f.ends_with("users.json"));
        if has_channels || has_users { 0.85 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "slack"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Walk for JSON files in channel subdirectories
        // Slack exports have: channel_name/YYYY-MM-DD.json
        for entry in WalkDir::new(path)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "json")
                    && e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n != "channels.json" && n != "users.json" && n != "integration_logs.json")
                        .unwrap_or(false)
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
                Err(_) => continue,
            };

            // Infer channel name from parent directory
            let channel = entry
                .path()
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            if let Some(messages) = value.as_array() {
                for msg in messages {
                    let text = msg.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    if text.trim().is_empty() {
                        continue;
                    }

                    let user = msg
                        .get("user")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let timestamp = msg
                        .get("ts")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.split('.').next())
                        .and_then(|s| s.parse::<i64>().ok())
                        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                        ;

                    let mut meta = serde_json::Map::new();
                    meta.insert("channel".into(), serde_json::Value::String(channel.clone()));
                    if let Some(subtype) = msg.get("subtype").and_then(|v| v.as_str()) {
                        meta.insert("subtype".into(), serde_json::Value::String(subtype.into()));
                    }

                    documents.push(parse_utils::build_document(
                        text.to_string(),
                        SourcePlatform::Slack,
                        timestamp,
                        vec![user],
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
    fn test_detect_slack() {
        let adapter = SlackAdapter;
        let files = vec!["channels.json", "users.json", "general/2024-01-01.json"];
        assert!(adapter.detect(&files) >= 0.85);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = SlackAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = SlackAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "slack");
        assert!(meta.handles_zip);
    }
}
