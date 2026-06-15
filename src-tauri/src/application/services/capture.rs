use crate::{
    domain::{
        ports::{SourceRepository, WorkspaceRepository},
        Source,
    },
    error::AppError,
};

pub trait CaptureService: Send + Sync {
    fn capture_text_source(&self, raw_content: String) -> Result<Source, AppError>;
}

pub struct DefaultCaptureService<'repository, WorkspaceRepo, SourceRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
{
    workspace_repository: &'repository WorkspaceRepo,
    source_repository: &'repository SourceRepo,
}

impl<'repository, WorkspaceRepo, SourceRepo>
    DefaultCaptureService<'repository, WorkspaceRepo, SourceRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
{
    pub const fn new(
        workspace_repository: &'repository WorkspaceRepo,
        source_repository: &'repository SourceRepo,
    ) -> Self {
        Self {
            workspace_repository,
            source_repository,
        }
    }
}

impl<WorkspaceRepo, SourceRepo> CaptureService
    for DefaultCaptureService<'_, WorkspaceRepo, SourceRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
{
    fn capture_text_source(&self, raw_content: String) -> Result<Source, AppError> {
        if raw_content.trim().is_empty() {
            return Err(AppError::Validation(
                "raw_content must not be empty".to_owned(),
            ));
        }

        let workspace = self.workspace_repository.ensure_default_workspace()?;
        self.source_repository
            .insert_text_source(&workspace.id, &raw_content, None)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{CaptureService, DefaultCaptureService};
    use crate::{
        application::services::{DefaultInboxService, InboxService},
        domain::{
            ports::{SourceRepository, WorkspaceRepository},
            InboxStatus, SourceType,
        },
        error::AppError,
        infrastructure::database::{
            repositories::{SqliteSourceRepository, SqliteWorkspaceRepository},
            Database,
        },
    };

    #[test]
    fn ensure_default_workspace_is_idempotent() {
        let database = test_database();
        let repository = SqliteWorkspaceRepository::new(&database);

        let first = repository
            .ensure_default_workspace()
            .expect("first default workspace creation should succeed");
        let second = repository
            .ensure_default_workspace()
            .expect("second default workspace creation should succeed");
        let found = repository
            .find_default_workspace()
            .expect("default workspace lookup should succeed")
            .expect("default workspace should exist");
        let workspace_count = database
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM workspaces WHERE name = 'Default'",
                        [],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(AppError::from)
            })
            .expect("workspace count should be readable");

        assert_eq!(first.id, second.id);
        assert_eq!(first.id, found.id);
        assert_eq!(workspace_count, 1);
    }

    #[test]
    fn insert_text_source_persists_an_unprocessed_text_source() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("default workspace should be created");

        let source = source_repository
            .insert_text_source(&workspace.id, "Phase 2A capture", None)
            .expect("text source should be inserted");

        assert_eq!(source.workspace_id, workspace.id);
        assert_eq!(source.source_type, SourceType::Text);
        assert_eq!(source.inbox_status, InboxStatus::Unprocessed);
        assert_eq!(source.raw_content, "Phase 2A capture");
        assert_eq!(source.metadata_json.as_deref(), Some("{}"));
        assert_eq!(source.content_hash.len(), 64);
    }

    #[test]
    fn capture_text_source_rejects_empty_content() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let service = DefaultCaptureService::new(&workspace_repository, &source_repository);

        let result = service.capture_text_source(" \n\t ".to_owned());

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn list_inbox_sources_only_returns_unprocessed_sources() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("default workspace should be created");
        let unprocessed = source_repository
            .insert_text_source(&workspace.id, "Keep in inbox", None)
            .expect("unprocessed source should be inserted");
        let processed = source_repository
            .insert_text_source(&workspace.id, "Already processed", None)
            .expect("processed source should be inserted");

        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE sources SET inbox_status = 'processed' WHERE id = ?1",
                    [&processed.id],
                )?;
                Ok(())
            })
            .expect("test source status should be updated");

        let service = DefaultInboxService::new(&workspace_repository, &source_repository);
        let sources = service
            .list_inbox_sources(None, 50)
            .expect("inbox sources should be listed");

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].id, unprocessed.id);
        assert_eq!(sources[0].inbox_status, InboxStatus::Unprocessed);
    }

    #[test]
    fn list_inbox_sources_orders_by_captured_at_descending() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("default workspace should be created");
        let older = source_repository
            .insert_text_source(&workspace.id, "Older", None)
            .expect("older source should be inserted");
        let newer = source_repository
            .insert_text_source(&workspace.id, "Newer", None)
            .expect("newer source should be inserted");

        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE sources SET captured_at = ?1 WHERE id = ?2",
                    params!["2026-06-14T01:00:00.000Z", older.id],
                )?;
                connection.execute(
                    "UPDATE sources SET captured_at = ?1 WHERE id = ?2",
                    params!["2026-06-14T02:00:00.000Z", newer.id],
                )?;
                Ok(())
            })
            .expect("test capture timestamps should be updated");

        let sources = source_repository
            .list_inbox_sources(&workspace.id, None, 50)
            .expect("inbox sources should be listed");

        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].id, newer.id);
        assert_eq!(sources[1].id, older.id);
    }

    #[test]
    fn identical_content_produces_identical_hashes() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("default workspace should be created");

        let first = source_repository
            .insert_text_source(&workspace.id, "Same content", None)
            .expect("first source should be inserted");
        let second = source_repository
            .insert_text_source(&workspace.id, "Same content", None)
            .expect("second source should be inserted");

        assert_eq!(first.content_hash, second.content_hash);
    }

    fn test_database() -> Database {
        Database::from_connection(
            Connection::open_in_memory().expect("in-memory database should open"),
        )
        .expect("test database should initialize")
    }
}
