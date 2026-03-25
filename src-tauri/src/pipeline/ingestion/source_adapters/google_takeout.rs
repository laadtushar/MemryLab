use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct GoogleTakeoutAdapter;

impl SourceAdapter for GoogleTakeoutAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "google_takeout".into(),
            display_name: "Google Takeout".into(),
            icon: "chrome".into(),
            takeout_url: Some("https://takeout.google.com/".into()),
            instructions: "Download your data from Google Takeout. Select the services you want (Keep, Chrome, YouTube, etc.).".into(),
            accepted_extensions: vec!["zip".into()],
            handles_zip: true,
            platform: SourcePlatform::GoogleTakeout,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_takeout = file_listing.iter().any(|f| f.starts_with("Takeout/") || f.contains("/Takeout/"));
        let has_archive_browser = file_listing.iter().any(|f| f.contains("archive_browser.html"));
        if has_takeout || has_archive_browser { 0.9 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "google_takeout"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Find the Takeout root
        let takeout_root = if path.join("Takeout").is_dir() {
            path.join("Takeout")
        } else {
            path.to_path_buf()
        };

        // Google Keep notes
        parse_keep_notes(&takeout_root, &mut documents);

        // Chrome browser history
        parse_chrome_history(&takeout_root, &mut documents);

        // YouTube watch history
        parse_youtube_history(&takeout_root, &mut documents);

        // Walk remaining JSON files for anything else interesting
        for entry in WalkDir::new(&takeout_root)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "json")
                    && !e.path().to_string_lossy().contains("Keep")
                    && !e.path().to_string_lossy().contains("Chrome")
                    && !e.path().to_string_lossy().contains("YouTube")
            })
        {
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let text = parse_utils::flatten_json_to_text(&value);
            if text.len() > 20 {
                let mut meta = serde_json::Map::new();
                meta.insert(
                    "source_file".into(),
                    serde_json::Value::String(entry.path().display().to_string()),
                );

                documents.push(parse_utils::build_document(
                    text,
                    SourcePlatform::GoogleTakeout,
                    Utc::now(),
                    vec![],
                    serde_json::Value::Object(meta),
                ));
            }
        }

        Ok(documents)
    }
}

fn parse_keep_notes(root: &Path, docs: &mut Vec<Document>) {
    let keep_dir = root.join("Keep");
    if !keep_dir.is_dir() {
        return;
    }

    for entry in WalkDir::new(&keep_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
    {
        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let title = value.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let text_content = value.get("textContent").and_then(|v| v.as_str()).unwrap_or("");

        let full_text = if title.is_empty() {
            text_content.to_string()
        } else {
            format!("{}\n\n{}", title, text_content)
        };

        if full_text.trim().is_empty() {
            continue;
        }

        let timestamp = value
            .get("userEditedTimestampUsec")
            .and_then(|v| v.as_i64())
            .and_then(|us| chrono::DateTime::from_timestamp(us / 1_000_000, 0))
            .unwrap_or_else(Utc::now);

        let mut meta = serde_json::Map::new();
        meta.insert("service".into(), serde_json::Value::String("Google Keep".into()));
        if !title.is_empty() {
            meta.insert("title".into(), serde_json::Value::String(title.into()));
        }

        docs.push(parse_utils::build_document(
            full_text,
            SourcePlatform::GoogleTakeout,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_chrome_history(root: &Path, docs: &mut Vec<Document>) {
    let history_path = root.join("Chrome").join("BrowserHistory.json");
    if !history_path.is_file() {
        return;
    }

    let content = match std::fs::read_to_string(&history_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    let items = match value.get("Browser History").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return,
    };

    for item in items {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");

        if title.is_empty() && url.is_empty() {
            continue;
        }

        let text = format!("{}\n{}", title, url);

        let timestamp = item
            .get("time_usec")
            .and_then(|v| v.as_i64())
            .and_then(|us| chrono::DateTime::from_timestamp(us / 1_000_000, 0))
            .unwrap_or_else(Utc::now);

        let mut meta = serde_json::Map::new();
        meta.insert("service".into(), serde_json::Value::String("Chrome".into()));
        meta.insert("url".into(), serde_json::Value::String(url.into()));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::GoogleTakeout,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_youtube_history(root: &Path, docs: &mut Vec<Document>) {
    // YouTube watch history may be in several locations
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            name == "watch-history.json" || name == "search-history.json"
        })
    {
        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(arr) = value.as_array() {
            for item in arr {
                let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
                if title.trim().is_empty() {
                    continue;
                }

                let timestamp = item
                    .get("time")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                let mut meta = serde_json::Map::new();
                meta.insert("service".into(), serde_json::Value::String("YouTube".into()));
                if let Some(url) = item.get("titleUrl").and_then(|v| v.as_str()) {
                    meta.insert("url".into(), serde_json::Value::String(url.into()));
                }

                docs.push(parse_utils::build_document(
                    title.to_string(),
                    SourcePlatform::GoogleTakeout,
                    timestamp,
                    vec![],
                    serde_json::Value::Object(meta),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_google_takeout() {
        let adapter = GoogleTakeoutAdapter;
        let files = vec!["Takeout/Keep/note1.json", "Takeout/Chrome/BrowserHistory.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_archive_browser() {
        let adapter = GoogleTakeoutAdapter;
        let files = vec!["archive_browser.html", "Keep/note.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = GoogleTakeoutAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = GoogleTakeoutAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "google_takeout");
        assert!(meta.takeout_url.is_some());
    }
}
