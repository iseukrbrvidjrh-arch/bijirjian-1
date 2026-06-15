use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{
        AiProviderService, AiProviderSettingsSummary, DefaultAiProviderService,
        ProviderConnectionResult,
    },
    domain::{validate_model_id, ProviderModelInfo, ProviderModelListResult, ProviderType},
    error::AppError,
    infrastructure::database::repositories::SqliteProviderSettingsRepository,
    state::AppState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderSettingsDto {
    provider_type: String,
    default_model: String,
    has_api_key: bool,
    updated_at: String,
}

impl From<AiProviderSettingsSummary> for AiProviderSettingsDto {
    fn from(settings: AiProviderSettingsSummary) -> Self {
        Self {
            provider_type: settings.provider_type.to_string(),
            default_model: settings.default_model,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelInfoDto {
    id: String,
    label: String,
    provider_type: String,
    source: String,
}

impl From<ProviderModelInfo> for ProviderModelInfoDto {
    fn from(model: ProviderModelInfo) -> Self {
        Self {
            id: model.id,
            label: model.label,
            provider_type: model.provider_type.to_string(),
            source: match model.source {
                crate::domain::ModelSource::Remote => "remote".to_owned(),
                crate::domain::ModelSource::Builtin => "builtin".to_owned(),
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelListDto {
    models: Vec<ProviderModelInfoDto>,
    used_fallback: bool,
    fallback_reason: Option<String>,
}

impl From<ProviderModelListResult> for ProviderModelListDto {
    fn from(result: ProviderModelListResult) -> Self {
        Self {
            models: result.models.into_iter().map(Into::into).collect(),
            used_fallback: result.used_fallback,
            fallback_reason: result.fallback_reason,
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
    default_model: String,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<AiProviderSettingsDto, AppError> {
    let provider_type =
        ProviderType::try_from(provider_type.as_str()).map_err(AppError::Validation)?;
    let default_model = validate_model_id(&default_model)?;
    let repository = SqliteProviderSettingsRepository::new(&state.database);
    let service =
        DefaultAiProviderService::new(&repository, &state.credential_store, &state.provider_router);

    service
        .save_settings(provider_type, default_model, api_key)
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

#[tauri::command]
pub fn list_ai_provider_models(
    provider_type: String,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<ProviderModelListDto, AppError> {
    let provider_type =
        ProviderType::try_from(provider_type.as_str()).map_err(AppError::Validation)?;
    let repository = SqliteProviderSettingsRepository::new(&state.database);
    let service =
        DefaultAiProviderService::new(&repository, &state.credential_store, &state.provider_router);

    service.list_models(provider_type, api_key).map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::AiProviderSettingsDto;
    use crate::{application::services::AiProviderSettingsSummary, domain::ProviderType};

    #[test]
    fn settings_dto_never_contains_an_api_key() {
        let dto = AiProviderSettingsDto::from(AiProviderSettingsSummary {
            provider_type: ProviderType::DeepSeek,
            default_model: "deepseek-v4-flash".to_owned(),
            has_api_key: true,
            updated_at: "2026-06-14T00:00:00.000Z".to_owned(),
        });
        let json = serde_json::to_value(dto).expect("serialize settings DTO");

        assert_eq!(json["providerType"], "deepseek");
        assert_eq!(json["defaultModel"], "deepseek-v4-flash");
        assert_eq!(json["hasApiKey"], true);
        assert!(json.get("apiKey").is_none());
    }
}
