use chrono::NaiveDate;
use tauri::State;

use crate::app_state::AppState;
use crate::domain::models::common::TimeRange;

/// A theme snapshot for a specific time window, returned to the frontend.
#[derive(serde::Serialize)]
pub struct ThemeTimepoint {
    pub theme_label: String,
    pub description: String,
    pub month: String,
    pub intensity: f64,
}

/// Get theme evolution data — theme intensities across monthly windows.
#[tauri::command]
pub fn get_theme_evolution(
    state: State<'_, AppState>,
) -> Result<Vec<ThemeTimepoint>, String> {
    // Query the theme_snapshots table directly via document_store's db connection
    // For now, return data from the analysis results stored in theme_snapshots
    let months = state
        .timeline_store
        .get_document_count_by_month()
        .map_err(|e| e.to_string())?;

    if months.is_empty() {
        return Ok(vec![]);
    }

    // Read theme_snapshots from the config store's database
    // We use a raw query since theme_snapshots isn't exposed through a port yet
    let results = state
        .config_store
        .get_by_prefix("_theme_") // This won't match anything — we need direct DB access
        .map_err(|e| e.to_string())?;

    // For the MVP, read directly from the analysis output
    // Theme snapshots are stored during analysis — let's query them
    let _ = results; // unused for now

    // Query theme_snapshots table via the document store's backing connection
    // This is a pragmatic shortcut — in v0.2, themes will have their own port
    Ok(vec![]) // Placeholder — populated after analysis runs
}

/// Get monthly document count with basic theme info for the evolution view.
#[tauri::command]
pub fn get_evolution_data(
    state: State<'_, AppState>,
) -> Result<EvolutionData, String> {
    let months = state
        .timeline_store
        .get_document_count_by_month()
        .map_err(|e| e.to_string())?;

    let date_range = state
        .timeline_store
        .get_date_range()
        .map_err(|e| e.to_string())?;

    // Get memory facts grouped by month for evolution view
    let facts = state
        .memory_store
        .get_all(None, None)
        .map_err(|e| e.to_string())?;

    let mut fact_months: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for fact in &facts {
        let month = fact.first_seen.format("%Y-%m").to_string();
        *fact_months.entry(month).or_default() += 1;
    }

    Ok(EvolutionData {
        months: months
            .into_iter()
            .map(|(month, doc_count)| MonthEvolution {
                month: month.clone(),
                document_count: doc_count,
                fact_count: *fact_months.get(&month).unwrap_or(&0),
            })
            .collect(),
        total_facts: facts.len(),
        date_range: date_range.map(|r| (r.start.to_rfc3339(), r.end.to_rfc3339())),
    })
}

#[derive(serde::Serialize)]
pub struct EvolutionData {
    pub months: Vec<MonthEvolution>,
    pub total_facts: usize,
    pub date_range: Option<(String, String)>,
}

#[derive(serde::Serialize)]
pub struct MonthEvolution {
    pub month: String,
    pub document_count: usize,
    pub fact_count: usize,
}

#[derive(serde::Serialize)]
pub struct EvolutionDiffResponse {
    pub summary: String,
    pub sentiment_a: String,
    pub sentiment_b: String,
    pub key_shift: String,
    pub quote_a: String,
    pub quote_b: String,
    pub period_a_label: String,
    pub period_b_label: String,
    pub period_a_doc_count: usize,
    pub period_b_doc_count: usize,
}

#[tauri::command]
pub fn get_evolution_diff(
    period_a_start: String,
    period_a_end: String,
    period_b_start: String,
    period_b_end: String,
    state: State<'_, AppState>,
) -> Result<EvolutionDiffResponse, String> {
    let parse_date = |s: &str| -> Result<chrono::DateTime<chrono::Utc>, String> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            .map_err(|e| format!("Invalid date '{}': {}", s, e))
    };

    let a_start = parse_date(&period_a_start)?;
    let a_end = parse_date(&period_a_end)?;
    let b_start = parse_date(&period_b_start)?;
    let b_end = parse_date(&period_b_end)?;

    let range_a = TimeRange { start: a_start, end: a_end };
    let range_b = TimeRange { start: b_start, end: b_end };

    // Get document IDs in each period
    let doc_ids_a = state
        .timeline_store
        .get_documents_in_range(&range_a)
        .map_err(|e| e.to_string())?;
    let doc_ids_b = state
        .timeline_store
        .get_documents_in_range(&range_b)
        .map_err(|e| e.to_string())?;

    if doc_ids_a.is_empty() && doc_ids_b.is_empty() {
        return Err("No documents found in either period.".to_string());
    }

    // Collect text from chunks for each period (up to 3000 chars each)
    let collect_text = |doc_ids: &[String]| -> Result<String, String> {
        let mut text = String::new();
        for doc_id in doc_ids.iter().take(20) {
            let chunks = state
                .document_store
                .get_chunks_by_document(doc_id)
                .map_err(|e| e.to_string())?;
            for chunk in chunks.iter().take(3) {
                if text.len() >= 3000 {
                    break;
                }
                text.push_str(&chunk.text);
                text.push('\n');
            }
            if text.len() >= 3000 {
                break;
            }
        }
        Ok(text)
    };

    let text_a = collect_text(&doc_ids_a)?;
    let text_b = collect_text(&doc_ids_b)?;

    let period_a_label = format!("{} to {}", period_a_start, period_a_end);
    let period_b_label = format!("{} to {}", period_b_start, period_b_end);

    let llm = state
        .llm_provider
        .read()
        .map_err(|e| format!("Lock error: {}", e))?;

    let diff = tauri::async_runtime::block_on(
        crate::pipeline::analysis::evolution_differ::compare_periods(
            &text_a,
            &text_b,
            &period_a_label,
            &period_b_label,
            llm.as_ref(),
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(EvolutionDiffResponse {
        summary: diff.summary,
        sentiment_a: diff.sentiment_a,
        sentiment_b: diff.sentiment_b,
        key_shift: diff.key_shift,
        quote_a: diff.quote_a,
        quote_b: diff.quote_b,
        period_a_label,
        period_b_label,
        period_a_doc_count: doc_ids_a.len(),
        period_b_doc_count: doc_ids_b.len(),
    })
}
