use tauri::State;

use crate::{
    application::services::{CaptureService, DefaultCaptureService},
    commands::dto::SourceDto,
    error::AppError,
    infrastructure::database::repositories::{SqliteSourceRepository, SqliteWorkspaceRepository},
    state::AppState,
};

#[tauri::command]
pub fn capture_text_source(
    raw_content: String,
    state: State<'_, AppState>,
) -> Result<SourceDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let service = DefaultCaptureService::new(&workspace_repository, &source_repository);

    service.capture_text_source(raw_content).map(Into::into)
}
