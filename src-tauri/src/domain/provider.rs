use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderType {
    DeepSeek,
}

impl ProviderType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek",
        }
    }
}

impl TryFrom<&str> for ProviderType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "deepseek" => Ok(Self::DeepSeek),
            _ => Err(format!("unsupported provider type: {value}")),
        }
    }
}

impl fmt::Display for ProviderType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderModel {
    DeepSeekV4Flash,
    DeepSeekV4Pro,
}

impl ProviderModel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeepSeekV4Flash => "deepseek-v4-flash",
            Self::DeepSeekV4Pro => "deepseek-v4-pro",
        }
    }

    pub const fn provider_type(self) -> ProviderType {
        match self {
            Self::DeepSeekV4Flash | Self::DeepSeekV4Pro => ProviderType::DeepSeek,
        }
    }
}

impl TryFrom<&str> for ProviderModel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "deepseek-v4-flash" => Ok(Self::DeepSeekV4Flash),
            "deepseek-v4-pro" => Ok(Self::DeepSeekV4Pro),
            _ => Err(format!("unsupported provider model: {value}")),
        }
    }
}

impl fmt::Display for ProviderModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSettings {
    pub provider_type: ProviderType,
    pub default_model: ProviderModel,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::{ProviderModel, ProviderType};

    #[test]
    fn parses_deepseek_provider_type() {
        assert_eq!(
            ProviderType::try_from("deepseek"),
            Ok(ProviderType::DeepSeek)
        );
    }

    #[test]
    fn rejects_unknown_provider_type() {
        assert!(ProviderType::try_from("unknown").is_err());
    }

    #[test]
    fn parses_supported_deepseek_models() {
        assert_eq!(
            ProviderModel::try_from("deepseek-v4-flash"),
            Ok(ProviderModel::DeepSeekV4Flash)
        );
        assert_eq!(
            ProviderModel::try_from("deepseek-v4-pro"),
            Ok(ProviderModel::DeepSeekV4Pro)
        );
    }

    #[test]
    fn rejects_unknown_provider_model() {
        assert!(ProviderModel::try_from("deepseek-chat").is_err());
        assert!(ProviderModel::try_from("unknown").is_err());
    }

    #[test]
    fn associates_supported_models_with_deepseek() {
        for model in [ProviderModel::DeepSeekV4Flash, ProviderModel::DeepSeekV4Pro] {
            assert_eq!(model.provider_type(), ProviderType::DeepSeek);
        }
    }
}
