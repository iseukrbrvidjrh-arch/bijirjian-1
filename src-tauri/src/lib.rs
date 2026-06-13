pub mod application;
pub mod commands;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod state;
pub mod worker;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let database = infrastructure::database::initialize(app.handle())?;
            app.manage(AppState::new(database));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Second Brain OS");
}
