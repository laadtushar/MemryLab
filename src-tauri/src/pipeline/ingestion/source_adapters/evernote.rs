use std::path::Path;

use chrono::NaiveDateTime;
use regex::Regex;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct EvernoteAdapter;

impl SourceAdapter for EvernoteAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "evernote".into(),
            display_name: "Evernote".into(),
            icon: "notebook".into(),
            takeout_url: None,
            instructions: "Export notes from Evernote Desktop (File > Export Notes > .enex format).".into(),
            accepted_extensions: vec!["enex".into(), "xml".into()],
            handles_zip: false,
            platform: SourcePlatform::Evernote,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_enex = file_listing.iter().any(|f| f.ends_with(".enex"));
        if has_enex { 0.9 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "evernote"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        let files: Vec<_> = if path.is_dir() {
            walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "enex" || ext == "xml")
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            vec![path.to_path_buf()]
        };

        for file in files {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Skipping {}: {}", file.display(), e);
                    continue;
                }
            };

            parse_enex(&content, &mut documents);
        }

        Ok(documents)
    }
}

fn parse_enex(content: &str, docs: &mut Vec<Document>) {
    // Simple XML parsing using regex for ENEX format
    // Each note is wrapped in <note>...</note>
    let note_re = Regex::new(r"(?s)<note>(.*?)</note>").unwrap();
    let title_re = Regex::new(r"(?s)<title>(.*?)</title>").unwrap();
    let content_re = Regex::new(r"(?s)<content>(.*?)</content>").unwrap();
    let created_re = Regex::new(r"<created>(\d{8}T\d{6}Z?)</created>").unwrap();

    for note_cap in note_re.captures_iter(content) {
        let note_xml = &note_cap[1];

        let title = title_re
            .captures(note_xml)
            .map(|c| c[1].trim().to_string())
            .unwrap_or_default();

        let raw_content = content_re
            .captures(note_xml)
            .map(|c| c[1].to_string())
            .unwrap_or_default();

        // Strip CDATA wrapper if present
        let html_content = raw_content
            .trim()
            .strip_prefix("<![CDATA[")
            .and_then(|s| s.strip_suffix("]]>"))
            .unwrap_or(&raw_content);

        let text = parse_utils::html_to_text(html_content);

        let full_text = if title.is_empty() {
            text
        } else {
            format!("{}\n\n{}", title, text)
        };

        if full_text.trim().is_empty() {
            continue;
        }

        let timestamp = created_re
            .captures(note_xml)
            .and_then(|c| {
                let s = &c[1];
                NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ")
                    .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S"))
                    .ok()
                    .map(|dt| dt.and_utc())
            })
            ;

        let mut meta = serde_json::Map::new();
        if !title.is_empty() {
            meta.insert("title".into(), serde_json::Value::String(title));
        }

        docs.push(parse_utils::build_document(
            full_text,
            SourcePlatform::Evernote,
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
    fn test_detect_evernote() {
        let adapter = EvernoteAdapter;
        let files = vec!["My Notes.enex"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = EvernoteAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_parse_enex() {
        let enex = r#"<?xml version="1.0" encoding="UTF-8"?>
<en-export>
<note>
<title>My First Note</title>
<content><![CDATA[<div>Hello <b>world</b></div>]]></content>
<created>20240315T143000Z</created>
</note>
<note>
<title>Second Note</title>
<content><![CDATA[<p>Some content here</p>]]></content>
<created>20240316T100000Z</created>
</note>
</en-export>"#;

        let mut docs = Vec::new();
        parse_enex(enex, &mut docs);
        assert_eq!(docs.len(), 2);
        assert!(docs[0].raw_text.contains("My First Note"));
        assert!(docs[0].raw_text.contains("Hello"));
    }

    #[test]
    fn test_metadata() {
        let adapter = EvernoteAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "evernote");
        assert!(!meta.handles_zip);
    }
}
