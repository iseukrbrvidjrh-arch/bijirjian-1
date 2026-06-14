use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{DefaultPromptDetails, DefaultPromptService, PromptService},
    domain::PromptVersion,
    error::AppError,
    infrastructure::database::repositories::SqlitePromptRepository,
    state::AppState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptVersionDto {
    id: String,
    prompt_id: String,
    version: i64,
    prompt_content: String,
    created_at: String,
}

impl From<PromptVersion> for PromptVersionDto {
    fn from(version: PromptVersion) -> Self {
        Self {
            id: version.id,
            prompt_id: version.prompt_id,
            version: version.version,
            prompt_content: version.prompt_content,
            created_at: version.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultPromptDto {
    id: String,
    prompt_key: String,
    name: String,
    description: Option<String>,
    active_version_id: String,
    active_version: PromptVersionDto,
    created_at: String,
    updated_at: String,
}

impl From<DefaultPromptDetails> for DefaultPromptDto {
    fn from(details: DefaultPromptDetails) -> Self {
        let active_version_id = details.active_version.id.clone();

        Self {
            id: details.prompt.id,
            prompt_key: details.prompt.prompt_key,
            name: details.prompt.name,
            description: details.prompt.description,
            active_version_id,
            active_version: details.active_version.into(),
            created_at: details.prompt.created_at,
            updated_at: details.prompt.updated_at,
        }
    }
}

#[tauri::command]
pub fn get_default_prompt(state: State<'_, AppState>) -> Result<DefaultPromptDto, AppError> {
    let repository = SqlitePromptRepository::new(&state.database);
    let service = DefaultPromptService::new(&repository);

    service.get_default_prompt().map(Into::into)
}

#[tauri::command]
pub fn list_prompt_versions(state: State<'_, AppState>) -> Result<Vec<PromptVersionDto>, AppError> {
    let repository = SqlitePromptRepository::new(&state.database);
    let service = DefaultPromptService::new(&repository);

    service
        .list_default_prompt_versions()
        .map(|versions| versions.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub fn create_prompt_version(
    prompt_content: String,
    state: State<'_, AppState>,
) -> Result<PromptVersionDto, AppError> {
    let repository = SqlitePromptRepository::new(&state.database);
    let service = DefaultPromptService::new(&repository);

    service
        .create_default_prompt_version(prompt_content)
        .map(Into::into)
}

#[tauri::command]
pub fn set_active_prompt_version(
    version_id: String,
    state: State<'_, AppState>,
) -> Result<DefaultPromptDto, AppError> {
    let repository = SqlitePromptRepository::new(&state.database);
    let service = DefaultPromptService::new(&repository);

    service
        .set_default_prompt_active_version(&version_id)
        .map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::DefaultPromptDto;
    use crate::{
        application::services::DefaultPromptDetails,
        domain::{Prompt, PromptVersion},
    };

    #[test]
    fn prompt_dto_uses_camel_case() {
        let active_version = PromptVersion {
            id: "version-1".to_owned(),
            prompt_id: "prompt-1".to_owned(),
            version: 1,
            prompt_content: "Prompt content".to_owned(),
            created_at: "2026-06-14T00:00:00.000Z".to_owned(),
        };
        let dto = DefaultPromptDto::from(DefaultPromptDetails {
            prompt: Prompt {
                id: "prompt-1".to_owned(),
                prompt_key: "source_summary".to_owned(),
                name: "Source Summary".to_owned(),
                description: Some("Description".to_owned()),
                active_version_id: Some("version-1".to_owned()),
                created_at: "2026-06-14T00:00:00.000Z".to_owned(),
                updated_at: "2026-06-14T00:00:00.000Z".to_owned(),
            },
            active_version,
        });
        let json = serde_json::to_value(dto).expect("serialize prompt DTO");

        assert_eq!(json["promptKey"], "source_summary");
        assert_eq!(json["activeVersionId"], "version-1");
        assert_eq!(json["activeVersion"]["promptId"], "prompt-1");
        assert_eq!(json["activeVersion"]["promptContent"], "Prompt content");
        assert!(json.get("prompt_key").is_none());
        assert!(json["activeVersion"].get("prompt_content").is_none());
    }
}
