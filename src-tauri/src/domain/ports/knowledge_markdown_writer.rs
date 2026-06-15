use crate::{domain::KnowledgeNode, error::AppError};

pub trait KnowledgeMarkdownWriter: Send + Sync {
    fn write_markdown(
        &self,
        vault_path: &str,
        knowledge: &KnowledgeNode,
    ) -> Result<String, AppError>;
}
