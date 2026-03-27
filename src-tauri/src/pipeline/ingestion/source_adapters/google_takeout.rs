use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::parse_utils;
use super::{SourceAdapter, SourceAdapterMeta};

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
        // Strong signals
        let has_takeout = file_listing
            .iter()
            .any(|f| f.starts_with("Takeout/") || f.contains("/Takeout/"));
        let has_archive_browser = file_listing
            .iter()
            .any(|f| f.contains("archive_browser.html"));
        if has_takeout || has_archive_browser {
            return 0.95;
        }

        // Medium signals: Google-specific folder names at root level
        let google_dirs = [
            "Keep/",
            "Chrome/",
            "YouTube and YouTube Music/",
            "YouTube/",
            "My Activity/",
            "Google Photos/",
            "Gmail/",
            "Drive/",
            "Google Fit/",
            "Maps/",
            "Location History/",
            "Calendar/",
            "Contacts/",
            "Google Play Store/",
            "Hangouts/",
            "Google Chat/",
            "Google Pay/",
            "Google Shopping/",
        ];
        let google_matches = google_dirs
            .iter()
            .filter(|dir| {
                file_listing
                    .iter()
                    .any(|f| f.starts_with(*dir) || f.contains(&format!("/{}", dir)))
            })
            .count();

        if google_matches >= 3 {
            return 0.9;
        }
        if google_matches >= 1 {
            return 0.6;
        }

        0.0
    }

    fn name(&self) -> &str {
        "google_takeout"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Find the Takeout root — could be path itself, or path/Takeout
        let takeout_root = if path.join("Takeout").is_dir() {
            path.join("Takeout")
        } else {
            path.to_path_buf()
        };

        // Google Keep notes (JSON with textContent)
        parse_keep_notes(&takeout_root, &mut documents);

        // Chrome browser history
        parse_chrome_history(&takeout_root, &mut documents);

        // YouTube watch/search history
        parse_youtube_history(&takeout_root, &mut documents);

        // My Activity — Google's activity logs across all services
        parse_my_activity(&takeout_root, &mut documents);

        // Gmail (MBOX format — extract subject lines and snippets)
        parse_gmail(&takeout_root, &mut documents);

        // Walk ALL remaining JSON/HTML/TXT/CSV/MD files for anything we missed
        parse_remaining_files(&takeout_root, &mut documents);

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

        let title = value
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let text_content = value
            .get("textContent")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Also check for list items
        let list_text = value
            .get("listContent")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n- ")
            })
            .unwrap_or_default();

        let full_text = [title, text_content, &list_text]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join("\n\n");

        if full_text.trim().is_empty() {
            continue;
        }

        let timestamp = value
            .get("userEditedTimestampUsec")
            .and_then(|v| v.as_i64())
            .and_then(|us| chrono::DateTime::from_timestamp(us / 1_000_000, 0))
            .or_else(|| {
                value
                    .get("createdTimestampUsec")
                    .and_then(|v| v.as_i64())
                    .and_then(|us| chrono::DateTime::from_timestamp(us / 1_000_000, 0))
            })
            ;

        let mut meta = serde_json::Map::new();
        meta.insert(
            "service".into(),
            serde_json::Value::String("Google Keep".into()),
        );
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

    let items = match value
        .get("Browser History")
        .and_then(|v| v.as_array())
    {
        Some(arr) => arr,
        None => return,
    };

    for item in items {
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");

        if title.is_empty() && url.is_empty() {
            continue;
        }

        let text = format!("{}\n{}", title, url);

        let timestamp = item
            .get("time_usec")
            .and_then(|v| v.as_i64())
            .and_then(|us| chrono::DateTime::from_timestamp(us / 1_000_000, 0))
            ;

        let mut meta = serde_json::Map::new();
        meta.insert(
            "service".into(),
            serde_json::Value::String("Chrome".into()),
        );
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
                let title = item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if title.trim().is_empty() {
                    continue;
                }

                let timestamp = item
                    .get("time")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    ;

                let mut meta = serde_json::Map::new();
                meta.insert(
                    "service".into(),
                    serde_json::Value::String("YouTube".into()),
                );
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

fn parse_my_activity(root: &Path, docs: &mut Vec<Document>) {
    let activity_dir = root.join("My Activity");
    if !activity_dir.is_dir() {
        return;
    }

    // My Activity contains HTML and JSON files per service
    for entry in WalkDir::new(&activity_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "json" => {
                let content = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
                };
                let value: serde_json::Value = match serde_json::from_str(&content) {
                    Ok(v) => v,
                    Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
                };

                // Activity JSON is typically an array of activity items
                let items = if let Some(arr) = value.as_array() {
                    arr.clone()
                } else {
                    vec![value]
                };

                for item in &items {
                    let title = item
                        .get("title")
                        .and_then(|v| v.as_str())
                        .or_else(|| item.get("header").and_then(|v| v.as_str()))
                        .unwrap_or("");

                    if title.trim().is_empty() || title.len() < 5 {
                        continue;
                    }

                    let timestamp = item
                        .get("time")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        ;

                    // Get the service name from the parent folder
                    let service = entry
                        .path()
                        .parent()
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Activity".into());

                    let mut meta = serde_json::Map::new();
                    meta.insert(
                        "service".into(),
                        serde_json::Value::String(service),
                    );

                    docs.push(parse_utils::build_document(
                        title.to_string(),
                        SourcePlatform::GoogleTakeout,
                        timestamp,
                        vec![],
                        serde_json::Value::Object(meta),
                    ));
                }
            }
            "html" => {
                let content = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
                };
                let text = parse_utils::html_to_text(&content);
                if text.len() > 30 {
                    let service = entry
                        .path()
                        .parent()
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Activity".into());

                    let mut meta = serde_json::Map::new();
                    meta.insert(
                        "service".into(),
                        serde_json::Value::String(service),
                    );
                    meta.insert(
                        "source_file".into(),
                        serde_json::Value::String(
                            entry.path().display().to_string(),
                        ),
                    );

                    docs.push(parse_utils::build_document(
                        text,
                        SourcePlatform::GoogleTakeout,
                        None, // time-agnostic when no date available
                        vec![],
                        serde_json::Value::Object(meta),
                    ));
                }
            }
            _ => {}
        }
    }
}

fn parse_gmail(root: &Path, docs: &mut Vec<Document>) {
    // Gmail exports as MBOX. Extract subject lines and first few lines of each message.
    let mail_dir = root.join("Mail");
    let gmail_dir = root.join("Gmail");
    let mbox_dir = if mail_dir.is_dir() {
        mail_dir
    } else if gmail_dir.is_dir() {
        gmail_dir
    } else {
        return;
    };

    for entry in WalkDir::new(&mbox_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "mbox" || ext == "eml")
        })
    {
        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Simple MBOX parser: split by "From " at line start, extract Subject and body preview
        for message in content.split("\nFrom ") {
            let subject = message
                .lines()
                .find(|l| l.starts_with("Subject:"))
                .map(|l| l.strip_prefix("Subject:").unwrap_or(l).trim().to_string())
                .unwrap_or_default();

            if subject.is_empty() {
                continue;
            }

            let from = message
                .lines()
                .find(|l| l.starts_with("From:"))
                .map(|l| l.strip_prefix("From:").unwrap_or(l).trim().to_string())
                .unwrap_or_default();

            // Get the first non-header content (after blank line)
            let body_preview = message
                .split("\n\n")
                .nth(1)
                .map(|b| {
                    b.lines()
                        .take(5)
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();

            let text = if body_preview.is_empty() {
                subject.clone()
            } else {
                format!("{}\n\n{}", subject, body_preview)
            };

            let mut meta = serde_json::Map::new();
            meta.insert(
                "service".into(),
                serde_json::Value::String("Gmail".into()),
            );
            meta.insert(
                "subject".into(),
                serde_json::Value::String(subject),
            );
            if !from.is_empty() {
                meta.insert(
                    "from".into(),
                    serde_json::Value::String(from.clone()),
                );
            }

            let participants = if from.is_empty() {
                vec![]
            } else {
                vec![from]
            };

            docs.push(parse_utils::build_document(
                text,
                SourcePlatform::GoogleTakeout,
                None, // time-agnostic
                participants,
                serde_json::Value::Object(meta),
            ));
        }
    }
}

/// Walk all remaining text files not already handled by specific parsers
fn parse_remaining_files(root: &Path, docs: &mut Vec<Document>) {
    let handled_dirs = [
        "Keep", "Chrome", "YouTube", "My Activity", "Mail", "Gmail",
    ];

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let rel_path = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");

        // Skip directories already handled
        if handled_dirs
            .iter()
            .any(|d| rel_path.starts_with(d) || rel_path.contains(&format!("/{}/", d)))
        {
            continue;
        }

        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let text = match ext.as_str() {
            "json" => {
                let content = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
                };
                let value: serde_json::Value = match serde_json::from_str(&content) {
                    Ok(v) => v,
                    Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
                };
                parse_utils::flatten_json_to_text(&value)
            }
            "html" | "htm" => {
                let content = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
                };
                parse_utils::html_to_text(&content)
            }
            "csv" => {
                let rows = match parse_utils::parse_csv_file(entry.path()) {
                    Ok(r) => r,
                    Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
                };
                rows.iter()
                    .map(|row| {
                        row.values()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" | ")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            "txt" | "md" | "text" => match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
            },
            "mbox" | "eml" | "vcf" | "ics" | "xml" | "yaml" | "yml"
            | "tsv" | "log" | "ndjson" | "jsonl" => {
                match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(e) => { log::warn!("Skipping {}: {}", entry.path().display(), e); continue; }
                }
            }
            _ => {
                // Try reading any remaining file as UTF-8 text
                match std::fs::read_to_string(entry.path()) {
                    Ok(c) if c.len() > 20 => c,
                    _ => continue,
                }
            }
        };

        if text.len() < 20 {
            continue;
        }

        // Determine service from folder name
        let service = entry
            .path()
            .parent()
            .and_then(|p| p.strip_prefix(root).ok())
            .and_then(|p| p.components().next())
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .unwrap_or_else(|| "Other".into());

        let mut meta = serde_json::Map::new();
        meta.insert(
            "service".into(),
            serde_json::Value::String(service),
        );
        meta.insert(
            "source_file".into(),
            serde_json::Value::String(rel_path),
        );

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::GoogleTakeout,
            None, // time-agnostic
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_google_takeout() {
        let adapter = GoogleTakeoutAdapter;
        let files = vec![
            "Takeout/Keep/note1.json",
            "Takeout/Chrome/BrowserHistory.json",
        ];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_inside_takeout_folder() {
        let adapter = GoogleTakeoutAdapter;
        // User opened the Takeout folder itself
        let files = vec![
            "Keep/note1.json",
            "Chrome/BrowserHistory.json",
            "My Activity/Search/MyActivity.json",
        ];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_single_service() {
        let adapter = GoogleTakeoutAdapter;
        let files = vec!["My Activity/Search/MyActivity.json"];
        assert!(adapter.detect(&files) >= 0.5);
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
