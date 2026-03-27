use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct DiscordAdapter;

impl SourceAdapter for DiscordAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "discord".into(),
            display_name: "Discord".into(),
            icon: "gamepad-2".into(),
            takeout_url: None,
            instructions: "Request your data from Discord (Settings > Privacy & Safety > Request all of my Data). Upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "csv".into()],
            handles_zip: true,
            platform: SourcePlatform::Discord,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_messages = file_listing.iter().any(|f| {
            f.contains("messages/") && f.ends_with("messages.csv")
        });
        if has_messages { 0.85 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "discord"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Walk for messages.csv files in channel directories
        let csv_files: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name().to_string_lossy() == "messages.csv"
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            vec![path.to_path_buf()]
        };

        for csv_path in csv_files {
            // Channel ID from parent directory
            let channel = csv_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Try to read channel.json for name
            let channel_name = csv_path
                .parent()
                .map(|p| p.join("channel.json"))
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(String::from))
                .unwrap_or(channel);

            let rows = match parse_utils::parse_csv_file(&csv_path) {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("Skipping {}: {}", csv_path.display(), e);
                    continue;
                }
            };

            for row in rows {
                let content = row.get("Contents").cloned().unwrap_or_default();
                if content.trim().is_empty() {
                    continue;
                }

                let timestamp = row
                    .get("Timestamp")
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    ;

                let mut meta = serde_json::Map::new();
                meta.insert("channel".into(), serde_json::Value::String(channel_name.clone()));

                documents.push(parse_utils::build_document(
                    content,
                    SourcePlatform::Discord,
                    timestamp,
                    vec![],
                    serde_json::Value::Object(meta),
                ));
            }
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_discord() {
        let adapter = DiscordAdapter;
        let files = vec!["messages/123456/messages.csv", "messages/123456/channel.json"];
        assert!(adapter.detect(&files) >= 0.85);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = DiscordAdapter;
        assert!(adapter.detect(&["random.csv"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = DiscordAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "discord");
        assert!(meta.handles_zip);
    }
}
