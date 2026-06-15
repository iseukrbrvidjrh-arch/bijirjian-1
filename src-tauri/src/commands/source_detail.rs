use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{DefaultSourceDetailService, SourceDetail, SourceDetailService},
    commands::{dto::SourceDto, knowledge::KnowledgeNodeDto, summary::LatestSourceSummaryDto},
    error::AppError,
    infrastructure::database::repositories::{
        SqliteAiRunRepository, SqliteKnowledgeRepository, SqliteSourceRepository,
        SqliteWorkspaceRepository,
    },
    state::AppState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDetailDto {
    source: SourceDto,
    latest_summary: Option<LatestSourceSummaryDto>,
    related_knowledge: Option<KnowledgeNodeDto>,
}

impl From<SourceDetail> for SourceDetailDto {
    fn from(detail: SourceDetail) -> Self {
        Self {
            source: detail.source.into(),
            latest_summary: detail.latest_summary.map(Into::into),
            related_knowledge: detail.related_knowledge.map(Into::into),
        }
    }
}

#[tauri::command]
pub fn get_source_detail(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<SourceDetailDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let ai_run_repository = SqliteAiRunRepository::new(&state.database);
    let knowledge_repository = SqliteKnowledgeRepository::new(&state.database);
    let service = DefaultSourceDetailService::new(
        &workspace_repository,
        &source_repository,
        &ai_run_repository,
        &knowledge_repository,
    );

    service.get_source_detail(source_id).map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::SourceDetailDto;
    use crate::{
        application::services::SourceDetail,
        domain::{
            AiRun, AiRunStatus, InboxStatus, KnowledgeNode, KnowledgeStatus, KnowledgeType,
            ProviderType, Source, SourceType,
        },
    };

    #[test]
    fn source_detail_dto_uses_camel_case_without_sensitive_data() {
        let dto = SourceDetailDto::from(SourceDetail {
            source: Source {
                id: "source-1".to_owned(),
                workspace_id: "workspace-1".to_owned(),
                source_type: SourceType::Pdf,
                raw_content: "Extracted PDF text".to_owned(),
                content_hash: "hash".to_owned(),
                metadata_json: Some(
                    r#"{"originalFileName":"guide.pdf","capturedVia":"pdf"}"#.to_owned(),
                ),
                inbox_status: InboxStatus::Unprocessed,
                captured_at: "2026-06-15T00:00:00.000Z".to_owned(),
                processed_at: None,
                created_at: "2026-06-15T00:00:00.000Z".to_owned(),
                updated_at: "2026-06-15T00:00:00.000Z".to_owned(),
                deleted_at: None,
            },
            latest_summary: Some(AiRun {
                id: "run-1".to_owned(),
                source_id: "source-1".to_owned(),
                prompt_version_id: Some("prompt-version-1".to_owned()),
                prompt_version: Some(1),
                provider_type: Some(ProviderType::DeepSeek),
                model: Some("deepseek-v4-flash".to_owned()),
                status: AiRunStatus::Succeeded,
                output_text: Some("Summary".to_owned()),
                error_message: None,
                created_at: "2026-06-15T00:00:01.000Z".to_owned(),
                completed_at: "2026-06-15T00:00:02.000Z".to_owned(),
            }),
            related_knowledge: Some(KnowledgeNode {
                id: "knowledge-1".to_owned(),
                workspace_id: "workspace-1".to_owned(),
                ai_run_id: Some("run-1".to_owned()),
                title: "Summary".to_owned(),
                content: "Summary".to_owned(),
                knowledge_type: KnowledgeType::Insight,
                status: KnowledgeStatus::Proposed,
                created_at: "2026-06-15T00:00:03.000Z".to_owned(),
                updated_at: "2026-06-15T00:00:03.000Z".to_owned(),
                archived_at: None,
            }),
        });

        let value = serde_json::to_value(dto).expect("serialize detail DTO");
        let object = value
            .as_object()
            .expect("source detail DTO should be an object");
        let serialized = value.to_string();

        assert_eq!(object.len(), 3);
        assert!(object.contains_key("source"));
        assert!(object.contains_key("latestSummary"));
        assert!(object.contains_key("relatedKnowledge"));
        assert!(!serialized.contains("apiKey"));
        assert!(!serialized.contains("promptContent"));
        assert!(!serialized.contains("rawResponse"));
        assert!(!serialized.contains("obsidian"));
    }
}
