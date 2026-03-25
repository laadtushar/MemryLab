use tauri::State;

use crate::app_state::AppState;

#[derive(serde::Serialize)]
pub struct TimelineDataResponse {
    pub months: Vec<MonthData>,
    pub total_documents: usize,
    pub date_range: Option<DateRange>,
}

#[derive(serde::Serialize)]
pub struct MonthData {
    pub month: String,
    pub document_count: usize,
}

#[derive(serde::Serialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(serde::Serialize)]
pub struct MemoryFactResponse {
    pub id: String,
    pub fact_text: String,
    pub confidence: f64,
    pub category: String,
    pub first_seen: String,
    pub last_updated: String,
    pub is_active: bool,
}

#[tauri::command]
pub fn get_timeline_data(
    state: State<'_, AppState>,
) -> Result<TimelineDataResponse, String> {
    let months = state
        .timeline_store
        .get_document_count_by_month()
        .map_err(|e| e.to_string())?;

    let date_range = state
        .timeline_store
        .get_date_range()
        .map_err(|e| e.to_string())?
        .map(|r| DateRange {
            start: r.start.to_rfc3339(),
            end: r.end.to_rfc3339(),
        });

    let total: usize = months.iter().map(|(_, c)| c).sum();

    Ok(TimelineDataResponse {
        months: months
            .into_iter()
            .map(|(month, count)| MonthData {
                month,
                document_count: count,
            })
            .collect(),
        total_documents: total,
        date_range,
    })
}

#[tauri::command]
pub fn get_memory_facts(
    category: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<MemoryFactResponse>, String> {
    let cat = category.as_ref().and_then(|c| match c.as_str() {
        "belief" => Some(crate::domain::models::memory::FactCategory::Belief),
        "preference" => Some(crate::domain::models::memory::FactCategory::Preference),
        "fact" => Some(crate::domain::models::memory::FactCategory::Fact),
        "self_description" => Some(crate::domain::models::memory::FactCategory::SelfDescription),
        "insight" => Some(crate::domain::models::memory::FactCategory::Insight),
        _ => None,
    });

    let facts = state
        .memory_store
        .get_all(cat.as_ref(), None)
        .map_err(|e| e.to_string())?;

    Ok(facts
        .into_iter()
        .map(|f| MemoryFactResponse {
            id: f.id,
            fact_text: f.fact_text,
            confidence: f.confidence,
            category: format!("{:?}", f.category).to_lowercase(),
            first_seen: f.first_seen.to_rfc3339(),
            last_updated: f.last_updated.to_rfc3339(),
            is_active: f.is_active,
        })
        .collect())
}

#[tauri::command]
pub fn delete_memory_fact(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .memory_store
        .forget(&id)
        .map_err(|e| e.to_string())
}
