use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Text,
}

impl SourceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
        }
    }
}

impl TryFrom<&str> for SourceType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "text" => Ok(Self::Text),
            _ => Err(format!("unsupported source type: {value}")),
        }
    }
}

impl fmt::Display for SourceType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxStatus {
    Unprocessed,
    Processed,
    Dismissed,
    Failed,
}

impl InboxStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unprocessed => "unprocessed",
            Self::Processed => "processed",
            Self::Dismissed => "dismissed",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for InboxStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "unprocessed" => Ok(Self::Unprocessed),
            "processed" => Ok(Self::Processed),
            "dismissed" => Ok(Self::Dismissed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("unsupported inbox status: {value}")),
        }
    }
}

impl fmt::Display for InboxStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub id: String,
    pub workspace_id: String,
    pub source_type: SourceType,
    pub raw_content: String,
    pub content_hash: String,
    pub metadata_json: Option<String>,
    pub inbox_status: InboxStatus,
    pub captured_at: String,
    pub processed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
