use tauri::State;

use crate::app_state::AppState;

#[derive(serde::Serialize)]
pub struct EntityResponse {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub mention_count: u32,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

#[derive(serde::Serialize)]
pub struct EntityGraphResponse {
    pub entities: Vec<EntityResponse>,
    pub relationships: Vec<RelationshipResponse>,
}

#[derive(serde::Serialize)]
pub struct RelationshipResponse {
    pub id: String,
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub rel_type: String,
    pub weight: f64,
}

/// List all entities sorted by mention count (most mentioned first).
#[tauri::command]
pub fn list_entities(
    entity_type: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<EntityResponse>, String> {
    // Use a time range that covers everything
    let range = crate::domain::models::common::TimeRange {
        start: chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
        end: chrono::Utc::now(),
    };

    let subgraph = state
        .graph_store
        .get_by_time_range(&range)
        .map_err(|e| e.to_string())?;

    let mut entities: Vec<EntityResponse> = subgraph
        .nodes
        .into_iter()
        .filter(|e| {
            entity_type.as_ref().map_or(true, |t| {
                format!("{:?}", e.entity_type).to_lowercase() == t.to_lowercase()
            })
        })
        .map(|e| EntityResponse {
            id: e.id,
            name: e.name,
            entity_type: format!("{:?}", e.entity_type).to_lowercase(),
            mention_count: e.mention_count,
            first_seen: e.first_seen.map(|d| d.to_rfc3339()),
            last_seen: e.last_seen.map(|d| d.to_rfc3339()),
        })
        .collect();

    entities.sort_by(|a, b| b.mention_count.cmp(&a.mention_count));
    Ok(entities)
}

/// Get the full entity graph (all entities up to limit, plus their relationships).
#[tauri::command]
pub fn get_full_graph(
    limit: Option<usize>,
    entity_type: Option<String>,
    state: State<'_, AppState>,
) -> Result<EntityGraphResponse, String> {
    let max = limit.unwrap_or(200);
    let subgraph = state
        .graph_store
        .get_all_entities(max, entity_type.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(EntityGraphResponse {
        entities: subgraph
            .nodes
            .into_iter()
            .map(|e| EntityResponse {
                id: e.id,
                name: e.name,
                entity_type: format!("{:?}", e.entity_type).to_lowercase(),
                mention_count: e.mention_count,
                first_seen: e.first_seen.map(|d| d.to_rfc3339()),
                last_seen: e.last_seen.map(|d| d.to_rfc3339()),
            })
            .collect(),
        relationships: subgraph
            .edges
            .into_iter()
            .map(|r| RelationshipResponse {
                id: r.id,
                source_entity_id: r.source_entity_id,
                target_entity_id: r.target_entity_id,
                rel_type: r.rel_type,
                weight: r.weight,
            })
            .collect(),
    })
}

/// Get an entity's neighborhood (connected entities and relationships).
#[tauri::command]
pub fn get_entity_graph(
    entity_id: String,
    depth: Option<usize>,
    state: State<'_, AppState>,
) -> Result<EntityGraphResponse, String> {
    let d = depth.unwrap_or(1);
    let subgraph = state
        .graph_store
        .get_neighbors(&entity_id, d, None)
        .map_err(|e| e.to_string())?;

    Ok(EntityGraphResponse {
        entities: subgraph
            .nodes
            .into_iter()
            .map(|e| EntityResponse {
                id: e.id,
                name: e.name,
                entity_type: format!("{:?}", e.entity_type).to_lowercase(),
                mention_count: e.mention_count,
                first_seen: e.first_seen.map(|d| d.to_rfc3339()),
                last_seen: e.last_seen.map(|d| d.to_rfc3339()),
            })
            .collect(),
        relationships: subgraph
            .edges
            .into_iter()
            .map(|r| RelationshipResponse {
                id: r.id,
                source_entity_id: r.source_entity_id,
                target_entity_id: r.target_entity_id,
                rel_type: r.rel_type,
                weight: r.weight,
            })
            .collect(),
    })
}
