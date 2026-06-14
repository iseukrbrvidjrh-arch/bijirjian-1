use crate::{domain::ProviderType, error::AppError};

pub trait ProviderRouter: Send + Sync {
    fn test_connection(&self, provider_type: ProviderType, api_key: &str) -> Result<(), AppError>;
}
