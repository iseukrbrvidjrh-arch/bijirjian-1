use crate::{
    domain::{Prompt, PromptVersion},
    error::AppError,
};

pub trait PromptRepository: Send + Sync {
    fn find_by_key(&self, prompt_key: &str) -> Result<Option<Prompt>, AppError>;

    fn get_active_version(&self, prompt_id: &str) -> Result<Option<PromptVersion>, AppError>;

    fn list_versions(&self, prompt_id: &str) -> Result<Vec<PromptVersion>, AppError>;

    fn create_version(
        &self,
        prompt_id: &str,
        prompt_content: &str,
    ) -> Result<PromptVersion, AppError>;

    fn set_active_version(&self, prompt_id: &str, version_id: &str) -> Result<Prompt, AppError>;
}
