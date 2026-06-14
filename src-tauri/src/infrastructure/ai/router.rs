use crate::{
    domain::{ports::ProviderRouter, ProviderType},
    error::AppError,
    infrastructure::ai::deepseek::DeepSeekAdapter,
};

pub struct DefaultProviderRouter {
    deepseek: DeepSeekAdapter,
}

impl DefaultProviderRouter {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            deepseek: DeepSeekAdapter::new()?,
        })
    }
}

impl ProviderRouter for DefaultProviderRouter {
    fn test_connection(&self, provider_type: ProviderType, api_key: &str) -> Result<(), AppError> {
        match provider_type {
            ProviderType::DeepSeek => self.deepseek.test_connection(api_key),
        }
    }
}
