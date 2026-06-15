use std::time::Duration;

use reqwest::{blocking::Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub struct OpenAiCompatibleClient {
    client: Client,
    base_url: String,
    provider_name: &'static str,
}

impl OpenAiCompatibleClient {
    pub fn new(base_url: &str, provider_name: &'static str) -> Result<Self, AppError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .user_agent("Second-Brain-OS/0.1")
            .build()
            .map_err(|_| {
                AppError::AiProvider(format!(
                    "could not initialize the {provider_name} HTTP client"
                ))
            })?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
            provider_name,
        })
    }

    pub fn fetch_model_ids(&self, api_key: &str) -> Result<Vec<String>, AppError> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .bearer_auth(api_key)
            .send()
            .map_err(|error| self.connection_error(error))?;

        self.ensure_success_status(response.status())?;

        let models = response.json::<ModelsResponse>().map_err(|_| {
            AppError::AiProvider(format!(
                "{} returned an invalid /models response",
                self.provider_name
            ))
        })?;

        if models.object != "list" {
            return Err(AppError::AiProvider(format!(
                "{} returned an invalid /models response",
                self.provider_name
            )));
        }

        Ok(models
            .data
            .into_iter()
            .map(|model| model.id.trim().to_owned())
            .filter(|id| !id.is_empty())
            .collect())
    }

    pub fn test_connection_via_models(&self, api_key: &str) -> Result<(), AppError> {
        let model_ids = self.fetch_model_ids(api_key)?;
        if model_ids.is_empty() {
            return Err(AppError::AiProvider(format!(
                "{} returned an invalid /models response",
                self.provider_name
            )));
        }
        Ok(())
    }

    pub fn test_connection_via_chat(
        &self,
        api_key: &str,
        model: &str,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<(), AppError> {
        let content = self.generate_text(model, api_key, system_prompt, user_content)?;
        if content.trim().is_empty() {
            return Err(AppError::AiProvider(format!(
                "{} returned an empty chat completions response during connection test",
                self.provider_name
            )));
        }
        Ok(())
    }

    pub fn generate_text(
        &self,
        model: &str,
        api_key: &str,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<String, AppError> {
        let request = ChatCompletionRequest {
            model,
            messages: [
                ChatMessage {
                    role: "system",
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user",
                    content: user_content,
                },
            ],
        };
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .map_err(|error| self.connection_error(error))?;

        self.ensure_success_status(response.status())?;

        let completion = response.json::<ChatCompletionResponse>().map_err(|_| {
            AppError::AiProvider(format!(
                "{} returned an invalid chat completions response",
                self.provider_name
            ))
        })?;
        let content = completion
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .map(|content| content.trim().to_owned())
            .filter(|content| !content.is_empty())
            .ok_or_else(|| {
                AppError::AiProvider(format!(
                    "{} returned an empty chat completions response",
                    self.provider_name
                ))
            })?;

        Ok(content)
    }

    fn connection_error(&self, error: reqwest::Error) -> AppError {
        if error.is_timeout() {
            AppError::AiProvider(format!("{} connection timed out", self.provider_name))
        } else {
            AppError::AiProvider(format!("could not connect to {}", self.provider_name))
        }
    }

    fn ensure_success_status(&self, status: StatusCode) -> Result<(), AppError> {
        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(AppError::AiProvider(format!(
                "{} authentication failed; check the saved API key",
                self.provider_name
            ))),
            StatusCode::TOO_MANY_REQUESTS => Err(AppError::AiProvider(format!(
                "{} rate limit was reached; try again later",
                self.provider_name
            ))),
            status if !status.is_success() => Err(AppError::AiProvider(format!(
                "{} returned HTTP status {status}",
                self.provider_name
            ))),
            _ => Ok(()),
        }
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest<'request> {
    model: &'request str,
    messages: [ChatMessage<'request>; 2],
}

#[derive(Serialize)]
struct ChatMessage<'message> {
    role: &'static str,
    content: &'message str,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ModelsResponse {
    object: String,
    data: Vec<ModelSummary>,
}

#[derive(Deserialize)]
struct ModelSummary {
    id: String,
}
