use crate::domain::models::common::TimeRange;
use crate::domain::models::document::Document;
use crate::error::AppError;

/// Port for temporal indexing and range queries
pub trait ITimelineStore: Send + Sync {
    fn index_document(&self, doc: &Document) -> Result<(), AppError>;
    fn get_documents_in_range(&self, range: &TimeRange) -> Result<Vec<String>, AppError>;
    fn get_document_count_by_month(&self) -> Result<Vec<(String, usize)>, AppError>;
    fn get_document_count_by_day(&self) -> Result<Vec<(String, usize)>, AppError>;
    fn get_document_count_by_week(&self) -> Result<Vec<(String, usize)>, AppError>;
    fn get_document_count_by_year(&self) -> Result<Vec<(String, usize)>, AppError>;
    fn get_date_range(&self) -> Result<Option<TimeRange>, AppError>;
}
