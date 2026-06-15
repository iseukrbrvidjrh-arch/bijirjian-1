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
        query: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Source>, AppError> {
        let limit = i64::try_from(limit)
            .map_err(|_| AppError::Validation("inbox limit is too large".to_owned()))?;
        let query_pattern = query.map(search_pattern);

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
                  AND (
                      ?3 IS NULL
                      OR LOWER(raw_content) LIKE ?3 ESCAPE '!'
                  )
                ORDER BY captured_at DESC
                LIMIT ?4
                ",
            )?;

            let sources = statement
                .query_map(
                    params![
                        workspace_id,
                        InboxStatus::Unprocessed.as_str(),
                        query_pattern,
                        limit
                    ],
                    map_source,
                )?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(sources)
        })
    }

    fn count_inbox_sources(&self, workspace_id: &str) -> Result<usize, AppError> {
        self.database.with_connection(|connection| {
            let count = connection.query_row(
                "
                SELECT COUNT(*)
                FROM sources
                WHERE workspace_id = ?1
                  AND inbox_status = ?2
                  AND deleted_at IS NULL
                ",
                params![workspace_id, InboxStatus::Unprocessed.as_str()],
                |row| row.get::<_, i64>(0),
            )?;

            usize::try_from(count)
                .map_err(|_| AppError::State("inbox source count is invalid".to_owned()))
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

fn search_pattern(query: &str) -> String {
    let escaped = query
        .to_lowercase()
        .replace('!', "!!")
        .replace('%', "!%")
        .replace('_', "!_");

    format!("%{escaped}%")
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

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::SqliteSourceRepository;
    use crate::{
        domain::{
            ports::{SourceRepository, WorkspaceRepository},
            InboxStatus,
        },
        infrastructure::database::{repositories::SqliteWorkspaceRepository, Database},
    };

    const OLD_TIMESTAMP: &str = "2026-06-14T01:00:00.000Z";
    const NEW_TIMESTAMP: &str = "2026-06-14T02:00:00.000Z";

    #[test]
    fn searches_raw_content_case_insensitively_and_escapes_wildcards() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let text_match = repository
            .insert_text_source(&workspace.id, "Local First source", None)
            .expect("insert text match");
        let percent_match = repository
            .insert_text_source(&workspace.id, "Progress is 100% complete", None)
            .expect("insert percent match");
        repository
            .insert_text_source(&workspace.id, "Progress is 100X complete", None)
            .expect("insert percent decoy");
        let underscore_match = repository
            .insert_text_source(&workspace.id, "snake_case source", None)
            .expect("insert underscore match");
        repository
            .insert_text_source(&workspace.id, "snakeXcase source", None)
            .expect("insert underscore decoy");

        assert_eq!(
            repository
                .list_inbox_sources(&workspace.id, Some("LOCAL FIRST"), 50)
                .expect("search case-insensitively"),
            vec![text_match]
        );
        assert_eq!(
            repository
                .list_inbox_sources(&workspace.id, Some("100%"), 50)
                .expect("search literal percent"),
            vec![percent_match]
        );
        assert_eq!(
            repository
                .list_inbox_sources(&workspace.id, Some("snake_case"), 50)
                .expect("search literal underscore"),
            vec![underscore_match]
        );
        assert!(repository
            .list_inbox_sources(&workspace.id, Some("missing"), 50)
            .expect("search missing content")
            .is_empty());
    }

    #[test]
    fn search_preserves_inbox_scope_workspace_order_limit_and_source_data() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "other-search-workspace");

        let older = repository
            .insert_text_source(&workspace.id, "Search needle older", None)
            .expect("insert older source");
        let newer = repository
            .insert_text_source(&workspace.id, "Search needle newer", None)
            .expect("insert newer source");
        let processed = repository
            .insert_text_source(&workspace.id, "Search needle processed", None)
            .expect("insert processed source");
        let dismissed = repository
            .insert_text_source(&workspace.id, "Search needle dismissed", None)
            .expect("insert dismissed source");
        let deleted = repository
            .insert_text_source(&workspace.id, "Search needle deleted", None)
            .expect("insert deleted source");
        repository
            .insert_text_source("other-search-workspace", "Search needle elsewhere", None)
            .expect("insert cross-workspace source");

        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE sources SET captured_at = ?1 WHERE id = ?2",
                    params![OLD_TIMESTAMP, older.id],
                )?;
                connection.execute(
                    "UPDATE sources SET captured_at = ?1 WHERE id = ?2",
                    params![NEW_TIMESTAMP, newer.id],
                )?;
                connection.execute(
                    "UPDATE sources SET inbox_status = ?1 WHERE id = ?2",
                    params![InboxStatus::Processed.as_str(), processed.id],
                )?;
                connection.execute(
                    "UPDATE sources SET inbox_status = ?1 WHERE id = ?2",
                    params![InboxStatus::Dismissed.as_str(), dismissed.id],
                )?;
                connection.execute(
                    "UPDATE sources SET deleted_at = ?1 WHERE id = ?2",
                    params![NEW_TIMESTAMP, deleted.id],
                )?;
                Ok(())
            })
            .expect("prepare inbox search states");
        let older_before = repository
            .find_source(&workspace.id, &older.id)
            .expect("read source before search");

        let matches = repository
            .list_inbox_sources(&workspace.id, Some("needle"), 50)
            .expect("search inbox scope");
        let limited = repository
            .list_inbox_sources(&workspace.id, Some("needle"), 1)
            .expect("limit search results");
        let older_after = repository
            .find_source(&workspace.id, &older.id)
            .expect("read source after search");

        assert_eq!(
            matches
                .iter()
                .map(|source| source.id.as_str())
                .collect::<Vec<_>>(),
            vec![newer.id.as_str(), older.id.as_str()]
        );
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, newer.id);
        assert_eq!(older_before, older_after);
    }

    #[test]
    fn lifecycle_transitions_remove_sources_from_matching_search_results() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let processed = repository
            .insert_text_source(&workspace.id, "Lifecycle needle processed", None)
            .expect("insert processed candidate");
        let dismissed = repository
            .insert_text_source(&workspace.id, "Lifecycle needle dismissed", None)
            .expect("insert dismissed candidate");

        assert_eq!(
            repository
                .list_inbox_sources(&workspace.id, Some("lifecycle needle"), 50)
                .expect("list matching sources")
                .len(),
            2
        );

        repository
            .mark_source_processed(&workspace.id, &processed.id)
            .expect("mark source processed");
        let after_processed = repository
            .list_inbox_sources(&workspace.id, Some("lifecycle needle"), 50)
            .expect("list after processed transition");
        assert_eq!(after_processed.len(), 1);
        assert_eq!(after_processed[0].id, dismissed.id);

        repository
            .mark_source_dismissed(&workspace.id, &dismissed.id)
            .expect("mark source dismissed");
        assert!(repository
            .list_inbox_sources(&workspace.id, Some("lifecycle needle"), 50)
            .expect("list after dismissed transition")
            .is_empty());
    }

    #[test]
    fn counts_only_current_workspace_unprocessed_non_deleted_sources() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "count-other-workspace");
        repository
            .insert_text_source(&workspace.id, "Count this source", None)
            .expect("insert unprocessed source");
        let processed = repository
            .insert_text_source(&workspace.id, "Processed source", None)
            .expect("insert processed source");
        let dismissed = repository
            .insert_text_source(&workspace.id, "Dismissed source", None)
            .expect("insert dismissed source");
        let deleted = repository
            .insert_text_source(&workspace.id, "Deleted source", None)
            .expect("insert deleted source");
        repository
            .insert_text_source("count-other-workspace", "Other workspace source", None)
            .expect("insert other workspace source");

        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE sources SET inbox_status = ?1 WHERE id = ?2",
                    params![InboxStatus::Processed.as_str(), processed.id],
                )?;
                connection.execute(
                    "UPDATE sources SET inbox_status = ?1 WHERE id = ?2",
                    params![InboxStatus::Dismissed.as_str(), dismissed.id],
                )?;
                connection.execute(
                    "UPDATE sources SET deleted_at = ?1 WHERE id = ?2",
                    params![NEW_TIMESTAMP, deleted.id],
                )?;
                Ok(())
            })
            .expect("prepare source count states");

        assert_eq!(
            repository
                .count_inbox_sources(&workspace.id)
                .expect("count inbox sources"),
            1
        );
        assert_eq!(
            repository
                .count_inbox_sources("count-other-workspace")
                .expect("count other workspace sources"),
            1
        );
    }

    fn seed_workspace(database: &Database, workspace_id: &str) {
        database
            .with_connection(|connection| {
                connection.execute(
                    "
                    INSERT INTO workspaces (
                        id,
                        name,
                        description,
                        created_at,
                        updated_at,
                        archived_at
                    )
                    VALUES (?1, ?2, NULL, ?3, ?3, NULL)
                    ",
                    params![
                        workspace_id,
                        format!("Workspace {workspace_id}"),
                        OLD_TIMESTAMP
                    ],
                )?;
                Ok(())
            })
            .expect("insert workspace");
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
