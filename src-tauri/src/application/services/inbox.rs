use crate::{
    domain::{
        ports::{SourceRepository, WorkspaceRepository},
        Source,
    },
    error::AppError,
};

pub trait InboxService: Send + Sync {
    fn list_inbox_sources(&self, limit: usize) -> Result<Vec<Source>, AppError>;
    fn mark_source_processed(&self, source_id: String) -> Result<Source, AppError>;
    fn mark_source_dismissed(&self, source_id: String) -> Result<Source, AppError>;
}

pub struct DefaultInboxService<'repository, WorkspaceRepo, SourceRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
{
    workspace_repository: &'repository WorkspaceRepo,
    source_repository: &'repository SourceRepo,
}

impl<'repository, WorkspaceRepo, SourceRepo>
    DefaultInboxService<'repository, WorkspaceRepo, SourceRepo>
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

impl<WorkspaceRepo, SourceRepo> InboxService for DefaultInboxService<'_, WorkspaceRepo, SourceRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
{
    fn list_inbox_sources(&self, limit: usize) -> Result<Vec<Source>, AppError> {
        let workspace = self.workspace_repository.ensure_default_workspace()?;
        self.source_repository
            .list_inbox_sources(&workspace.id, limit)
    }

    fn mark_source_processed(&self, source_id: String) -> Result<Source, AppError> {
        validate_source_id(&source_id)?;
        let workspace = self.workspace_repository.ensure_default_workspace()?;
        self.source_repository
            .mark_source_processed(&workspace.id, &source_id)
    }

    fn mark_source_dismissed(&self, source_id: String) -> Result<Source, AppError> {
        validate_source_id(&source_id)?;
        let workspace = self.workspace_repository.ensure_default_workspace()?;
        self.source_repository
            .mark_source_dismissed(&workspace.id, &source_id)
    }
}

fn validate_source_id(source_id: &str) -> Result<(), AppError> {
    if source_id.trim().is_empty() {
        return Err(AppError::Validation(
            "source_id must not be empty".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{DefaultInboxService, InboxService};
    use crate::{
        domain::{
            ports::{SourceRepository, WorkspaceRepository},
            InboxStatus,
        },
        error::AppError,
        infrastructure::database::{
            repositories::{SqliteSourceRepository, SqliteWorkspaceRepository},
            Database,
        },
    };

    const OLD_TIMESTAMP: &str = "2020-01-01T00:00:00.000Z";

    #[test]
    fn marks_an_unprocessed_source_as_processed_and_removes_it_from_inbox() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("default workspace should exist");
        let source = source_repository
            .insert_text_source(&workspace.id, "Process this source", None)
            .expect("source should be inserted");
        set_updated_at(&database, &source.id, OLD_TIMESTAMP);
        let service = DefaultInboxService::new(&workspace_repository, &source_repository);

        let processed = service
            .mark_source_processed(source.id.clone())
            .expect("unprocessed source should be marked as processed");

        assert_eq!(processed.inbox_status, InboxStatus::Processed);
        assert!(processed.processed_at.is_some());
        assert_ne!(processed.updated_at, OLD_TIMESTAMP);
        assert!(service
            .list_inbox_sources(50)
            .expect("inbox query should succeed")
            .is_empty());
        assert!(matches!(
            service.mark_source_processed(source.id),
            Err(AppError::Conflict(_))
        ));
    }

    #[test]
    fn marks_an_unprocessed_source_as_dismissed_without_setting_processed_at() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("default workspace should exist");
        let source = source_repository
            .insert_text_source(&workspace.id, "Dismiss this source", None)
            .expect("source should be inserted");
        set_updated_at(&database, &source.id, OLD_TIMESTAMP);
        let service = DefaultInboxService::new(&workspace_repository, &source_repository);

        let dismissed = service
            .mark_source_dismissed(source.id.clone())
            .expect("unprocessed source should be marked as dismissed");

        assert_eq!(dismissed.inbox_status, InboxStatus::Dismissed);
        assert!(dismissed.processed_at.is_none());
        assert_ne!(dismissed.updated_at, OLD_TIMESTAMP);
        assert!(service
            .list_inbox_sources(50)
            .expect("inbox query should succeed")
            .is_empty());
        assert!(matches!(
            service.mark_source_processed(source.id),
            Err(AppError::Conflict(_))
        ));
    }

    #[test]
    fn refuses_to_transition_a_source_from_another_workspace() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let default_workspace = workspace_repository
            .ensure_default_workspace()
            .expect("default workspace should exist");
        let other_workspace_id = "other-workspace";

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
                    params![other_workspace_id, OLD_TIMESTAMP],
                )?;
                Ok(())
            })
            .expect("other workspace should be inserted");

        let source = source_repository
            .insert_text_source(other_workspace_id, "Other workspace source", None)
            .expect("source should be inserted in other workspace");

        assert!(matches!(
            source_repository.mark_source_processed(&default_workspace.id, &source.id),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn returns_not_found_for_a_missing_source() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let service = DefaultInboxService::new(&workspace_repository, &source_repository);

        assert!(matches!(
            service.mark_source_dismissed("missing-source".to_owned()),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn refuses_to_transition_a_deleted_source() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("default workspace should exist");
        let source = source_repository
            .insert_text_source(&workspace.id, "Deleted source", None)
            .expect("source should be inserted");

        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE sources SET deleted_at = ?1 WHERE id = ?2",
                    params![OLD_TIMESTAMP, source.id],
                )?;
                Ok(())
            })
            .expect("test source should be marked as deleted");

        assert!(matches!(
            source_repository.mark_source_dismissed(&workspace.id, &source.id),
            Err(AppError::Conflict(_))
        ));
    }

    fn set_updated_at(database: &Database, source_id: &str, updated_at: &str) {
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE sources SET updated_at = ?1 WHERE id = ?2",
                    params![updated_at, source_id],
                )?;
                Ok(())
            })
            .expect("test source timestamp should be updated");
    }

    fn test_database() -> Database {
        Database::from_connection(
            Connection::open_in_memory().expect("in-memory database should open"),
        )
        .expect("test database should initialize")
    }
}
