use std::io;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, types::Type, OptionalExtension, Transaction, TransactionBehavior};
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

    fn find_source(&self, workspace_id: &str, source_id: &str) -> Result<Source, AppError> {
        self.database.with_connection(|connection| {
            if let Some(source) =
                find_source_in_connection(connection, workspace_id, source_id, false)?
            {
                return Ok(source);
            }

            let source_state = connection
                .query_row(
                    "
                    SELECT workspace_id, deleted_at
                    FROM sources
                    WHERE id = ?1
                    ",
                    [source_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
                )
                .optional()?;

            match source_state {
                Some((source_workspace_id, Some(_))) if source_workspace_id == workspace_id => {
                    Err(AppError::Conflict(format!(
                        "source {source_id} is deleted and cannot be summarized"
                    )))
                }
                _ => Err(AppError::NotFound(format!(
                    "source {source_id} does not exist in the current workspace"
                ))),
            }
        })
    }

    fn mark_source_processed(
        &self,
        workspace_id: &str,
        source_id: &str,
    ) -> Result<Source, AppError> {
        self.transition_source(workspace_id, source_id, InboxStatus::Processed)
    }

    fn mark_source_dismissed(
        &self,
        workspace_id: &str,
        source_id: &str,
    ) -> Result<Source, AppError> {
        self.transition_source(workspace_id, source_id, InboxStatus::Dismissed)
    }
}

impl SqliteSourceRepository<'_> {
    fn transition_source(
        &self,
        workspace_id: &str,
        source_id: &str,
        target_status: InboxStatus,
    ) -> Result<Source, AppError> {
        self.database.with_connection(|connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let now = current_timestamp();

            let changed_rows = match target_status {
                InboxStatus::Processed => transaction.execute(
                    "
                    UPDATE sources
                    SET inbox_status = ?1,
                        processed_at = ?2,
                        updated_at = ?2
                    WHERE id = ?3
                      AND workspace_id = ?4
                      AND inbox_status = ?5
                      AND deleted_at IS NULL
                    ",
                    params![
                        target_status.as_str(),
                        now,
                        source_id,
                        workspace_id,
                        InboxStatus::Unprocessed.as_str(),
                    ],
                )?,
                InboxStatus::Dismissed => transaction.execute(
                    "
                    UPDATE sources
                    SET inbox_status = ?1,
                        updated_at = ?2
                    WHERE id = ?3
                      AND workspace_id = ?4
                      AND inbox_status = ?5
                      AND deleted_at IS NULL
                    ",
                    params![
                        target_status.as_str(),
                        now,
                        source_id,
                        workspace_id,
                        InboxStatus::Unprocessed.as_str(),
                    ],
                )?,
                _ => {
                    return Err(AppError::Validation(format!(
                        "unsupported source lifecycle target: {target_status}"
                    )));
                }
            };

            if changed_rows == 0 {
                return Err(source_transition_error(
                    &transaction,
                    workspace_id,
                    source_id,
                    target_status,
                )?);
            }

            let source = find_source_in_connection(&transaction, workspace_id, source_id, true)?
                .ok_or_else(|| {
                    AppError::State(format!(
                        "source {source_id} was updated but could not be loaded"
                    ))
                })?;

            transaction.commit()?;
            Ok(source)
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

fn find_source_in_connection(
    connection: &rusqlite::Connection,
    workspace_id: &str,
    source_id: &str,
    include_deleted: bool,
) -> Result<Option<Source>, AppError> {
    connection
        .query_row(
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
            WHERE id = ?1
              AND workspace_id = ?2
              AND (?3 = 1 OR deleted_at IS NULL)
            ",
            params![source_id, workspace_id, include_deleted],
            map_source,
        )
        .optional()
        .map_err(AppError::from)
}

fn source_transition_error(
    transaction: &Transaction<'_>,
    workspace_id: &str,
    source_id: &str,
    target_status: InboxStatus,
) -> Result<AppError, AppError> {
    let current_state = transaction
        .query_row(
            "
            SELECT inbox_status, deleted_at
            FROM sources
            WHERE id = ?1
              AND workspace_id = ?2
            ",
            params![source_id, workspace_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()?;

    Ok(match current_state {
        None => AppError::NotFound(format!(
            "source {source_id} does not exist in the current workspace"
        )),
        Some((_, Some(_))) => AppError::Conflict(format!(
            "source {source_id} is deleted and cannot be marked as {target_status}"
        )),
        Some((status, None)) => AppError::Conflict(format!(
            "source {source_id} has status {status} and cannot be marked as {target_status}"
        )),
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
