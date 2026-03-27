use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct RedditAdapter;

impl SourceAdapter for RedditAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "reddit".into(),
            display_name: "Reddit".into(),
            icon: "message-square".into(),
            takeout_url: Some("https://www.reddit.com/settings/data-request".into()),
            instructions: "Request your data from Reddit Settings > Privacy & Security. Upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "csv".into()],
            handles_zip: true,
            platform: SourcePlatform::Reddit,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_comments = file_listing.iter().any(|f| f.ends_with("comments.csv"));
        let has_posts = file_listing.iter().any(|f| f.ends_with("posts.csv"));
        if has_comments && has_posts {
            0.9
        } else if has_comments || has_posts {
            0.5
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "reddit"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        let csv_files: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "csv"))
                .map(|e| e.path().to_path_buf())
                .collect()
        } else if path.extension().is_some_and(|ext| ext == "csv") {
            vec![path.to_path_buf()]
        } else {
            vec![]
        };

        for csv_path in csv_files {
            let file_name = csv_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            let rows = match parse_utils::parse_csv_file(&csv_path) {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("Skipping {}: {}", csv_path.display(), e);
                    continue;
                }
            };

            for row in rows {
                let text = if file_name.contains("comments") {
                    row.get("body").cloned().unwrap_or_default()
                } else if file_name.contains("posts") {
                    let title = row.get("title").cloned().unwrap_or_default();
                    let selftext = row.get("selftext").or_else(|| row.get("body")).cloned().unwrap_or_default();
                    if selftext.is_empty() { title } else { format!("{}\n\n{}", title, selftext) }
                } else {
                    // Generic CSV — join all values
                    row.values().cloned().collect::<Vec<_>>().join(" ")
                };

                if text.trim().is_empty() {
                    continue;
                }

                let subreddit = row.get("subreddit").cloned().unwrap_or_default();

                let timestamp = row
                    .get("date")
                    .or_else(|| row.get("timestamp"))
                    .or_else(|| row.get("created_utc"))
                    .and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                            .or_else(|| {
                                s.parse::<i64>()
                                    .ok()
                                    .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                            })
                    })
                    ;

                let mut meta = serde_json::Map::new();
                if !subreddit.is_empty() {
                    meta.insert("subreddit".into(), serde_json::Value::String(subreddit));
                }
                meta.insert(
                    "type".into(),
                    serde_json::Value::String(
                        if file_name.contains("comments") { "comment" } else { "post" }.into(),
                    ),
                );
                if let Some(id) = row.get("id") {
                    meta.insert("reddit_id".into(), serde_json::Value::String(id.clone()));
                }

                documents.push(parse_utils::build_document(
                    text,
                    SourcePlatform::Reddit,
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
    fn test_detect_reddit_both() {
        let adapter = RedditAdapter;
        let files = vec!["comments.csv", "posts.csv", "statistics.csv"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_reddit_partial() {
        let adapter = RedditAdapter;
        let files = vec!["comments.csv"];
        let score = adapter.detect(&files);
        assert!(score >= 0.4 && score <= 0.6);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = RedditAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = RedditAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "reddit");
        assert!(meta.takeout_url.is_some());
    }
}
