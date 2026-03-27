use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct MicrosoftAdapter;

impl SourceAdapter for MicrosoftAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "microsoft".into(),
            display_name: "Microsoft".into(),
            icon: "monitor".into(),
            takeout_url: Some("https://account.microsoft.com/privacy/download-data".into()),
            instructions: "Request your data from Microsoft (Privacy Dashboard > Download your data). Download and upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "json".into(), "csv".into()],
            handles_zip: true,
            platform: SourcePlatform::Microsoft,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_search = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("searchrequestsandquery") || lower.contains("search_requests")
        });
        let has_browse = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("browsehistory") || lower.contains("browse_history")
        });
        let has_ms_path = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("microsoft") || lower.contains("cortana") || lower.contains("outlook")
        });

        if has_search || has_browse { 0.9 }
        else if has_ms_path { 0.7 }
        else { 0.0 }
    }

    fn name(&self) -> &str {
        "microsoft"
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

            let rel_path = entry.path().to_string_lossy().replace('\\', "/").to_lowercase();

            match ext.as_str() {
                "json" => parse_ms_json(entry.path(), &rel_path, &mut documents),
                "csv" => parse_ms_csv(entry.path(), &rel_path, &mut documents),
                _ => {}
            }
        }

        Ok(documents)
    }
}

fn parse_ms_json(path: &Path, rel_path: &str, docs: &mut Vec<Document>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Skipping Microsoft JSON {}: {}", path.display(), e);
            return;
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => { log::warn!("Skipping Microsoft JSON {}: {}", path.display(), e); return; }
    };

    let doc_type = if rel_path.contains("search") {
        "search_query"
    } else if rel_path.contains("browse") {
        "browse_history"
    } else if rel_path.contains("cortana") {
        "voice_command"
    } else if rel_path.contains("outlook") || rel_path.contains("mail") {
        "email"
    } else {
        "microsoft_data"
    };

    let items = if let Some(arr) = value.as_array() {
        arr.clone()
    } else {
        vec![value]
    };

    for item in &items {
        // Try to extract structured search/browse data
        let text = if doc_type == "search_query" {
            item.get("SearchQuery")
                .or_else(|| item.get("searchQuery"))
                .or_else(|| item.get("Text"))
                .and_then(|v| v.as_str())
                .map(|s| format!("Searched: {}", s))
                .unwrap_or_else(|| parse_utils::flatten_json_to_text(item))
        } else if doc_type == "browse_history" {
            let title = item.get("PageTitle")
                .or_else(|| item.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let url = item.get("NavigatedToUrl")
                .or_else(|| item.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if title.is_empty() && url.is_empty() {
                parse_utils::flatten_json_to_text(item)
            } else if title.is_empty() {
                format!("Visited: {}", url)
            } else {
                format!("Visited: {} ({})", title, url)
            }
        } else {
            parse_utils::flatten_json_to_text(item)
        };

        if text.trim().is_empty() {
            log::debug!("Skipping empty content in Microsoft JSON");
            continue;
        }

        let timestamp = item.get("DateTime")
            .or_else(|| item.get("dateTime"))
            .or_else(|| item.get("Timestamp"))
            .or_else(|| item.get("timestamp"))
            .and_then(|v| v.as_str())
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| {
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                            .ok()
                            .map(|dt| dt.and_utc())
                    })
            })
            ;

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String(doc_type.into()));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Microsoft,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_ms_csv(path: &Path, rel_path: &str, docs: &mut Vec<Document>) {
    let rows = match parse_utils::parse_csv_file(path) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Skipping Microsoft CSV {}: {}", path.display(), e);
            return;
        }
    };

    let file_name = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let doc_type = if rel_path.contains("search") {
        "search_query"
    } else if rel_path.contains("browse") {
        "browse_history"
    } else {
        "microsoft_data"
    };

    for row in rows {
        let text_parts: Vec<String> = row
            .iter()
            .filter(|(_, v)| !v.trim().is_empty())
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();

        let text = text_parts.join("; ");
        if text.trim().is_empty() {
            log::debug!("Skipping empty content in Microsoft CSV");
            continue;
        }

        let timestamp = row.get("DateTime")
            .or_else(|| row.get("Date"))
            .or_else(|| row.get("Timestamp"))
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
        meta.insert("type".into(), serde_json::Value::String(doc_type.into()));
        meta.insert("source_file".into(), serde_json::Value::String(file_name.clone()));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Microsoft,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_microsoft() {
        let adapter = MicrosoftAdapter;
        let files = vec!["SearchRequestsAndQuery.json", "BrowseHistory.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_ms_path() {
        let adapter = MicrosoftAdapter;
        let files = vec!["Microsoft/Outlook/emails.json"];
        assert!(adapter.detect(&files) >= 0.7);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = MicrosoftAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }
}
