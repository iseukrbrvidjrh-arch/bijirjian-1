pub mod migrations;
pub mod repositories;

use std::{fs, sync::Mutex, time::Duration};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::AppError;

pub struct Database {
    connection: Mutex<Connection>,
}

pub fn initialize(app: &AppHandle) -> Result<Database, AppError> {
    let app_data_dir = app.path().app_data_dir()?;
    fs::create_dir_all(&app_data_dir)?;

    Database::from_connection(Connection::open(
        app_data_dir.join("second-brain-os.sqlite3"),
    )?)
}

impl Database {
    pub(crate) fn from_connection(mut connection: Connection) -> Result<Self, AppError> {
        connection.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            ",
        )?;
        connection.busy_timeout(Duration::from_secs(5))?;
        migrations::run(&mut connection)?;

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub(crate) fn with_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| AppError::State("database connection lock is poisoned".to_owned()))?;

        operation(&mut connection)
    }
}
