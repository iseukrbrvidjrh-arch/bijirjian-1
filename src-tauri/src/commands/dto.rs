use serde::Serialize;

use crate::domain::Source;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDto {
    pub id: String,
    pub workspace_id: String,
    pub source_type: String,
    pub raw_content: String,
    pub content_hash: String,
    pub metadata_json: Option<String>,
    pub inbox_status: String,
    pub captured_at: String,
    pub processed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl From<Source> for SourceDto {
    fn from(source: Source) -> Self {
        Self {
            id: source.id,
            workspace_id: source.workspace_id,
            source_type: source.source_type.to_string(),
            raw_content: source.raw_content,
            content_hash: source.content_hash,
            metadata_json: source.metadata_json,
            inbox_status: source.inbox_status.to_string(),
            captured_at: source.captured_at,
            processed_at: source.processed_at,
            created_at: source.created_at,
            updated_at: source.updated_at,
            deleted_at: source.deleted_at,
        }
    }
}
