use crate::{
    domain::{ProviderModelInfo, ProviderModelListResult, ProviderType},
    error::AppError,
};

pub trait ProviderRouter: Send + Sync {
    fn test_connection(&self, provider_type: ProviderType, api_key: &str) -> Result<(), AppError>;

    fn list_models(&self, provider_type: ProviderType, api_key: &str) -> ProviderModelListResult;

    fn generate_text(
        &self,
        provider_type: ProviderType,
        model_id: &str,
        api_key: &str,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<String, AppError>;
}
