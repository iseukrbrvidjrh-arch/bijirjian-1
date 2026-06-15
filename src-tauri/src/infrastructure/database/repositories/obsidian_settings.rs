use chrono::{SecondsFormat, Utc};
use rusqlite::{params, OptionalExtension, TransactionBehavior};

use crate::{
    domain::{ports::ObsidianSettingsRepository, ObsidianSettings},
    error::AppError,
    infrastructure::database::Database,
};

pub struct SqliteObsidianSettingsRepository<'database> {
    database: &'database Database,
}

impl<'database> SqliteObsidianSettingsRepository<'database> {
    pub const fn new(database: &'database Database) -> Self {
        Self { database }
    }
}

impl ObsidianSettingsRepository for SqliteObsidianSettingsRepository<'_> {
    fn find_by_workspace(&self, workspace_id: &str) -> Result<Option<ObsidianSettings>, AppError> {
        self.database.with_connection(|connection| {
            find_by_workspace(connection, workspace_id).map_err(AppError::from)
        })
    }

    fn upsert(&self, workspace_id: &str, vault_path: &str) -> Result<ObsidianSettings, AppError> {
        self.database.with_connection(|connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let now = current_timestamp();

            transaction.execute(
                "
                INSERT INTO obsidian_settings (
                    workspace_id,
                    vault_path,
                    created_at,
                    updated_at
                )
                VALUES (?1, ?2, ?3, ?3)
                ON CONFLICT(workspace_id) DO UPDATE SET
                    vault_path = excluded.vault_path,
                    updated_at = excluded.updated_at
                ",
                params![workspace_id, vault_path, now],
            )?;

            let settings = find_by_workspace(&transaction, workspace_id)?.ok_or_else(|| {
                AppError::State("Obsidian settings were saved but could not be loaded".to_owned())
            })?;
            transaction.commit()?;

            Ok(settings)
        })
    }
}

fn find_by_workspace(
    connection: &rusqlite::Connection,
    workspace_id: &str,
) -> rusqlite::Result<Option<ObsidianSettings>> {
    connection
        .query_row(
            "
            SELECT workspace_id, vault_path, created_at, updated_at
            FROM obsidian_settings
            WHERE workspace_id = ?1
            ",
            [workspace_id],
            |row| {
                Ok(ObsidianSettings {
                    workspace_id: row.get(0)?,
                    vault_path: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        )
        .optional()
}

fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::SqliteObsidianSettingsRepository;
    use crate::{
        domain::ports::{ObsidianSettingsRepository, WorkspaceRepository},
        infrastructure::database::{repositories::SqliteWorkspaceRepository, Database},
    };

    #[test]
    fn inserts_updates_and_reloads_workspace_settings() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteObsidianSettingsRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");

        let inserted = repository
            .upsert(&workspace.id, "/vault/one")
            .expect("insert Obsidian settings");
        let updated = repository
            .upsert(&workspace.id, "/vault/two")
            .expect("update Obsidian settings");
        let reloaded = repository
            .find_by_workspace(&workspace.id)
            .expect("reload Obsidian settings")
            .expect("settings should exist");

        assert_eq!(inserted.vault_path, "/vault/one");
        assert_eq!(updated.vault_path, "/vault/two");
        assert_eq!(updated.created_at, inserted.created_at);
        assert_eq!(reloaded, updated);
    }

    #[test]
    fn isolates_settings_by_workspace() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteObsidianSettingsRepository::new(&database);
        let default_workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
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
                    params!["other-workspace", "Other", "2026-06-15T00:00:00.000Z"],
                )?;
                Ok(())
            })
            .expect("insert other workspace");

        repository
            .upsert(&default_workspace.id, "/vault/default")
            .expect("save default settings");
        repository
            .upsert("other-workspace", "/vault/other")
            .expect("save other settings");

        assert_eq!(
            repository
                .find_by_workspace(&default_workspace.id)
                .expect("read default settings")
                .expect("default settings should exist")
                .vault_path,
            "/vault/default"
        );
        assert_eq!(
            repository
                .find_by_workspace("other-workspace")
                .expect("read other settings")
                .expect("other settings should exist")
                .vault_path,
            "/vault/other"
        );
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
