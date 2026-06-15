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
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::ai_provider::get_ai_provider_settings,
            commands::ai_provider::save_ai_provider_settings,
            commands::ai_provider::test_ai_provider_connection,
            commands::capture::capture_text_source,
            commands::capture::capture_pdf_source,
            commands::dashboard::get_dashboard_summary,
            commands::export::export_knowledge_node,
            commands::export::get_latest_export_record_for_knowledge,
            commands::inbox::list_inbox_sources,
            commands::inbox::mark_source_processed,
            commands::inbox::mark_source_dismissed,
            commands::knowledge::accept_knowledge_node,
            commands::knowledge::archive_knowledge_node,
            commands::knowledge::create_knowledge_node,
            commands::knowledge::create_knowledge_draft_from_latest_summary,
            commands::knowledge::list_knowledge_nodes,
            commands::obsidian_settings::get_obsidian_settings,
            commands::obsidian_settings::save_obsidian_settings,
            commands::prompt::get_default_prompt,
            commands::prompt::list_prompt_versions,
            commands::prompt::create_prompt_version,
            commands::prompt::set_active_prompt_version,
            commands::summary::summarize_source,
            commands::summary::get_latest_source_summary,
            commands::source_detail::get_source_detail
        ])
        .setup(|app| {
            let database = infrastructure::database::initialize(app.handle())?;
            SqliteWorkspaceRepository::new(&database).ensure_default_workspace()?;
            let credential_store = infrastructure::keyring::SystemCredentialStore::new()?;
            let provider_router = infrastructure::ai::DefaultProviderRouter::new()?;
            app.manage(AppState::new(database, credential_store, provider_router));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Second Brain OS");
}
