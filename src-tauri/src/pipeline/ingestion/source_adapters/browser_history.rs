//! Browser history import adapters for Chrome, Edge, Firefox, and Safari.
//!
//! All browsers store history in SQLite databases. These adapters copy the
//! database to a temp location (to avoid lock conflicts with running browsers),
//! then read visits and create documents grouped by domain.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::params;
use uuid::Uuid;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::parse_utils;
use super::{SourceAdapter, SourceAdapterMeta};

/// A raw browser visit record
struct BrowserVisit {
    url: String,
    title: String,
    timestamp: Option<DateTime<Utc>>,
}

// ─── Shared helpers ───

/// Copy a browser SQLite database to a temp file to avoid lock conflicts.
/// Also copies WAL/journal files if present.
fn copy_db_to_temp(db_path: &Path) -> Result<std::path::PathBuf, AppError> {
    let temp_dir = std::env::temp_dir().join("memrylab_browser_import");
    std::fs::create_dir_all(&temp_dir)?;
    let dest = temp_dir.join(format!("history_{}.sqlite", Uuid::new_v4()));
    std::fs::copy(db_path, &dest)?;

    // Copy WAL and journal if they exist
    for suffix in &["-wal", "-journal", "-shm"] {
        let wal_src = db_path.with_extension(format!("sqlite{}", suffix));
        if wal_src.exists() {
            let wal_dest = dest.with_extension(format!("sqlite{}", suffix));
            let _ = std::fs::copy(&wal_src, &wal_dest);
        }
        // Also try without .sqlite extension (Chrome uses "History-journal" etc.)
        let name = db_path.file_name().unwrap_or_default().to_string_lossy();
        let wal_src2 = db_path.with_file_name(format!("{}{}", name, suffix));
        if wal_src2.exists() {
            let dest_name = dest.file_name().unwrap_or_default().to_string_lossy();
            let wal_dest2 = dest.with_file_name(format!("{}{}", dest_name, suffix));
            let _ = std::fs::copy(&wal_src2, &wal_dest2);
        }
    }
    Ok(dest)
}

/// Extract domain from URL
fn extract_domain(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

/// Convert visits into Documents, grouping by domain within 1-hour windows
fn visits_to_documents(
    visits: Vec<BrowserVisit>,
    platform: SourcePlatform,
    browser_name: &str,
) -> Vec<Document> {
    // Group by domain
    let mut by_domain: HashMap<String, Vec<&BrowserVisit>> = HashMap::new();
    for visit in &visits {
        let domain = extract_domain(&visit.url);
        by_domain.entry(domain).or_default().push(visit);
    }

    let mut documents = Vec::new();

    for (domain, domain_visits) in &by_domain {
        // Create one document per domain (summarizing all visits)
        let titles: Vec<&str> = domain_visits
            .iter()
            .filter(|v| !v.title.is_empty())
            .map(|v| v.title.as_str())
            .collect::<Vec<_>>();

        // Deduplicate titles
        let mut unique_titles: Vec<&str> = Vec::new();
        for t in &titles {
            if !unique_titles.contains(t) {
                unique_titles.push(t);
            }
        }
        unique_titles.truncate(20); // Limit to avoid huge documents

        if unique_titles.is_empty() {
            continue;
        }

        let text = format!(
            "Browser history: {}\nDomain: {}\nPages visited ({}):\n{}",
            browser_name,
            domain,
            domain_visits.len(),
            unique_titles.iter().map(|t| format!("- {}", t)).collect::<Vec<_>>().join("\n")
        );

        // Use earliest visit timestamp
        let timestamp = domain_visits
            .iter()
            .filter_map(|v| v.timestamp)
            .min();

        let mut meta = serde_json::Map::new();
        meta.insert("browser".into(), serde_json::Value::String(browser_name.to_string()));
        meta.insert("domain".into(), serde_json::Value::String(domain.clone()));
        meta.insert("visit_count".into(), serde_json::Value::Number(domain_visits.len().into()));

        documents.push(parse_utils::build_document(
            text,
            platform.clone(),
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }

    documents
}

// ─── Chromium parser (Chrome + Edge) ───

fn parse_chromium_history(path: &Path, platform: SourcePlatform, browser_name: &str) -> Result<Vec<Document>, AppError> {
    // Find the History file
    let history_file = find_file(path, "History")?;
    let temp_db = copy_db_to_temp(&history_file)?;

    let conn = rusqlite::Connection::open_with_flags(
        &temp_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).map_err(|e| AppError::Other(format!("Failed to open browser history: {}", e)))?;

    let mut stmt = conn.prepare(
        "SELECT urls.url, urls.title, visits.visit_time
         FROM urls JOIN visits ON urls.id = visits.url
         ORDER BY visits.visit_time DESC
         LIMIT 50000"
    ).map_err(|e| AppError::Other(format!("SQL error: {}", e)))?;

    let visits: Vec<BrowserVisit> = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: String = row.get::<_, String>(1).unwrap_or_default();
        let visit_time: i64 = row.get(2)?;
        // WebKit timestamp: microseconds since 1601-01-01
        // Convert: subtract 11644473600000000 (us between 1601 and 1970), divide by 1000000
        let unix_secs = (visit_time - 11_644_473_600_000_000) / 1_000_000;
        let timestamp = Utc.timestamp_opt(unix_secs, 0).single();
        Ok(BrowserVisit { url, title, timestamp })
    }).map_err(|e| AppError::Other(format!("Query error: {}", e)))?
    .filter_map(|r| r.ok())
    .collect();

    // Cleanup temp file
    let _ = std::fs::remove_file(&temp_db);

    log::info!("{}: found {} visits", browser_name, visits.len());
    Ok(visits_to_documents(visits, platform, browser_name))
}

// ─── Firefox parser ───

fn parse_firefox_history(path: &Path) -> Result<Vec<Document>, AppError> {
    let history_file = find_file(path, "places.sqlite")?;
    let temp_db = copy_db_to_temp(&history_file)?;

    let conn = rusqlite::Connection::open_with_flags(
        &temp_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).map_err(|e| AppError::Other(format!("Failed to open Firefox history: {}", e)))?;

    let mut stmt = conn.prepare(
        "SELECT moz_places.url, moz_places.title, moz_historyvisits.visit_date
         FROM moz_places
         JOIN moz_historyvisits ON moz_places.id = moz_historyvisits.place_id
         WHERE moz_places.url NOT LIKE 'place:%'
         ORDER BY moz_historyvisits.visit_date DESC
         LIMIT 50000"
    ).map_err(|e| AppError::Other(format!("SQL error: {}", e)))?;

    let visits: Vec<BrowserVisit> = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: String = row.get::<_, String>(1).unwrap_or_default();
        let visit_date: i64 = row.get(2)?;
        // Firefox: microseconds since Unix epoch
        let unix_secs = visit_date / 1_000_000;
        let timestamp = Utc.timestamp_opt(unix_secs, 0).single();
        Ok(BrowserVisit { url, title, timestamp })
    }).map_err(|e| AppError::Other(format!("Query error: {}", e)))?
    .filter_map(|r| r.ok())
    .collect();

    let _ = std::fs::remove_file(&temp_db);

    log::info!("Firefox: found {} visits", visits.len());
    Ok(visits_to_documents(visits, SourcePlatform::FirefoxHistory, "Firefox"))
}

// ─── Safari parser ───

fn parse_safari_history(path: &Path) -> Result<Vec<Document>, AppError> {
    let history_file = find_file(path, "History.db")?;
    let temp_db = copy_db_to_temp(&history_file)?;

    let conn = rusqlite::Connection::open_with_flags(
        &temp_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).map_err(|e| AppError::Other(format!("Failed to open Safari history: {}", e)))?;

    let mut stmt = conn.prepare(
        "SELECT history_items.url, history_visits.title, history_visits.visit_time
         FROM history_items
         JOIN history_visits ON history_items.id = history_visits.history_item
         ORDER BY history_visits.visit_time DESC
         LIMIT 50000"
    ).map_err(|e| AppError::Other(format!("SQL error: {}", e)))?;

    let visits: Vec<BrowserVisit> = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: String = row.get::<_, String>(1).unwrap_or_default();
        let visit_time: f64 = row.get(2)?;
        // Safari/Core Data: seconds since 2001-01-01
        let unix_secs = visit_time as i64 + 978_307_200;
        let timestamp = Utc.timestamp_opt(unix_secs, 0).single();
        Ok(BrowserVisit { url, title, timestamp })
    }).map_err(|e| AppError::Other(format!("Query error: {}", e)))?
    .filter_map(|r| r.ok())
    .collect();

    let _ = std::fs::remove_file(&temp_db);

    log::info!("Safari: found {} visits", visits.len());
    Ok(visits_to_documents(visits, SourcePlatform::SafariHistory, "Safari"))
}

// ─── File finder ───

fn find_file(path: &Path, name: &str) -> Result<std::path::PathBuf, AppError> {
    // Check if the path itself is the file
    if path.is_file() && path.file_name().map(|n| n.to_string_lossy()) == Some(name.into()) {
        return Ok(path.to_path_buf());
    }
    // Walk directory to find the file
    for entry in walkdir::WalkDir::new(path).max_depth(5).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_string_lossy() == name {
            return Ok(entry.path().to_path_buf());
        }
    }
    Err(AppError::Other(format!("Could not find '{}' in {}", name, path.display())))
}

// ═══════════════════════════════════════════════════════
// Adapter implementations
// ═══════════════════════════════════════════════════════

// ─── Chrome ───

pub struct ChromeHistoryAdapter;

impl SourceAdapter for ChromeHistoryAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "chrome_history".to_string(),
            display_name: "Google Chrome".to_string(),
            icon: "chrome".to_string(),
            takeout_url: None,
            instructions: "Point MemryLab at your Chrome profile folder. On Windows: %LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default. On macOS: ~/Library/Application Support/Google/Chrome/Default. On Linux: ~/.config/google-chrome/Default. Or copy the 'History' file to a folder and import that.".to_string(),
            accepted_extensions: vec![],
            handles_zip: false,
            platform: SourcePlatform::ChromeHistory,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_history = file_listing.iter().any(|f| {
            let name = f.rsplit('/').next().unwrap_or(f);
            name == "History" && !f.to_lowercase().contains("firefox") && !f.to_lowercase().contains("edge")
        });
        let has_bookmarks = file_listing.iter().any(|f| f.rsplit('/').next().unwrap_or(f) == "Bookmarks");
        let has_chrome_marker = file_listing.iter().any(|f| f.to_lowercase().contains("chrome") || f.contains("Google"));

        if has_history && (has_bookmarks || has_chrome_marker) { 0.90 }
        else if has_history && !file_listing.iter().any(|f| f.to_lowercase().contains("edge")) { 0.70 }
        else { 0.0 }
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        parse_chromium_history(path, SourcePlatform::ChromeHistory, "Chrome")
    }

    fn name(&self) -> &str { "chrome_history" }
}

// ─── Edge ───

pub struct EdgeHistoryAdapter;

impl SourceAdapter for EdgeHistoryAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "edge_history".to_string(),
            display_name: "Microsoft Edge".to_string(),
            icon: "globe".to_string(),
            takeout_url: None,
            instructions: "Point MemryLab at your Edge profile folder. On Windows: %LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default. On macOS: ~/Library/Application Support/Microsoft Edge/Default. On Linux: ~/.config/microsoft-edge/Default.".to_string(),
            accepted_extensions: vec![],
            handles_zip: false,
            platform: SourcePlatform::EdgeHistory,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_history = file_listing.iter().any(|f| f.rsplit('/').next().unwrap_or(f) == "History");
        let has_edge = file_listing.iter().any(|f| f.to_lowercase().contains("edge"));
        if has_history && has_edge { 0.92 } else { 0.0 }
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        parse_chromium_history(path, SourcePlatform::EdgeHistory, "Edge")
    }

    fn name(&self) -> &str { "edge_history" }
}

// ─── Firefox ───

pub struct FirefoxHistoryAdapter;

impl SourceAdapter for FirefoxHistoryAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "firefox_history".to_string(),
            display_name: "Mozilla Firefox".to_string(),
            icon: "flame".to_string(),
            takeout_url: None,
            instructions: "Point MemryLab at your Firefox profile folder. On Windows: %APPDATA%\\Mozilla\\Firefox\\Profiles\\*.default-release. On macOS: ~/Library/Application Support/Firefox/Profiles/. On Linux: ~/.mozilla/firefox/*.default-release. Or copy the 'places.sqlite' file to a folder.".to_string(),
            accepted_extensions: vec!["sqlite".to_string()],
            handles_zip: false,
            platform: SourcePlatform::FirefoxHistory,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        if file_listing.iter().any(|f| f.rsplit('/').next().unwrap_or(f) == "places.sqlite") {
            0.92
        } else {
            0.0
        }
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        parse_firefox_history(path)
    }

    fn name(&self) -> &str { "firefox_history" }
}

// ─── Safari ───

pub struct SafariHistoryAdapter;

impl SourceAdapter for SafariHistoryAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "safari_history".to_string(),
            display_name: "Safari".to_string(),
            icon: "compass".to_string(),
            takeout_url: None,
            instructions: "Point MemryLab at your Safari data folder: ~/Library/Safari. You may need to grant Full Disk Access in System Settings > Privacy & Security. (macOS only)".to_string(),
            accepted_extensions: vec!["db".to_string()],
            handles_zip: false,
            platform: SourcePlatform::SafariHistory,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        if file_listing.iter().any(|f| f.rsplit('/').next().unwrap_or(f) == "History.db") {
            0.90
        } else {
            0.0
        }
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        parse_safari_history(path)
    }

    fn name(&self) -> &str { "safari_history" }
}
