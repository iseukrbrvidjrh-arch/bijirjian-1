use crate::{
    domain::{
        ports::{
            KnowledgeRepository, ObsidianSettingsRepository, SourceRepository, WorkspaceRepository,
        },
        KnowledgeNode, KnowledgeStatusCounts, Source,
    },
    error::AppError,
};

const RECENT_ITEM_LIMIT: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardSummary {
    pub inbox_unprocessed_count: usize,
    pub knowledge_counts: KnowledgeStatusCounts,
    pub recent_knowledge: Vec<KnowledgeNode>,
    pub recent_inbox_sources: Vec<Source>,
    pub obsidian_vault_configured: bool,
}

pub trait DashboardService: Send + Sync {
    fn get_dashboard_summary(&self) -> Result<DashboardSummary, AppError>;
}

pub struct DefaultDashboardService<
    'repository,
    WorkspaceRepo,
    SourceRepo,
    KnowledgeRepo,
    SettingsRepo,
> where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
    SettingsRepo: ObsidianSettingsRepository + ?Sized,
{
    workspace_repository: &'repository WorkspaceRepo,
    source_repository: &'repository SourceRepo,
    knowledge_repository: &'repository KnowledgeRepo,
    settings_repository: &'repository SettingsRepo,
}

impl<'repository, WorkspaceRepo, SourceRepo, KnowledgeRepo, SettingsRepo>
    DefaultDashboardService<'repository, WorkspaceRepo, SourceRepo, KnowledgeRepo, SettingsRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
    SettingsRepo: ObsidianSettingsRepository + ?Sized,
{
    pub const fn new(
        workspace_repository: &'repository WorkspaceRepo,
        source_repository: &'repository SourceRepo,
        knowledge_repository: &'repository KnowledgeRepo,
        settings_repository: &'repository SettingsRepo,
    ) -> Self {
        Self {
            workspace_repository,
            source_repository,
            knowledge_repository,
            settings_repository,
        }
    }
}

impl<WorkspaceRepo, SourceRepo, KnowledgeRepo, SettingsRepo> DashboardService
    for DefaultDashboardService<'_, WorkspaceRepo, SourceRepo, KnowledgeRepo, SettingsRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SourceRepo: SourceRepository + ?Sized,
    KnowledgeRepo: KnowledgeRepository + ?Sized,
    SettingsRepo: ObsidianSettingsRepository + ?Sized,
{
    fn get_dashboard_summary(&self) -> Result<DashboardSummary, AppError> {
        let workspace = self.workspace_repository.ensure_default_workspace()?;
        let inbox_unprocessed_count = self.source_repository.count_inbox_sources(&workspace.id)?;
        let knowledge_counts = self
            .knowledge_repository
            .count_nodes_by_status(&workspace.id)?;
        let recent_knowledge = self.knowledge_repository.list_nodes(
            &workspace.id,
            None,
            None,
            None,
            RECENT_ITEM_LIMIT,
        )?;
        let recent_inbox_sources =
            self.source_repository
                .list_inbox_sources(&workspace.id, None, RECENT_ITEM_LIMIT)?;
        let obsidian_vault_configured = self
            .settings_repository
            .find_by_workspace(&workspace.id)?
            .is_some();

        Ok(DashboardSummary {
            inbox_unprocessed_count,
            knowledge_counts,
            recent_knowledge,
            recent_inbox_sources,
            obsidian_vault_configured,
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{DashboardService, DefaultDashboardService};
    use crate::{
        domain::{
            ports::{ObsidianSettingsRepository, SourceRepository, WorkspaceRepository},
            InboxStatus,
        },
        infrastructure::database::{
            repositories::{
                SqliteKnowledgeRepository, SqliteObsidianSettingsRepository,
                SqliteSourceRepository, SqliteWorkspaceRepository,
            },
            Database,
        },
    };

    const TIMESTAMP: &str = "2026-06-15T00:00:00.000Z";

    #[test]
    fn returns_zero_counts_and_empty_recent_lists_for_an_empty_workspace() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let settings_repository = SqliteObsidianSettingsRepository::new(&database);
        let service = DefaultDashboardService::new(
            &workspace_repository,
            &source_repository,
            &knowledge_repository,
            &settings_repository,
        );

        let summary = service
            .get_dashboard_summary()
            .expect("load empty dashboard");

        assert_eq!(summary.inbox_unprocessed_count, 0);
        assert_eq!(summary.knowledge_counts.total, 0);
        assert_eq!(summary.knowledge_counts.proposed, 0);
        assert_eq!(summary.knowledge_counts.accepted, 0);
        assert_eq!(summary.knowledge_counts.archived, 0);
        assert!(summary.recent_knowledge.is_empty());
        assert!(summary.recent_inbox_sources.is_empty());
        assert!(!summary.obsidian_vault_configured);
    }

    #[test]
    fn combines_workspace_counts_recent_items_and_saved_vault_without_file_access() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let source_repository = SqliteSourceRepository::new(&database);
        let knowledge_repository = SqliteKnowledgeRepository::new(&database);
        let settings_repository = SqliteObsidianSettingsRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "dashboard-other-workspace");
        seed_knowledge(&database, &workspace.id);
        seed_knowledge_node(
            &database,
            "other-knowledge",
            "dashboard-other-workspace",
            "accepted",
            "2026-06-15T10:00:00.000Z",
        );
        seed_sources(&database, &source_repository, &workspace.id);
        source_repository
            .insert_text_source(
                "dashboard-other-workspace",
                "Other workspace inbox source",
                None,
            )
            .expect("insert other workspace source");
        settings_repository
            .upsert(&workspace.id, "/offline/dashboard-vault")
            .expect("save offline vault path");
        let rows_before = table_row_counts(&database);
        let service = DefaultDashboardService::new(
            &workspace_repository,
            &source_repository,
            &knowledge_repository,
            &settings_repository,
        );

        let summary = service
            .get_dashboard_summary()
            .expect("load populated dashboard");
        let rows_after = table_row_counts(&database);

        assert_eq!(summary.inbox_unprocessed_count, 7);
        assert_eq!(summary.knowledge_counts.total, 6);
        assert_eq!(summary.knowledge_counts.proposed, 2);
        assert_eq!(summary.knowledge_counts.accepted, 3);
        assert_eq!(summary.knowledge_counts.archived, 1);
        assert_eq!(
            summary
                .recent_knowledge
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "knowledge-6",
                "knowledge-5",
                "knowledge-4",
                "knowledge-3",
                "knowledge-2",
            ]
        );
        assert_eq!(
            summary
                .recent_inbox_sources
                .iter()
                .map(|source| source.raw_content.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Inbox source 7",
                "Inbox source 6",
                "Inbox source 5",
                "Inbox source 4",
                "Inbox source 3",
            ]
        );
        assert!(summary.obsidian_vault_configured);
        assert_eq!(rows_before, rows_after);
    }

    fn seed_knowledge(database: &Database, workspace_id: &str) {
        for (index, status) in [
            "proposed", "accepted", "archived", "accepted", "proposed", "accepted",
        ]
        .into_iter()
        .enumerate()
        {
            seed_knowledge_node(
                database,
                &format!("knowledge-{}", index + 1),
                workspace_id,
                status,
                TIMESTAMP,
            );
        }
    }

    fn seed_knowledge_node(
        database: &Database,
        id: &str,
        workspace_id: &str,
        status: &str,
        created_at: &str,
    ) {
        database
            .with_connection(|connection| {
                connection.execute(
                    "
                    INSERT INTO knowledge_nodes (
                        id,
                        workspace_id,
                        ai_run_id,
                        title,
                        content,
                        knowledge_type,
                        status,
                        created_at,
                        updated_at,
                        archived_at
                    )
                    VALUES (
                        ?1,
                        ?2,
                        NULL,
                        ?1,
                        'Dashboard knowledge content',
                        'concept',
                        ?3,
                        ?4,
                        ?4,
                        CASE WHEN ?3 = 'archived' THEN ?4 ELSE NULL END
                    )
                    ",
                    params![id, workspace_id, status, created_at],
                )?;
                Ok(())
            })
            .expect("insert dashboard knowledge");
    }

    fn seed_sources(
        database: &Database,
        repository: &SqliteSourceRepository<'_>,
        workspace_id: &str,
    ) {
        for index in 1..=7 {
            let source = repository
                .insert_text_source(workspace_id, &format!("Inbox source {index}"), None)
                .expect("insert recent inbox source");
            database
                .with_connection(|connection| {
                    connection.execute(
                        "UPDATE sources SET captured_at = ?1 WHERE id = ?2",
                        params![format!("2026-06-15T00:00:0{index}.000Z"), source.id],
                    )?;
                    Ok(())
                })
                .expect("set inbox source timestamp");
        }

        for (label, status, deleted_at) in [
            ("Processed", InboxStatus::Processed, None),
            ("Dismissed", InboxStatus::Dismissed, None),
            ("Deleted", InboxStatus::Unprocessed, Some(TIMESTAMP)),
        ] {
            let source = repository
                .insert_text_source(workspace_id, &format!("{label} source"), None)
                .expect("insert excluded source");
            database
                .with_connection(|connection| {
                    connection.execute(
                        "
                        UPDATE sources
                        SET inbox_status = ?1,
                            deleted_at = ?2
                        WHERE id = ?3
                        ",
                        params![status.as_str(), deleted_at, source.id],
                    )?;
                    Ok(())
                })
                .expect("set excluded source state");
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
                    VALUES (?1, ?2, NULL, ?3, ?3, NULL)
                    ",
                    params![workspace_id, format!("Workspace {workspace_id}"), TIMESTAMP],
                )?;
                Ok(())
            })
            .expect("insert dashboard workspace");
    }

    fn table_row_counts(database: &Database) -> (i64, i64, i64) {
        database
            .with_connection(|connection| {
                Ok((
                    connection.query_row("SELECT COUNT(*) FROM sources", [], |row| row.get(0))?,
                    connection
                        .query_row("SELECT COUNT(*) FROM knowledge_nodes", [], |row| row.get(0))?,
                    connection.query_row("SELECT COUNT(*) FROM obsidian_settings", [], |row| {
                        row.get(0)
                    })?,
                ))
            })
            .expect("read dashboard table counts")
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
