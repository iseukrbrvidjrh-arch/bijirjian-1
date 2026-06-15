use serde::Serialize;
use tauri::State;

use crate::{
    application::services::{DashboardService, DashboardSummary, DefaultDashboardService},
    commands::{dto::SourceDto, knowledge::KnowledgeNodeDto},
    error::AppError,
    infrastructure::database::repositories::{
        SqliteKnowledgeRepository, SqliteObsidianSettingsRepository, SqliteSourceRepository,
        SqliteWorkspaceRepository,
    },
    state::AppState,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummaryDto {
    inbox_unprocessed_count: usize,
    knowledge_total_count: usize,
    proposed_knowledge_count: usize,
    accepted_knowledge_count: usize,
    archived_knowledge_count: usize,
    recent_knowledge: Vec<KnowledgeNodeDto>,
    recent_inbox_sources: Vec<SourceDto>,
    obsidian_vault_configured: bool,
}

impl From<DashboardSummary> for DashboardSummaryDto {
    fn from(summary: DashboardSummary) -> Self {
        Self {
            inbox_unprocessed_count: summary.inbox_unprocessed_count,
            knowledge_total_count: summary.knowledge_counts.total,
            proposed_knowledge_count: summary.knowledge_counts.proposed,
            accepted_knowledge_count: summary.knowledge_counts.accepted,
            archived_knowledge_count: summary.knowledge_counts.archived,
            recent_knowledge: summary
                .recent_knowledge
                .into_iter()
                .map(Into::into)
                .collect(),
            recent_inbox_sources: summary
                .recent_inbox_sources
                .into_iter()
                .map(Into::into)
                .collect(),
            obsidian_vault_configured: summary.obsidian_vault_configured,
        }
    }
}

#[tauri::command]
pub fn get_dashboard_summary(state: State<'_, AppState>) -> Result<DashboardSummaryDto, AppError> {
    let workspace_repository = SqliteWorkspaceRepository::new(&state.database);
    let source_repository = SqliteSourceRepository::new(&state.database);
    let knowledge_repository = SqliteKnowledgeRepository::new(&state.database);
    let settings_repository = SqliteObsidianSettingsRepository::new(&state.database);
    let service = DefaultDashboardService::new(
        &workspace_repository,
        &source_repository,
        &knowledge_repository,
        &settings_repository,
    );

    service.get_dashboard_summary().map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::DashboardSummaryDto;
    use crate::{application::services::DashboardSummary, domain::KnowledgeStatusCounts};

    #[test]
    fn dashboard_summary_dto_uses_camel_case_without_sensitive_data() {
        let dto = DashboardSummaryDto::from(DashboardSummary {
            inbox_unprocessed_count: 2,
            knowledge_counts: KnowledgeStatusCounts {
                total: 4,
                proposed: 1,
                accepted: 2,
                archived: 1,
            },
            recent_knowledge: Vec::new(),
            recent_inbox_sources: Vec::new(),
            obsidian_vault_configured: true,
        });
        let json = serde_json::to_value(dto).expect("serialize dashboard DTO");
        let object = json.as_object().expect("dashboard DTO should be an object");

        assert_eq!(json["inboxUnprocessedCount"], 2);
        assert_eq!(json["knowledgeTotalCount"], 4);
        assert_eq!(json["proposedKnowledgeCount"], 1);
        assert_eq!(json["acceptedKnowledgeCount"], 2);
        assert_eq!(json["archivedKnowledgeCount"], 1);
        assert_eq!(json["recentKnowledge"], serde_json::json!([]));
        assert_eq!(json["recentInboxSources"], serde_json::json!([]));
        assert_eq!(json["obsidianVaultConfigured"], true);
        assert_eq!(object.len(), 8);
        assert!(object.get("apiKey").is_none());
        assert!(object.get("rawResponse").is_none());
        assert!(object.get("vaultContents").is_none());
    }
}
