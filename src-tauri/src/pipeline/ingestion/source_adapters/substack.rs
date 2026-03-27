use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct SubstackAdapter;

impl SourceAdapter for SubstackAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "substack".into(),
            display_name: "Substack".into(),
            icon: "book-open".into(),
            takeout_url: Some("https://substack.com/settings".into()),
            instructions: "Export your Substack data from Settings > Export. You will receive a ZIP containing posts.csv and your post HTML files.".into(),
            accepted_extensions: vec!["zip".into(), "csv".into()],
            handles_zip: true,
            platform: SourcePlatform::Substack,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_posts_csv = file_listing.iter().any(|f| f.ends_with("posts.csv"));
        let has_substack_marker = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("substack") || lower.contains("subscribers")
        });
        if has_posts_csv && has_substack_marker {
            0.9
        } else if has_posts_csv {
            0.4
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "substack"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Find posts.csv
        let csv_files: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let fname = e.path().file_name().and_then(|s| s.to_str()).unwrap_or("");
                    fname == "posts.csv"
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        } else if path.extension().is_some_and(|ext| ext == "csv") {
            vec![path.to_path_buf()]
        } else {
            vec![]
        };

        for csv_path in &csv_files {
            let rows = match parse_utils::parse_csv_file(csv_path) {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("Skipping {}: {}", csv_path.display(), e);
                    continue;
                }
            };

            for row in rows {
                let title = row.get("title").cloned().unwrap_or_default();
                let subtitle = row.get("subtitle").cloned().unwrap_or_default();
                let body_html = row.get("body_html").or_else(|| row.get("body")).cloned().unwrap_or_default();

                let body_text = if body_html.contains('<') {
                    parse_utils::html_to_text(&body_html)
                } else {
                    body_html
                };

                let text = if subtitle.is_empty() {
                    if body_text.is_empty() { title.clone() } else { format!("{}\n\n{}", title, body_text) }
                } else if body_text.is_empty() {
                    format!("{}\n{}", title, subtitle)
                } else {
                    format!("{}\n{}\n\n{}", title, subtitle, body_text)
                };

                if text.trim().is_empty() {
                    continue;
                }

                let timestamp = row
                    .get("post_date")
                    .or_else(|| row.get("date"))
                    .or_else(|| row.get("publish_date"))
                    .and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                            .or_else(|| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
                                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()))
                            .or_else(|| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").ok()
                                .map(|dt| dt.and_utc()))
                    })
                    ;

                let is_published = row.get("is_published").or_else(|| row.get("status"))
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                let mut meta = serde_json::Map::new();
                meta.insert("type".into(), serde_json::Value::String("newsletter_post".into()));
                if !title.is_empty() {
                    meta.insert("title".into(), serde_json::Value::String(title));
                }
                if !is_published.is_empty() {
                    meta.insert("status".into(), serde_json::Value::String(is_published));
                }
                if let Some(url) = row.get("post_url").or_else(|| row.get("url")) {
                    meta.insert("url".into(), serde_json::Value::String(url.clone()));
                }
                if let Some(slug) = row.get("slug") {
                    meta.insert("slug".into(), serde_json::Value::String(slug.clone()));
                }

                documents.push(parse_utils::build_document(
                    text,
                    SourcePlatform::Substack,
                    timestamp,
                    vec![],
                    serde_json::Value::Object(meta),
                ));
            }
        }

        // Also parse standalone HTML post files if present
        if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "html"))
            {
                let html = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let text = parse_utils::html_to_text(&html);
                if text.trim().is_empty() || text.len() < 20 {
                    continue;
                }

                let file_name = entry.path().file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("untitled")
                    .to_string();

                let mut meta = serde_json::Map::new();
                meta.insert("type".into(), serde_json::Value::String("newsletter_post".into()));
                meta.insert("source_file".into(), serde_json::Value::String(file_name));

                documents.push(parse_utils::build_document(
                    text,
                    SourcePlatform::Substack,
                    None,
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
    fn test_detect_substack() {
        let adapter = SubstackAdapter;
        let files = vec!["posts.csv", "subscribers.csv"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_ambiguous() {
        let adapter = SubstackAdapter;
        assert!(adapter.detect(&["posts.csv"]) >= 0.3);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = SubstackAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = SubstackAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "substack");
        assert_eq!(meta.platform, SourcePlatform::Substack);
    }
}
