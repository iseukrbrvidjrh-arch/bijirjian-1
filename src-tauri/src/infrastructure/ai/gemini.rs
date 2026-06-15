use std::time::Duration;

use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;

use crate::{
    domain::{ModelSource, ProviderModelInfo, ProviderType},
    error::AppError,
    infrastructure::ai::{
        model_list::{filter_gemini_models, to_provider_model_infos},
        openai_compatible::OpenAiCompatibleClient,
    },
};

const GEMINI_OPENAI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";
const GEMINI_MODELS_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const GEMINI_CONNECTION_TEST_MODEL: &str = "gemini-2.0-flash";

pub struct GeminiAdapter {
    chat_client: OpenAiCompatibleClient,
    models_client: Client,
}

impl GeminiAdapter {
    pub fn new() -> Result<Self, AppError> {
        let models_client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .user_agent("Second-Brain-OS/0.1")
            .build()
            .map_err(|_| {
                AppError::AiProvider("could not initialize the Gemini HTTP client".to_owned())
            })?;

        Ok(Self {
            chat_client: OpenAiCompatibleClient::new(GEMINI_OPENAI_BASE_URL, "Gemini")?,
            models_client,
        })
    }

    pub fn test_connection(&self, api_key: &str) -> Result<(), AppError> {
        self.chat_client.test_connection_via_chat(
            api_key,
            GEMINI_CONNECTION_TEST_MODEL,
            "You are a connection test.",
            "Reply with ok.",
        )
    }

    pub fn list_models(&self, api_key: &str) -> Result<Vec<ProviderModelInfo>, AppError> {
        let model_ids = filter_gemini_models(self.fetch_native_model_ids(api_key)?);
        if model_ids.is_empty() {
            return Err(AppError::AiProvider(
                "Gemini returned no supported chat models".to_owned(),
            ));
        }
        Ok(to_provider_model_infos(
            ProviderType::Gemini,
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
        self.chat_client
            .generate_text(model_id, api_key, system_prompt, user_content)
    }

    fn fetch_native_model_ids(&self, api_key: &str) -> Result<Vec<String>, AppError> {
        let response = self
            .models_client
            .get(format!("{GEMINI_MODELS_BASE_URL}/models"))
            .bearer_auth(api_key)
            .send()
            .map_err(connection_error)?;

        ensure_success_status(response.status())?;

        let models = response.json::<GeminiModelsResponse>().map_err(|_| {
            AppError::AiProvider("Gemini returned an invalid /models response".to_owned())
        })?;

        Ok(models
            .models
            .into_iter()
            .filter(|model| {
                model
                    .supported_generation_methods
                    .iter()
                    .any(|method| method == "generateContent")
            })
            .filter_map(|model| normalize_gemini_model_name(&model.name))
            .collect())
    }
}

fn normalize_gemini_model_name(name: &str) -> Option<String> {
    let id = name.trim().trim_start_matches("models/").trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_owned())
    }
}

fn connection_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::AiProvider("Gemini connection timed out".to_owned())
    } else {
        AppError::AiProvider("could not connect to Gemini".to_owned())
    }
}

fn ensure_success_status(status: StatusCode) -> Result<(), AppError> {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(AppError::AiProvider(
            "Gemini authentication failed; check the saved API key".to_owned(),
        )),
        StatusCode::TOO_MANY_REQUESTS => Err(AppError::AiProvider(
            "Gemini rate limit was reached; try again later".to_owned(),
        )),
        status if !status.is_success() => Err(AppError::AiProvider(format!(
            "Gemini returned HTTP status {status}"
        ))),
        _ => Ok(()),
    }
}

#[derive(Deserialize)]
struct GeminiModelsResponse {
    models: Vec<GeminiModelSummary>,
}

#[derive(Deserialize)]
struct GeminiModelSummary {
    name: String,
    #[serde(default, rename = "supportedGenerationMethods")]
    supported_generation_methods: Vec<String>,
}
