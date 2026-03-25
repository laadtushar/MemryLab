use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct TumblrAdapter;

impl SourceAdapter for TumblrAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "tumblr".into(),
            display_name: "Tumblr".into(),
            icon: "tumblr".into(),
            takeout_url: Some("https://www.tumblr.com/settings/blog/".into()),
            instructions: "Export your blog from Tumblr (Settings > your blog > Export). You will receive a ZIP with HTML post files and media.".into(),
            accepted_extensions: vec!["zip".into(), "html".into()],
            handles_zip: true,
            platform: SourcePlatform::Tumblr,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_posts = file_listing.iter().any(|f| {
            let normalized = f.replace('\\', "/").to_lowercase();
            (normalized.contains("posts/") || normalized.contains("post/"))
                && (normalized.ends_with(".html") || normalized.ends_with(".json"))
        });
        let has_tumblr_marker = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("tumblr") || lower.contains("avatar") || lower.contains("theme")
        });
        if has_posts && has_tumblr_marker {
            0.9
        } else if has_posts {
            0.4
        } else if has_tumblr_marker {
            0.3
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "tumblr"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        // Parse HTML post files
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "html"))
        {
            let html = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Skipping {}: {}", entry.path().display(), e);
                    continue;
                }
            };

            let text = parse_utils::html_to_text(&html);
            if text.trim().is_empty() || text.len() < 10 {
                continue;
            }

            let file_stem = entry.path().file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string();

            let timestamp = extract_tumblr_date(&html)
                .unwrap_or_else(Utc::now);

            let post_type = detect_tumblr_post_type(&html);

            let mut meta = serde_json::Map::new();
            meta.insert("type".into(), serde_json::Value::String(post_type));
            meta.insert("source_file".into(), serde_json::Value::String(file_stem));

            documents.push(parse_utils::build_document(
                text,
                SourcePlatform::Tumblr,
                timestamp,
                vec![],
                serde_json::Value::Object(meta),
            ));
        }

        // Parse JSON post data if available
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let p = e.path().to_string_lossy().replace('\\', "/").to_lowercase();
                e.path().extension().is_some_and(|ext| ext == "json")
                    && (p.contains("posts") || p.contains("post"))
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

            parse_tumblr_json_posts(&value, &mut documents);
        }

        Ok(documents)
    }
}

fn extract_tumblr_date(html: &str) -> Option<chrono::DateTime<Utc>> {
    // Look for datetime attribute or date-like patterns
    let re = regex::Regex::new(r#"datetime="(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2})"#).ok()?;
    if let Some(caps) = re.captures(html) {
        let s = &caps[1];
        return chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .ok()
            .map(|dt| dt.and_utc());
    }
    // Fallback: look for date in tumblr post headers
    let re_date = regex::Regex::new(r"(\d{4}-\d{2}-\d{2})").ok()?;
    re_date.captures(html).and_then(|caps| {
        chrono::NaiveDate::parse_from_str(&caps[1], "%Y-%m-%d")
            .ok()
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
    })
}

fn detect_tumblr_post_type(html: &str) -> String {
    let lower = html.to_lowercase();
    if lower.contains("<blockquote") || lower.contains("class=\"quote") {
        "quote".into()
    } else if lower.contains("<img") || lower.contains("class=\"photo") {
        "photo".into()
    } else if lower.contains("<a") && lower.contains("class=\"link") {
        "link".into()
    } else if lower.contains("<audio") || lower.contains("class=\"audio") {
        "audio".into()
    } else if lower.contains("<video") || lower.contains("class=\"video") {
        "video".into()
    } else {
        "text".into()
    }
}

fn parse_tumblr_json_posts(value: &serde_json::Value, docs: &mut Vec<Document>) {
    let posts = if let Some(arr) = value.as_array() {
        arr.clone()
    } else if let Some(arr) = value.get("posts").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        return;
    };

    for post in &posts {
        let body = post.get("body")
            .or_else(|| post.get("caption"))
            .or_else(|| post.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let text = if body.contains('<') {
            parse_utils::html_to_text(body)
        } else {
            body.to_string()
        };

        if text.trim().is_empty() {
            continue;
        }

        let timestamp = post.get("timestamp")
            .and_then(|v| v.as_i64())
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
            .or_else(|| {
                post.get("date")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
            })
            .unwrap_or_else(Utc::now);

        let post_type = post.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("text")
            .to_string();

        let blog_name = post.get("blog_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String(post_type));
        if let Some(slug) = post.get("slug").and_then(|v| v.as_str()) {
            meta.insert("slug".into(), serde_json::Value::String(slug.into()));
        }
        if let Some(tags) = post.get("tags").and_then(|v| v.as_array()) {
            let tag_strs: Vec<String> = tags.iter()
                .filter_map(|t| t.as_str().map(String::from))
                .collect();
            meta.insert("tags".into(), serde_json::json!(tag_strs));
        }

        let participants = if blog_name.is_empty() { vec![] } else { vec![blog_name] };

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Tumblr,
            timestamp,
            participants,
            serde_json::Value::Object(meta),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_tumblr() {
        let adapter = TumblrAdapter;
        let files = vec!["posts/my-post.html", "avatar.png", "tumblr_theme.html"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = TumblrAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_detect_post_type() {
        assert_eq!(detect_tumblr_post_type("<blockquote>test</blockquote>"), "quote");
        assert_eq!(detect_tumblr_post_type("<p>just text</p>"), "text");
        assert_eq!(detect_tumblr_post_type("<img src='photo.jpg'>"), "photo");
    }

    #[test]
    fn test_parse_tumblr_json() {
        let value = serde_json::json!({
            "posts": [
                {
                    "body": "<p>Hello from tumblr!</p>",
                    "timestamp": 1710504600,
                    "type": "text",
                    "blog_name": "myblog",
                    "tags": ["life", "thoughts"]
                }
            ]
        });
        let mut docs = Vec::new();
        parse_tumblr_json_posts(&value, &mut docs);
        assert_eq!(docs.len(), 1);
        assert!(docs[0].raw_text.contains("Hello from tumblr"));
    }
}
