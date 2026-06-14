use crate::{
    domain::{AiRun, ProviderModel, ProviderType},
    error::AppError,
};

pub trait AiRunRepository: Send + Sync {
    fn insert_success(
        &self,
        source_id: &str,
        prompt_version_id: &str,
        provider_type: ProviderType,
        model: ProviderModel,
        output_text: &str,
    ) -> Result<AiRun, AppError>;

    fn insert_failure(
        &self,
        source_id: &str,
        prompt_version_id: Option<&str>,
        provider_type: Option<ProviderType>,
        model: Option<ProviderModel>,
        error_message: &str,
    ) -> Result<AiRun, AppError>;

    fn find_latest_for_source(&self, source_id: &str) -> Result<Option<AiRun>, AppError>;
}
