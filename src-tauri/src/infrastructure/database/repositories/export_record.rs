use std::io;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, types::Type, OptionalExtension};
use uuid::Uuid;

use crate::{
    domain::{ports::ExportRecordRepository, ExportRecord, ExportStatus},
    error::AppError,
    infrastructure::database::Database,
};

pub struct SqliteExportRecordRepository<'database> {
    database: &'database Database,
}

impl<'database> SqliteExportRecordRepository<'database> {
    pub const fn new(database: &'database Database) -> Self {
        Self { database }
    }
}

impl ExportRecordRepository for SqliteExportRecordRepository<'_> {
    fn insert_success(
        &self,
        workspace_id: &str,
        knowledge_node_id: &str,
        export_path: &str,
    ) -> Result<ExportRecord, AppError> {
        insert_record(
            self.database,
            workspace_id,
            knowledge_node_id,
            Some(export_path),
            ExportStatus::Succeeded,
            None,
        )
    }

    fn insert_failure(
        &self,
        workspace_id: &str,
        knowledge_node_id: &str,
        export_path: Option<&str>,
        error_message: &str,
    ) -> Result<ExportRecord, AppError> {
        insert_record(
            self.database,
            workspace_id,
            knowledge_node_id,
            export_path,
            ExportStatus::Failed,
            Some(error_message),
        )
    }

    fn find_latest_for_knowledge(
        &self,
        workspace_id: &str,
        knowledge_node_id: &str,
    ) -> Result<Option<ExportRecord>, AppError> {
        self.database.with_connection(|connection| {
            connection
                .query_row(
                    "
                    SELECT
                        id,
                        workspace_id,
                        knowledge_node_id,
                        export_path,
                        status,
                        error_message,
                        created_at
                    FROM export_records
                    WHERE workspace_id = ?1
                      AND knowledge_node_id = ?2
                    ORDER BY created_at DESC, rowid DESC
                    LIMIT 1
                    ",
                    params![workspace_id, knowledge_node_id],
                    map_export_record,
                )
                .optional()
                .map_err(AppError::from)
        })
    }
}

fn insert_record(
    database: &Database,
    workspace_id: &str,
    knowledge_node_id: &str,
    export_path: Option<&str>,
    status: ExportStatus,
    error_message: Option<&str>,
) -> Result<ExportRecord, AppError> {
    let record = ExportRecord {
        id: Uuid::new_v4().to_string(),
        workspace_id: workspace_id.to_owned(),
        knowledge_node_id: knowledge_node_id.to_owned(),
        export_path: export_path.map(str::to_owned),
        status,
        error_message: error_message.map(str::to_owned),
        created_at: current_timestamp(),
    };

    database.with_connection(|connection| {
        connection.execute(
            "
            INSERT INTO export_records (
                id,
                workspace_id,
                knowledge_node_id,
                export_path,
                status,
                error_message,
                created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                record.id,
                record.workspace_id,
                record.knowledge_node_id,
                record.export_path,
                record.status.as_str(),
                record.error_message,
                record.created_at,
            ],
        )?;

        Ok(record)
    })
}

fn map_export_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExportRecord> {
    let status = row.get::<_, String>(4)?;

    Ok(ExportRecord {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        knowledge_node_id: row.get(2)?,
        export_path: row.get(3)?,
        status: ExportStatus::try_from(status.as_str())
            .map_err(|message| invalid_enum_value(4, message))?,
        error_message: row.get(5)?,
        created_at: row.get(6)?,
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

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::SqliteExportRecordRepository;
    use crate::{
        domain::{
            ports::{ExportRecordRepository, KnowledgeRepository, WorkspaceRepository},
            ExportStatus, KnowledgeType,
        },
        infrastructure::database::{
            repositories::{SqliteKnowledgeRepository, SqliteWorkspaceRepository},
            Database,
        },
    };

    #[test]
    fn inserts_success_and_failure_records_and_returns_the_latest() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let repository = SqliteExportRecordRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let knowledge = knowledge_repository
            .insert_manual_node(
                &workspace.id,
                "Exportable",
                "Exportable content",
                KnowledgeType::Concept,
            )
            .expect("insert knowledge");

        let succeeded = repository
            .insert_success(&workspace.id, &knowledge.id, "/vault/knowledge.md")
            .expect("insert success");
        let failed = repository
            .insert_failure(
                &workspace.id,
                &knowledge.id,
                Some("/vault/knowledge.md"),
                "disk unavailable",
            )
            .expect("insert failure");
        let latest = repository
            .find_latest_for_knowledge(&workspace.id, &knowledge.id)
            .expect("find latest")
            .expect("latest should exist");

        assert_eq!(succeeded.status, ExportStatus::Succeeded);
        assert_eq!(
            succeeded.export_path.as_deref(),
            Some("/vault/knowledge.md")
        );
        assert!(succeeded.error_message.is_none());
        assert_eq!(failed.status, ExportStatus::Failed);
        assert_eq!(failed.error_message.as_deref(), Some("disk unavailable"));
        assert_eq!(latest, failed);
    }

    #[test]
    fn latest_record_uses_created_at_then_rowid_descending() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let repository = SqliteExportRecordRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let knowledge = knowledge_repository
            .insert_manual_node(
                &workspace.id,
                "Ordered exports",
                "Exportable content",
                KnowledgeType::Concept,
            )
            .expect("insert knowledge");
        let first = repository
            .insert_success(&workspace.id, &knowledge.id, "/vault/first.md")
            .expect("insert first record");
        let second = repository
            .insert_failure(
                &workspace.id,
                &knowledge.id,
                Some("/vault/second.md"),
                "later row",
            )
            .expect("insert second record");

        set_created_at(&database, &first.id, "2026-06-15T02:00:00.000Z");
        set_created_at(&database, &second.id, "2026-06-15T01:00:00.000Z");
        assert_eq!(
            repository
                .find_latest_for_knowledge(&workspace.id, &knowledge.id)
                .expect("query latest by timestamp")
                .expect("latest record should exist")
                .id,
            first.id
        );

        set_created_at(&database, &first.id, "2026-06-15T03:00:00.000Z");
        set_created_at(&database, &second.id, "2026-06-15T03:00:00.000Z");
        assert_eq!(
            repository
                .find_latest_for_knowledge(&workspace.id, &knowledge.id)
                .expect("query latest by rowid")
                .expect("latest record should exist")
                .id,
            second.id
        );
    }

    #[test]
    fn isolates_latest_records_by_workspace() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let repository = SqliteExportRecordRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let knowledge = knowledge_repository
            .insert_manual_node(
                &workspace.id,
                "Exportable",
                "Exportable content",
                KnowledgeType::Concept,
            )
            .expect("insert knowledge");
        seed_workspace(&database, "other-workspace");

        repository
            .insert_success(&workspace.id, &knowledge.id, "/vault/knowledge.md")
            .expect("insert success");

        assert!(repository
            .find_latest_for_knowledge("other-workspace", &knowledge.id)
            .expect("query other workspace")
            .is_none());
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
                    VALUES (?1, 'Other', NULL, ?2, ?2, NULL)
                    ",
                    params![workspace_id, "2026-06-15T00:00:00.000Z"],
                )?;
                Ok(())
            })
            .expect("seed workspace");
    }

    fn set_created_at(database: &Database, export_id: &str, created_at: &str) {
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE export_records SET created_at = ?1 WHERE id = ?2",
                    params![created_at, export_id],
                )?;
                Ok(())
            })
            .expect("set export timestamp");
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
