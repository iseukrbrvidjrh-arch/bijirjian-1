use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{
        DefaultObsidianSettingsService, ObsidianSettingsService, ObsidianSettingsSummary,
    },
    error::AppError,
    infrastructure::database::repositories::{
        SqliteObsidianSettingsRepository, SqliteWorkspaceRepository,
    },
    state::AppState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsidianSettingsDto {
    workspace_id: String,
    vault_path: String,
    has_obsidian_directory: bool,
    created_at: String,
    updated_at: String,
}

impl From<ObsidianSettingsSummary> for ObsidianSettingsDto {
    fn from(summary: ObsidianSettingsSummary) -> Self {
        Self {
            workspace_id: summary.settings.workspace_id,
            vault_path: summary.settings.vault_path,
            has_obsidian_directory: summary.has_obsidian_directory,
            created_at: summary.settings.created_at,
            updated_at: summary.settings.updated_at,
        }
    }
}

#[tauri::command]
pub fn get_obsidian_settings(
    state: State<'_, AppState>,
) -> Result<Option<ObsidianSettingsDto>, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let settings_repository = SqliteObsidianSettingsRepository::new(&state.database);
    let service = DefaultObsidianSettingsService::new(&workspace_repository, &settings_repository);

    service
        .get_obsidian_settings()
        .map(|settings| settings.map(Into::into))
}

#[tauri::command]
pub fn save_obsidian_settings(
    vault_path: String,
    state: State<'_, AppState>,
) -> Result<ObsidianSettingsDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let settings_repository = SqliteObsidianSettingsRepository::new(&state.database);
    let service = DefaultObsidianSettingsService::new(&workspace_repository, &settings_repository);

    service.save_obsidian_settings(vault_path).map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::ObsidianSettingsDto;
    use crate::{application::services::ObsidianSettingsSummary, domain::ObsidianSettings};

    #[test]
    fn obsidian_settings_dto_uses_camel_case() {
        let dto = ObsidianSettingsDto::from(ObsidianSettingsSummary {
            settings: ObsidianSettings {
                workspace_id: "workspace-1".to_owned(),
                vault_path: "/vault".to_owned(),
                created_at: "2026-06-15T00:00:00.000Z".to_owned(),
                updated_at: "2026-06-15T01:00:00.000Z".to_owned(),
            },
            has_obsidian_directory: true,
        });
        let json = serde_json::to_value(dto).expect("serialize Obsidian settings DTO");

        assert_eq!(json["workspaceId"], "workspace-1");
        assert_eq!(json["vaultPath"], "/vault");
        assert_eq!(json["hasObsidianDirectory"], true);
        assert_eq!(json["createdAt"], "2026-06-15T00:00:00.000Z");
        assert_eq!(json["updatedAt"], "2026-06-15T01:00:00.000Z");
        assert!(json.get("vaultContents").is_none());
    }
}
