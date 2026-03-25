use std::path::Path;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use regex::Regex;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};

pub struct ObsidianAdapter;

impl SourceAdapter for ObsidianAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "obsidian".into(),
            display_name: "Obsidian".into(),
            icon: "vault".into(),
            takeout_url: None,
            instructions: "Select your Obsidian vault folder. Markdown files with frontmatter, tags, and wikilinks are parsed.".into(),
            accepted_extensions: vec!["md".into()],
            handles_zip: false,
            platform: SourcePlatform::Obsidian,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_obsidian_config = file_listing.iter().any(|f| f.contains(".obsidian/"));
        let md_count = file_listing.iter().filter(|f| f.ends_with(".md")).count();
        if has_obsidian_config { 0.95 } else if md_count > 5 { 0.3 } else { 0.0 }
    }

    fn name(&self) -> &str { "obsidian" }

    fn parse(&self, vault_path: &Path) -> Result<Vec<Document>, AppError> {
        if !vault_path.is_dir() {
            return Err(AppError::Import(format!(
                "Obsidian vault path is not a directory: {}",
                vault_path.display()
            )));
        }

        let mut documents = Vec::new();

        for entry in WalkDir::new(vault_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "md")
                    && !e
                        .path()
                        .components()
                        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
            })
        {
            match parse_obsidian_file(entry.path()) {
                Ok(doc) => documents.push(doc),
                Err(e) => {
                    log::warn!("Skipping file {}: {}", entry.path().display(), e);
                }
            }
        }

        Ok(documents)
    }
}

fn parse_obsidian_file(path: &Path) -> Result<Document, AppError> {
    let content = std::fs::read_to_string(path)?;

    // Parse YAML frontmatter
    let (frontmatter, body) = split_frontmatter(&content);
    let timestamp = extract_timestamp(&frontmatter, path)?;

    // Extract tags from body
    let tags = extract_tags(&body);
    // Strip wikilinks for clean text
    let clean_text = strip_wikilinks(&body);

    let mut hasher = Sha256::new();
    hasher.update(clean_text.as_bytes());
    let content_hash = format!("{:x}", hasher.finalize());

    let mut metadata = serde_json::Map::new();
    if !tags.is_empty() {
        metadata.insert(
            "tags".to_string(),
            serde_json::Value::Array(tags.into_iter().map(serde_json::Value::String).collect()),
        );
    }
    if let Some(title) = path.file_stem().and_then(|s| s.to_str()) {
        metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
    }
    metadata.insert(
        "source_path".to_string(),
        serde_json::Value::String(path.display().to_string()),
    );

    Ok(Document {
        id: Uuid::new_v4().to_string(),
        source_platform: SourcePlatform::Obsidian,
        raw_text: clean_text,
        timestamp,
        participants: vec![],
        metadata: serde_json::Value::Object(metadata),
        content_hash,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    })
}

/// Split content into optional YAML frontmatter and body.
fn split_frontmatter(content: &str) -> (Option<String>, String) {
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let fm = content[3..3 + end].trim().to_string();
            let body = content[3 + end + 3..].trim().to_string();
            return (Some(fm), body);
        }
    }
    (None, content.to_string())
}

/// Try to extract a timestamp from frontmatter, falling back to file metadata.
fn extract_timestamp(frontmatter: &Option<String>, path: &Path) -> Result<DateTime<Utc>, AppError> {
    if let Some(fm) = frontmatter {
        // Try common frontmatter date fields
        if let Some(date) = extract_date_from_yaml(fm, "date")
            .or_else(|| extract_date_from_yaml(fm, "created"))
            .or_else(|| extract_date_from_yaml(fm, "created_at"))
            .or_else(|| extract_date_from_yaml(fm, "publish_date"))
        {
            return Ok(date);
        }
    }

    // Try to extract date from filename (common pattern: YYYY-MM-DD-title.md)
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if let Some(date) = parse_date_prefix(stem) {
            return Ok(date);
        }
    }

    // Fall back to file modification time
    let metadata = std::fs::metadata(path)?;
    let modified = metadata
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    Ok(DateTime::<Utc>::from(modified))
}

/// Extract a date value from a YAML key.
fn extract_date_from_yaml(yaml: &str, key: &str) -> Option<DateTime<Utc>> {
    let pattern = format!(r"(?m)^{}:\s*(.+)$", regex::escape(key));
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(yaml)?;
    let value = caps.get(1)?.as_str().trim().trim_matches('"').trim_matches('\'');
    parse_flexible_date(value)
}

/// Parse dates from filename prefix like "2024-01-15" or "20240115".
fn parse_date_prefix(filename: &str) -> Option<DateTime<Utc>> {
    // Try YYYY-MM-DD prefix
    if filename.len() >= 10 {
        if let Ok(date) = NaiveDate::parse_from_str(&filename[..10], "%Y-%m-%d") {
            return Some(date.and_hms_opt(12, 0, 0)?.and_utc());
        }
    }
    // Try YYYYMMDD prefix
    if filename.len() >= 8 {
        if let Ok(date) = NaiveDate::parse_from_str(&filename[..8], "%Y%m%d") {
            return Some(date.and_hms_opt(12, 0, 0)?.and_utc());
        }
    }
    None
}

/// Parse a date string in various common formats.
fn parse_flexible_date(s: &str) -> Option<DateTime<Utc>> {
    // ISO 8601 full
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // Common datetime formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
    ];
    for fmt in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt.and_utc());
        }
    }
    // Date only
    let date_formats = ["%Y-%m-%d", "%Y/%m/%d", "%d/%m/%Y", "%m/%d/%Y"];
    for fmt in &date_formats {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d.and_hms_opt(12, 0, 0)?.and_utc());
        }
    }
    None
}

/// Extract #tags from markdown body.
fn extract_tags(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?:^|\s)#([a-zA-Z][a-zA-Z0-9_/-]*)").unwrap();
    re.captures_iter(text)
        .map(|c| c[1].to_string())
        .collect()
}

/// Strip [[wikilinks]] to plain text, keeping display text.
fn strip_wikilinks(text: &str) -> String {
    // [[Page|Display text]] → Display text
    // [[Page]] → Page
    let re = Regex::new(r"\[\[([^\]|]+)\|([^\]]+)\]\]").unwrap();
    let result = re.replace_all(text, "$2");
    let re2 = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re2.replace_all(&result, "$1").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter() {
        let content = "---\ndate: 2024-01-15\ntags: [journal]\n---\n\nHello world";
        let (fm, body) = split_frontmatter(content);
        assert!(fm.is_some());
        assert!(fm.unwrap().contains("date: 2024-01-15"));
        assert_eq!(body, "Hello world");
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "Just regular markdown content.";
        let (fm, body) = split_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_extract_tags() {
        let text = "Today I worked on #project-alpha and #coding. Also #life/journal.";
        let tags = extract_tags(text);
        assert!(tags.contains(&"project-alpha".to_string()));
        assert!(tags.contains(&"coding".to_string()));
        assert!(tags.contains(&"life/journal".to_string()));
    }

    #[test]
    fn test_strip_wikilinks() {
        let text = "I talked to [[Alice]] about [[Project X|the project]].";
        let clean = strip_wikilinks(text);
        assert_eq!(clean, "I talked to Alice about the project.");
    }

    #[test]
    fn test_parse_date_prefix() {
        let result = parse_date_prefix("2024-03-15-my-note");
        assert!(result.is_some());
        assert_eq!(result.unwrap().date_naive(), NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn test_parse_flexible_date() {
        assert!(parse_flexible_date("2024-01-15").is_some());
        assert!(parse_flexible_date("2024-01-15 14:30:00").is_some());
        assert!(parse_flexible_date("2024-01-15T14:30:00Z").is_some());
        assert!(parse_flexible_date("not a date").is_none());
    }
}
