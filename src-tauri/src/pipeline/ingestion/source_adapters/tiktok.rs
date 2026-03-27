use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct TikTokAdapter;

impl SourceAdapter for TikTokAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "tiktok".into(),
            display_name: "TikTok".into(),
            icon: "music".into(),
            takeout_url: Some("https://www.tiktok.com/setting/download-your-data".into()),
            instructions: "Request your data from TikTok (Settings > Account > Download your data). Choose JSON format.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::TikTok,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_activity = file_listing.iter().any(|f| f.contains("Activity/") || f.contains("activity/"));
        if has_activity { 0.85 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "tiktok"
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
                Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
            };

            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
            };

            let file_name = entry
                .file_name()
                .to_string_lossy()
                .to_lowercase();

            let category = if file_name.contains("comment") {
                "comment"
            } else if file_name.contains("chat") || file_name.contains("message") {
                "message"
            } else if file_name.contains("browse") || file_name.contains("video") {
                "browse_history"
            } else if file_name.contains("like") || file_name.contains("favorite") {
                "likes"
            } else {
                "activity"
            };

            // TikTok data can be deeply nested; walk the JSON for text content
            parse_tiktok_json(&value, category, &mut documents);
        }

        Ok(documents)
    }
}

fn parse_tiktok_json(value: &serde_json::Value, category: &str, docs: &mut Vec<Document>) {
    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_tiktok_item(item, category, docs);
            }
        }
        serde_json::Value::Object(obj) => {
            // Walk into nested objects looking for arrays
            for (key, val) in obj {
                let sub_category = if key.to_lowercase().contains("comment") {
                    "comment"
                } else if key.to_lowercase().contains("chat") {
                    "message"
                } else {
                    category
                };

                if let Some(arr) = val.as_array() {
                    for item in arr {
                        extract_tiktok_item(item, sub_category, docs);
                    }
                } else if val.is_object() {
                    parse_tiktok_json(val, sub_category, docs);
                }
            }
        }
        _ => {}
    }
}

fn extract_tiktok_item(item: &serde_json::Value, category: &str, docs: &mut Vec<Document>) {
    // Try common field names for text content
    let text = item
        .get("Comment")
        .or_else(|| item.get("comment"))
        .or_else(|| item.get("Content")  )
        .or_else(|| item.get("content"))
        .or_else(|| item.get("Text"))
        .or_else(|| item.get("text"))
        .or_else(|| item.get("Description"))
        .or_else(|| item.get("VideoDescription"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if text.trim().is_empty() || text.len() < 3 {
        return;
    }

    let timestamp = item
        .get("Date")
        .or_else(|| item.get("date"))
        .or_else(|| item.get("CreateDate"))
        .and_then(|v| v.as_str())
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|dt| dt.and_utc())
                })
        })
        ;

    let mut meta = serde_json::Map::new();
    meta.insert("category".into(), serde_json::Value::String(category.into()));

    docs.push(parse_utils::build_document(
        text.to_string(),
        SourcePlatform::TikTok,
        timestamp,
        vec![],
        serde_json::Value::Object(meta),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_tiktok() {
        let adapter = TikTokAdapter;
        let files = vec!["Activity/Video Browsing History.json", "Activity/Like List.json"];
        assert!(adapter.detect(&files) >= 0.85);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = TikTokAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_extract_tiktok_item() {
        let item = serde_json::json!({
            "Comment": "This is funny!",
            "Date": "2024-03-15 14:30:00"
        });
        let mut docs = Vec::new();
        extract_tiktok_item(&item, "comment", &mut docs);
        assert_eq!(docs.len(), 1);
        assert!(docs[0].raw_text.contains("funny"));
    }

    #[test]
    fn test_metadata() {
        let adapter = TikTokAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "tiktok");
        assert!(meta.takeout_url.is_some());
    }
}
