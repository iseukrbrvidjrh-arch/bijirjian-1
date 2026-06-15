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

#[cfg(test)]
mod tests {
    use super::SourceDto;
    use crate::domain::{InboxStatus, Source, SourceType};

    #[test]
    fn pdf_source_dto_uses_camel_case_and_pdf_source_type() {
        let dto = SourceDto::from(Source {
            id: "source-1".to_owned(),
            workspace_id: "workspace-1".to_owned(),
            source_type: SourceType::Pdf,
            raw_content: "Extracted PDF text".to_owned(),
            content_hash: "hash".to_owned(),
            metadata_json: Some(r#"{"originalFileName":"guide.pdf"}"#.to_owned()),
            inbox_status: InboxStatus::Unprocessed,
            captured_at: "2026-06-15T00:00:00.000Z".to_owned(),
            processed_at: None,
            created_at: "2026-06-15T00:00:00.000Z".to_owned(),
            updated_at: "2026-06-15T00:00:00.000Z".to_owned(),
            deleted_at: None,
        });

        let value = serde_json::to_value(dto).expect("serialize source DTO");

        assert_eq!(value["sourceType"], "pdf");
        assert_eq!(value["workspaceId"], "workspace-1");
        assert_eq!(value["metadataJson"], r#"{"originalFileName":"guide.pdf"}"#);
        assert!(value.get("source_type").is_none());
    }
}
