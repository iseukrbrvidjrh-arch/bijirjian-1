#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub id: String,
    pub prompt_key: String,
    pub name: String,
    pub description: Option<String>,
    pub active_version_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptVersion {
    pub id: String,
    pub prompt_id: String,
    pub version: i64,
    pub prompt_content: String,
    pub created_at: String,
}
