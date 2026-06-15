use crate::{
    domain::{ModelSource, ProviderModelInfo, ProviderType},
    error::AppError,
    infrastructure::ai::{
        model_list::{filter_qwen_models, to_provider_model_infos},
        openai_compatible::OpenAiCompatibleClient,
    },
};

const QWEN_BASE_URL: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1";

pub struct QwenAdapter {
    client: OpenAiCompatibleClient,
}

impl QwenAdapter {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            client: OpenAiCompatibleClient::new(QWEN_BASE_URL, "Qwen")?,
        })
    }

    pub fn test_connection(&self, api_key: &str) -> Result<(), AppError> {
        self.client.test_connection_via_models(api_key)
    }

    pub fn list_models(&self, api_key: &str) -> Result<Vec<ProviderModelInfo>, AppError> {
        let model_ids = filter_qwen_models(self.client.fetch_model_ids(api_key)?);
        if model_ids.is_empty() {
            return Err(AppError::AiProvider(
                "Qwen returned no supported chat models".to_owned(),
            ));
        }
        Ok(to_provider_model_infos(
            ProviderType::Qwen,
            model_ids,
            ModelSource::Remote,
        ))
    }

    pub fn generate_text(
        &self,
        model_id: &str,
        api_key: &str,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<String, AppError> {
        self.client
            .generate_text(model_id, api_key, system_prompt, user_content)
    }
}
