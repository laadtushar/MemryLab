use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct TwitterAdapter;

impl SourceAdapter for TwitterAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "twitter".into(),
            display_name: "Twitter / X".into(),
            icon: "twitter".into(),
            takeout_url: Some("https://x.com/settings/download_your_data".into()),
            instructions: "Upload your Twitter/X data archive (ZIP). Request it from Settings > Your Account.".into(),
            accepted_extensions: vec!["zip".into()],
            handles_zip: true,
            platform: SourcePlatform::Twitter,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_tweets_js = file_listing.iter().any(|f| {
            f.contains("data/tweets.js") || f.contains("data/tweet.js")
        });
        if has_tweets_js { 0.95 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "twitter"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        let data_dir = if path.join("data").is_dir() {
            path.join("data")
        } else {
            path.to_path_buf()
        };

        for entry in WalkDir::new(&data_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "js"))
        {
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Skipping {}: {}", entry.path().display(), e);
                    continue;
                }
            };

            let value = match parse_utils::unwrap_twitter_js(&content) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Skipping {}: {}", entry.path().display(), e);
                    continue;
                }
            };

            if let Some(arr) = value.as_array() {
                for item in arr {
                    let tweet = item.get("tweet").unwrap_or(item);

                    let full_text = tweet
                        .get("full_text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if full_text.trim().is_empty() {
                        continue;
                    }

                    let timestamp = tweet
                        .get("created_at")
                        .and_then(|v| v.as_str())
                        .and_then(|s| {
                            chrono::DateTime::parse_from_str(s, "%a %b %d %H:%M:%S %z %Y").ok()
                        })
                        .map(|dt| dt.with_timezone(&Utc))
                        ;

                    let mut meta = serde_json::Map::new();
                    if let Some(id) = tweet.get("id_str").and_then(|v| v.as_str()) {
                        meta.insert("tweet_id".into(), serde_json::Value::String(id.into()));
                    }
                    if let Some(rt) = tweet.get("retweet_count") {
                        meta.insert("retweet_count".into(), rt.clone());
                    }
                    if let Some(fav) = tweet.get("favorite_count") {
                        meta.insert("favorite_count".into(), fav.clone());
                    }
                    meta.insert(
                        "source_file".into(),
                        serde_json::Value::String(entry.path().display().to_string()),
                    );

                    documents.push(parse_utils::build_document(
                        full_text.to_string(),
                        SourcePlatform::Twitter,
                        timestamp,
                        vec![],
                        serde_json::Value::Object(meta),
                    ));
                }
            }
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_twitter() {
        let adapter = TwitterAdapter;
        let files = vec!["data/tweets.js", "data/profile.js", "data/account.js"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = TwitterAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = TwitterAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "twitter");
        assert!(meta.takeout_url.is_some());
        assert!(meta.handles_zip);
    }
}
