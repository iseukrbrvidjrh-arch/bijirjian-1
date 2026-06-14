use crate::{domain::ProviderType, error::AppError};

pub trait CredentialStore: Send + Sync {
    fn get_api_key(&self, provider_type: ProviderType) -> Result<Option<String>, AppError>;
    fn set_api_key(&self, provider_type: ProviderType, api_key: &str) -> Result<(), AppError>;
}
