use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelSource {
    Remote,
    Builtin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelInfo {
    pub id: String,
    pub label: String,
    pub provider_type: ProviderType,
    pub source: ModelSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderModelListResult {
    pub models: Vec<ProviderModelInfo>,
    pub used_fallback: bool,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    DeepSeek,
    Qwen,
    OpenAI,
    Gemini,
}

impl ProviderType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek",
            Self::Qwen => "qwen",
            Self::OpenAI => "openai",
            Self::Gemini => "gemini",
        }
    }
}

impl TryFrom<&str> for ProviderType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "deepseek" => Ok(Self::DeepSeek),
            "qwen" => Ok(Self::Qwen),
            "openai" => Ok(Self::OpenAI),
            "gemini" => Ok(Self::Gemini),
            _ => Err(format!("unsupported provider type: {value}")),
        }
    }
}

impl fmt::Display for ProviderType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSettings {
    pub provider_type: ProviderType,
    pub default_model: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn validate_model_id(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "model id must not be empty".to_owned(),
        ));
    }
    Ok(trimmed.to_owned())
}

pub fn default_model_for_provider(provider_type: ProviderType) -> String {
    match provider_type {
        ProviderType::DeepSeek => "deepseek-v4-flash".to_owned(),
        ProviderType::Qwen => "qwen-plus".to_owned(),
        ProviderType::OpenAI => "gpt-4o".to_owned(),
        ProviderType::Gemini => "gemini-2.5-flash".to_owned(),
    }
}

pub fn builtin_models(provider_type: ProviderType) -> Vec<ProviderModelInfo> {
    let entries: &[(&str, &str)] = match provider_type {
        ProviderType::DeepSeek => &[
            ("deepseek-v4-flash", "DeepSeek V4 Flash"),
            ("deepseek-v4-pro", "DeepSeek V4 Pro"),
        ],
        ProviderType::Qwen => &[
            ("qwen-plus", "Qwen Plus"),
            ("qwen-turbo", "Qwen Turbo"),
            ("qwen-max", "Qwen Max"),
            ("qwen-flash", "Qwen Flash"),
            ("qwen-long", "Qwen Long"),
            ("qwen-coder-plus", "Qwen Coder Plus"),
        ],
        ProviderType::OpenAI => &[
            ("gpt-4.1", "GPT-4.1"),
            ("gpt-4o", "GPT-4o"),
            ("gpt-4o-mini", "GPT-4o Mini"),
            ("gpt-4.1-mini", "GPT-4.1 Mini"),
            ("gpt-4-turbo", "GPT-4 Turbo"),
            ("o3-mini", "o3-mini"),
        ],
        ProviderType::Gemini => &[
            ("gemini-2.5-flash", "Gemini 2.5 Flash"),
            ("gemini-2.5-pro", "Gemini 2.5 Pro"),
            ("gemini-2.0-flash", "Gemini 2.0 Flash"),
            ("gemini-1.5-flash", "Gemini 1.5 Flash"),
            ("gemini-1.5-pro", "Gemini 1.5 Pro"),
        ],
    };

    entries
        .iter()
        .map(|(id, label)| ProviderModelInfo {
            id: (*id).to_owned(),
            label: (*label).to_owned(),
            provider_type,
            source: ModelSource::Builtin,
        })
        .collect()
}

pub fn model_label(provider_type: ProviderType, model_id: &str) -> String {
    builtin_models(provider_type)
        .into_iter()
        .find(|model| model.id == model_id)
        .map(|model| model.label)
        .unwrap_or_else(|| model_id.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        builtin_models, default_model_for_provider, validate_model_id, ModelSource, ProviderType,
    };

    #[test]
    fn parses_supported_provider_types() {
        assert_eq!(
            ProviderType::try_from("deepseek"),
            Ok(ProviderType::DeepSeek)
        );
        assert_eq!(ProviderType::try_from("qwen"), Ok(ProviderType::Qwen));
        assert_eq!(ProviderType::try_from("openai"), Ok(ProviderType::OpenAI));
        assert_eq!(ProviderType::try_from("gemini"), Ok(ProviderType::Gemini));
    }

    #[test]
    fn rejects_unknown_provider_type() {
        assert!(ProviderType::try_from("unknown").is_err());
    }

    #[test]
    fn validates_non_empty_model_ids() {
        assert_eq!(
            validate_model_id("gpt-4o").expect("valid model id"),
            "gpt-4o"
        );
        assert!(validate_model_id("   ").is_err());
    }

    #[test]
    fn builtin_models_cover_each_provider() {
        for provider_type in [
            ProviderType::DeepSeek,
            ProviderType::Qwen,
            ProviderType::OpenAI,
            ProviderType::Gemini,
        ] {
            let models = builtin_models(provider_type);
            assert!(!models.is_empty());
            assert!(models
                .iter()
                .all(|model| model.source == ModelSource::Builtin));
            assert!(models
                .iter()
                .all(|model| model.provider_type == provider_type));
        }
    }

    #[test]
    fn default_models_are_present_in_builtin_lists() {
        for provider_type in [
            ProviderType::DeepSeek,
            ProviderType::Qwen,
            ProviderType::OpenAI,
            ProviderType::Gemini,
        ] {
            let default_model = default_model_for_provider(provider_type);
            assert!(builtin_models(provider_type)
                .iter()
                .any(|model| model.id == default_model));
        }
    }
}
