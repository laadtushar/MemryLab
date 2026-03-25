use crate::domain::models::common::TimeRange;
use crate::domain::models::entity::{Entity, Relationship};
use crate::error::AppError;

/// A subgraph returned from graph queries
#[derive(Debug, Clone)]
pub struct SubGraph {
    pub nodes: Vec<Entity>,
    pub edges: Vec<Relationship>,
}

/// Port for entity and relationship graph
pub trait IGraphStore: Send + Sync {
    fn add_node(&self, entity: &Entity) -> Result<(), AppError>;
    fn add_edge(&self, relationship: &Relationship) -> Result<(), AppError>;
    fn get_neighbors(
        &self,
        node_id: &str,
        depth: usize,
        rel_type: Option<&str>,
    ) -> Result<SubGraph, AppError>;
    fn get_by_time_range(&self, range: &TimeRange) -> Result<SubGraph, AppError>;
    fn get_all_entities(
        &self,
        limit: usize,
        type_filter: Option<&str>,
    ) -> Result<SubGraph, AppError>;
    fn find_entity_by_name(&self, name: &str) -> Result<Option<Entity>, AppError>;
    fn update_entity(&self, entity: &Entity) -> Result<(), AppError>;
}
