use crate::{domain::Source, error::AppError};

pub trait SourceRepository: Send + Sync {
    fn insert_text_source(
        &self,
        workspace_id: &str,
        raw_content: &str,
        metadata_json: Option<&str>,
    ) -> Result<Source, AppError>;

    fn list_inbox_sources(&self, workspace_id: &str, limit: usize)
        -> Result<Vec<Source>, AppError>;
}
