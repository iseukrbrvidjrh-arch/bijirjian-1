use crate::{domain::Source, error::AppError};

pub trait SourceRepository: Send + Sync {
    fn insert_text_source(
        &self,
        workspace_id: &str,
        raw_content: &str,
        metadata_json: Option<&str>,
    ) -> Result<Source, AppError>;

    fn insert_pdf_source(
        &self,
        workspace_id: &str,
        extracted_text: &str,
        metadata_json: &str,
    ) -> Result<Source, AppError>;

    fn list_inbox_sources(
        &self,
        workspace_id: &str,
        query: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Source>, AppError>;

    fn count_inbox_sources(&self, workspace_id: &str) -> Result<usize, AppError>;

    fn find_source(&self, workspace_id: &str, source_id: &str) -> Result<Source, AppError>;

    fn mark_source_processed(
        &self,
        workspace_id: &str,
        source_id: &str,
    ) -> Result<Source, AppError>;

    fn mark_source_dismissed(
        &self,
        workspace_id: &str,
        source_id: &str,
    ) -> Result<Source, AppError>;
}
