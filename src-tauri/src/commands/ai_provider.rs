use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{
        AiProviderService, AiProviderSettingsSummary, DefaultAiProviderService,
        ProviderConnectionResult,
    },
    domain::ProviderType,
    error::AppError,
    infrastructure::database::repositories::SqliteProviderSettingsRepository,
    state::AppState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderSettingsDto {
    provider_type: String,
    has_api_key: bool,
    updated_at: String,
}

impl From<AiProviderSettingsSummary> for AiProviderSettingsDto {
    fn from(settings: AiProviderSettingsSummary) -> Self {
        Self {
            provider_type: settings.provider_type.to_string(),
            has_api_key: settings.has_api_key,
            updated_at: settings.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConnectionResultDto {
    provider_type: String,
    message: String,
}

impl From<ProviderConnectionResult> for ProviderConnectionResultDto {
    fn from(result: ProviderConnectionResult) -> Self {
        Self {
            provider_type: result.provider_type.to_string(),
            message: result.message,
        }
    }
}

#[tauri::command]
pub fn get_ai_provider_settings(
    state: State<'_, AppState>,
) -> Result<Option<AiProviderSettingsDto>, AppError> {
    let repository = SqliteProviderSettingsRepository::new(&state.database);
    let service =
        DefaultAiProviderService::new(&repository, &state.credential_store, &state.provider_router);

    service
        .get_settings()
        .map(|settings| settings.map(Into::into))
}

#[tauri::command]
pub fn save_ai_provider_settings(
    provider_type: String,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<AiProviderSettingsDto, AppError> {
    let provider_type =
        ProviderType::try_from(provider_type.as_str()).map_err(AppError::Validation)?;
    let repository = SqliteProviderSettingsRepository::new(&state.database);
    let service =
        DefaultAiProviderService::new(&repository, &state.credential_store, &state.provider_router);

    service
        .save_settings(provider_type, api_key)
        .map(Into::into)
}

#[tauri::command]
pub fn test_ai_provider_connection(
    state: State<'_, AppState>,
) -> Result<ProviderConnectionResultDto, AppError> {
    let repository = SqliteProviderSettingsRepository::new(&state.database);
    let service =
        DefaultAiProviderService::new(&repository, &state.credential_store, &state.provider_router);

    service.test_connection().map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::AiProviderSettingsDto;
    use crate::{application::services::AiProviderSettingsSummary, domain::ProviderType};

    #[test]
    fn settings_dto_never_contains_an_api_key() {
        let dto = AiProviderSettingsDto::from(AiProviderSettingsSummary {
            provider_type: ProviderType::DeepSeek,
            has_api_key: true,
            updated_at: "2026-06-14T00:00:00.000Z".to_owned(),
        });
        let json = serde_json::to_value(dto).expect("serialize settings DTO");

        assert_eq!(json["providerType"], "deepseek");
        assert_eq!(json["hasApiKey"], true);
        assert!(json.get("apiKey").is_none());
    }
}
