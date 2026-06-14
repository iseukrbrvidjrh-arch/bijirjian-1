use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{
        DefaultPromptService, DefaultSummaryService, SourceSummary, SummaryService,
    },
    error::AppError,
    infrastructure::database::repositories::{
        SqlitePromptRepository, SqliteProviderSettingsRepository, SqliteSourceRepository,
        SqliteWorkspaceRepository,
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

#[tauri::command]
pub fn summarize_source(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<SourceSummaryDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let prompt_repository = SqlitePromptRepository::new(&state.database);
    let settings_repository = SqliteProviderSettingsRepository::new(&state.database);
    let prompt_service = DefaultPromptService::new(&prompt_repository);
    let service = DefaultSummaryService::new(
        &workspace_repository,
        &source_repository,
        &prompt_service,
        &settings_repository,
        &state.credential_store,
        &state.provider_router,
    );

    service.summarize_source(source_id).map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::SourceSummaryDto;
    use crate::{
        application::services::SourceSummary,
        domain::{ProviderModel, ProviderType},
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
}
