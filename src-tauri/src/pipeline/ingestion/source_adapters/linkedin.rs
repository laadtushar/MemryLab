use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct LinkedInAdapter;

impl SourceAdapter for LinkedInAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "linkedin".into(),
            display_name: "LinkedIn".into(),
            icon: "linkedin".into(),
            takeout_url: Some("https://www.linkedin.com/mypreferences/d/download-my-data".into()),
            instructions: "Download your LinkedIn data from Settings > Data Privacy > Get a copy of your data. Choose the complete archive.".into(),
            accepted_extensions: vec!["zip".into(), "csv".into()],
            handles_zip: true,
            platform: SourcePlatform::LinkedIn,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_connections = file_listing.iter().any(|f| f.ends_with("Connections.csv"));
        let has_messages = file_listing.iter().any(|f| f.ends_with("Messages.csv") || f.ends_with("messages.csv"));
        let has_profile = file_listing.iter().any(|f| f.ends_with("Profile.csv") || f.ends_with("profile.csv"));
        let matches = [has_connections, has_messages, has_profile].iter().filter(|&&b| b).count();
        if matches >= 2 {
            0.9
        } else if matches == 1 {
            0.5
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "linkedin"
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
                .unwrap_or("")
                .to_lowercase();

            let rows = match parse_utils::parse_csv_file(&csv_path) {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("Skipping {}: {}", csv_path.display(), e);
                    continue;
                }
            };

            for row in rows {
                let (text, doc_type) = if file_name.contains("connections") {
                    let name = row.get("First Name").cloned().unwrap_or_default();
                    let last = row.get("Last Name").cloned().unwrap_or_default();
                    let company = row.get("Company").cloned().unwrap_or_default();
                    let position = row.get("Position").cloned().unwrap_or_default();
                    let t = format!("{} {} — {} at {}", name, last, position, company);
                    (t, "connection")
                } else if file_name.contains("messages") {
                    let content = row.get("CONTENT").or_else(|| row.get("Content")).or_else(|| row.get("content")).cloned().unwrap_or_default();
                    (content, "message")
                } else if file_name.contains("shares") || file_name.contains("posts") {
                    let text = row.get("ShareCommentary").or_else(|| row.get("Commentary")).or_else(|| row.get("Text")).cloned().unwrap_or_default();
                    (text, "post")
                } else if file_name.contains("profile") {
                    let headline = row.get("Headline").cloned().unwrap_or_default();
                    let summary = row.get("Summary").cloned().unwrap_or_default();
                    let t = if summary.is_empty() { headline } else { format!("{}\n\n{}", headline, summary) };
                    (t, "profile")
                } else {
                    let t = row.values().cloned().collect::<Vec<_>>().join(" ");
                    (t, "other")
                };

                if text.trim().is_empty() {
                    continue;
                }

                let sender = row.get("FROM").or_else(|| row.get("From")).or_else(|| row.get("First Name")).cloned().unwrap_or_default();

                let timestamp = row
                    .get("Date")
                    .or_else(|| row.get("DATE"))
                    .or_else(|| row.get("Connected On"))
                    .or_else(|| row.get("date"))
                    .and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                            .or_else(|| chrono::NaiveDate::parse_from_str(s, "%d %b %Y").ok()
                                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()))
                            .or_else(|| chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y").ok()
                                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()))
                            .or_else(|| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
                                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()))
                    })
                    .unwrap_or_else(Utc::now);

                let mut meta = serde_json::Map::new();
                meta.insert("type".into(), serde_json::Value::String(doc_type.into()));
                if let Some(url) = row.get("URL").or_else(|| row.get("url")) {
                    meta.insert("url".into(), serde_json::Value::String(url.clone()));
                }

                let participants = if sender.is_empty() { vec![] } else { vec![sender] };

                documents.push(parse_utils::build_document(
                    text,
                    SourcePlatform::LinkedIn,
                    timestamp,
                    participants,
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
    fn test_detect_linkedin() {
        let adapter = LinkedInAdapter;
        let files = vec!["Connections.csv", "Messages.csv", "Profile.csv"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_partial() {
        let adapter = LinkedInAdapter;
        assert!(adapter.detect(&["Connections.csv"]) >= 0.4);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = LinkedInAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = LinkedInAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "linkedin");
        assert!(meta.takeout_url.is_some());
    }
}
