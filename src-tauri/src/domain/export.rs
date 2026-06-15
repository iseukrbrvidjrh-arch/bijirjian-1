use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStatus {
    Succeeded,
    Failed,
}

impl ExportStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for ExportStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("unsupported export status: {value}")),
        }
    }
}

impl fmt::Display for ExportStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRecord {
    pub id: String,
    pub workspace_id: String,
    pub knowledge_node_id: String,
    pub export_path: Option<String>,
    pub status: ExportStatus,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::ExportStatus;

    #[test]
    fn parses_supported_export_statuses() {
        assert_eq!(
            ExportStatus::try_from("succeeded"),
            Ok(ExportStatus::Succeeded)
        );
        assert_eq!(ExportStatus::try_from("failed"), Ok(ExportStatus::Failed));
        assert!(ExportStatus::try_from("pending").is_err());
    }
}
