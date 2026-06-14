use crate::{
    domain::{ports::ProviderRouter, ProviderModel, ProviderType},
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

    fn generate_text(
        &self,
        provider_type: ProviderType,
        model: ProviderModel,
        api_key: &str,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<String, AppError> {
        if model.provider_type() != provider_type {
            return Err(AppError::Validation(format!(
                "model {model} is not supported by provider {provider_type}"
            )));
        }

        match provider_type {
            ProviderType::DeepSeek => {
                self.deepseek
                    .generate_text(model, api_key, system_prompt, user_content)
            }
        }
    }
}
