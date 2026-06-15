use crate::{
    domain::{
        ports::{AiRunRepository, KnowledgeRepository, SourceRepository, WorkspaceRepository},
        KnowledgeNode, KnowledgeType,
    },
    error::AppError,
};

const MAX_TITLE_CHARACTERS: usize = 80;

pub trait KnowledgeDraftService: Send + Sync {
    fn create_from_latest_summary(&self, source_id: String) -> Result<KnowledgeNode, AppError>;
}

pub struct DefaultKnowledgeDraftService<
    'service,
    WorkspaceRepo,
    SourceRepo,
    AiRunRepo,
    KnowledgeRepo,
> where
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
    DefaultKnowledgeDraftService<'service, WorkspaceRepo, SourceRepo, AiRunRepo, KnowledgeRepo>
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

impl<WorkspaceRepo, SourceRepo, AiRunRepo, KnowledgeRepo> KnowledgeDraftService
    for DefaultKnowledgeDraftService<'_, WorkspaceRepo, SourceRepo, AiRunRepo, KnowledgeRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    AiRunRepo: AiRunRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
{
    fn create_from_latest_summary(&self, source_id: String) -> Result<KnowledgeNode, AppError> {
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
        let ai_run = self
            .ai_run_repository
            .find_latest_succeeded_for_source(&source.id)?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "source {} does not have a successful AI summary",
                    source.id
                ))
            })?;
        let content = ai_run
            .output_text
            .as_deref()
            .map(str::trim)
            .filter(|content| !content.is_empty())
            .ok_or_else(|| {
                AppError::State(format!(
                    "successful AI run {} does not contain summary text",
                    ai_run.id
                ))
            })?;
        let title = title_from_summary(content)?;

        self.knowledge_repository.insert_proposed_node(
            &workspace.id,
            &ai_run.id,
            &title,
            content,
            KnowledgeType::Insight,
        )
    }
}

fn title_from_summary(summary: &str) -> Result<String, AppError> {
    let first_line = summary
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .find(|line| !line.is_empty())
        .ok_or_else(|| {
            AppError::Validation(
                "AI summary must contain text that can be used as a knowledge title".to_owned(),
            )
        })?;
    let character_count = first_line.chars().count();

    if character_count <= MAX_TITLE_CHARACTERS {
        return Ok(first_line);
    }

    let mut title = first_line
        .chars()
        .take(MAX_TITLE_CHARACTERS - 1)
        .collect::<String>();
    title.push('…');
    Ok(title)
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{
        title_from_summary, DefaultKnowledgeDraftService, KnowledgeDraftService,
        MAX_TITLE_CHARACTERS,
    };
    use crate::{
        domain::{
            ports::{AiRunRepository, SourceRepository, WorkspaceRepository},
            AiRunStatus, KnowledgeStatus, KnowledgeType, ProviderModel, ProviderType,
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
    fn creates_a_proposed_insight_from_the_latest_successful_summary() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let ai_run_repository = SqliteAiRunRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let source = source_repository
            .insert_text_source(&workspace.id, "Original source", None)
            .expect("insert source");
        let successful_run = ai_run_repository
            .insert_success(
                &source.id,
                "builtin-source-summary-v1",
                ProviderType::DeepSeek,
                ProviderModel::DeepSeekV4Flash,
                "\n  First   summary   line  \nSecond line",
            )
            .expect("insert successful AI run");
        ai_run_repository
            .insert_failure(
                &source.id,
                Some("builtin-source-summary-v1"),
                Some(ProviderType::DeepSeek),
                Some(ProviderModel::DeepSeekV4Flash),
                "Newer failure",
            )
            .expect("insert newer failed AI run");
        let service = DefaultKnowledgeDraftService::new(
            &workspace_repository,
            &source_repository,
            &ai_run_repository,
            &knowledge_repository,
        );
        let source_before = source_repository
            .find_source(&workspace.id, &source.id)
            .expect("read source before draft");
        let run_before = ai_run_repository
            .find_latest_succeeded_for_source(&source.id)
            .expect("read AI run before draft")
            .expect("successful AI run should exist");

        let draft = service
            .create_from_latest_summary(source.id.clone())
            .expect("create knowledge draft");
        let source_after = source_repository
            .find_source(&workspace.id, &source.id)
            .expect("read source after draft");
        let run_after = ai_run_repository
            .find_latest_succeeded_for_source(&source.id)
            .expect("read AI run after draft")
            .expect("successful AI run should still exist");

        assert_eq!(draft.workspace_id, workspace.id);
        assert_eq!(draft.ai_run_id.as_deref(), Some(successful_run.id.as_str()));
        assert_eq!(draft.title, "First summary line");
        assert_eq!(draft.content, "First   summary   line  \nSecond line");
        assert_eq!(draft.knowledge_type, KnowledgeType::Insight);
        assert_eq!(draft.status, KnowledgeStatus::Proposed);
        assert_eq!(source_before, source_after);
        assert_eq!(run_before, run_after);
        assert_eq!(run_after.status, AiRunStatus::Succeeded);
    }

    #[test]
    fn rejects_missing_successful_summaries_and_duplicate_drafts() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let ai_run_repository = SqliteAiRunRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let source = source_repository
            .insert_text_source(&workspace.id, "Source", None)
            .expect("insert source");
        let service = DefaultKnowledgeDraftService::new(
            &workspace_repository,
            &source_repository,
            &ai_run_repository,
            &knowledge_repository,
        );

        assert!(matches!(
            service.create_from_latest_summary(source.id.clone()),
            Err(AppError::NotFound(_))
        ));

        ai_run_repository
            .insert_success(
                &source.id,
                "builtin-source-summary-v1",
                ProviderType::DeepSeek,
                ProviderModel::DeepSeekV4Flash,
                "Summary",
            )
            .expect("insert successful AI run");
        service
            .create_from_latest_summary(source.id.clone())
            .expect("create first draft");

        assert!(matches!(
            service.create_from_latest_summary(source.id),
            Err(AppError::Conflict(_))
        ));
    }

    #[test]
    fn rejects_cross_workspace_deleted_and_empty_source_ids() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let ai_run_repository = SqliteAiRunRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let default_workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "other-workspace", "Other");
        let other_source = source_repository
            .insert_text_source("other-workspace", "Other source", None)
            .expect("insert cross-workspace source");
        let deleted_source = source_repository
            .insert_text_source(&default_workspace.id, "Deleted source", None)
            .expect("insert deleted source");
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE sources SET deleted_at = ?1 WHERE id = ?2",
                    params!["2026-06-14T00:00:00.000Z", deleted_source.id],
                )?;
                Ok(())
            })
            .expect("mark source deleted");
        let service = DefaultKnowledgeDraftService::new(
            &workspace_repository,
            &source_repository,
            &ai_run_repository,
            &knowledge_repository,
        );

        assert!(matches!(
            service.create_from_latest_summary(" \n".to_owned()),
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            service.create_from_latest_summary(other_source.id),
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            service.create_from_latest_summary(deleted_source.id),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn creates_a_unicode_title_with_an_eighty_character_limit() {
        let short = title_from_summary("\n  一   个 标题 \n内容").expect("create short title");
        let long_line = "知".repeat(100);
        let long = title_from_summary(&long_line).expect("create truncated title");

        assert_eq!(short, "一 个 标题");
        assert_eq!(long.chars().count(), MAX_TITLE_CHARACTERS);
        assert!(long.ends_with('…'));
        assert_eq!(
            long.chars().filter(|character| *character == '知').count(),
            79
        );
    }

    fn seed_workspace(database: &Database, id: &str, name: &str) {
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
                    VALUES (?1, ?2, NULL, ?3, ?3, NULL)
                    ",
                    params![id, name, "2026-06-14T00:00:00.000Z"],
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
