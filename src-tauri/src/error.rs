use std::{error::Error, fmt};

#[derive(Debug)]
pub enum AppError {
    AiProvider(String),
    Conflict(String),
    Credential(String),
    Database(rusqlite::Error),
    Io(std::io::Error),
    Migration(String),
    NotFound(String),
    State(String),
    Tauri(tauri::Error),
    Validation(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AiProvider(message) => write!(formatter, "AI provider error: {message}"),
            Self::Conflict(message) => write!(formatter, "conflict: {message}"),
            Self::Credential(message) => write!(formatter, "credential store error: {message}"),
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Migration(message) => write!(formatter, "migration error: {message}"),
            Self::NotFound(message) => write!(formatter, "not found: {message}"),
            Self::State(message) => write!(formatter, "application state error: {message}"),
            Self::Tauri(error) => write!(formatter, "Tauri error: {error}"),
            Self::Validation(message) => write!(formatter, "validation error: {message}"),
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

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
