use std::{error::Error, fmt};

#[derive(Debug)]
pub enum AppError {
    Database(rusqlite::Error),
    Io(std::io::Error),
    Migration(String),
    Tauri(tauri::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Migration(message) => write!(formatter, "migration error: {message}"),
            Self::Tauri(error) => write!(formatter, "Tauri error: {error}"),
        }
    }
}

impl Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<tauri::Error> for AppError {
    fn from(error: tauri::Error) -> Self {
        Self::Tauri(error)
    }
}
