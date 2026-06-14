use std::io;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, types::Type};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    domain::{ports::SourceRepository, InboxStatus, Source, SourceType},
    error::AppError,
    infrastructure::database::Database,
};

pub struct SqliteSourceRepository<'database> {
    database: &'database Database,
}

impl<'database> SqliteSourceRepository<'database> {
    pub const fn new(database: &'database Database) -> Self {
        Self { database }
    }
}

impl SourceRepository for SqliteSourceRepository<'_> {
    fn insert_text_source(
        &self,
        workspace_id: &str,
        raw_content: &str,
        metadata_json: Option<&str>,
    ) -> Result<Source, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = current_timestamp();
        let content_hash = content_hash(raw_content);
        let metadata_json = metadata_json.unwrap_or("{}").to_owned();

        self.database.with_connection(|connection| {
            connection.execute(
                "
                INSERT INTO sources (
                    id,
                    workspace_id,
                    source_type,
                    raw_content,
                    content_hash,
                    metadata_json,
                    inbox_status,
                    captured_at,
                    processed_at,
                    created_at,
                    updated_at,
                    deleted_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?8, ?8, NULL)
                ",
                params![
                    id,
                    workspace_id,
                    SourceType::Text.as_str(),
                    raw_content,
                    content_hash,
                    metadata_json,
                    InboxStatus::Unprocessed.as_str(),
                    now,
                ],
            )?;

            Ok(Source {
                id,
                workspace_id: workspace_id.to_owned(),
                source_type: SourceType::Text,
                raw_content: raw_content.to_owned(),
                content_hash,
                metadata_json: Some(metadata_json),
                inbox_status: InboxStatus::Unprocessed,
                captured_at: now.clone(),
                processed_at: None,
                created_at: now.clone(),
                updated_at: now,
                deleted_at: None,
            })
        })
    }

    fn list_inbox_sources(
        &self,
        workspace_id: &str,
        limit: usize,
    ) -> Result<Vec<Source>, AppError> {
        let limit = i64::try_from(limit)
            .map_err(|_| AppError::Validation("inbox limit is too large".to_owned()))?;

        self.database.with_connection(|connection| {
            let mut statement = connection.prepare(
                "
                SELECT
                    id,
                    workspace_id,
                    source_type,
                    raw_content,
                    content_hash,
                    metadata_json,
                    inbox_status,
                    captured_at,
                    processed_at,
                    created_at,
                    updated_at,
                    deleted_at
                FROM sources
                WHERE workspace_id = ?1
                  AND inbox_status = ?2
                  AND deleted_at IS NULL
                ORDER BY captured_at DESC
                LIMIT ?3
                ",
            )?;

            let sources = statement
                .query_map(
                    params![workspace_id, InboxStatus::Unprocessed.as_str(), limit],
                    map_source,
                )?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(sources)
        })
    }
}

fn map_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<Source> {
    let source_type: String = row.get(2)?;
    let inbox_status: String = row.get(6)?;

    Ok(Source {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        source_type: SourceType::try_from(source_type.as_str())
            .map_err(|message| invalid_enum_value(2, message))?,
        raw_content: row.get(3)?,
        content_hash: row.get(4)?,
        metadata_json: row.get(5)?,
        inbox_status: InboxStatus::try_from(inbox_status.as_str())
            .map_err(|message| invalid_enum_value(6, message))?,
        captured_at: row.get(7)?,
        processed_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        deleted_at: row.get(11)?,
    })
}

fn invalid_enum_value(column: usize, message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        Type::Text,
        Box::new(io::Error::new(io::ErrorKind::InvalidData, message)),
    )
}

fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn content_hash(raw_content: &str) -> String {
    format!("{:x}", Sha256::digest(raw_content.as_bytes()))
}
