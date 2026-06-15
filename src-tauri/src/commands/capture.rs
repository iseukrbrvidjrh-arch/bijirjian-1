use tauri::State;

use crate::{
    application::services::{
        CaptureService, DefaultCaptureService, DefaultPdfCaptureService, PdfCaptureService,
    },
    commands::dto::SourceDto,
    error::AppError,
    infrastructure::{
        database::repositories::{SqliteSourceRepository, SqliteWorkspaceRepository},
        pdf::PdfExtractAdapter,
    },
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

#[tauri::command]
pub fn capture_pdf_source(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<SourceDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let extractor = PdfExtractAdapter::new();
    let service =
        DefaultPdfCaptureService::new(&workspace_repository, &source_repository, &extractor);

    service.capture_pdf_source(file_path).map(Into::into)
}
