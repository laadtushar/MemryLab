use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct AppleAdapter;

impl SourceAdapter for AppleAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "apple".into(),
            display_name: "Apple".into(),
            icon: "apple".into(),
            takeout_url: Some("https://privacy.apple.com/".into()),
            instructions: "Request your data from Apple (privacy.apple.com > Request a copy of your data). Download and upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "csv".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Apple,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_apple = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("apple") || lower.contains("apple_id")
        });
        let has_icloud = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("icloud") || lower.contains("apple_media_services")
        });
        if has_apple && has_icloud {
            0.9
        } else if has_apple || has_icloud {
            0.7
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "apple"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let ext = entry.path()
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            match ext.as_str() {
                "csv" => parse_apple_csv(entry.path(), &mut documents),
                "json" => parse_apple_json(entry.path(), &mut documents),
                _ => {}
            }
        }

        Ok(documents)
    }
}

fn parse_apple_csv(path: &Path, docs: &mut Vec<Document>) {
    let rows = match parse_utils::parse_csv_file(path) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Skipping Apple CSV {}: {}", path.display(), e);
            return;
        }
    };

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let doc_type = if file_name.to_lowercase().contains("note") {
        "note"
    } else if file_name.to_lowercase().contains("health") {
        "health"
    } else if file_name.to_lowercase().contains("purchase") || file_name.to_lowercase().contains("transaction") {
        "purchase"
    } else {
        "data"
    };

    for row in rows {
        // Build text from all non-empty values
        let text_parts: Vec<String> = row
            .iter()
            .filter(|(_, v)| !v.trim().is_empty())
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();

        let text = text_parts.join("; ");
        if text.trim().is_empty() {
            log::debug!("Skipping empty content in Apple CSV");
            continue;
        }

        let timestamp = row
            .get("Date")
            .or_else(|| row.get("date"))
            .or_else(|| row.get("Timestamp"))
            .or_else(|| row.get("Event Date"))
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| {
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                            .ok()
                            .map(|dt| dt.and_utc())
                    })
                    .or_else(|| {
                        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                            .ok()
                            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
                    })
            })
            ;

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String(doc_type.into()));
        meta.insert("source_file".into(), serde_json::Value::String(file_name.clone()));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Apple,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_apple_json(path: &Path, docs: &mut Vec<Document>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Skipping Apple JSON {}: {}", path.display(), e);
            return;
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => { log::warn!("Skipping Apple JSON {}: {}", path.display(), e); return; }
    };

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Handle array of items or a single object
    let items = if let Some(arr) = value.as_array() {
        arr.clone()
    } else {
        vec![value]
    };

    for item in &items {
        let text = parse_utils::flatten_json_to_text(item);
        if text.trim().is_empty() {
            log::debug!("Skipping empty content in Apple JSON");
            continue;
        }

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("apple_data".into()));
        meta.insert("source_file".into(), serde_json::Value::String(file_name.clone()));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Apple,
            None,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_apple() {
        let adapter = AppleAdapter;
        let files = vec!["Apple_ID/apple_id_account.csv", "iCloud/notes.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = AppleAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = AppleAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "apple");
        assert!(meta.takeout_url.is_some());
    }
}
