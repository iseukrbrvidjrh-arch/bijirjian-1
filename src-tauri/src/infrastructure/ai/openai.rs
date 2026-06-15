use crate::{
    domain::{ModelSource, ProviderModelInfo, ProviderType},
    error::AppError,
    infrastructure::ai::{
        model_list::{filter_openai_models, to_provider_model_infos},
        openai_compatible::OpenAiCompatibleClient,
    },
};

const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

pub struct OpenAiAdapter {
    client: OpenAiCompatibleClient,
}

impl OpenAiAdapter {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            client: OpenAiCompatibleClient::new(OPENAI_BASE_URL, "OpenAI")?,
        })
    }

    pub fn test_connection(&self, api_key: &str) -> Result<(), AppError> {
        self.client.test_connection_via_models(api_key)
    }

    pub fn list_models(&self, api_key: &str) -> Result<Vec<ProviderModelInfo>, AppError> {
        let model_ids = filter_openai_models(self.client.fetch_model_ids(api_key)?);
        if model_ids.is_empty() {
            return Err(AppError::AiProvider(
                "OpenAI returned no supported chat models".to_owned(),
            ));
        }
        Ok(to_provider_model_infos(
            ProviderType::OpenAI,
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
