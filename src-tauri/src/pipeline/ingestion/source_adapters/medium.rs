use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct MediumAdapter;

impl SourceAdapter for MediumAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "medium".into(),
            display_name: "Medium".into(),
            icon: "edit-3".into(),
            takeout_url: Some("https://medium.com/me/settings/security".into()),
            instructions: "Download your Medium data from Settings > Security and apps > Download your information. You will receive a ZIP with HTML posts.".into(),
            accepted_extensions: vec!["zip".into(), "html".into()],
            handles_zip: true,
            platform: SourcePlatform::Medium,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_posts_dir = file_listing.iter().any(|f| {
            let normalized = f.replace('\\', "/");
            normalized.contains("posts/") && normalized.ends_with(".html")
        });
        let has_profile = file_listing.iter().any(|f| {
            let normalized = f.replace('\\', "/");
            normalized.contains("profile/profile.json") || normalized.contains("profile.json")
        });
        if has_posts_dir && has_profile {
            0.95
        } else if has_posts_dir {
            0.8
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "medium"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Parse profile.json for author info
        let mut author_name = String::new();
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let fname = e.path().file_name().and_then(|s| s.to_str()).unwrap_or("");
                fname == "profile.json"
            })
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    author_name = val.get("name")
                        .or_else(|| val.get("displayName"))
                        .or_else(|| val.get("username"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                }
            }
        }

        // Parse HTML post files
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let p = e.path().to_string_lossy().replace('\\', "/");
                e.path().extension().is_some_and(|ext| ext == "html")
                    && p.contains("posts/")
            })
        {
            let html = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Skipping {}: {}", entry.path().display(), e);
                    continue;
                }
            };

            let text = parse_utils::html_to_text(&html);
            if text.trim().is_empty() || text.len() < 20 {
                continue;
            }

            // Extract title from filename (Medium uses format: draft_xxxx_title-slug.html or yyyy-mm-dd_title-slug_hash.html)
            let file_stem = entry.path().file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled");

            // Try to extract date from filename pattern: yyyy-mm-dd_...
            let timestamp = extract_date_from_filename(file_stem)
                .or_else(|| extract_date_from_html(&html))
                ;

            // Clean up the title from the filename slug
            let title = file_stem
                .split('_')
                .skip(1) // skip date or "draft" prefix
                .next()
                .unwrap_or(file_stem)
                .replace('-', " ");

            let mut meta = serde_json::Map::new();
            meta.insert("type".into(), serde_json::Value::String("article".into()));
            if !title.is_empty() {
                meta.insert("title".into(), serde_json::Value::String(title));
            }
            meta.insert("source_file".into(), serde_json::Value::String(
                entry.path().file_name().and_then(|s| s.to_str()).unwrap_or("").to_string()
            ));
            if file_stem.starts_with("draft") {
                meta.insert("status".into(), serde_json::Value::String("draft".into()));
            }

            let participants = if author_name.is_empty() { vec![] } else { vec![author_name.clone()] };

            documents.push(parse_utils::build_document(
                text,
                SourcePlatform::Medium,
                timestamp,
                participants,
                serde_json::Value::Object(meta),
            ));
        }

        Ok(documents)
    }
}

fn extract_date_from_filename(stem: &str) -> Option<chrono::DateTime<Utc>> {
    // Medium filenames often start with yyyy-mm-dd
    if stem.len() >= 10 {
        let date_part = &stem[..10];
        chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
            .ok()
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
    } else {
        None
    }
}

fn extract_date_from_html(html: &str) -> Option<chrono::DateTime<Utc>> {
    // Try to find a datetime attribute in the HTML
    let re = regex::Regex::new(r#"datetime="(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2})"#).ok()?;
    re.captures(html).and_then(|caps| {
        let s = &caps[1];
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .ok()
            .map(|dt| dt.and_utc())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_medium() {
        let adapter = MediumAdapter;
        let files = vec!["posts/2024-01-15_My-Article_abc123.html", "profile/profile.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_posts_only() {
        let adapter = MediumAdapter;
        let files = vec!["posts/draft_my-post.html"];
        assert!(adapter.detect(&files) >= 0.7);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = MediumAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_extract_date_from_filename() {
        let dt = extract_date_from_filename("2024-03-15_my-article_abc123");
        assert!(dt.is_some());
        let dt = dt.unwrap();
        assert_eq!(dt.date_naive().to_string(), "2024-03-15");
    }

    #[test]
    fn test_extract_date_no_match() {
        assert!(extract_date_from_filename("draft_my-post").is_none());
    }
}
