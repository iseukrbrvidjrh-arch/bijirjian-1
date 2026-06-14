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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSettings {
    pub provider_type: ProviderType,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::ProviderType;

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
}
