pub mod migrations;
pub mod repositories;

use std::{fs, sync::Mutex, time::Duration};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::AppError;

pub struct Database {
    _connection: Mutex<Connection>,
}

pub fn initialize(app: &AppHandle) -> Result<Database, AppError> {
    let app_data_dir = app.path().app_data_dir()?;
    fs::create_dir_all(&app_data_dir)?;

    let mut connection = Connection::open(app_data_dir.join("second-brain-os.sqlite3"))?;
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        ",
    )?;
    connection.busy_timeout(Duration::from_secs(5))?;
    migrations::run(&mut connection)?;

    Ok(Database {
        _connection: Mutex::new(connection),
    })
}
