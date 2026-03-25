use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct AmazonAdapter;

impl SourceAdapter for AmazonAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "amazon".into(),
            display_name: "Amazon".into(),
            icon: "shopping-cart".into(),
            takeout_url: Some("https://www.amazon.com/hz/privacy-central/data-requests/preview.html".into()),
            instructions: "Request your data from Amazon (Privacy Central > Request Your Data). Download and upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "csv".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Amazon,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_order = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("retail.orderhistory")
                || lower.contains("digital-ordering")
                || lower.contains("retail-order")
        });
        let has_audible = file_listing.iter().any(|f| {
            f.to_lowercase().contains("audible")
        });
        let has_kindle = file_listing.iter().any(|f| {
            let lower = f.to_lowercase();
            lower.contains("kindle") || lower.contains("reading")
        });

        if has_order { 0.9 }
        else if has_audible || has_kindle { 0.8 }
        else { 0.0 }
    }

    fn name(&self) -> &str {
        "amazon"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let ext = entry.path()
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let rel_path = entry.path().to_string_lossy().replace('\\', "/").to_lowercase();

            match ext.as_str() {
                "csv" => {
                    if rel_path.contains("order") || rel_path.contains("retail") {
                        parse_order_csv(entry.path(), &mut documents);
                    } else {
                        parse_generic_csv(entry.path(), &mut documents);
                    }
                }
                "json" => parse_amazon_json(entry.path(), &rel_path, &mut documents),
                _ => {}
            }
        }

        Ok(documents)
    }
}

fn parse_order_csv(path: &Path, docs: &mut Vec<Document>) {
    let rows = match parse_utils::parse_csv_file(path) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Skipping Amazon CSV {}: {}", path.display(), e);
            return;
        }
    };

    for row in rows {
        let product = row.get("Product Name")
            .or_else(|| row.get("Title"))
            .or_else(|| row.get("Item Description"))
            .cloned()
            .unwrap_or_default();

        let category = row.get("Category")
            .or_else(|| row.get("Product Category"))
            .cloned()
            .unwrap_or_default();

        let text = if product.is_empty() {
            // Fallback: join all values
            row.values().cloned().collect::<Vec<_>>().join(", ")
        } else if category.is_empty() {
            format!("Ordered: {}", product)
        } else {
            format!("Ordered: {} ({})", product, category)
        };

        if text.trim().is_empty() {
            continue;
        }

        let timestamp = row.get("Order Date")
            .or_else(|| row.get("Purchase Date"))
            .or_else(|| row.get("date"))
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| {
                        chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y")
                            .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                            .ok()
                            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
                    })
            })
            .unwrap_or_else(Utc::now);

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String("order".into()));
        if !product.is_empty() {
            meta.insert("product".into(), serde_json::Value::String(product));
        }

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Amazon,
            timestamp,
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_generic_csv(path: &Path, docs: &mut Vec<Document>) {
    let rows = match parse_utils::parse_csv_file(path) {
        Ok(r) => r,
        Err(_) => return,
    };

    let file_name = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let doc_type = if file_name.to_lowercase().contains("audible") {
        "audiobook"
    } else if file_name.to_lowercase().contains("kindle") {
        "highlight"
    } else {
        "amazon_data"
    };

    for row in rows {
        let text: String = row.values().cloned().collect::<Vec<_>>().join(", ");
        if text.trim().is_empty() {
            continue;
        }

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String(doc_type.into()));
        meta.insert("source_file".into(), serde_json::Value::String(file_name.clone()));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Amazon,
            Utc::now(),
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

fn parse_amazon_json(path: &Path, rel_path: &str, docs: &mut Vec<Document>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    let doc_type = if rel_path.contains("kindle") {
        "highlight"
    } else if rel_path.contains("audible") {
        "audiobook"
    } else {
        "amazon_data"
    };

    let items = if let Some(arr) = value.as_array() {
        arr.clone()
    } else {
        vec![value]
    };

    for item in &items {
        let text = parse_utils::flatten_json_to_text(item);
        if text.trim().is_empty() {
            continue;
        }

        let mut meta = serde_json::Map::new();
        meta.insert("type".into(), serde_json::Value::String(doc_type.into()));

        docs.push(parse_utils::build_document(
            text,
            SourcePlatform::Amazon,
            Utc::now(),
            vec![],
            serde_json::Value::Object(meta),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_amazon() {
        let adapter = AmazonAdapter;
        let files = vec!["Retail.OrderHistory.1/Retail.OrderHistory.1.csv"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_audible() {
        let adapter = AmazonAdapter;
        let files = vec!["Audible/Library.csv"];
        assert!(adapter.detect(&files) >= 0.8);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = AmazonAdapter;
        assert!(adapter.detect(&["random.csv"]) < 0.1);
    }
}
