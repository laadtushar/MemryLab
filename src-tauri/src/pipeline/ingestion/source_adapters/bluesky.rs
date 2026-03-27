use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct BlueskyAdapter;

impl SourceAdapter for BlueskyAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "bluesky".into(),
            display_name: "Bluesky".into(),
            icon: "cloud".into(),
            takeout_url: Some("https://bsky.app/settings".into()),
            instructions: "Export your data from Bluesky (Settings > Account > Export My Data). Upload the resulting file.".into(),
            accepted_extensions: vec!["zip".into(), "json".into(), "car".into()],
            handles_zip: true,
            platform: SourcePlatform::Bluesky,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_car = file_listing.iter().any(|f| f.ends_with(".car"));
        let has_actor_json = file_listing.iter().any(|f| {
            f.contains("actor") || f.contains("post")
        });
        if has_car || has_actor_json { 0.7 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "bluesky"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Check for .car files (CAR format — Content Addressable aRchive)
        let has_car = if path.is_dir() {
            WalkDir::new(path)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| e.path().extension().is_some_and(|ext| ext == "car"))
        } else {
            path.extension().is_some_and(|ext| ext == "car")
        };

        if has_car {
            log::info!("Bluesky CAR format detected. Full CAR parsing is not yet implemented — extracting available JSON data.");
        }

        // Parse any JSON files available
        let json_files: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
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
                Err(_) => continue,
            };

            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue,
            };

            extract_bsky_posts(&value, &mut documents);
        }

        Ok(documents)
    }
}

fn extract_bsky_posts(value: &serde_json::Value, docs: &mut Vec<Document>) {
    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_single_post(item, docs);
            }
        }
        serde_json::Value::Object(_) => {
            // Could be a single post or a container
            if value.get("text").is_some() || value.get("$type").is_some() {
                extract_single_post(value, docs);
            } else {
                // Walk nested objects for post arrays
                for (_, val) in value.as_object().unwrap() {
                    if let Some(arr) = val.as_array() {
                        for item in arr {
                            extract_single_post(item, docs);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn extract_single_post(item: &serde_json::Value, docs: &mut Vec<Document>) {
    let text = item
        .get("text")
        .or_else(|| item.get("value").and_then(|v| v.get("text")))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if text.trim().is_empty() {
        return;
    }

    let timestamp = item
        .get("createdAt")
        .or_else(|| item.get("value").and_then(|v| v.get("createdAt")))
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        ;

    let mut meta = serde_json::Map::new();
    meta.insert("type".into(), serde_json::Value::String("post".into()));

    docs.push(parse_utils::build_document(
        text.to_string(),
        SourcePlatform::Bluesky,
        timestamp,
        vec![],
        serde_json::Value::Object(meta),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_bluesky_car() {
        let adapter = BlueskyAdapter;
        let files = vec!["repo.car"];
        assert!(adapter.detect(&files) >= 0.7);
    }

    #[test]
    fn test_detect_bluesky_json() {
        let adapter = BlueskyAdapter;
        let files = vec!["actor.json", "post_records.json"];
        assert!(adapter.detect(&files) >= 0.7);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = BlueskyAdapter;
        assert!(adapter.detect(&["random.txt"]) < 0.1);
    }

    #[test]
    fn test_extract_single_post() {
        let item = serde_json::json!({
            "text": "Hello from Bluesky!",
            "createdAt": "2024-03-15T14:30:00Z"
        });
        let mut docs = Vec::new();
        extract_single_post(&item, &mut docs);
        assert_eq!(docs.len(), 1);
        assert!(docs[0].raw_text.contains("Bluesky"));
    }

    #[test]
    fn test_metadata() {
        let adapter = BlueskyAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "bluesky");
        assert!(meta.takeout_url.is_some());
    }
}
