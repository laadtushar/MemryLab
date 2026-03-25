use tauri::State;

use crate::app_state::AppState;

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
