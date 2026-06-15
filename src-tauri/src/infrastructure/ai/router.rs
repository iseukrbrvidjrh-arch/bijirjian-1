use crate::{
    domain::{
        builtin_models, ports::ProviderRouter, ProviderModelInfo, ProviderModelListResult,
        ProviderType,
    },
    error::AppError,
    infrastructure::ai::{
        deepseek::DeepSeekAdapter, gemini::GeminiAdapter, openai::OpenAiAdapter, qwen::QwenAdapter,
    },
};

pub struct DefaultProviderRouter {
    deepseek: DeepSeekAdapter,
    qwen: QwenAdapter,
    openai: OpenAiAdapter,
    gemini: GeminiAdapter,
}

impl DefaultProviderRouter {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            deepseek: DeepSeekAdapter::new()?,
            qwen: QwenAdapter::new()?,
            openai: OpenAiAdapter::new()?,
            gemini: GeminiAdapter::new()?,
        })
    }
}

impl ProviderRouter for DefaultProviderRouter {
    fn test_connection(&self, provider_type: ProviderType, api_key: &str) -> Result<(), AppError> {
        match provider_type {
            ProviderType::DeepSeek => self.deepseek.test_connection(api_key),
            ProviderType::Qwen => self.qwen.test_connection(api_key),
            ProviderType::OpenAI => self.openai.test_connection(api_key),
            ProviderType::Gemini => self.gemini.test_connection(api_key),
        }
    }

    fn list_models(&self, provider_type: ProviderType, api_key: &str) -> ProviderModelListResult {
        let remote = match provider_type {
            ProviderType::DeepSeek => self.deepseek.list_models(api_key),
            ProviderType::Qwen => self.qwen.list_models(api_key),
            ProviderType::OpenAI => self.openai.list_models(api_key),
            ProviderType::Gemini => self.gemini.list_models(api_key),
        };

        match remote {
            Ok(models) if !models.is_empty() => ProviderModelListResult {
                models,
                used_fallback: false,
                fallback_reason: None,
            },
            Ok(_) => fallback_models(
                provider_type,
                Some("the provider returned an empty model list".to_owned()),
            ),
            Err(error) => fallback_models(provider_type, Some(error.to_string())),
        }
    }

    fn generate_text(
        &self,
        provider_type: ProviderType,
        model_id: &str,
        api_key: &str,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<String, AppError> {
        match provider_type {
            ProviderType::DeepSeek => {
                self.deepseek
                    .generate_text(model_id, api_key, system_prompt, user_content)
            }
            ProviderType::Qwen => {
                self.qwen
                    .generate_text(model_id, api_key, system_prompt, user_content)
            }
            ProviderType::OpenAI => {
                self.openai
                    .generate_text(model_id, api_key, system_prompt, user_content)
            }
            ProviderType::Gemini => {
                self.gemini
                    .generate_text(model_id, api_key, system_prompt, user_content)
            }
        }
    }
}

fn fallback_models(
    provider_type: ProviderType,
    fallback_reason: Option<String>,
) -> ProviderModelListResult {
    ProviderModelListResult {
        models: builtin_models(provider_type),
        used_fallback: true,
        fallback_reason,
    }
}
