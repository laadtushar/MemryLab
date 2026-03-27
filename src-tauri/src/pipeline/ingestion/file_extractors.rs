//! Extractors for binary/rich file formats: PDF, DOCX, PPTX, XLSX, images (EXIF), emails (.eml).
//! Each extractor returns extracted text + metadata, or None if the file can't be processed.

use std::collections::HashMap;
use std::path::Path;

/// Result of extracting content from a file.
pub struct ExtractedContent {
    /// Main text content extracted from the file.
    pub text: String,
    /// Metadata key-value pairs (author, date, GPS, device, etc.)
    pub metadata: HashMap<String, String>,
    /// Timestamp extracted from the file, if available.
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

// ─── PDF ───────────────────────────────────────────────────────────────────────

pub fn extract_pdf(path: &Path) -> Option<ExtractedContent> {
    let bytes = std::fs::read(path).ok()?;
    let text = pdf_extract::extract_text_from_mem(&bytes).ok()?;
    if text.trim().is_empty() {
        log::debug!("PDF has no extractable text: {}", path.display());
        return None;
    }

    let mut metadata = HashMap::new();
    metadata.insert("format".into(), "pdf".into());
    metadata.insert("source_file".into(), path.file_name()?.to_string_lossy().into_owned());

    // Try to get page count from the text (rough estimate by form feeds)
    let page_count = text.matches('\u{000C}').count().max(1);
    metadata.insert("pages".into(), page_count.to_string());

    let file_ts = std::fs::metadata(path).ok()
        .and_then(|m| m.modified().ok())
        .map(chrono::DateTime::<chrono::Utc>::from);

    log::info!("PDF: extracted {} chars, ~{} pages from {}", text.len(), page_count, path.display());

    Some(ExtractedContent { text, metadata, timestamp: file_ts })
}

// ─── DOCX / PPTX / XLSX (Office Open XML — ZIP with XML inside) ───────────────

pub fn extract_office_doc(path: &Path) -> Option<ExtractedContent> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let ext = path.extension()?.to_str()?.to_lowercase();
    let mut all_text = String::new();
    let mut metadata = HashMap::new();
    metadata.insert("format".into(), ext.clone());
    metadata.insert("source_file".into(), path.file_name()?.to_string_lossy().into_owned());

    match ext.as_str() {
        "docx" => {
            // Main document body
            if let Ok(mut entry) = archive.by_name("word/document.xml") {
                let mut xml = String::new();
                std::io::Read::read_to_string(&mut entry, &mut xml).ok()?;
                all_text.push_str(&strip_xml_tags(&xml));
            }
            // Headers/footers
            for i in 1..=3 {
                for kind in &["header", "footer"] {
                    let name = format!("word/{}{}. xml", kind, i);
                    if let Ok(mut entry) = archive.by_name(&name) {
                        let mut xml = String::new();
                        if std::io::Read::read_to_string(&mut entry, &mut xml).is_ok() {
                            all_text.push('\n');
                            all_text.push_str(&strip_xml_tags(&xml));
                        }
                    }
                }
            }
        }
        "pptx" => {
            // Iterate slide XML files
            for i in 0..archive.len() {
                let name = archive.by_index(i).ok().map(|e| e.name().to_string());
                if let Some(name) = name {
                    if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                        if let Ok(mut entry) = archive.by_name(&name) {
                            let mut xml = String::new();
                            if std::io::Read::read_to_string(&mut entry, &mut xml).is_ok() {
                                all_text.push('\n');
                                all_text.push_str(&strip_xml_tags(&xml));
                            }
                        }
                    }
                    // Also get slide notes
                    if name.starts_with("ppt/notesSlides/") && name.ends_with(".xml") {
                        if let Ok(mut entry) = archive.by_name(&name) {
                            let mut xml = String::new();
                            if std::io::Read::read_to_string(&mut entry, &mut xml).is_ok() {
                                all_text.push('\n');
                                all_text.push_str(&strip_xml_tags(&xml));
                            }
                        }
                    }
                }
            }
        }
        "xlsx" => {
            // Shared strings table
            let mut shared_strings = Vec::new();
            if let Ok(mut entry) = archive.by_name("xl/sharedStrings.xml") {
                let mut xml = String::new();
                if std::io::Read::read_to_string(&mut entry, &mut xml).is_ok() {
                    // Extract <t>...</t> values
                    for segment in xml.split("<t") {
                        if let Some(start) = segment.find('>') {
                            if let Some(end) = segment[start..].find("</t>") {
                                shared_strings.push(segment[start + 1..start + end].to_string());
                            }
                        }
                    }
                }
            }
            all_text = shared_strings.join("\t");
        }
        _ => return None,
    }

    // Extract core metadata (docProps/core.xml)
    if let Ok(mut entry) = archive.by_name("docProps/core.xml") {
        let mut xml = String::new();
        if std::io::Read::read_to_string(&mut entry, &mut xml).is_ok() {
            if let Some(author) = extract_xml_value(&xml, "dc:creator") {
                metadata.insert("author".into(), author);
            }
            if let Some(title) = extract_xml_value(&xml, "dc:title") {
                metadata.insert("title".into(), title);
            }
            if let Some(subject) = extract_xml_value(&xml, "dc:subject") {
                metadata.insert("subject".into(), subject);
            }
            if let Some(modified) = extract_xml_value(&xml, "dcterms:modified") {
                metadata.insert("modified".into(), modified);
            }
        }
    }

    if all_text.trim().is_empty() {
        log::debug!("Office doc has no extractable text: {}", path.display());
        return None;
    }

    let file_ts = std::fs::metadata(path).ok()
        .and_then(|m| m.modified().ok())
        .map(chrono::DateTime::<chrono::Utc>::from);

    log::info!("Office: extracted {} chars from {} ({})", all_text.len(), path.display(), ext);

    Some(ExtractedContent { text: all_text, metadata, timestamp: file_ts })
}

// ─── Image EXIF ────────────────────────────────────────────────────────────────

pub fn extract_image_exif(path: &Path) -> Option<ExtractedContent> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exif_reader = exif::Reader::new();
    let exif_data = exif_reader.read_from_container(&mut bufreader).ok()?;

    let mut metadata = HashMap::new();
    metadata.insert("format".into(), "image".into());
    metadata.insert("source_file".into(), path.file_name()?.to_string_lossy().into_owned());

    let mut text_parts = Vec::new();

    // Date/time
    if let Some(field) = exif_data.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        let val = field.display_value().to_string();
        metadata.insert("date_taken".into(), val.clone());
        text_parts.push(format!("Photo taken: {}", val));
    } else if let Some(field) = exif_data.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
        let val = field.display_value().to_string();
        metadata.insert("date_taken".into(), val.clone());
        text_parts.push(format!("Photo date: {}", val));
    }

    // Camera / device
    if let Some(field) = exif_data.get_field(exif::Tag::Make, exif::In::PRIMARY) {
        let make = field.display_value().to_string().replace('"', "");
        metadata.insert("camera_make".into(), make.clone());
        text_parts.push(format!("Camera: {}", make));
    }
    if let Some(field) = exif_data.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        let model = field.display_value().to_string().replace('"', "");
        metadata.insert("camera_model".into(), model.clone());
        text_parts.push(format!("Model: {}", model));
    }

    // GPS coordinates
    let lat = extract_gps_coord(&exif_data, exif::Tag::GPSLatitude, exif::Tag::GPSLatitudeRef);
    let lng = extract_gps_coord(&exif_data, exif::Tag::GPSLongitude, exif::Tag::GPSLongitudeRef);
    if let (Some(lat), Some(lng)) = (lat, lng) {
        metadata.insert("gps_latitude".into(), format!("{:.6}", lat));
        metadata.insert("gps_longitude".into(), format!("{:.6}", lng));
        text_parts.push(format!("Location: {:.6}, {:.6}", lat, lng));
    }

    // Image dimensions
    if let Some(w) = exif_data.get_field(exif::Tag::PixelXDimension, exif::In::PRIMARY) {
        if let Some(h) = exif_data.get_field(exif::Tag::PixelYDimension, exif::In::PRIMARY) {
            let dims = format!("{}x{}", w.display_value(), h.display_value());
            metadata.insert("dimensions".into(), dims.clone());
            text_parts.push(format!("Size: {}", dims));
        }
    }

    // Lens info
    if let Some(field) = exif_data.get_field(exif::Tag::LensModel, exif::In::PRIMARY) {
        let lens = field.display_value().to_string().replace('"', "");
        metadata.insert("lens".into(), lens.clone());
        text_parts.push(format!("Lens: {}", lens));
    }

    // Image description / user comment
    if let Some(field) = exif_data.get_field(exif::Tag::ImageDescription, exif::In::PRIMARY) {
        let desc = field.display_value().to_string().replace('"', "");
        if !desc.is_empty() {
            metadata.insert("description".into(), desc.clone());
            text_parts.push(format!("Description: {}", desc));
        }
    }

    if text_parts.is_empty() {
        log::debug!("Image has no useful EXIF data: {}", path.display());
        return None;
    }

    let file_name = path.file_name()?.to_string_lossy();
    let text = format!("Image: {}\n{}", file_name, text_parts.join("\n"));

    // Parse timestamp from EXIF date
    let timestamp = metadata.get("date_taken")
        .and_then(|d| chrono::NaiveDateTime::parse_from_str(d, "%Y-%m-%d %H:%M:%S").ok())
        .map(|dt| dt.and_utc());

    log::info!("EXIF: extracted {} fields from {}", metadata.len(), path.display());

    Some(ExtractedContent { text, metadata, timestamp })
}

fn extract_gps_coord(exif: &exif::Exif, coord_tag: exif::Tag, ref_tag: exif::Tag) -> Option<f64> {
    let field = exif.get_field(coord_tag, exif::In::PRIMARY)?;
    let ref_field = exif.get_field(ref_tag, exif::In::PRIMARY)?;

    // GPS coordinates stored as [degrees, minutes, seconds] rationals
    if let exif::Value::Rational(ref vals) = field.value {
        if vals.len() >= 3 {
            let deg = vals[0].to_f64();
            let min = vals[1].to_f64();
            let sec = vals[2].to_f64();
            let mut coord = deg + min / 60.0 + sec / 3600.0;

            let ref_str = ref_field.display_value().to_string().replace('"', "");
            if ref_str == "S" || ref_str == "W" {
                coord = -coord;
            }
            return Some(coord);
        }
    }
    None
}

// ─── Email (.eml) ──────────────────────────────────────────────────────────────

pub fn extract_email(path: &Path) -> Option<ExtractedContent> {
    let raw = std::fs::read(path).ok()?;
    let parsed = mailparse::parse_mail(&raw).ok()?;

    let mut metadata = HashMap::new();
    metadata.insert("format".into(), "email".into());
    metadata.insert("source_file".into(), path.file_name()?.to_string_lossy().into_owned());

    let mut text_parts = Vec::new();

    // Extract headers
    for header in parsed.get_headers() {
        let key: String = header.get_key().to_lowercase();
        let val: String = header.get_value();
        match key.as_str() {
            "from" => {
                metadata.insert("from".into(), val.clone());
                text_parts.push(format!("From: {}", val));
            }
            "to" => {
                metadata.insert("to".into(), val.clone());
                text_parts.push(format!("To: {}", val));
            }
            "subject" => {
                metadata.insert("subject".into(), val.clone());
                text_parts.push(format!("Subject: {}", val));
            }
            "date" => {
                metadata.insert("date".into(), val.clone());
                text_parts.push(format!("Date: {}", val));
            }
            "cc" => {
                metadata.insert("cc".into(), val.clone());
            }
            _ => {}
        }
    }

    // Extract body text
    let body = extract_email_body(&parsed);
    if !body.trim().is_empty() {
        text_parts.push(String::new());
        text_parts.push(body);
    }

    if text_parts.is_empty() {
        log::debug!("Email has no extractable content: {}", path.display());
        return None;
    }

    let text = text_parts.join("\n");

    // Parse date from header
    let timestamp = metadata.get("date")
        .and_then(|d| mailparse::dateparse(d).ok())
        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0));

    log::info!("Email: extracted from {} (subject: {})", path.display(),
        metadata.get("subject").unwrap_or(&"<none>".into()));

    Some(ExtractedContent { text, metadata, timestamp })
}

fn extract_email_body(mail: &mailparse::ParsedMail) -> String {
    // Prefer plain text, fall back to HTML stripped
    if mail.subparts.is_empty() {
        return mail.get_body().unwrap_or_default();
    }

    // Look for text/plain first
    for part in &mail.subparts {
        let ct = part.ctype.mimetype.to_lowercase();
        if ct == "text/plain" {
            if let Ok(body) = part.get_body() {
                if !body.trim().is_empty() {
                    return body;
                }
            }
        }
    }

    // Fall back to text/html stripped of tags
    for part in &mail.subparts {
        let ct = part.ctype.mimetype.to_lowercase();
        if ct == "text/html" {
            if let Ok(body) = part.get_body() {
                return strip_html_tags(&body);
            }
        }
    }

    // Recurse into multipart
    for part in &mail.subparts {
        let body = extract_email_body(part);
        if !body.trim().is_empty() {
            return body;
        }
    }

    String::new()
}

// ─── Image Vision (Ollama multimodal) ──────────────────────────────────────────

/// Describe an image using a multimodal LLM (llava via Ollama).
/// Returns None if no vision model is available.
pub async fn describe_image_with_vision(
    path: &Path,
    ollama_url: &str,
) -> Option<ExtractedContent> {
    let bytes = std::fs::read(path).ok()?;
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

    let ext = path.extension()?.to_str()?.to_lowercase();
    if !["jpg", "jpeg", "png", "gif", "webp", "bmp"].contains(&ext.as_str()) {
        return None;
    }

    let file_name = path.file_name()?.to_string_lossy().into_owned();

    let payload = serde_json::json!({
        "model": "llava",
        "prompt": "Describe this image in detail. Include any text visible in the image, objects, people, locations, and context. Be thorough.",
        "images": [b64],
        "stream": false,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .ok()?;

    let resp = client
        .post(format!("{}/api/generate", ollama_url))
        .json(&payload)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        log::warn!("Vision: Ollama returned {}", resp.status());
        return None;
    }

    let body: serde_json::Value = resp.json().await.ok()?;
    let description = body.get("response")?.as_str()?.to_string();

    if description.trim().is_empty() {
        return None;
    }

    let mut metadata = HashMap::new();
    metadata.insert("format".into(), "image_vision".into());
    metadata.insert("source_file".into(), file_name.clone());
    metadata.insert("vision_model".into(), "llava".into());

    let text = format!("Image: {}\n\n{}", file_name, description);

    let file_ts = std::fs::metadata(path).ok()
        .and_then(|m| m.modified().ok())
        .map(chrono::DateTime::<chrono::Utc>::from);

    log::info!("Vision: described {} ({} chars)", path.display(), description.len());

    Some(ExtractedContent { text, metadata, timestamp: file_ts })
}

// ─── Helpers ───────────────────────────────────────────────────────────────────

/// Strip XML/HTML tags, keeping only text content.
fn strip_xml_tags(xml: &str) -> String {
    let mut result = String::with_capacity(xml.len() / 2);
    let mut in_tag = false;
    let mut last_was_space = true;

    for ch in xml.chars() {
        if ch == '<' {
            in_tag = true;
            if !last_was_space && !result.is_empty() {
                result.push(' ');
                last_was_space = true;
            }
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            if ch.is_whitespace() {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            } else {
                result.push(ch);
                last_was_space = false;
            }
        }
    }

    // Decode common XML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#10;", "\n")
}

fn strip_html_tags(html: &str) -> String {
    strip_xml_tags(html)
}

/// Extract text between <tag>...</tag> from XML string.
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let after_open = xml[start..].find('>')? + start + 1;
    let end = xml[after_open..].find(&close)? + after_open;
    let value = xml[after_open..end].trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}
