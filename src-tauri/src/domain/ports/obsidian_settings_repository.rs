use crate::{domain::ObsidianSettings, error::AppError};

pub trait ObsidianSettingsRepository: Send + Sync {
    fn find_by_workspace(&self, workspace_id: &str) -> Result<Option<ObsidianSettings>, AppError>;

    fn upsert(&self, workspace_id: &str, vault_path: &str) -> Result<ObsidianSettings, AppError>;
}
