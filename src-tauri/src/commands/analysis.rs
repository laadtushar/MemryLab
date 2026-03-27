use rusqlite::params;
use tauri::{Emitter, Manager, State};

use crate::adapters::sqlite::activity_store::ActivityEntry;
use crate::app_state::AppState;
use crate::domain::models::common::TimeGranularity;
use crate::pipeline::analysis::orchestrator::{self, AnalysisConfig, AnalysisResult};
use crate::pipeline::pii_detector::PiiDetector;
use crate::services::task_manager::TaskManager;

#[derive(Clone, serde::Serialize)]
struct AnalysisProgress {
    stage: String,
    message: String,
}

#[tauri::command]
pub async fn run_analysis(
    granularity: Option<String>,
    task_id: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<AnalysisResult, String> {
    tracing::info!(granularity = ?granularity, "Starting analysis");
    let start = std::time::Instant::now();

    // Check for last analysis timestamp for incremental mode
    let last_analysis = {
        let state = app_handle.state::<AppState>();
        state.config_store.get("analysis.last_run_at").ok().flatten()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
    };

    let config = AnalysisConfig {
        granularity: TimeGranularity::from_str_opt(granularity.as_deref()),
        since: last_analysis,
    };

    let mgr = app_handle.state::<TaskManager>();
    let id = task_id.unwrap_or_else(|| format!("analysis-{}", uuid::Uuid::new_v4()));
    let token = mgr.register_task(&id, "analysis", "Running analysis pipeline");
    let ah2 = app_handle.clone();
    let id2 = id.clone();

    // Spawn on blocking thread pool — retrieve state via app_handle inside
    let result = tokio::task::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let llm = state.llm_provider.read().map_err(|e| format!("Lock error: {}", e))?;
        let ah = app_handle.clone();
        let ct = token.clone();
        let result = tauri::async_runtime::block_on(orchestrator::run_analysis_with_progress(
            state.document_store.as_ref(),
            state.timeline_store.as_ref(),
            state.memory_store.as_ref(),
            state.graph_store.as_ref(),
            llm.as_ref(),
            Some(config),
            move |stage, message| {
                if ct.is_cancelled() { return; }
                let _ = ah.emit("analysis-progress", AnalysisProgress {
                    stage: stage.to_string(),
                    message: message.to_string(),
                });
            },
        ))
        .map_err(|e| {
            tracing::error!(error = %e, "Analysis failed");
            e.to_string()
        })?;

        // Check if cancelled during execution
        if token.is_cancelled() {
            return Err("Task cancelled".to_string());
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        tracing::info!(
            themes = result.themes_extracted,
            beliefs = result.beliefs_extracted,
            sentiments = result.sentiments_classified,
            entities = result.entities_extracted,
            insights = result.insights_generated,
            contradictions = result.contradictions_found,
            narratives = result.narratives_generated,
            duration_ms = duration_ms,
            "Analysis complete"
        );

        // Save last analysis timestamp for incremental mode
        let _ = state.config_store.set("analysis.last_run_at", &chrono::Utc::now().to_rfc3339());

        let _ = state.activity_store.log_activity(&ActivityEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            action_type: "analysis".to_string(),
            title: "Ran analysis".to_string(),
            description: String::new(),
            result_summary: format!(
                "{} themes, {} beliefs, {} entities, {} insights",
                result.themes_extracted, result.beliefs_extracted, result.entities_extracted, result.insights_generated
            ),
            metadata: serde_json::json!({}),
            duration_ms: duration_ms as i64,
            status: "success".to_string(),
        });

        Ok(result)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    let mgr2 = ah2.state::<TaskManager>();
    match &result {
        Ok(_) => mgr2.complete_task(&id2, None),
        Err(e) => mgr2.complete_task(&id2, Some(e)),
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
pub async fn scan_pii(app_handle: tauri::AppHandle) -> Result<PiiScanResult, String> {
    tokio::task::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        scan_pii_blocking(&state)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

fn scan_pii_blocking(state: &AppState) -> Result<PiiScanResult, String> {
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
