use tauri::State;

use crate::{
    application::services::{DefaultInboxService, InboxService},
    commands::dto::SourceDto,
    error::AppError,
    infrastructure::database::repositories::{SqliteSourceRepository, SqliteWorkspaceRepository},
    state::AppState,
};

const DEFAULT_INBOX_LIMIT: usize = 50;

#[tauri::command]
pub fn list_inbox_sources(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SourceDto>, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let service = DefaultInboxService::new(&workspace_repository, &source_repository);

    service
        .list_inbox_sources(limit.unwrap_or(DEFAULT_INBOX_LIMIT))
        .map(|sources| sources.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub fn mark_source_processed(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<SourceDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let service = DefaultInboxService::new(&workspace_repository, &source_repository);

    service.mark_source_processed(source_id).map(Into::into)
}

#[tauri::command]
pub fn mark_source_dismissed(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<SourceDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let service = DefaultInboxService::new(&workspace_repository, &source_repository);

    service.mark_source_dismissed(source_id).map(Into::into)
}
