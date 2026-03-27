use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct MastodonAdapter;

impl SourceAdapter for MastodonAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "mastodon".into(),
            display_name: "Mastodon".into(),
            icon: "mastodon".into(),
            takeout_url: Some("https://mastodon.social/settings/export".into()),
            instructions: "Export your data from Mastodon (Settings > Import and export > Data export). URL varies by instance.".into(),
            accepted_extensions: vec!["zip".into(), "json".into(), "csv".into()],
            handles_zip: true,
            platform: SourcePlatform::Mastodon,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_outbox = file_listing.iter().any(|f| f.ends_with("outbox.json"));
        let has_following = file_listing.iter().any(|f| f.contains("following_accounts.csv"));
        let has_actor = file_listing.iter().any(|f| f.ends_with("actor.json"));
        if has_outbox {
            0.9
        } else if has_following || has_actor {
            0.7
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "mastodon"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Look for outbox.json (ActivityPub format)
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let fname = e.path().file_name().and_then(|s| s.to_str()).unwrap_or("");
                fname == "outbox.json"
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

            // ActivityPub OrderedCollection: { "orderedItems": [ { "type": "Create", "object": { "type": "Note", ... } } ] }
            let items = value
                .get("orderedItems")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            for item in &items {
                let activity_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if activity_type != "Create" {
                    continue;
                }

                let object = match item.get("object") {
                    Some(obj) => obj,
                    None => continue,
                };

                // Extract content from the Note object (HTML)
                let html_content = object
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let text = parse_utils::html_to_text(html_content);
                if text.trim().is_empty() {
                    continue;
                }

                let timestamp = item
                    .get("published")
                    .or_else(|| object.get("published"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    })
                    ;

                let mut meta = serde_json::Map::new();
                meta.insert("type".into(), serde_json::Value::String("toot".into()));
                if let Some(id) = object.get("id").and_then(|v| v.as_str()) {
                    meta.insert("url".into(), serde_json::Value::String(id.into()));
                }
                if let Some(in_reply_to) = object.get("inReplyTo").and_then(|v| v.as_str()) {
                    meta.insert("in_reply_to".into(), serde_json::Value::String(in_reply_to.into()));
                }
                if let Some(sensitive) = object.get("sensitive").and_then(|v| v.as_bool()) {
                    meta.insert("sensitive".into(), serde_json::Value::Bool(sensitive));
                }

                let attributed_to = object
                    .get("attributedTo")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let participants = if attributed_to.is_empty() { vec![] } else { vec![attributed_to] };

                documents.push(parse_utils::build_document(
                    text,
                    SourcePlatform::Mastodon,
                    timestamp,
                    participants,
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
    fn test_detect_mastodon() {
        let adapter = MastodonAdapter;
        let files = vec!["outbox.json", "actor.json", "following_accounts.csv"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_partial() {
        let adapter = MastodonAdapter;
        assert!(adapter.detect(&["following_accounts.csv"]) >= 0.5);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = MastodonAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = MastodonAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "mastodon");
        assert_eq!(meta.platform, SourcePlatform::Mastodon);
    }
}
