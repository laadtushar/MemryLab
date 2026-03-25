use chrono::{DateTime, Datelike, Duration, Utc};
use uuid::Uuid;

use crate::domain::models::common::{TimeGranularity, TimeRange};
use crate::domain::models::document::Chunk;
use crate::domain::models::theme::ThemeSnapshot;
use crate::domain::ports::document_store::IDocumentStore;
use crate::domain::ports::llm_provider::{ILlmProvider, LlmParams};
use crate::domain::ports::timeline_store::ITimelineStore;
use crate::error::AppError;
use crate::prompts::templates::{render_template, THEME_EXTRACTION_V1};

/// Extract themes from chunks grouped by time windows based on granularity.
pub async fn extract_themes(
    document_store: &dyn IDocumentStore,
    timeline_store: &dyn ITimelineStore,
    llm: &dyn ILlmProvider,
    max_chunks_per_window: usize,
    granularity: &TimeGranularity,
) -> Result<Vec<ThemeSnapshot>, AppError> {
    let date_range = timeline_store
        .get_date_range()?
        .ok_or_else(|| AppError::Analysis("No documents to analyze".to_string()))?;

    let windows = generate_windows(&date_range, granularity);
    let mut all_themes = Vec::new();

    for window in &windows {
        let doc_ids = timeline_store.get_documents_in_range(window)?;
        if doc_ids.is_empty() {
            continue;
        }

        // Gather chunks from documents in this window
        let mut window_chunks: Vec<Chunk> = Vec::new();
        for doc_id in &doc_ids {
            let chunks = document_store.get_chunks_by_document(doc_id)?;
            window_chunks.extend(chunks);
        }

        if window_chunks.is_empty() {
            continue;
        }

        // Limit chunks to avoid exceeding token budget
        window_chunks.truncate(max_chunks_per_window);

        let chunks_text = window_chunks
            .iter()
            .enumerate()
            .map(|(i, c)| format!("[{}] {}", i + 1, c.text))
            .collect::<Vec<_>>()
            .join("\n\n");

        let window_label = format!(
            "{} to {}",
            window.start.format("%Y-%m-%d"),
            window.end.format("%Y-%m-%d")
        );

        let prompt = render_template(
            THEME_EXTRACTION_V1,
            &[
                ("time_window", &window_label),
                ("chunk_count", &window_chunks.len().to_string()),
                ("chunks", &chunks_text),
            ],
        );

        let params = LlmParams {
            temperature: Some(0.3),
            max_tokens: Some(2048),
            ..Default::default()
        };

        match llm.complete(&prompt, &params).await {
            Ok(response) => {
                let themes = parse_theme_response(&response, window);
                all_themes.extend(themes);
            }
            Err(e) => {
                log::warn!("Theme extraction failed for {}: {}", window_label, e);
            }
        }
    }

    Ok(all_themes)
}

/// Parse the LLM JSON response into ThemeSnapshot records.
fn parse_theme_response(response: &str, window: &TimeRange) -> Vec<ThemeSnapshot> {
    // Try to extract JSON array from response (LLM may include extra text)
    let json_str = extract_json_array(response);
    let parsed: Result<Vec<ThemeEntry>, _> = serde_json::from_str(&json_str);

    match parsed {
        Ok(entries) => entries
            .into_iter()
            .map(|entry| ThemeSnapshot {
                id: Uuid::new_v4().to_string(),
                theme_label: entry.label,
                description: Some(entry.description),
                time_window_start: window.start,
                time_window_end: window.end,
                intensity_score: entry.intensity_score.clamp(0.0, 1.0),
                representative_chunks: vec![],
                created_at: Utc::now(),
            })
            .collect(),
        Err(e) => {
            log::warn!("Failed to parse theme response: {}", e);
            Vec::new()
        }
    }
}

#[derive(serde::Deserialize)]
struct ThemeEntry {
    label: String,
    description: String,
    intensity_score: f64,
}

/// Generate time windows from a date range using the given granularity.
fn generate_windows(range: &TimeRange, granularity: &TimeGranularity) -> Vec<TimeRange> {
    match granularity {
        TimeGranularity::Monthly => generate_monthly_windows(range),
        _ => {
            let days = granularity.window_days();
            let mut windows = Vec::new();
            let mut current = range.start;
            while current < range.end {
                let next = current + Duration::days(days);
                windows.push(TimeRange {
                    start: current,
                    end: std::cmp::min(next, range.end),
                });
                current = next;
            }
            windows
        }
    }
}

/// Generate monthly time windows from a date range.
fn generate_monthly_windows(range: &TimeRange) -> Vec<TimeRange> {
    let mut windows = Vec::new();
    let mut current = first_of_month(range.start);

    while current < range.end {
        let next = next_month(current);
        windows.push(TimeRange {
            start: current,
            end: std::cmp::min(next, range.end),
        });
        current = next;
    }

    windows
}

fn first_of_month(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt.with_day(1)
        .unwrap_or(dt)
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
}

fn next_month(dt: DateTime<Utc>) -> DateTime<Utc> {
    if dt.month() == 12 {
        dt.with_year(dt.year() + 1)
            .and_then(|d| d.with_month(1))
            .unwrap_or(dt + Duration::days(31))
    } else {
        dt.with_month(dt.month() + 1)
            .unwrap_or(dt + Duration::days(31))
    }
}

/// Extract a JSON array from a response that may contain extra text.
fn extract_json_array(text: &str) -> String {
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            return text[start..=end].to_string();
        }
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_monthly_windows() {
        let range = TimeRange {
            start: chrono::TimeZone::with_ymd_and_hms(&Utc, 2024, 1, 15, 0, 0, 0).unwrap(),
            end: chrono::TimeZone::with_ymd_and_hms(&Utc, 2024, 4, 10, 0, 0, 0).unwrap(),
        };
        let windows = generate_monthly_windows(&range);
        assert_eq!(windows.len(), 4); // Jan, Feb, Mar, Apr
    }

    #[test]
    fn test_parse_theme_response() {
        let response = r#"[
            {"label": "Career anxiety", "description": "Worrying about job prospects", "intensity_score": 0.8},
            {"label": "Health focus", "description": "Interest in exercise and diet", "intensity_score": 0.6}
        ]"#;
        let window = TimeRange {
            start: Utc::now(),
            end: Utc::now(),
        };
        let themes = parse_theme_response(response, &window);
        assert_eq!(themes.len(), 2);
        assert_eq!(themes[0].theme_label, "Career anxiety");
        assert!((themes[0].intensity_score - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_extract_json_array_with_preamble() {
        let text = "Here are the themes:\n[{\"label\":\"test\"}]\n\nDone.";
        let json = extract_json_array(text);
        assert_eq!(json, "[{\"label\":\"test\"}]");
    }
}
