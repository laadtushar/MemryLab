use rusqlite::params;
use tauri::{Emitter, State};

use crate::adapters::sqlite::activity_store::ActivityEntry;
use crate::app_state::AppState;
use crate::domain::models::common::TimeGranularity;
use crate::pipeline::analysis::orchestrator::{self, AnalysisConfig, AnalysisResult};
use crate::pipeline::pii_detector::PiiDetector;

#[derive(Clone, serde::Serialize)]
struct AnalysisProgress {
    stage: String,
    message: String,
}

#[tauri::command]
pub fn run_analysis(
    granularity: Option<String>,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    tracing::info!(granularity = ?granularity, "Starting analysis");
    let start = std::time::Instant::now();

    let _ = app_handle.emit("analysis-progress", AnalysisProgress {
        stage: "starting".into(), message: "Starting analysis...".into(),
    });

    let llm = state
        .llm_provider
        .read()
        .map_err(|e| format!("Lock error: {}", e))?;

    let config = AnalysisConfig {
        granularity: TimeGranularity::from_str_opt(granularity.as_deref()),
    };

    let result = tauri::async_runtime::block_on(orchestrator::run_analysis(
        state.document_store.as_ref(),
        state.timeline_store.as_ref(),
        state.memory_store.as_ref(),
        state.graph_store.as_ref(),
        llm.as_ref(),
        Some(config),
    ))
    .map_err(|e| {
        tracing::error!(error = %e, "Analysis failed");
        e.to_string()
    });

    let duration_ms = start.elapsed().as_millis() as u64;

    if let Ok(ref r) = result {
        tracing::info!(
            themes = r.themes_extracted,
            beliefs = r.beliefs_extracted,
            sentiments = r.sentiments_classified,
            entities = r.entities_extracted,
            insights = r.insights_generated,
            contradictions = r.contradictions_found,
            narratives = r.narratives_generated,
            duration_ms = duration_ms,
            "Analysis complete"
        );

        let _ = state.activity_store.log_activity(&ActivityEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            action_type: "analysis".to_string(),
            title: "Ran analysis".to_string(),
            description: String::new(),
            result_summary: format!(
                "{} themes, {} beliefs, {} entities, {} insights",
                r.themes_extracted, r.beliefs_extracted, r.entities_extracted, r.insights_generated
            ),
            metadata: serde_json::json!({}),
            duration_ms: duration_ms as i64,
            status: "success".to_string(),
        });
    }

    result
}

#[derive(serde::Serialize)]
pub struct PiiScanResult {
    pub total_scanned: usize,
    pub total_flagged: usize,
    pub flagged_facts: Vec<PiiFlaggedFact>,
}

#[derive(serde::Serialize)]
pub struct PiiFlaggedFact {
    pub fact_id: String,
    pub pii_types: Vec<String>,
}

#[tauri::command]
pub fn scan_pii(state: State<'_, AppState>) -> Result<PiiScanResult, String> {
    let detector = PiiDetector::new();

    let facts = state
        .memory_store
        .get_all(None, None)
        .map_err(|e| e.to_string())?;

    let total_scanned = facts.len();
    let mut flagged_facts = Vec::new();

    state
        .db
        .with_conn(|conn| {
            for fact in &facts {
                let pii_types = detector.scan(&fact.fact_text);
                let pii_json =
                    serde_json::to_string(&pii_types).map_err(|e| crate::error::AppError::Other(e.to_string()))?;
                conn.execute(
                    "INSERT OR REPLACE INTO pii_scan_results (fact_id, pii_types, scanned_at) VALUES (?1, ?2, datetime('now'))",
                    params![fact.id, pii_json],
                )?;
                if !pii_types.is_empty() {
                    flagged_facts.push(PiiFlaggedFact {
                        fact_id: fact.id.clone(),
                        pii_types,
                    });
                }
            }
            Ok(())
        })
        .map_err(|e| e.to_string())?;

    let total_flagged = flagged_facts.len();

    Ok(PiiScanResult {
        total_scanned,
        total_flagged,
        flagged_facts,
    })
}

#[tauri::command]
pub fn get_pii_flags(state: State<'_, AppState>) -> Result<Vec<PiiFlaggedFact>, String> {
    state
        .db
        .with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT fact_id, pii_types FROM pii_scan_results WHERE pii_types != '[]'")
                .map_err(crate::error::AppError::Database)?;
            let rows = stmt
                .query_map([], |row| {
                    let fact_id: String = row.get(0)?;
                    let pii_types_str: String = row.get(1)?;
                    Ok((fact_id, pii_types_str))
                })
                .map_err(crate::error::AppError::Database)?;

            let mut results = Vec::new();
            for row in rows {
                let (fact_id, pii_types_str) = row.map_err(crate::error::AppError::Database)?;
                let pii_types: Vec<String> = serde_json::from_str(&pii_types_str)
                    .unwrap_or_default();
                results.push(PiiFlaggedFact { fact_id, pii_types });
            }
            Ok(results)
        })
        .map_err(|e| e.to_string())
}
