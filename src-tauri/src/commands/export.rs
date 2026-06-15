use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{DefaultExportService, ExportService},
    domain::ExportRecord,
    error::AppError,
    infrastructure::{
        database::repositories::{
            SqliteExportRecordRepository, SqliteKnowledgeRepository,
            SqliteObsidianSettingsRepository, SqliteWorkspaceRepository,
        },
        obsidian::FileSystemKnowledgeMarkdownWriter,
    },
    state::AppState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecordDto {
    id: String,
    workspace_id: String,
    knowledge_node_id: String,
    export_path: Option<String>,
    status: String,
    error_message: Option<String>,
    created_at: String,
}

impl From<ExportRecord> for ExportRecordDto {
    fn from(record: ExportRecord) -> Self {
        Self {
            id: record.id,
            workspace_id: record.workspace_id,
            knowledge_node_id: record.knowledge_node_id,
            export_path: record.export_path,
            status: record.status.to_string(),
            error_message: record.error_message,
            created_at: record.created_at,
        }
    }
}

#[tauri::command]
pub fn export_knowledge_node(
    knowledge_id: String,
    state: State<'_, AppState>,
) -> Result<ExportRecordDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let knowledge_repository = SqliteKnowledgeRepository::new(&state.database);
    let settings_repository = SqliteObsidianSettingsRepository::new(&state.database);
    let export_repository = SqliteExportRecordRepository::new(&state.database);
    let markdown_writer = FileSystemKnowledgeMarkdownWriter::new();
    let service = DefaultExportService::new(
        &workspace_repository,
        &knowledge_repository,
        &settings_repository,
        &export_repository,
        &markdown_writer,
    );

    service.export_knowledge_node(knowledge_id).map(Into::into)
}

#[tauri::command]
pub fn get_latest_export_record_for_knowledge(
    knowledge_id: String,
    state: State<'_, AppState>,
) -> Result<Option<ExportRecordDto>, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let knowledge_repository = SqliteKnowledgeRepository::new(&state.database);
    let settings_repository = SqliteObsidianSettingsRepository::new(&state.database);
    let export_repository = SqliteExportRecordRepository::new(&state.database);
    let markdown_writer = FileSystemKnowledgeMarkdownWriter::new();
    let service = DefaultExportService::new(
        &workspace_repository,
        &knowledge_repository,
        &settings_repository,
        &export_repository,
        &markdown_writer,
    );

    service
        .get_latest_export_record_for_knowledge(knowledge_id)
        .map(|record| record.map(Into::into))
}

#[cfg(test)]
mod tests {
    use super::ExportRecordDto;
    use crate::domain::{ExportRecord, ExportStatus};

    #[test]
    fn export_record_dto_uses_camel_case_and_excludes_sensitive_data() {
        let dto = ExportRecordDto::from(ExportRecord {
            id: "export-1".to_owned(),
            workspace_id: "workspace-1".to_owned(),
            knowledge_node_id: "knowledge-1".to_owned(),
            export_path: Some("/vault/SecondBrainOS/Knowledge/node.md".to_owned()),
            status: ExportStatus::Succeeded,
            error_message: None,
            created_at: "2026-06-15T00:00:00.000Z".to_owned(),
        });
        let json = serde_json::to_value(dto).expect("serialize export record DTO");
        let object = json.as_object().expect("export DTO should be an object");

        assert_eq!(json["workspaceId"], "workspace-1");
        assert_eq!(json["knowledgeNodeId"], "knowledge-1");
        assert_eq!(json["exportPath"], "/vault/SecondBrainOS/Knowledge/node.md");
        assert_eq!(json["status"], "succeeded");
        assert_eq!(json["createdAt"], "2026-06-15T00:00:00.000Z");
        assert_eq!(object.len(), 7);
        assert!(object.get("apiKey").is_none());
        assert!(object.get("sourceContent").is_none());
        assert!(object.get("rawResponse").is_none());
    }

    #[test]
    fn latest_export_response_serializes_none_as_null() {
        let response: Option<ExportRecordDto> = None;
        let json = serde_json::to_value(response).expect("serialize empty latest export response");

        assert!(json.is_null());
    }
}
