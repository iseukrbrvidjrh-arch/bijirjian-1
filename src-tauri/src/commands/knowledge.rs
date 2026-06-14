use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{
        DefaultKnowledgeDraftService, DefaultKnowledgeService, KnowledgeDraftService,
        KnowledgeService,
    },
    domain::KnowledgeNode,
    error::AppError,
    infrastructure::database::repositories::{
        SqliteAiRunRepository, SqliteKnowledgeRepository, SqliteSourceRepository,
        SqliteWorkspaceRepository,
    },
    state::AppState,
};

const DEFAULT_KNOWLEDGE_LIMIT: usize = 50;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeNodeDto {
    id: String,
    workspace_id: String,
    ai_run_id: Option<String>,
    title: String,
    content: String,
    knowledge_type: String,
    status: String,
    created_at: String,
    updated_at: String,
    archived_at: Option<String>,
}

impl From<KnowledgeNode> for KnowledgeNodeDto {
    fn from(node: KnowledgeNode) -> Self {
        Self {
            id: node.id,
            workspace_id: node.workspace_id,
            ai_run_id: node.ai_run_id,
            title: node.title,
            content: node.content,
            knowledge_type: node.knowledge_type.to_string(),
            status: node.status.to_string(),
            created_at: node.created_at,
            updated_at: node.updated_at,
            archived_at: node.archived_at,
        }
    }
}

#[tauri::command]
pub fn accept_knowledge_node(
    knowledge_id: String,
    state: State<'_, AppState>,
) -> Result<KnowledgeNodeDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let knowledge_repository = SqliteKnowledgeRepository::new(&state.database);
    let service = DefaultKnowledgeService::new(&workspace_repository, &knowledge_repository);

    service.accept_knowledge_node(knowledge_id).map(Into::into)
}

#[tauri::command]
pub fn archive_knowledge_node(
    knowledge_id: String,
    state: State<'_, AppState>,
) -> Result<KnowledgeNodeDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let knowledge_repository = SqliteKnowledgeRepository::new(&state.database);
    let service = DefaultKnowledgeService::new(&workspace_repository, &knowledge_repository);

    service.archive_knowledge_node(knowledge_id).map(Into::into)
}

#[tauri::command]
pub fn create_knowledge_draft_from_latest_summary(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<KnowledgeNodeDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let ai_run_repository = SqliteAiRunRepository::new(&state.database);
    let knowledge_repository = SqliteKnowledgeRepository::new(&state.database);
    let service = DefaultKnowledgeDraftService::new(
        &workspace_repository,
        &source_repository,
        &ai_run_repository,
        &knowledge_repository,
    );

    service
        .create_from_latest_summary(source_id)
        .map(Into::into)
}

#[tauri::command]
pub fn create_knowledge_node(
    title: String,
    content: String,
    knowledge_type: String,
    state: State<'_, AppState>,
) -> Result<KnowledgeNodeDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let knowledge_repository = SqliteKnowledgeRepository::new(&state.database);
    let service = DefaultKnowledgeService::new(&workspace_repository, &knowledge_repository);

    service
        .create_knowledge_node(title, content, knowledge_type)
        .map(Into::into)
}

#[tauri::command]
pub fn list_knowledge_nodes(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<KnowledgeNodeDto>, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let knowledge_repository = SqliteKnowledgeRepository::new(&state.database);
    let service = DefaultKnowledgeService::new(&workspace_repository, &knowledge_repository);

    service
        .list_knowledge_nodes(limit.unwrap_or(DEFAULT_KNOWLEDGE_LIMIT))
        .map(|nodes| nodes.into_iter().map(Into::into).collect())
}

#[cfg(test)]
mod tests {
    use super::KnowledgeNodeDto;
    use crate::domain::{KnowledgeNode, KnowledgeStatus, KnowledgeType};

    #[test]
    fn knowledge_node_dto_uses_camel_case() {
        let dto = KnowledgeNodeDto::from(KnowledgeNode {
            id: "knowledge-1".to_owned(),
            workspace_id: "workspace-1".to_owned(),
            ai_run_id: Some("ai-run-1".to_owned()),
            title: "Local First".to_owned(),
            content: "Data stays local.".to_owned(),
            knowledge_type: KnowledgeType::Concept,
            status: KnowledgeStatus::Archived,
            created_at: "2026-06-14T00:00:00.000Z".to_owned(),
            updated_at: "2026-06-14T01:00:00.000Z".to_owned(),
            archived_at: Some("2026-06-14T01:00:00.000Z".to_owned()),
        });
        let json = serde_json::to_value(dto).expect("serialize knowledge DTO");
        let object = json.as_object().expect("knowledge DTO should be an object");

        assert_eq!(json["id"], "knowledge-1");
        assert_eq!(json["workspaceId"], "workspace-1");
        assert_eq!(json["aiRunId"], "ai-run-1");
        assert_eq!(json["knowledgeType"], "concept");
        assert_eq!(json["status"], "archived");
        assert_eq!(json["createdAt"], "2026-06-14T00:00:00.000Z");
        assert_eq!(json["updatedAt"], "2026-06-14T01:00:00.000Z");
        assert_eq!(json["archivedAt"], "2026-06-14T01:00:00.000Z");
        assert_eq!(object.len(), 10);
        assert!(object.get("apiKey").is_none());
        assert!(object.get("sourceContent").is_none());
        assert!(object.get("rawResponse").is_none());
    }
}
