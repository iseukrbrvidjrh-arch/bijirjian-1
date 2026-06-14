pub mod application;
pub mod commands;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod state;
pub mod worker;

use state::AppState;
use tauri::Manager;

use crate::{
    domain::ports::WorkspaceRepository,
    infrastructure::database::repositories::SqliteWorkspaceRepository,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::capture::capture_text_source,
            commands::inbox::list_inbox_sources
        ])
        .setup(|app| {
            let database = infrastructure::database::initialize(app.handle())?;
            SqliteWorkspaceRepository::new(&database).ensure_default_workspace()?;
            app.manage(AppState::new(database));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Second Brain OS");
}
