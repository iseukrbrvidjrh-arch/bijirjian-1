use std::fmt;

use crate::domain::{ProviderModel, ProviderType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiRunStatus {
    Succeeded,
    Failed,
}

impl AiRunStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for AiRunStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("unsupported AI run status: {value}")),
        }
    }
}

impl fmt::Display for AiRunStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiRun {
    pub id: String,
    pub source_id: String,
    pub prompt_version_id: Option<String>,
    pub prompt_version: Option<i64>,
    pub provider_type: Option<ProviderType>,
    pub model: Option<ProviderModel>,
    pub status: AiRunStatus,
    pub output_text: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: String,
}

#[cfg(test)]
mod tests {
    use super::AiRunStatus;

    #[test]
    fn parses_supported_ai_run_statuses() {
        assert_eq!(
            AiRunStatus::try_from("succeeded"),
            Ok(AiRunStatus::Succeeded)
        );
        assert_eq!(AiRunStatus::try_from("failed"), Ok(AiRunStatus::Failed));
        assert!(AiRunStatus::try_from("pending").is_err());
    }
}
