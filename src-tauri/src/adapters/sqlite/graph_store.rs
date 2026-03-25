use std::sync::Arc;

use rusqlite::{params, OptionalExtension};

use crate::domain::models::common::TimeRange;
use crate::domain::models::entity::{Entity, EntityType, Relationship};
use crate::domain::ports::graph_store::{IGraphStore, SubGraph};
use crate::error::AppError;

use super::connection::SqliteConnection;

pub struct SqliteGraphStore {
    db: Arc<SqliteConnection>,
}

impl SqliteGraphStore {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

impl IGraphStore for SqliteGraphStore {
    fn add_node(&self, entity: &Entity) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO entities (id, name, entity_type, first_seen, last_seen, mention_count, aliases, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    entity.id,
                    entity.name,
                    entity_type_to_str(&entity.entity_type),
                    entity.first_seen.map(|d| d.to_rfc3339()),
                    entity.last_seen.map(|d| d.to_rfc3339()),
                    entity.mention_count,
                    serde_json::to_string(&entity.aliases)?,
                    entity.metadata.to_string(),
                ],
            )?;
            Ok(())
        })
    }

    fn add_edge(&self, rel: &Relationship) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO relationships (id, source_entity_id, target_entity_id, rel_type, weight, first_seen, context_chunks)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    rel.id,
                    rel.source_entity_id,
                    rel.target_entity_id,
                    rel.rel_type,
                    rel.weight,
                    rel.first_seen.map(|d| d.to_rfc3339()),
                    serde_json::to_string(&rel.context_chunks)?,
                ],
            )?;
            Ok(())
        })
    }

    fn get_neighbors(
        &self,
        node_id: &str,
        depth: usize,
        rel_type: Option<&str>,
    ) -> Result<SubGraph, AppError> {
        self.db.with_conn(|conn| {
            // Use recursive CTE for multi-hop traversal
            let rel_filter = match rel_type {
                Some(rt) => format!("AND r.rel_type = '{}'", rt.replace('\'', "''")),
                None => String::new(),
            };

            let sql = format!(
                "WITH RECURSIVE neighbors(entity_id, depth) AS (
                    SELECT ?1, 0
                    UNION
                    SELECT CASE
                        WHEN r.source_entity_id = neighbors.entity_id THEN r.target_entity_id
                        ELSE r.source_entity_id
                    END, neighbors.depth + 1
                    FROM relationships r
                    JOIN neighbors ON (r.source_entity_id = neighbors.entity_id OR r.target_entity_id = neighbors.entity_id)
                        {}
                    WHERE neighbors.depth < ?2
                )
                SELECT DISTINCT e.id, e.name, e.entity_type, e.first_seen, e.last_seen, e.mention_count, e.aliases, e.metadata
                FROM neighbors n
                JOIN entities e ON e.id = n.entity_id",
                rel_filter
            );

            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![node_id, depth as i64], |row| {
                Ok(row_to_entity(row))
            })?;
            let mut nodes = Vec::new();
            for row in rows {
                nodes.push(row??);
            }

            // Get edges between the found nodes
            let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
            let edges = if node_ids.is_empty() {
                Vec::new()
            } else {
                get_edges_between(conn, &node_ids)?
            };

            Ok(SubGraph { nodes, edges })
        })
    }

    fn get_by_time_range(&self, range: &TimeRange) -> Result<SubGraph, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, entity_type, first_seen, last_seen, mention_count, aliases, metadata
                 FROM entities
                 WHERE first_seen IS NOT NULL AND first_seen >= ?1 AND first_seen <= ?2
                 ORDER BY first_seen ASC",
            )?;
            let rows = stmt.query_map(
                params![range.start.to_rfc3339(), range.end.to_rfc3339()],
                |row| Ok(row_to_entity(row)),
            )?;
            let mut nodes = Vec::new();
            for row in rows {
                nodes.push(row??);
            }

            let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
            let edges = if node_ids.is_empty() {
                Vec::new()
            } else {
                get_edges_between(conn, &node_ids)?
            };

            Ok(SubGraph { nodes, edges })
        })
    }

    fn get_all_entities(
        &self,
        limit: usize,
        type_filter: Option<&str>,
    ) -> Result<SubGraph, AppError> {
        self.db.with_conn(|conn| {
            let (sql, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
                match type_filter {
                    Some(et) => (
                        "SELECT id, name, entity_type, first_seen, last_seen, mention_count, aliases, metadata \
                         FROM entities WHERE entity_type = ?1 ORDER BY mention_count DESC LIMIT ?2"
                            .to_string(),
                        vec![
                            Box::new(et.to_string()) as Box<dyn rusqlite::types::ToSql>,
                            Box::new(limit as i64),
                        ],
                    ),
                    None => (
                        "SELECT id, name, entity_type, first_seen, last_seen, mention_count, aliases, metadata \
                         FROM entities ORDER BY mention_count DESC LIMIT ?1"
                            .to_string(),
                        vec![Box::new(limit as i64) as Box<dyn rusqlite::types::ToSql>],
                    ),
                };

            let params_refs: Vec<&dyn rusqlite::types::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_refs.as_slice(), |row| Ok(row_to_entity(row)))?;
            let mut nodes = Vec::new();
            for row in rows {
                nodes.push(row??);
            }

            let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
            let edges = if node_ids.is_empty() {
                Vec::new()
            } else {
                get_edges_between(conn, &node_ids)?
            };

            Ok(SubGraph { nodes, edges })
        })
    }

    fn find_entity_by_name(&self, name: &str) -> Result<Option<Entity>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, entity_type, first_seen, last_seen, mention_count, aliases, metadata
                 FROM entities WHERE name = ?1 COLLATE NOCASE",
            )?;
            let result = stmt
                .query_row(params![name], |row| Ok(row_to_entity(row)))
                .optional()?;
            match result {
                Some(entity) => Ok(Some(entity?)),
                None => Ok(None),
            }
        })
    }

    fn update_entity(&self, entity: &Entity) -> Result<(), AppError> {
        self.add_node(entity)
    }
}

fn entity_type_to_str(et: &EntityType) -> &'static str {
    match et {
        EntityType::Person => "person",
        EntityType::Place => "place",
        EntityType::Organization => "organization",
        EntityType::Concept => "concept",
        EntityType::Topic => "topic",
    }
}

fn str_to_entity_type(s: &str) -> EntityType {
    match s {
        "person" => EntityType::Person,
        "place" => EntityType::Place,
        "organization" => EntityType::Organization,
        "concept" => EntityType::Concept,
        "topic" => EntityType::Topic,
        _ => EntityType::Concept,
    }
}

fn row_to_entity(row: &rusqlite::Row) -> Result<Entity, AppError> {
    let entity_type_str: String = row.get(2)?;
    let first_seen_str: Option<String> = row.get(3)?;
    let last_seen_str: Option<String> = row.get(4)?;
    let aliases_str: String = row.get(6)?;
    let metadata_str: String = row.get(7)?;

    Ok(Entity {
        id: row.get(0)?,
        name: row.get(1)?,
        entity_type: str_to_entity_type(&entity_type_str),
        first_seen: first_seen_str
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&chrono::Utc)),
        last_seen: last_seen_str
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&chrono::Utc)),
        mention_count: row.get::<_, i64>(5)? as u32,
        aliases: serde_json::from_str(&aliases_str)?,
        metadata: serde_json::from_str(&metadata_str)?,
    })
}

fn get_edges_between(
    conn: &rusqlite::Connection,
    node_ids: &[String],
) -> Result<Vec<Relationship>, AppError> {
    if node_ids.is_empty() {
        return Ok(Vec::new());
    }
    let n = node_ids.len();
    let placeholders_a: Vec<String> = (1..=n).map(|i| format!("?{}", i)).collect();
    let placeholders_b: Vec<String> = (n + 1..=2 * n).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "SELECT id, source_entity_id, target_entity_id, rel_type, weight, first_seen, context_chunks
         FROM relationships
         WHERE source_entity_id IN ({}) AND target_entity_id IN ({})",
        placeholders_a.join(", "),
        placeholders_b.join(", ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut all_params: Vec<&dyn rusqlite::types::ToSql> = Vec::new();
    for id in node_ids {
        all_params.push(id as &dyn rusqlite::types::ToSql);
    }
    for id in node_ids {
        all_params.push(id as &dyn rusqlite::types::ToSql);
    }
    let rows = stmt.query_map(all_params.as_slice(), |row| {
        let first_seen_str: Option<String> = row.get(5)?;
        let context_str: String = row.get(6)?;
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, f64>(4)?,
            first_seen_str,
            context_str,
        ))
    })?;

    let mut edges = Vec::new();
    for row in rows {
        let (id, source, target, rel_type, weight, first_seen_str, context_str) = row?;
        edges.push(Relationship {
            id,
            source_entity_id: source,
            target_entity_id: target,
            rel_type,
            weight,
            first_seen: first_seen_str
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&chrono::Utc)),
            context_chunks: serde_json::from_str(&context_str)?,
        });
    }
    Ok(edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_add_and_find_entity() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteGraphStore::new(db);

        let entity = Entity {
            id: "e1".to_string(),
            name: "Alice".to_string(),
            entity_type: EntityType::Person,
            first_seen: Some(Utc::now()),
            last_seen: Some(Utc::now()),
            mention_count: 5,
            aliases: vec!["Ali".to_string()],
            metadata: serde_json::json!({}),
        };
        store.add_node(&entity).unwrap();

        let found = store.find_entity_by_name("Alice").unwrap().unwrap();
        assert_eq!(found.id, "e1");
        assert_eq!(found.mention_count, 5);
    }

    #[test]
    fn test_add_edge_and_get_neighbors() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteGraphStore::new(db);

        let alice = Entity {
            id: "e1".to_string(),
            name: "Alice".to_string(),
            entity_type: EntityType::Person,
            first_seen: Some(Utc::now()),
            last_seen: None,
            mention_count: 1,
            aliases: vec![],
            metadata: serde_json::json!({}),
        };
        let bob = Entity {
            id: "e2".to_string(),
            name: "Bob".to_string(),
            entity_type: EntityType::Person,
            first_seen: Some(Utc::now()),
            last_seen: None,
            mention_count: 1,
            aliases: vec![],
            metadata: serde_json::json!({}),
        };
        store.add_node(&alice).unwrap();
        store.add_node(&bob).unwrap();

        let rel = Relationship {
            id: "r1".to_string(),
            source_entity_id: "e1".to_string(),
            target_entity_id: "e2".to_string(),
            rel_type: "knows".to_string(),
            weight: 1.0,
            first_seen: Some(Utc::now()),
            context_chunks: vec!["c1".to_string()],
        };
        store.add_edge(&rel).unwrap();

        let subgraph = store.get_neighbors("e1", 1, None).unwrap();
        assert_eq!(subgraph.nodes.len(), 2); // Alice and Bob
        assert_eq!(subgraph.edges.len(), 1);
    }
}
