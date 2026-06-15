use crate::{
    domain::{
        ports::{AiRunRepository, KnowledgeRepository, SourceRepository, WorkspaceRepository},
        AiRun, KnowledgeNode, Source,
    },
    error::AppError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDetail {
    pub source: Source,
    pub latest_summary: Option<AiRun>,
    pub related_knowledge: Option<KnowledgeNode>,
}

pub trait SourceDetailService: Send + Sync {
    fn get_source_detail(&self, source_id: String) -> Result<SourceDetail, AppError>;
}

pub struct DefaultSourceDetailService<'service, WorkspaceRepo, SourceRepo, AiRunRepo, KnowledgeRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    AiRunRepo: AiRunRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
{
    workspace_repository: &'service WorkspaceRepo,
    source_repository: &'service SourceRepo,
    ai_run_repository: &'service AiRunRepo,
    knowledge_repository: &'service KnowledgeRepo,
}

impl<'service, WorkspaceRepo, SourceRepo, AiRunRepo, KnowledgeRepo>
    DefaultSourceDetailService<'service, WorkspaceRepo, SourceRepo, AiRunRepo, KnowledgeRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    AiRunRepo: AiRunRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
{
    pub const fn new(
        workspace_repository: &'service WorkspaceRepo,
        source_repository: &'service SourceRepo,
        ai_run_repository: &'service AiRunRepo,
        knowledge_repository: &'service KnowledgeRepo,
    ) -> Self {
        Self {
            workspace_repository,
            source_repository,
            ai_run_repository,
            knowledge_repository,
        }
    }
}

impl<WorkspaceRepo, SourceRepo, AiRunRepo, KnowledgeRepo> SourceDetailService
    for DefaultSourceDetailService<'_, WorkspaceRepo, SourceRepo, AiRunRepo, KnowledgeRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    AiRunRepo: AiRunRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
{
    fn get_source_detail(&self, source_id: String) -> Result<SourceDetail, AppError> {
        let source_id = source_id.trim();
        if source_id.is_empty() {
            return Err(AppError::Validation(
                "source_id must not be empty".to_owned(),
            ));
        }

        let workspace = self.workspace_repository.ensure_default_workspace()?;
        let source = self
            .source_repository
            .find_source(&workspace.id, source_id)?;
        let latest_summary = self.ai_run_repository.find_latest_for_source(&source.id)?;
        let related_knowledge = self
            .knowledge_repository
            .find_latest_for_source(&workspace.id, &source.id)?;

        Ok(SourceDetail {
            source,
            latest_summary,
            related_knowledge,
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{DefaultSourceDetailService, SourceDetailService};
    use crate::{
        application::services::{DefaultInboxService, InboxService},
        domain::{
            ports::{AiRunRepository, KnowledgeRepository, SourceRepository, WorkspaceRepository},
            AiRunStatus, InboxStatus, KnowledgeStatus, KnowledgeType, ProviderType, SourceType,
        },
        error::AppError,
        infrastructure::database::{
            repositories::{
                SqliteAiRunRepository, SqliteKnowledgeRepository, SqliteSourceRepository,
                SqliteWorkspaceRepository,
            },
            Database,
        },
    };

    #[test]
    fn returns_text_and_pdf_details_without_a_summary() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let ai_run_repository = SqliteAiRunRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let text = source_repository
            .insert_text_source(&workspace.id, "Text source", None)
            .expect("insert text source");
        let pdf_metadata = r#"{"originalFileName":"guide.pdf","fileSize":1024,"extractedTextLength":18,"capturedVia":"pdf"}"#;
        let pdf = source_repository
            .insert_pdf_source(&workspace.id, "Extracted PDF text", pdf_metadata)
            .expect("insert PDF source");
        let service = DefaultSourceDetailService::new(
            &workspace_repository,
            &source_repository,
            &ai_run_repository,
            &knowledge_repository,
        );

        let text_detail = service.get_source_detail(text.id).expect("get text detail");
        let pdf_detail = service.get_source_detail(pdf.id).expect("get PDF detail");

        assert_eq!(text_detail.source.source_type, SourceType::Text);
        assert!(text_detail.latest_summary.is_none());
        assert!(text_detail.related_knowledge.is_none());
        assert_eq!(pdf_detail.source.source_type, SourceType::Pdf);
        assert_eq!(
            pdf_detail.source.metadata_json.as_deref(),
            Some(pdf_metadata)
        );
        assert!(pdf_detail.latest_summary.is_none());
        assert!(pdf_detail.related_knowledge.is_none());
    }

    #[test]
    fn rejects_empty_missing_cross_workspace_and_deleted_sources() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let ai_run_repository = SqliteAiRunRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "other-workspace");
        let other = source_repository
            .insert_text_source("other-workspace", "Other source", None)
            .expect("insert cross-workspace source");
        let deleted = source_repository
            .insert_text_source(&workspace.id, "Deleted source", None)
            .expect("insert deleted source");
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE sources SET deleted_at = ?1 WHERE id = ?2",
                    params!["2026-06-15T00:00:00.000Z", deleted.id],
                )?;
                Ok(())
            })
            .expect("mark source deleted");
        let service = DefaultSourceDetailService::new(
            &workspace_repository,
            &source_repository,
            &ai_run_repository,
            &knowledge_repository,
        );

        assert!(matches!(
            service.get_source_detail(" \n".to_owned()),
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            service.get_source_detail("missing".to_owned()),
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            service.get_source_detail(other.id),
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            service.get_source_detail(deleted.id),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn returns_the_latest_successful_and_failed_ai_runs() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let ai_run_repository = SqliteAiRunRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let successful_source = source_repository
            .insert_text_source(&workspace.id, "Successful source", None)
            .expect("insert successful source");
        let failed_source = source_repository
            .insert_text_source(&workspace.id, "Failed source", None)
            .expect("insert failed source");
        let successful_run = insert_success(&ai_run_repository, &successful_source.id, "Summary");
        let failed_run = ai_run_repository
            .insert_failure(
                &failed_source.id,
                Some("builtin-source-summary-v1"),
                Some(ProviderType::DeepSeek),
                Some("deepseek-v4-flash"),
                "Provider unavailable",
            )
            .expect("insert failed AI run");
        let service = DefaultSourceDetailService::new(
            &workspace_repository,
            &source_repository,
            &ai_run_repository,
            &knowledge_repository,
        );

        assert_eq!(
            service
                .get_source_detail(successful_source.id)
                .expect("get successful detail")
                .latest_summary,
            Some(successful_run)
        );
        let latest_failed = service
            .get_source_detail(failed_source.id)
            .expect("get failed detail")
            .latest_summary
            .expect("failed AI run should exist");
        assert_eq!(latest_failed, failed_run);
        assert_eq!(latest_failed.status, AiRunStatus::Failed);
    }

    #[test]
    fn returns_related_knowledge_in_proposed_accepted_and_archived_states() {
        for expected_status in [
            KnowledgeStatus::Proposed,
            KnowledgeStatus::Accepted,
            KnowledgeStatus::Archived,
        ] {
            let database = test_database();
            let workspace_repository = SqliteWorkspaceRepository::new(&database);
            let source_repository = SqliteSourceRepository::new(&database);
            let ai_run_repository = SqliteAiRunRepository::new(&database);
            let knowledge_repository = SqliteKnowledgeRepository::new(&database);
            let workspace = workspace_repository
                .ensure_default_workspace()
                .expect("create default workspace");
            let source = source_repository
                .insert_text_source(&workspace.id, "Knowledge source", None)
                .expect("insert source");
            let run = insert_success(&ai_run_repository, &source.id, "Summary");
            let proposed = knowledge_repository
                .insert_proposed_node(
                    &workspace.id,
                    &run.id,
                    "Related",
                    "Summary",
                    KnowledgeType::Insight,
                )
                .expect("insert proposed knowledge");
            let expected = match expected_status {
                KnowledgeStatus::Proposed => proposed,
                KnowledgeStatus::Accepted => knowledge_repository
                    .accept_proposed_node(&workspace.id, &proposed.id)
                    .expect("accept knowledge"),
                KnowledgeStatus::Archived => knowledge_repository
                    .archive_proposed_node(&workspace.id, &proposed.id)
                    .expect("archive knowledge"),
            };
            let service = DefaultSourceDetailService::new(
                &workspace_repository,
                &source_repository,
                &ai_run_repository,
                &knowledge_repository,
            );

            assert_eq!(
                service
                    .get_source_detail(source.id)
                    .expect("get detail")
                    .related_knowledge,
                Some(expected)
            );
        }
    }

    #[test]
    fn returns_latest_related_knowledge_without_mutating_records() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let ai_run_repository = SqliteAiRunRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let source = source_repository
            .insert_text_source(&workspace.id, "Multiple summaries", None)
            .expect("insert source");
        let older_run = insert_success(&ai_run_repository, &source.id, "Older summary");
        let older_node = knowledge_repository
            .insert_proposed_node(
                &workspace.id,
                &older_run.id,
                "Older",
                "Older summary",
                KnowledgeType::Insight,
            )
            .expect("insert older knowledge");
        let newer_run = insert_success(&ai_run_repository, &source.id, "Newer summary");
        let newer_node = knowledge_repository
            .insert_proposed_node(
                &workspace.id,
                &newer_run.id,
                "Newer",
                "Newer summary",
                KnowledgeType::Insight,
            )
            .expect("insert newer knowledge");
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE knowledge_nodes SET created_at = ?1 WHERE id IN (?2, ?3)",
                    params!["2026-06-15T00:00:00.000Z", older_node.id, newer_node.id],
                )?;
                Ok(())
            })
            .expect("align knowledge timestamps");
        let source_before = source_repository
            .find_source(&workspace.id, &source.id)
            .expect("read source before detail");
        let run_before = ai_run_repository
            .find_latest_for_source(&source.id)
            .expect("read AI run before detail");
        let knowledge_before = knowledge_repository
            .find_node(&workspace.id, &newer_node.id)
            .expect("read knowledge before detail");
        let service = DefaultSourceDetailService::new(
            &workspace_repository,
            &source_repository,
            &ai_run_repository,
            &knowledge_repository,
        );

        let detail = service
            .get_source_detail(source.id.clone())
            .expect("get source detail");

        assert_eq!(
            detail.related_knowledge.as_ref().map(|node| &node.id),
            Some(&newer_node.id)
        );
        assert_eq!(
            source_repository
                .find_source(&workspace.id, &source.id)
                .expect("read source after detail"),
            source_before
        );
        assert_eq!(
            ai_run_repository
                .find_latest_for_source(&source.id)
                .expect("read AI run after detail"),
            run_before
        );
        assert_eq!(
            knowledge_repository
                .find_node(&workspace.id, &newer_node.id)
                .expect("read knowledge after detail"),
            knowledge_before
        );
    }

    #[test]
    fn reflects_processed_and_dismissed_lifecycle_states() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let ai_run_repository = SqliteAiRunRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let processed_source = source_repository
            .insert_text_source(&workspace.id, "Process", None)
            .expect("insert processed candidate");
        let dismissed_source = source_repository
            .insert_text_source(&workspace.id, "Dismiss", None)
            .expect("insert dismissed candidate");
        let inbox_service = DefaultInboxService::new(&workspace_repository, &source_repository);
        let detail_service = DefaultSourceDetailService::new(
            &workspace_repository,
            &source_repository,
            &ai_run_repository,
            &knowledge_repository,
        );

        inbox_service
            .mark_source_processed(processed_source.id.clone())
            .expect("mark source processed");
        inbox_service
            .mark_source_dismissed(dismissed_source.id.clone())
            .expect("dismiss source");

        let processed = detail_service
            .get_source_detail(processed_source.id)
            .expect("get processed detail")
            .source;
        let dismissed = detail_service
            .get_source_detail(dismissed_source.id)
            .expect("get dismissed detail")
            .source;

        assert_eq!(processed.inbox_status, InboxStatus::Processed);
        assert!(processed.processed_at.is_some());
        assert_eq!(dismissed.inbox_status, InboxStatus::Dismissed);
        assert!(dismissed.processed_at.is_none());
    }

    fn insert_success(
        repository: &SqliteAiRunRepository<'_>,
        source_id: &str,
        summary: &str,
    ) -> crate::domain::AiRun {
        repository
            .insert_success(
                source_id,
                "builtin-source-summary-v1",
                ProviderType::DeepSeek,
                "deepseek-v4-flash",
                summary,
            )
            .expect("insert successful AI run")
    }

    fn seed_workspace(database: &Database, workspace_id: &str) {
        database
            .with_connection(|connection| {
                connection.execute(
                    "
                    INSERT INTO workspaces (
                        id,
                        name,
                        description,
                        created_at,
                        updated_at,
                        archived_at
                    )
                    VALUES (?1, 'Other', NULL, ?2, ?2, NULL)
                    ",
                    params![workspace_id, "2026-06-15T00:00:00.000Z"],
                )?;
                Ok(())
            })
            .expect("insert workspace");
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
