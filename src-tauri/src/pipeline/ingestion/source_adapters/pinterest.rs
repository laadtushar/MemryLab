use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct PinterestAdapter;

impl SourceAdapter for PinterestAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "pinterest".into(),
            display_name: "Pinterest".into(),
            icon: "pin".into(),
            takeout_url: Some("https://www.pinterest.com/settings/privacy".into()),
            instructions: "Request your data from Pinterest (Settings > Privacy > Request your data). Download and upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Pinterest,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_pins = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("pins") || lower.contains("/pin/")
        });
        let has_boards = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("boards") || lower.contains("board")
        });
        if has_pins && has_boards {
            0.9
        } else if has_pins || has_boards {
            0.7
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "pinterest"
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

            let rel_path = entry.path().to_string_lossy().replace('\\', "/").to_lowercase();

            if rel_path.contains("pins") || rel_path.contains("pin") {
                parse_pins(&value, &mut documents);
            } else if rel_path.contains("boards") || rel_path.contains("board") {
                parse_boards(&value, &mut documents);
            }
        }

        Ok(documents)
    }
}

fn parse_pins(value: &serde_json::Value, docs: &mut Vec<Document>) {
    let items = match value.as_array() {
        Some(arr) => arr,
        None => return,
    };

    for pin in items {
        let description = pin
            .get("description")
            .or_else(|| pin.get("note"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let link = pin
            .get("link")
            .or_else(|| pin.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let text = if description.is_empty() && link.is_empty() {
            parse_utils::flatten_json_to_text(pin)
        } else if description.is_empty() {
            format!("Pinned: {}", link)
        } else {
            description.to_string()
        };

        if text.trim().is_empty() {
            log::debug!("Skipping empty content in Pinterest pin");
            continue;
        }

        let board = pin
            .get("board")
            .and_then(|b| b.get("name").or_else(|| b.as_str().map(|_| b)))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Board");

        let timestamp = pin
            .get("created_at")
            .or_else(|| pin.get("date"))
            .and_then(|v| v.as_str())
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            })
            ;

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("pin".into()));
        meta.insert("board".into(), serde_json::Value::String(board.into()));
        if !link.is_empty() {
            meta.insert("link".into(), serde_json::Value::String(link.into()));
        }

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Pinterest,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_boards(value: &serde_json::Value, docs: &mut Vec<Document>) {
    let items = match value.as_array() {
        Some(arr) => arr,
        None => return,
    };

    for board in items {
        let name = board.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let description = board.get("description").and_then(|v| v.as_str()).unwrap_or("");

        let text = if description.is_empty() {
            if name.is_empty() { continue; } else { format!("Board: {}", name) }
        } else {
            format!("Board: {} - {}", name, description)
        };

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("board".into()));
        meta.insert("board_name".into(), serde_json::Value::String(name.into()));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Pinterest,
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
    fn test_detect_pinterest() {
        let adapter = PinterestAdapter;
        let files = vec!["Pins/pins.json", "Boards/boards.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = PinterestAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_parse_pins() {
        let value = serde_json::json!([
            {
                "description": "Cool recipe idea",
                "board": {"name": "Food"},
                "link": "https://example.com/recipe"
            }
        ]);
        let mut docs = Vec::new();
        parse_pins(&value, &mut docs);
        assert_eq!(docs.len(), 1);
        assert!(docs[0].raw_text.contains("Cool recipe"));
    }
}
