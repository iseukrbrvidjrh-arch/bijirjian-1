use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{
        DefaultPromptService, DefaultSummaryService, SourceSummary, SummaryService,
    },
    domain::AiRun,
    error::AppError,
    infrastructure::database::repositories::{
        SqliteAiRunRepository, SqlitePromptRepository, SqliteProviderSettingsRepository,
        SqliteSourceRepository, SqliteWorkspaceRepository,
    },
    state::AppState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSummaryDto {
    source_id: String,
    summary: String,
    provider_type: String,
    model: String,
    prompt_version_id: String,
    prompt_version: i64,
}

impl From<SourceSummary> for SourceSummaryDto {
    fn from(summary: SourceSummary) -> Self {
        Self {
            source_id: summary.source_id,
            summary: summary.summary,
            provider_type: summary.provider_type.to_string(),
            model: summary.model.to_string(),
            prompt_version_id: summary.prompt_version_id,
            prompt_version: summary.prompt_version,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestSourceSummaryDto {
    run_id: String,
    source_id: String,
    summary: Option<String>,
    status: String,
    error_message: Option<String>,
    provider_type: Option<String>,
    model: Option<String>,
    prompt_version_id: Option<String>,
    prompt_version: Option<i64>,
    created_at: String,
    completed_at: String,
}

impl From<AiRun> for LatestSourceSummaryDto {
    fn from(ai_run: AiRun) -> Self {
        Self {
            run_id: ai_run.id,
            source_id: ai_run.source_id,
            summary: ai_run.output_text,
            status: ai_run.status.to_string(),
            error_message: ai_run.error_message,
            provider_type: ai_run
                .provider_type
                .map(|provider_type| provider_type.to_string()),
            model: ai_run.model.map(|model| model.to_string()),
            prompt_version_id: ai_run.prompt_version_id,
            prompt_version: ai_run.prompt_version,
            created_at: ai_run.created_at,
            completed_at: ai_run.completed_at,
        }
    }
}

#[tauri::command]
pub fn summarize_source(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<SourceSummaryDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let prompt_repository = SqlitePromptRepository::new(&state.database);
    let settings_repository = SqliteProviderSettingsRepository::new(&state.database);
    let ai_run_repository = SqliteAiRunRepository::new(&state.database);
    let prompt_service = DefaultPromptService::new(&prompt_repository);
    let service = DefaultSummaryService::new(
        &workspace_repository,
        &source_repository,
        &prompt_service,
        &settings_repository,
        &state.credential_store,
        &state.provider_router,
        &ai_run_repository,
    );

    service.summarize_source(source_id).map(Into::into)
}

#[tauri::command]
pub fn get_latest_source_summary(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<Option<LatestSourceSummaryDto>, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let prompt_repository = SqlitePromptRepository::new(&state.database);
    let settings_repository = SqliteProviderSettingsRepository::new(&state.database);
    let ai_run_repository = SqliteAiRunRepository::new(&state.database);
    let prompt_service = DefaultPromptService::new(&prompt_repository);
    let service = DefaultSummaryService::new(
        &workspace_repository,
        &source_repository,
        &prompt_service,
        &settings_repository,
        &state.credential_store,
        &state.provider_router,
        &ai_run_repository,
    );

    service
        .get_latest_source_summary(source_id)
        .map(|ai_run| ai_run.map(Into::into))
}

#[cfg(test)]
mod tests {
    use super::{LatestSourceSummaryDto, SourceSummaryDto};
    use crate::{
        application::services::SourceSummary,
        domain::{AiRun, AiRunStatus, ProviderModel, ProviderType},
    };

    #[test]
    fn summary_dto_uses_camel_case_without_sensitive_inputs() {
        let dto = SourceSummaryDto::from(SourceSummary {
            source_id: "source-1".to_owned(),
            summary: "Summary text".to_owned(),
            provider_type: ProviderType::DeepSeek,
            model: ProviderModel::DeepSeekV4Flash,
            prompt_version_id: "prompt-version-1".to_owned(),
            prompt_version: 1,
        });
        let json = serde_json::to_value(dto).expect("serialize summary DTO");
        let object = json.as_object().expect("summary DTO should be an object");

        assert_eq!(json["sourceId"], "source-1");
        assert_eq!(json["summary"], "Summary text");
        assert_eq!(json["providerType"], "deepseek");
        assert_eq!(json["model"], "deepseek-v4-flash");
        assert_eq!(json["promptVersionId"], "prompt-version-1");
        assert_eq!(json["promptVersion"], 1);
        assert_eq!(object.len(), 6);
        assert!(object.get("apiKey").is_none());
        assert!(object.get("promptContent").is_none());
        assert!(object.get("rawContent").is_none());
    }

    #[test]
    fn latest_summary_dto_uses_camel_case_without_sensitive_inputs() {
        let dto = LatestSourceSummaryDto::from(AiRun {
            id: "run-1".to_owned(),
            source_id: "source-1".to_owned(),
            prompt_version_id: Some("prompt-version-1".to_owned()),
            prompt_version: Some(1),
            provider_type: Some(ProviderType::DeepSeek),
            model: Some(ProviderModel::DeepSeekV4Flash),
            status: AiRunStatus::Succeeded,
            output_text: Some("Persisted summary".to_owned()),
            error_message: None,
            created_at: "2026-06-14T00:00:00.000Z".to_owned(),
            completed_at: "2026-06-14T00:00:01.000Z".to_owned(),
        });
        let json = serde_json::to_value(dto).expect("serialize latest summary DTO");
        let object = json
            .as_object()
            .expect("latest summary DTO should be an object");

        assert_eq!(json["runId"], "run-1");
        assert_eq!(json["sourceId"], "source-1");
        assert_eq!(json["summary"], "Persisted summary");
        assert_eq!(json["status"], "succeeded");
        assert_eq!(json["providerType"], "deepseek");
        assert_eq!(json["model"], "deepseek-v4-flash");
        assert_eq!(json["promptVersionId"], "prompt-version-1");
        assert_eq!(json["promptVersion"], 1);
        assert_eq!(json["createdAt"], "2026-06-14T00:00:00.000Z");
        assert_eq!(json["completedAt"], "2026-06-14T00:00:01.000Z");
        assert_eq!(object.len(), 11);
        assert!(object.get("apiKey").is_none());
        assert!(object.get("promptContent").is_none());
        assert!(object.get("rawContent").is_none());
        assert!(object.get("rawResponse").is_none());
    }
}
