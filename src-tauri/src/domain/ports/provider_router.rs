use crate::{
    domain::{ProviderModel, ProviderType},
    error::AppError,
};

pub trait ProviderRouter: Send + Sync {
    fn test_connection(&self, provider_type: ProviderType, api_key: &str) -> Result<(), AppError>;

    fn generate_text(
        &self,
        provider_type: ProviderType,
        model: ProviderModel,
        api_key: &str,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<String, AppError>;
}
