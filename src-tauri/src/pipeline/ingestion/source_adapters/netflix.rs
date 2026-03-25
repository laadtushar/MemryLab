use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct NetflixAdapter;

impl SourceAdapter for NetflixAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "netflix".into(),
            display_name: "Netflix".into(),
            icon: "tv".into(),
            takeout_url: Some("https://www.netflix.com/account/getmyinfo".into()),
            instructions: "Request your data from Netflix (Account > Get My Info). Download the ZIP and upload it.".into(),
            accepted_extensions: vec!["zip".into(), "csv".into()],
            handles_zip: true,
            platform: SourcePlatform::Netflix,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_viewing = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("viewingactivity") || lower.contains("viewing_activity")
        });
        let has_content = file_listing.iter().any(|f| {
            f.to_lowercase().contains("content_interaction")
        });
        let has_mylist = file_listing.iter().any(|f| {
            f.to_lowercase().contains("mylist")
        });

        if has_viewing || has_content { 0.9 }
        else if has_mylist { 0.8 }
        else { 0.0 }
    }

    fn name(&self) -> &str {
        "netflix"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "csv"))
        {
            let file_name = entry.path()
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            if file_name.contains("viewingactivity") || file_name.contains("viewing_activity") {
                parse_viewing_activity(entry.path(), &mut documents);
            } else if file_name.contains("mylist") || file_name.contains("my_list") {
                parse_my_list(entry.path(), &mut documents);
            } else if file_name.contains("rating") {
                parse_ratings(entry.path(), &mut documents);
            }
        }

        Ok(documents)
    }
}

fn parse_viewing_activity(path: &Path, docs: &mut Vec<Document>) {
    let rows = match parse_utils::parse_csv_file(path) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Skipping Netflix ViewingActivity {}: {}", path.display(), e);
            return;
        }
    };

    for row in rows {
        let title = row.get("Title")
            .or_else(|| row.get("title"))
            .cloned()
            .unwrap_or_default();

        if title.trim().is_empty() {
            continue;
        }

        let text = format!("Watched: {}", title);

        let timestamp = row.get("Start Time")
            .or_else(|| row.get("Date"))
            .or_else(|| row.get("date"))
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| {
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                            .ok()
                            .map(|dt| dt.and_utc())
                    })
                    .or_else(|| {
                        chrono::NaiveDate::parse_from_str(s, "%m/%d/%y")
                            .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                            .ok()
                            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
                    })
            })
            .unwrap_or_else(Utc::now);

        let duration = row.get("Duration")
            .or_else(|| row.get("Bookmark"))
            .cloned()
            .unwrap_or_default();

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("viewing".into()));
        meta.insert("title".into(), serde_json::Value::String(title));
        if !duration.is_empty() {
            meta.insert("duration".into(), serde_json::Value::String(duration));
        }

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Netflix,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_my_list(path: &Path, docs: &mut Vec<Document>) {
    let rows = match parse_utils::parse_csv_file(path) {
        Ok(r) => r,
        Err(_) => return,
    };

    for row in rows {
        let title = row.get("Title")
            .or_else(|| row.get("title"))
            .or_else(|| row.get("Name"))
            .cloned()
            .unwrap_or_default();

        if title.trim().is_empty() {
            continue;
        }

        let text = format!("Added to My List: {}", title);

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("my_list".into()));
        meta.insert("title".into(), serde_json::Value::String(title));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Netflix,
            Utc::now(),
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_ratings(path: &Path, docs: &mut Vec<Document>) {
    let rows = match parse_utils::parse_csv_file(path) {
        Ok(r) => r,
        Err(_) => return,
    };

    for row in rows {
        let title = row.get("Title")
            .or_else(|| row.get("title"))
            .cloned()
            .unwrap_or_default();

        let rating = row.get("Rating")
            .or_else(|| row.get("Your Rating"))
            .cloned()
            .unwrap_or_default();

        if title.trim().is_empty() {
            continue;
        }

        let text = if rating.is_empty() {
            format!("Rated: {}", title)
        } else {
            format!("Rated: {} ({})", title, rating)
        };

        let timestamp = row.get("Date")
            .or_else(|| row.get("Timestamp"))
            .and_then(|s| {
                chrono::NaiveDate::parse_from_str(s, "%m/%d/%y")
                    .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                    .ok()
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .unwrap_or_else(Utc::now);

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("rating".into()));
        meta.insert("title".into(), serde_json::Value::String(title));
        if !rating.is_empty() {
            meta.insert("rating".into(), serde_json::Value::String(rating));
        }

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Netflix,
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
    fn test_detect_netflix() {
        let adapter = NetflixAdapter;
        let files = vec!["CONTENT_INTERACTION/ViewingActivity.csv", "MyList.csv"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = NetflixAdapter;
        assert!(adapter.detect(&["random.csv"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = NetflixAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "netflix");
        assert_eq!(meta.platform, SourcePlatform::Netflix);
    }
}
