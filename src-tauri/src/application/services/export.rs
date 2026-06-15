use crate::{
    domain::{
        ports::{
            ExportRecordRepository, KnowledgeMarkdownWriter, KnowledgeRepository,
            ObsidianSettingsRepository, WorkspaceRepository,
        },
        ExportRecord, KnowledgeNode, KnowledgeStatus,
    },
    error::AppError,
};

pub trait ExportService: Send + Sync {
    fn export_knowledge_node(&self, knowledge_id: String) -> Result<ExportRecord, AppError>;
}

pub struct DefaultExportService<
    'service,
    WorkspaceRepo,
    KnowledgeRepo,
    SettingsRepo,
    ExportRepo,
    MarkdownWriter,
> where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
    SettingsRepo: ObsidianSettingsRepository + ?Sized,
    ExportRepo: ExportRecordRepository + ?Sized,
    MarkdownWriter: KnowledgeMarkdownWriter + ?Sized,
{
    workspace_repository: &'service WorkspaceRepo,
    knowledge_repository: &'service KnowledgeRepo,
    settings_repository: &'service SettingsRepo,
    export_repository: &'service ExportRepo,
    markdown_writer: &'service MarkdownWriter,
}

impl<'service, WorkspaceRepo, KnowledgeRepo, SettingsRepo, ExportRepo, MarkdownWriter>
    DefaultExportService<
        'service,
        WorkspaceRepo,
        KnowledgeRepo,
        SettingsRepo,
        ExportRepo,
        MarkdownWriter,
    >
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
    SettingsRepo: ObsidianSettingsRepository + ?Sized,
    ExportRepo: ExportRecordRepository + ?Sized,
    MarkdownWriter: KnowledgeMarkdownWriter + ?Sized,
{
    pub const fn new(
        workspace_repository: &'service WorkspaceRepo,
        knowledge_repository: &'service KnowledgeRepo,
        settings_repository: &'service SettingsRepo,
        export_repository: &'service ExportRepo,
        markdown_writer: &'service MarkdownWriter,
    ) -> Self {
        Self {
            workspace_repository,
            knowledge_repository,
            settings_repository,
            export_repository,
            markdown_writer,
        }
    }

    fn perform_export(&self, knowledge: &KnowledgeNode) -> Result<String, AppError> {
        let settings = self
            .settings_repository
            .find_by_workspace(&knowledge.workspace_id)?
            .ok_or_else(|| {
                AppError::Validation(
                    "Obsidian vault path is not configured for the current workspace".to_owned(),
                )
            })?;

        self.markdown_writer
            .write_markdown(&settings.vault_path, knowledge)
    }

    fn record_failure(
        &self,
        knowledge: &KnowledgeNode,
        export_error: &AppError,
    ) -> Result<(), AppError> {
        let error_message = export_error.to_string();
        self.export_repository
            .insert_failure(
                &knowledge.workspace_id,
                &knowledge.id,
                None,
                &error_message,
            )
            .map(|_| ())
            .map_err(|record_error| {
                AppError::State(format!(
                    "{export_error}; additionally failed to record the export failure: {record_error}"
                ))
            })
    }
}

impl<WorkspaceRepo, KnowledgeRepo, SettingsRepo, ExportRepo, MarkdownWriter> ExportService
    for DefaultExportService<
        '_,
        WorkspaceRepo,
        KnowledgeRepo,
        SettingsRepo,
        ExportRepo,
        MarkdownWriter,
    >
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
    SettingsRepo: ObsidianSettingsRepository + ?Sized,
    ExportRepo: ExportRecordRepository + ?Sized,
    MarkdownWriter: KnowledgeMarkdownWriter + ?Sized,
{
    fn export_knowledge_node(&self, knowledge_id: String) -> Result<ExportRecord, AppError> {
        let knowledge_id = knowledge_id.trim();
        if knowledge_id.is_empty() {
            return Err(AppError::Validation(
                "knowledge_id must not be empty".to_owned(),
            ));
        }

        let workspace = self.workspace_repository.ensure_default_workspace()?;
        let knowledge = self
            .knowledge_repository
            .find_node(&workspace.id, knowledge_id)?;
        if knowledge.status != KnowledgeStatus::Accepted {
            return Err(AppError::Conflict(format!(
                "knowledge node {} has status {} and cannot be exported",
                knowledge.id, knowledge.status
            )));
        }

        match self.perform_export(&knowledge) {
            Ok(export_path) => {
                self.export_repository
                    .insert_success(&workspace.id, &knowledge.id, &export_path)
            }
            Err(error) => {
                self.record_failure(&knowledge, &error)?;
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use rusqlite::{params, Connection};

    use super::{DefaultExportService, ExportService};
    use crate::{
        domain::{
            ports::{
                ExportRecordRepository, KnowledgeRepository, ObsidianSettingsRepository,
                WorkspaceRepository,
            },
            ExportStatus, KnowledgeType,
        },
        error::AppError,
        infrastructure::{
            database::{
                repositories::{
                    SqliteExportRecordRepository, SqliteKnowledgeRepository,
                    SqliteObsidianSettingsRepository, SqliteWorkspaceRepository,
                },
                Database,
            },
            obsidian::FileSystemKnowledgeMarkdownWriter,
        },
    };

    #[test]
    fn exports_an_accepted_node_and_preserves_the_node() {
        let fixture = VaultFixture::new();
        let context = TestContext::new();
        let knowledge = context.insert_accepted("Local First", "Knowledge content.");
        context.configure_vault(&fixture.path);
        let before = context.find_knowledge(&knowledge.id);

        let record = context
            .export(&knowledge.id)
            .expect("export accepted knowledge");
        let after = context.find_knowledge(&knowledge.id);
        let export_path = record
            .export_path
            .as_deref()
            .expect("success should include path");
        let markdown = fs::read_to_string(export_path).expect("read exported Markdown");

        assert_eq!(record.status, ExportStatus::Succeeded);
        assert_eq!(before, after);
        assert!(export_path.contains("SecondBrainOS/Knowledge"));
        assert!(markdown.contains("title: \"Local First\""));
        assert!(markdown.contains("# Local First"));
        assert!(markdown.contains("Knowledge content."));
    }

    #[test]
    fn rejects_non_accepted_missing_cross_workspace_and_empty_ids_without_records() {
        let fixture = VaultFixture::new();
        let context = TestContext::new();
        context.configure_vault(&fixture.path);
        let proposed = context.insert_proposed("proposed-run", "Proposed");
        let archived = context.archive(proposed.clone());
        let proposed = context.insert_proposed("another-proposed-run", "Still proposed");
        let other = context.insert_other_workspace_node();

        for (knowledge_id, expected) in [
            (proposed.id.as_str(), "conflict"),
            (archived.id.as_str(), "conflict"),
            ("missing", "not found"),
            (other.id.as_str(), "not found"),
            (" \n", "validation"),
        ] {
            let error = context
                .export(knowledge_id)
                .expect_err("reject invalid export");
            assert!(
                error.to_string().contains(expected),
                "expected {expected:?}, got {error}"
            );
        }

        assert_eq!(context.export_record_count(), 0);
    }

    #[test]
    fn records_failures_after_an_accepted_node_is_confirmed() {
        let context = TestContext::new();
        let knowledge = context.insert_accepted("Missing vault", "Content");

        let error = context
            .export(&knowledge.id)
            .expect_err("missing settings should fail");
        let record = context.latest_export(&knowledge.id);

        assert!(matches!(error, AppError::Validation(_)));
        assert_eq!(record.status, ExportStatus::Failed);
        assert!(record.export_path.is_none());
        assert!(record
            .error_message
            .as_deref()
            .is_some_and(|message| message.contains("not configured")));
    }

    #[test]
    fn records_offline_non_directory_and_write_failures() {
        let fixture = VaultFixture::new();
        let context = TestContext::new();
        let knowledge = context.insert_accepted("Filesystem failures", "Content");

        let missing = fixture.path.join("missing");
        context.configure_vault(&missing);
        assert!(matches!(
            context.export(&knowledge.id),
            Err(AppError::Validation(_))
        ));

        let ordinary_file = fixture.path.join("ordinary-file");
        fs::write(&ordinary_file, "not a directory").expect("create ordinary file");
        context.configure_vault(&ordinary_file);
        assert!(matches!(
            context.export(&knowledge.id),
            Err(AppError::Validation(_))
        ));

        let blocked_vault = fixture.path.join("blocked-vault");
        fs::create_dir(&blocked_vault).expect("create blocked vault");
        fs::write(
            blocked_vault.join("SecondBrainOS"),
            "blocks directory creation",
        )
        .expect("create blocking file");
        context.configure_vault(&blocked_vault);
        assert!(matches!(
            context.export(&knowledge.id),
            Err(AppError::Io(_))
        ));

        assert_eq!(context.export_record_count(), 3);
        let latest = context.latest_export(&knowledge.id);
        assert_eq!(latest.status, ExportStatus::Failed);
        assert!(latest.error_message.is_some());
    }

    #[test]
    fn repeated_exports_overwrite_one_file_and_append_records() {
        let fixture = VaultFixture::new();
        let context = TestContext::new();
        let knowledge = context.insert_accepted("Repeatable export", "Content");
        context.configure_vault(&fixture.path);

        let first = context.export(&knowledge.id).expect("first export");
        let second = context.export(&knowledge.id).expect("second export");

        assert_eq!(first.export_path, second.export_path);
        assert_eq!(context.export_record_count(), 2);
        assert_eq!(
            fs::read_dir(fixture.path.join("SecondBrainOS/Knowledge"))
                .expect("read export directory")
                .count(),
            1
        );
    }

    struct TestContext {
        database: Database,
        workspace_id: String,
    }

    impl TestContext {
        fn new() -> Self {
            let database = test_database();
            let workspace_repository = SqliteWorkspaceRepository::new(&database);
            let workspace = workspace_repository
                .ensure_default_workspace()
                .expect("create default workspace");

            Self {
                database,
                workspace_id: workspace.id,
            }
        }

        fn export(&self, knowledge_id: &str) -> Result<crate::domain::ExportRecord, AppError> {
            let workspace_repository = SqliteWorkspaceRepository::new(&self.database);
            let knowledge_repository = SqliteKnowledgeRepository::new(&self.database);
            let settings_repository = SqliteObsidianSettingsRepository::new(&self.database);
            let export_repository = SqliteExportRecordRepository::new(&self.database);
            let writer = FileSystemKnowledgeMarkdownWriter::new();
            let service = DefaultExportService::new(
                &workspace_repository,
                &knowledge_repository,
                &settings_repository,
                &export_repository,
                &writer,
            );

            service.export_knowledge_node(knowledge_id.to_owned())
        }

        fn insert_accepted(&self, title: &str, content: &str) -> crate::domain::KnowledgeNode {
            SqliteKnowledgeRepository::new(&self.database)
                .insert_manual_node(&self.workspace_id, title, content, KnowledgeType::Concept)
                .expect("insert accepted knowledge")
        }

        fn insert_proposed(&self, ai_run_id: &str, title: &str) -> crate::domain::KnowledgeNode {
            seed_ai_run(&self.database, &self.workspace_id, ai_run_id);
            SqliteKnowledgeRepository::new(&self.database)
                .insert_proposed_node(
                    &self.workspace_id,
                    ai_run_id,
                    title,
                    "Proposed content",
                    KnowledgeType::Insight,
                )
                .expect("insert proposed knowledge")
        }

        fn archive(&self, knowledge: crate::domain::KnowledgeNode) -> crate::domain::KnowledgeNode {
            SqliteKnowledgeRepository::new(&self.database)
                .archive_proposed_node(&self.workspace_id, &knowledge.id)
                .expect("archive knowledge")
        }

        fn insert_other_workspace_node(&self) -> crate::domain::KnowledgeNode {
            seed_workspace(&self.database, "other-workspace");
            SqliteKnowledgeRepository::new(&self.database)
                .insert_manual_node(
                    "other-workspace",
                    "Other",
                    "Other content",
                    KnowledgeType::Concept,
                )
                .expect("insert other workspace knowledge")
        }

        fn configure_vault(&self, path: &Path) {
            SqliteObsidianSettingsRepository::new(&self.database)
                .upsert(&self.workspace_id, &path.to_string_lossy())
                .expect("configure vault");
        }

        fn find_knowledge(&self, knowledge_id: &str) -> crate::domain::KnowledgeNode {
            SqliteKnowledgeRepository::new(&self.database)
                .find_node(&self.workspace_id, knowledge_id)
                .expect("find knowledge")
        }

        fn latest_export(&self, knowledge_id: &str) -> crate::domain::ExportRecord {
            SqliteExportRecordRepository::new(&self.database)
                .find_latest_for_knowledge(&self.workspace_id, knowledge_id)
                .expect("find latest export")
                .expect("export record should exist")
        }

        fn export_record_count(&self) -> i64 {
            self.database
                .with_connection(|connection| {
                    connection
                        .query_row("SELECT COUNT(*) FROM export_records", [], |row| row.get(0))
                        .map_err(AppError::from)
                })
                .expect("count export records")
        }
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
            .expect("seed workspace");
    }

    fn seed_ai_run(database: &Database, workspace_id: &str, ai_run_id: &str) {
        let source_id = format!("source-{ai_run_id}");
        database
            .with_connection(|connection| {
                connection.execute(
                    "
                    INSERT INTO sources (
                        id,
                        workspace_id,
                        source_type,
                        raw_content,
                        content_hash,
                        metadata_json,
                        inbox_status,
                        captured_at,
                        processed_at,
                        created_at,
                        updated_at,
                        deleted_at
                    )
                    VALUES (
                        ?1, ?2, 'text', 'Source content', ?3, NULL, 'unprocessed',
                        ?4, NULL, ?4, ?4, NULL
                    )
                    ",
                    params![
                        source_id,
                        workspace_id,
                        format!("hash-{ai_run_id}"),
                        "2026-06-15T00:00:00.000Z"
                    ],
                )?;
                connection.execute(
                    "
                    INSERT INTO ai_runs (
                        id,
                        source_id,
                        prompt_version_id,
                        provider_type,
                        model,
                        status,
                        output_text,
                        error_message,
                        created_at,
                        completed_at
                    )
                    VALUES (
                        ?1, ?2, 'builtin-source-summary-v1', 'deepseek',
                        'deepseek-v4-flash', 'succeeded', 'Summary', NULL, ?3, ?3
                    )
                    ",
                    params![ai_run_id, source_id, "2026-06-15T00:00:00.000Z"],
                )?;
                Ok(())
            })
            .expect("seed AI run");
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }

    struct VaultFixture {
        path: PathBuf,
    }

    impl VaultFixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should follow Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "second-brain-os-export-service-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create vault fixture");
            Self { path }
        }
    }

    impl Drop for VaultFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
