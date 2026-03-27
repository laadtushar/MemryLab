use std::path::Path;

use chrono::{DateTime, Utc};
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;
use crate::pipeline::ingestion::file_extractors;

pub struct GenericAdapter;

impl SourceAdapter for GenericAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "generic".into(),
            display_name: "Generic / Other".into(),
            icon: "file-text".into(),
            takeout_url: None,
            instructions: "Import any text-based files (TXT, JSON, CSV, HTML, Markdown). The adapter will attempt to extract text from supported formats.".into(),
            accepted_extensions: vec![
                // Text formats
                "txt".into(), "json".into(), "csv".into(),
                "html".into(), "htm".into(), "md".into(),
                "xml".into(), "yaml".into(), "yml".into(),
                "toml".into(), "ini".into(), "cfg".into(),
                "log".into(), "rtf".into(), "tex".into(),
                "rst".into(), "org".into(), "tsv".into(),
                "eml".into(), "mbox".into(), "vcf".into(),
                "ics".into(), "bib".into(), "srt".into(),
                "vtt".into(), "plist".into(), "opml".into(),
                "ndjson".into(), "jsonl".into(),
                // Binary formats we can extract from
                "pdf".into(),
                "docx".into(), "pptx".into(), "xlsx".into(),
                "jpg".into(), "jpeg".into(), "png".into(),
                "heic".into(), "tiff".into(), "webp".into(),
            ],
            handles_zip: true,
            platform: SourcePlatform::Custom,
        }
    }

    fn detect(&self, _file_listing: &[&str]) -> f32 {
        // Lowest priority fallback — always returns a tiny score
        0.1
    }

    fn name(&self) -> &str {
        "generic"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        let entries: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            vec![path.to_path_buf()]
        };

        for file_path in entries {
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Try binary extractors first (PDF, Office, images, email)
            match ext.as_str() {
                "pdf" => {
                    if let Some(extracted) = file_extractors::extract_pdf(&file_path) {
                        let mut meta = serde_json::Map::new();
                        for (k, v) in &extracted.metadata {
                            meta.insert(k.clone(), serde_json::Value::String(v.clone()));
                        }
                        documents.push(parse_utils::build_document(
                            extracted.text, SourcePlatform::Custom, extracted.timestamp, vec![],
                            serde_json::Value::Object(meta),
                        ));
                    } else {
                        log::warn!("Failed to extract PDF: {}", file_path.display());
                    }
                    continue;
                }
                "docx" | "pptx" | "xlsx" => {
                    if let Some(extracted) = file_extractors::extract_office_doc(&file_path) {
                        let mut meta = serde_json::Map::new();
                        for (k, v) in &extracted.metadata {
                            meta.insert(k.clone(), serde_json::Value::String(v.clone()));
                        }
                        documents.push(parse_utils::build_document(
                            extracted.text, SourcePlatform::Custom, extracted.timestamp, vec![],
                            serde_json::Value::Object(meta),
                        ));
                    } else {
                        log::warn!("Failed to extract Office doc: {}", file_path.display());
                    }
                    continue;
                }
                "jpg" | "jpeg" | "png" | "heic" | "tiff" | "webp" => {
                    if let Some(extracted) = file_extractors::extract_image_exif(&file_path) {
                        let mut meta = serde_json::Map::new();
                        for (k, v) in &extracted.metadata {
                            meta.insert(k.clone(), serde_json::Value::String(v.clone()));
                        }
                        documents.push(parse_utils::build_document(
                            extracted.text, SourcePlatform::Custom, extracted.timestamp, vec![],
                            serde_json::Value::Object(meta),
                        ));
                    }
                    // No warning — many images simply don't have EXIF
                    continue;
                }
                "eml" => {
                    if let Some(extracted) = file_extractors::extract_email(&file_path) {
                        let participants: Vec<String> = extracted.metadata.get("from")
                            .into_iter().chain(extracted.metadata.get("to"))
                            .cloned().collect();
                        let mut meta = serde_json::Map::new();
                        for (k, v) in &extracted.metadata {
                            meta.insert(k.clone(), serde_json::Value::String(v.clone()));
                        }
                        documents.push(parse_utils::build_document(
                            extracted.text, SourcePlatform::Custom, extracted.timestamp, participants,
                            serde_json::Value::Object(meta),
                        ));
                    } else {
                        log::warn!("Failed to parse email: {}", file_path.display());
                    }
                    continue;
                }
                _ => {} // Fall through to text extraction below
            }

            // Read file content — skip binary files that fail UTF-8
            let read_text = |p: &Path| -> Option<String> {
                match std::fs::read_to_string(p) {
                    Ok(c) => Some(c),
                    Err(e) => {
                        log::warn!("Skipping {}: {}", p.display(), e);
                        None
                    }
                }
            };

            let text = match ext.as_str() {
                "json" | "ndjson" | "jsonl" => {
                    let content = match read_text(&file_path) { Some(c) => c, None => continue };
                    if ext == "ndjson" || ext == "jsonl" {
                        // Newline-delimited JSON: flatten each line
                        content.lines()
                            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                            .map(|v| parse_utils::flatten_json_to_text(&v))
                            .collect::<Vec<_>>()
                            .join("\n")
                    } else {
                        match serde_json::from_str::<serde_json::Value>(&content) {
                            Ok(v) => parse_utils::flatten_json_to_text(&v),
                            Err(_) => content,
                        }
                    }
                }
                "html" | "htm" => {
                    let content = match read_text(&file_path) { Some(c) => c, None => continue };
                    parse_utils::html_to_text(&content)
                }
                "xml" | "opml" | "plist" => {
                    // Strip XML tags, keep text content
                    let content = match read_text(&file_path) { Some(c) => c, None => continue };
                    parse_utils::html_to_text(&content)
                }
                "csv" | "tsv" => {
                    match parse_utils::parse_csv_file(&file_path) {
                        Ok(rows) => {
                            let lines: Vec<String> = rows
                                .iter()
                                .map(|row| {
                                    row.values()
                                        .cloned()
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                })
                                .collect();
                            lines.join("\n")
                        }
                        Err(e) => {
                            log::warn!("Skipping CSV {}: {}", file_path.display(), e);
                            continue;
                        }
                    }
                }
                "rtf" => {
                    // Strip RTF control words, keep text
                    let content = match read_text(&file_path) { Some(c) => c, None => continue };
                    let stripped: String = content
                        .replace("\\par", "\n")
                        .replace("\\tab", "\t")
                        .chars()
                        .fold((String::new(), false), |(mut acc, in_ctrl), c| {
                            if c == '\\' { (acc, true) }
                            else if in_ctrl && c == ' ' { (acc, false) }
                            else if in_ctrl { (acc, true) }
                            else if c == '{' || c == '}' { (acc, false) }
                            else { acc.push(c); (acc, false) }
                        }).0;
                    stripped
                }
                // All plain-text formats
                "txt" | "md" | "text" | "log" | "yaml" | "yml" | "toml"
                | "ini" | "cfg" | "tex" | "rst" | "org" | "eml" | "mbox"
                | "vcf" | "ics" | "bib" | "srt" | "vtt" => {
                    match read_text(&file_path) { Some(c) => c, None => continue }
                }
                _ => {
                    // Last resort: try reading as UTF-8 text
                    match std::fs::read_to_string(&file_path) {
                        Ok(c) if c.len() > 20 && c.is_ascii() || c.chars().take(500).all(|c| !c.is_control() || c == '\n' || c == '\r' || c == '\t') => c,
                        _ => continue,
                    }
                }
            };

            if text.trim().is_empty() {
                log::debug!("Skipping empty file: {}", file_path.display());
                continue;
            }

            log::debug!("Generic: parsed {} ({}, {} chars)", file_path.display(), ext, text.len());

            let file_name = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let mut meta = serde_json::Map::new();
            meta.insert("source_file".into(), serde_json::Value::String(file_name));
            meta.insert("format".into(), serde_json::Value::String(ext.clone()));

            // Try file modification time, fall back to None (time-agnostic)
            let file_ts = std::fs::metadata(&file_path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(DateTime::<Utc>::from);
            documents.push(parse_utils::build_document(
                text,
                SourcePlatform::Custom,
                file_ts,
                vec![],
                serde_json::Value::Object(meta),
            ));
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_generic() {
        let adapter = GenericAdapter;
        // Always returns 0.1 regardless of input
        assert!((adapter.detect(&["anything.txt"]) - 0.1).abs() < f32::EPSILON);
        assert!((adapter.detect(&[]) - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_metadata() {
        let adapter = GenericAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "generic");
        assert!(meta.takeout_url.is_none());
        assert_eq!(meta.platform, SourcePlatform::Custom);
    }
}
