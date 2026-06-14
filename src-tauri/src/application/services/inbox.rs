use crate::{
    domain::{
        ports::{SourceRepository, WorkspaceRepository},
        Source,
    },
    error::AppError,
};

pub trait InboxService: Send + Sync {
    fn list_inbox_sources(&self, limit: usize) -> Result<Vec<Source>, AppError>;
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
}
