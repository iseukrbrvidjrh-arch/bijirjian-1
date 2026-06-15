use crate::{domain::ExportRecord, error::AppError};

pub trait ExportRecordRepository: Send + Sync {
    fn insert_success(
        &self,
        workspace_id: &str,
        knowledge_node_id: &str,
        export_path: &str,
    ) -> Result<ExportRecord, AppError>;

    fn insert_failure(
        &self,
        workspace_id: &str,
        knowledge_node_id: &str,
        export_path: Option<&str>,
        error_message: &str,
    ) -> Result<ExportRecord, AppError>;

    fn find_latest_for_knowledge(
        &self,
        workspace_id: &str,
        knowledge_node_id: &str,
    ) -> Result<Option<ExportRecord>, AppError>;
}
