use crate::{domain::Workspace, error::AppError};

pub trait WorkspaceRepository: Send + Sync {
    fn ensure_default_workspace(&self) -> Result<Workspace, AppError>;
    fn find_default_workspace(&self) -> Result<Option<Workspace>, AppError>;
}
