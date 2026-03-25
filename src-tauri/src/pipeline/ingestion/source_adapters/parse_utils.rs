use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;

/// Strip HTML tags and decode common entities to plain text.
pub fn html_to_text(html: &str) -> String {
    // Remove script/style blocks
    let re_script = regex::Regex::new(r"(?is)<(script|style)[^>]*>.*?</\1>").unwrap();
    let text = re_script.replace_all(html, "");
    // Remove tags
    let re_tags = regex::Regex::new(r"<[^>]+>").unwrap();
    let text = re_tags.replace_all(&text, " ");
    // Decode common entities
    let text = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&#x27;", "'");
    // Decode numeric entities &#NNN;
    let re_num = regex::Regex::new(r"&#(\d+);").unwrap();
    let text = re_num.replace_all(&text, |caps: &regex::Captures| {
        caps[1]
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .map(|c| c.to_string())
            .unwrap_or_default()
    });
    // Collapse whitespace
    let re_ws = regex::Regex::new(r"\s+").unwrap();
    re_ws.replace_all(&text, " ").trim().to_string()
}

/// Parse a CSV file into rows of (header → value) maps.
pub fn parse_csv_file(path: &std::path::Path) -> Result<Vec<std::collections::HashMap<String, String>>, crate::error::AppError> {
    let content = std::fs::read_to_string(path)?;
    parse_csv_string(&content)
}

/// Parse CSV content string into rows.
pub fn parse_csv_string(content: &str) -> Result<Vec<std::collections::HashMap<String, String>>, crate::error::AppError> {
    let mut lines = content.lines();
    let header_line = match lines.next() {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let headers: Vec<String> = split_csv_line(header_line);
    let mut rows = Vec::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let values = split_csv_line(line);
        let mut row = std::collections::HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            let val = values.get(i).cloned().unwrap_or_default();
            row.insert(header.clone(), val);
        }
        rows.push(row);
    }

    Ok(rows)
}

/// Split a CSV line respecting quoted fields.
fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_quotes => {
                in_quotes = true;
            }
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Recursively extract all string values from a JSON value.
pub fn flatten_json_to_text(value: &serde_json::Value) -> String {
    let mut parts = Vec::new();
    collect_strings(value, &mut parts);
    parts.join(" ")
}

fn collect_strings(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) => {
            let trimmed = s.trim();
            if !trimmed.is_empty() && trimmed.len() > 2 {
                out.push(trimmed.to_string());
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                collect_strings(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, v) in map {
                collect_strings(v, out);
            }
        }
        _ => {}
    }
}

/// Strip Twitter's JS wrapper: `window.YTD.tweet.part0 = [...]` → `[...]`
pub fn unwrap_twitter_js(content: &str) -> Result<serde_json::Value, crate::error::AppError> {
    let trimmed = content.trim();
    // Find the first `[` or `{`
    if let Some(pos) = trimmed.find('[').or_else(|| trimmed.find('{')) {
        let json_str = &trimmed[pos..];
        Ok(serde_json::from_str(json_str)?)
    } else {
        Err(crate::error::AppError::Other("No JSON found in JS file".to_string()))
    }
}

/// Fix Facebook's broken UTF-8 encoding in JSON exports.
/// Facebook encodes UTF-8 bytes as \uXXXX sequences (e.g., \u00c3\u00a9 for é).
pub fn fix_facebook_encoding(text: &str) -> String {
    // Try to re-interpret the string as raw bytes
    let bytes: Vec<u8> = text.chars().map(|c| c as u8).collect();
    String::from_utf8(bytes).unwrap_or_else(|_| text.to_string())
}

/// Build a Document from common fields (reduces adapter boilerplate).
pub fn build_document(
    text: String,
    platform: SourcePlatform,
    timestamp: DateTime<Utc>,
    participants: Vec<String>,
    metadata: serde_json::Value,
) -> Document {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    Document {
        id: Uuid::new_v4().to_string(),
        source_platform: platform,
        raw_text: text,
        timestamp,
        participants,
        metadata,
        content_hash: hash,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_text() {
        assert_eq!(
            html_to_text("<p>Hello <b>world</b></p>"),
            "Hello world"
        );
        assert_eq!(
            html_to_text("&amp; &lt; &gt; &quot;"),
            "& < > \""
        );
        assert_eq!(
            html_to_text("<script>var x=1;</script>Real content"),
            "Real content"
        );
    }

    #[test]
    fn test_csv_parsing() {
        let csv = "name,age,city\nAlice,30,London\nBob,25,\"New York\"";
        let rows = parse_csv_string(csv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["name"], "Alice");
        assert_eq!(rows[1]["city"], "New York");
    }

    #[test]
    fn test_csv_quoted_commas() {
        let csv = "text,date\n\"Hello, world\",2024-01-01\n\"She said \"\"hi\"\"\",2024-02-01";
        let rows = parse_csv_string(csv).unwrap();
        assert_eq!(rows[0]["text"], "Hello, world");
        assert_eq!(rows[1]["text"], "She said \"hi\"");
    }

    #[test]
    fn test_flatten_json() {
        let val: serde_json::Value = serde_json::json!({
            "name": "Alice",
            "posts": [{"text": "Hello world"}, {"text": "Another post"}],
            "count": 5
        });
        let text = flatten_json_to_text(&val);
        assert!(text.contains("Alice"));
        assert!(text.contains("Hello world"));
        assert!(text.contains("Another post"));
    }

    #[test]
    fn test_unwrap_twitter_js() {
        let js = r#"window.YTD.tweet.part0 = [{"tweet": {"full_text": "Hello"}}]"#;
        let val = unwrap_twitter_js(js).unwrap();
        assert!(val.is_array());
    }

    #[test]
    fn test_build_document() {
        let doc = build_document(
            "Test text".to_string(),
            SourcePlatform::Reddit,
            Utc::now(),
            vec![],
            serde_json::json!({}),
        );
        assert_eq!(doc.raw_text, "Test text");
        assert_eq!(doc.source_platform, SourcePlatform::Reddit);
        assert!(!doc.content_hash.is_empty());
    }
}
