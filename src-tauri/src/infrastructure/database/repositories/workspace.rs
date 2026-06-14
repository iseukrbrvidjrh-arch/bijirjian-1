use chrono::{SecondsFormat, Utc};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use uuid::Uuid;

use crate::{
    domain::{ports::WorkspaceRepository, Workspace},
    error::AppError,
    infrastructure::database::Database,
};

const DEFAULT_WORKSPACE_NAME: &str = "Default";

pub struct SqliteWorkspaceRepository<'database> {
    database: &'database Database,
}

impl<'database> SqliteWorkspaceRepository<'database> {
    pub const fn new(database: &'database Database) -> Self {
        Self { database }
    }
}

impl WorkspaceRepository for SqliteWorkspaceRepository<'_> {
    fn ensure_default_workspace(&self) -> Result<Workspace, AppError> {
        self.database.with_connection(|connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let now = current_timestamp();

            transaction.execute(
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
                ON CONFLICT(name) DO NOTHING
                ",
                params![Uuid::new_v4().to_string(), DEFAULT_WORKSPACE_NAME, now],
            )?;

            let workspace =
                find_default_workspace_in_connection(&transaction)?.ok_or_else(|| {
                    AppError::State("default workspace could not be created or loaded".to_owned())
                })?;

            transaction.commit()?;
            Ok(workspace)
        })
    }

    fn find_default_workspace(&self) -> Result<Option<Workspace>, AppError> {
        self.database
            .with_connection(|connection| find_default_workspace_in_connection(connection))
    }
}

fn find_default_workspace_in_connection(
    connection: &rusqlite::Connection,
) -> Result<Option<Workspace>, AppError> {
    connection
        .query_row(
            "
            SELECT id, name, description, created_at, updated_at, archived_at
            FROM workspaces
            WHERE name = ?1
            ",
            [DEFAULT_WORKSPACE_NAME],
            |row| {
                Ok(Workspace {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    archived_at: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(AppError::from)
}

fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
