use crate::{
    domain::{KnowledgeNode, KnowledgeStatus, KnowledgeStatusCounts, KnowledgeType},
    error::AppError,
};

pub trait KnowledgeRepository: Send + Sync {
    fn find_node(&self, workspace_id: &str, knowledge_id: &str) -> Result<KnowledgeNode, AppError>;

    fn insert_manual_node(
        &self,
        workspace_id: &str,
        title: &str,
        content: &str,
        knowledge_type: KnowledgeType,
    ) -> Result<KnowledgeNode, AppError>;

    fn insert_proposed_node(
        &self,
        workspace_id: &str,
        ai_run_id: &str,
        title: &str,
        content: &str,
        knowledge_type: KnowledgeType,
    ) -> Result<KnowledgeNode, AppError>;

    fn accept_proposed_node(
        &self,
        workspace_id: &str,
        knowledge_id: &str,
    ) -> Result<KnowledgeNode, AppError>;

    fn archive_proposed_node(
        &self,
        workspace_id: &str,
        knowledge_id: &str,
    ) -> Result<KnowledgeNode, AppError>;

    fn list_nodes(
        &self,
        workspace_id: &str,
        status: Option<KnowledgeStatus>,
        knowledge_type: Option<KnowledgeType>,
        query: Option<&str>,
        limit: usize,
    ) -> Result<Vec<KnowledgeNode>, AppError>;

    fn count_nodes_by_status(&self, workspace_id: &str) -> Result<KnowledgeStatusCounts, AppError>;
}
