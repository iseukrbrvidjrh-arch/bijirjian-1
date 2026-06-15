use std::io;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, types::Type, OptionalExtension, TransactionBehavior};
use uuid::Uuid;

use crate::{
    domain::{
        ports::KnowledgeRepository, KnowledgeNode, KnowledgeStatus, KnowledgeStatusCounts,
        KnowledgeType,
    },
    error::AppError,
    infrastructure::database::Database,
};

pub struct SqliteKnowledgeRepository<'database> {
    database: &'database Database,
}

impl<'database> SqliteKnowledgeRepository<'database> {
    pub const fn new(database: &'database Database) -> Self {
        Self { database }
    }
}

impl KnowledgeRepository for SqliteKnowledgeRepository<'_> {
    fn find_node(&self, workspace_id: &str, knowledge_id: &str) -> Result<KnowledgeNode, AppError> {
        self.database.with_connection(|connection| {
            connection
                .query_row(
                    "
                    SELECT
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
                    FROM knowledge_nodes
                    WHERE id = ?1
                      AND workspace_id = ?2
                    ",
                    params![knowledge_id, workspace_id],
                    map_knowledge_node,
                )
                .optional()?
                .ok_or_else(|| {
                    AppError::NotFound(format!(
                        "knowledge node {knowledge_id} does not exist in the current workspace"
                    ))
                })
        })
    }

    fn find_latest_for_source(
        &self,
        workspace_id: &str,
        source_id: &str,
    ) -> Result<Option<KnowledgeNode>, AppError> {
        self.database.with_connection(|connection| {
            connection
                .query_row(
                    "
                    SELECT
                        node.id,
                        node.workspace_id,
                        node.ai_run_id,
                        node.title,
                        node.content,
                        node.knowledge_type,
                        node.status,
                        node.created_at,
                        node.updated_at,
                        node.archived_at
                    FROM knowledge_nodes AS node
                    INNER JOIN ai_runs AS run
                      ON run.id = node.ai_run_id
                    WHERE node.workspace_id = ?1
                      AND run.source_id = ?2
                    ORDER BY node.created_at DESC, node.rowid DESC
                    LIMIT 1
                    ",
                    params![workspace_id, source_id],
                    map_knowledge_node,
                )
                .optional()
                .map_err(AppError::from)
        })
    }

    fn insert_manual_node(
        &self,
        workspace_id: &str,
        title: &str,
        content: &str,
        knowledge_type: KnowledgeType,
    ) -> Result<KnowledgeNode, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = current_timestamp();

        self.database.with_connection(|connection| {
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
                VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?7, NULL)
                ",
                params![
                    id,
                    workspace_id,
                    title,
                    content,
                    knowledge_type.as_str(),
                    KnowledgeStatus::Accepted.as_str(),
                    now,
                ],
            )?;

            Ok(KnowledgeNode {
                id,
                workspace_id: workspace_id.to_owned(),
                ai_run_id: None,
                title: title.to_owned(),
                content: content.to_owned(),
                knowledge_type,
                status: KnowledgeStatus::Accepted,
                created_at: now.clone(),
                updated_at: now,
                archived_at: None,
            })
        })
    }

    fn insert_proposed_node(
        &self,
        workspace_id: &str,
        ai_run_id: &str,
        title: &str,
        content: &str,
        knowledge_type: KnowledgeType,
    ) -> Result<KnowledgeNode, AppError> {
        if knowledge_type != KnowledgeType::Insight {
            return Err(AppError::Validation(
                "AI summary drafts must use the insight knowledge type".to_owned(),
            ));
        }

        let id = Uuid::new_v4().to_string();
        let now = current_timestamp();

        self.database.with_connection(|connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let existing_node_id = transaction
                .query_row(
                    "
                    SELECT id
                    FROM knowledge_nodes
                    WHERE ai_run_id = ?1
                    ",
                    [ai_run_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;

            if let Some(existing_node_id) = existing_node_id {
                return Err(AppError::Conflict(format!(
                    "AI run {ai_run_id} already created knowledge node {existing_node_id}"
                )));
            }

            transaction.execute(
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
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, NULL)
                ",
                params![
                    id,
                    workspace_id,
                    ai_run_id,
                    title,
                    content,
                    knowledge_type.as_str(),
                    KnowledgeStatus::Proposed.as_str(),
                    now,
                ],
            )?;

            transaction.commit()?;
            Ok(KnowledgeNode {
                id,
                workspace_id: workspace_id.to_owned(),
                ai_run_id: Some(ai_run_id.to_owned()),
                title: title.to_owned(),
                content: content.to_owned(),
                knowledge_type,
                status: KnowledgeStatus::Proposed,
                created_at: now.clone(),
                updated_at: now,
                archived_at: None,
            })
        })
    }

    fn accept_proposed_node(
        &self,
        workspace_id: &str,
        knowledge_id: &str,
    ) -> Result<KnowledgeNode, AppError> {
        transition_proposed_node(
            self.database,
            workspace_id,
            knowledge_id,
            KnowledgeStatus::Accepted,
        )
    }

    fn archive_proposed_node(
        &self,
        workspace_id: &str,
        knowledge_id: &str,
    ) -> Result<KnowledgeNode, AppError> {
        transition_proposed_node(
            self.database,
            workspace_id,
            knowledge_id,
            KnowledgeStatus::Archived,
        )
    }

    fn list_nodes(
        &self,
        workspace_id: &str,
        status: Option<KnowledgeStatus>,
        knowledge_type: Option<KnowledgeType>,
        query: Option<&str>,
        limit: usize,
    ) -> Result<Vec<KnowledgeNode>, AppError> {
        let limit = i64::try_from(limit)
            .map_err(|_| AppError::Validation("knowledge limit is too large".to_owned()))?;
        let query_pattern = query.map(search_pattern);

        self.database.with_connection(|connection| {
            let mut statement = connection.prepare(
                "
                SELECT
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
                FROM knowledge_nodes
                WHERE workspace_id = ?1
                  AND (?2 IS NULL OR status = ?2)
                  AND (?3 IS NULL OR knowledge_type = ?3)
                  AND (
                      ?4 IS NULL
                      OR LOWER(title) LIKE ?4 ESCAPE '!'
                      OR LOWER(content) LIKE ?4 ESCAPE '!'
                  )
                ORDER BY created_at DESC, rowid DESC
                LIMIT ?5
                ",
            )?;
            let nodes = statement
                .query_map(
                    params![
                        workspace_id,
                        status.map(KnowledgeStatus::as_str),
                        knowledge_type.map(KnowledgeType::as_str),
                        query_pattern,
                        limit
                    ],
                    map_knowledge_node,
                )?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(nodes)
        })
    }

    fn count_nodes_by_status(&self, workspace_id: &str) -> Result<KnowledgeStatusCounts, AppError> {
        self.database.with_connection(|connection| {
            let counts = connection.query_row(
                "
                SELECT
                    COUNT(*),
                    COUNT(CASE WHEN status = ?2 THEN 1 END),
                    COUNT(CASE WHEN status = ?3 THEN 1 END),
                    COUNT(CASE WHEN status = ?4 THEN 1 END)
                FROM knowledge_nodes
                WHERE workspace_id = ?1
                ",
                params![
                    workspace_id,
                    KnowledgeStatus::Proposed.as_str(),
                    KnowledgeStatus::Accepted.as_str(),
                    KnowledgeStatus::Archived.as_str(),
                ],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )?;

            Ok(KnowledgeStatusCounts {
                total: valid_count(counts.0, "total")?,
                proposed: valid_count(counts.1, "proposed")?,
                accepted: valid_count(counts.2, "accepted")?,
                archived: valid_count(counts.3, "archived")?,
            })
        })
    }
}

fn valid_count(value: i64, label: &str) -> Result<usize, AppError> {
    usize::try_from(value)
        .map_err(|_| AppError::State(format!("{label} knowledge node count is invalid")))
}

fn search_pattern(query: &str) -> String {
    let escaped = query
        .to_lowercase()
        .replace('!', "!!")
        .replace('%', "!%")
        .replace('_', "!_");

    format!("%{escaped}%")
}

fn map_knowledge_node(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeNode> {
    let knowledge_type = row.get::<_, String>(5)?;
    let status = row.get::<_, String>(6)?;

    Ok(KnowledgeNode {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        ai_run_id: row.get(2)?,
        title: row.get(3)?,
        content: row.get(4)?,
        knowledge_type: KnowledgeType::try_from(knowledge_type.as_str())
            .map_err(|message| invalid_enum_value(5, message))?,
        status: KnowledgeStatus::try_from(status.as_str())
            .map_err(|message| invalid_enum_value(6, message))?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        archived_at: row.get(9)?,
    })
}

fn transition_proposed_node(
    database: &Database,
    workspace_id: &str,
    knowledge_id: &str,
    target_status: KnowledgeStatus,
) -> Result<KnowledgeNode, AppError> {
    let now = current_timestamp();

    database.with_connection(|connection| {
        let transaction =
            connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let affected_rows = match target_status {
            KnowledgeStatus::Accepted => transaction.execute(
                "
                UPDATE knowledge_nodes
                SET status = ?1,
                    updated_at = ?2,
                    archived_at = NULL
                WHERE id = ?3
                  AND workspace_id = ?4
                  AND status = ?5
                ",
                params![
                    KnowledgeStatus::Accepted.as_str(),
                    now,
                    knowledge_id,
                    workspace_id,
                    KnowledgeStatus::Proposed.as_str(),
                ],
            )?,
            KnowledgeStatus::Archived => transaction.execute(
                "
                UPDATE knowledge_nodes
                SET status = ?1,
                    updated_at = ?2,
                    archived_at = ?2
                WHERE id = ?3
                  AND workspace_id = ?4
                  AND status = ?5
                ",
                params![
                    KnowledgeStatus::Archived.as_str(),
                    now,
                    knowledge_id,
                    workspace_id,
                    KnowledgeStatus::Proposed.as_str(),
                ],
            )?,
            KnowledgeStatus::Proposed => {
                return Err(AppError::State(
                    "proposed is not a valid knowledge review target".to_owned(),
                ));
            }
        };

        if affected_rows == 0 {
            let node_state = transaction
                .query_row(
                    "
                    SELECT workspace_id, status
                    FROM knowledge_nodes
                    WHERE id = ?1
                    ",
                    [knowledge_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?;

            return match node_state {
                Some((node_workspace_id, status)) if node_workspace_id == workspace_id => {
                    Err(AppError::Conflict(format!(
                        "knowledge node {knowledge_id} has status {status} and cannot transition to {}",
                        target_status.as_str()
                    )))
                }
                _ => Err(AppError::NotFound(format!(
                    "knowledge node {knowledge_id} does not exist in the current workspace"
                ))),
            };
        }

        let node = transaction.query_row(
            "
            SELECT
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
            FROM knowledge_nodes
            WHERE id = ?1
              AND workspace_id = ?2
            ",
            params![knowledge_id, workspace_id],
            map_knowledge_node,
        )?;
        transaction.commit()?;

        Ok(node)
    })
}

fn invalid_enum_value(column: usize, message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        Type::Text,
        Box::new(io::Error::new(io::ErrorKind::InvalidData, message)),
    )
}

fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::SqliteKnowledgeRepository;
    use crate::{
        domain::{
            ports::{AiRunRepository, KnowledgeRepository, WorkspaceRepository},
            KnowledgeStatus, KnowledgeType, ProviderModel, ProviderType,
        },
        infrastructure::database::{
            repositories::{SqliteAiRunRepository, SqliteWorkspaceRepository},
            Database,
        },
    };

    #[test]
    fn inserts_a_manual_node_as_accepted() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");

        let node = repository
            .insert_manual_node(
                &workspace.id,
                "SQLite",
                "SQLite is an embedded relational database.",
                KnowledgeType::Tool,
            )
            .expect("insert manual knowledge node");

        assert_eq!(node.workspace_id, workspace.id);
        assert!(node.ai_run_id.is_none());
        assert_eq!(node.knowledge_type, KnowledgeType::Tool);
        assert_eq!(node.status, KnowledgeStatus::Accepted);
        assert!(node.archived_at.is_none());
        assert_eq!(
            repository
                .find_node(&workspace.id, &node.id)
                .expect("find inserted node"),
            node
        );
    }

    #[test]
    fn find_node_rejects_missing_and_cross_workspace_nodes() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "other-workspace", "Other");
        let other_node = repository
            .insert_manual_node(
                "other-workspace",
                "Other node",
                "Other content",
                KnowledgeType::Concept,
            )
            .expect("insert other node");

        assert!(matches!(
            repository.find_node(&workspace.id, "missing"),
            Err(crate::error::AppError::NotFound(_))
        ));
        assert!(matches!(
            repository.find_node(&workspace.id, &other_node.id),
            Err(crate::error::AppError::NotFound(_))
        ));
    }

    #[test]
    fn finds_latest_knowledge_for_a_source_with_workspace_isolation() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let ai_run_repository = SqliteAiRunRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "other-workspace", "Other");
        seed_ai_run_with_source(&database, &workspace.id, "detail-source", "detail-run-1");
        let older = repository
            .insert_proposed_node(
                &workspace.id,
                "detail-run-1",
                "Older",
                "Older content",
                KnowledgeType::Insight,
            )
            .expect("insert older related knowledge");
        let newer_run = ai_run_repository
            .insert_success(
                "detail-source",
                "builtin-source-summary-v1",
                ProviderType::DeepSeek,
                ProviderModel::DeepSeekV4Flash,
                "Newer summary",
            )
            .expect("insert newer AI run");
        let newer = repository
            .insert_proposed_node(
                &workspace.id,
                &newer_run.id,
                "Newer",
                "Newer content",
                KnowledgeType::Insight,
            )
            .expect("insert newer related knowledge");
        seed_ai_run_with_source(
            &database,
            "other-workspace",
            "other-detail-source",
            "other-detail-run",
        );
        repository
            .insert_proposed_node(
                "other-workspace",
                "other-detail-run",
                "Other",
                "Other content",
                KnowledgeType::Insight,
            )
            .expect("insert other workspace knowledge");
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE knowledge_nodes SET created_at = ?1 WHERE id IN (?2, ?3)",
                    params!["2026-06-15T00:00:00.000Z", older.id, newer.id],
                )?;
                Ok(())
            })
            .expect("align knowledge timestamps");

        assert_eq!(
            repository
                .find_latest_for_source(&workspace.id, "detail-source")
                .expect("find latest related knowledge")
                .map(|node| node.id),
            Some(newer.id)
        );
        assert!(repository
            .find_latest_for_source(&workspace.id, "other-detail-source")
            .expect("isolate other workspace knowledge")
            .is_none());
        assert!(repository
            .find_latest_for_source(&workspace.id, "missing-source")
            .expect("return none for missing relation")
            .is_none());
    }

    #[test]
    fn inserts_one_proposed_insight_per_ai_run() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_ai_run(&database, &workspace.id, "ai-run-1");

        let node = repository
            .insert_proposed_node(
                &workspace.id,
                "ai-run-1",
                "Summary title",
                "Summary content",
                KnowledgeType::Insight,
            )
            .expect("insert proposed node");
        let duplicate = repository.insert_proposed_node(
            &workspace.id,
            "ai-run-1",
            "Duplicate",
            "Duplicate content",
            KnowledgeType::Insight,
        );

        assert_eq!(node.ai_run_id.as_deref(), Some("ai-run-1"));
        assert_eq!(node.status, KnowledgeStatus::Proposed);
        assert_eq!(node.knowledge_type, KnowledgeType::Insight);
        assert!(matches!(
            duplicate,
            Err(crate::error::AppError::Conflict(_))
        ));
    }

    #[test]
    fn accepts_only_a_proposed_node_and_preserves_archive_state() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_ai_run(&database, &workspace.id, "accept-run");
        let proposed = repository
            .insert_proposed_node(
                &workspace.id,
                "accept-run",
                "Proposed",
                "Content",
                KnowledgeType::Insight,
            )
            .expect("insert proposed node");
        set_updated_at(&database, &proposed.id, "2026-06-14T00:00:00.000Z");

        let accepted = repository
            .accept_proposed_node(&workspace.id, &proposed.id)
            .expect("accept proposed node");

        assert_eq!(accepted.status, KnowledgeStatus::Accepted);
        assert_ne!(accepted.updated_at, "2026-06-14T00:00:00.000Z");
        assert!(accepted.archived_at.is_none());
        assert!(matches!(
            repository.accept_proposed_node(&workspace.id, &proposed.id),
            Err(crate::error::AppError::Conflict(_))
        ));
        assert!(matches!(
            repository.archive_proposed_node(&workspace.id, &proposed.id),
            Err(crate::error::AppError::Conflict(_))
        ));
    }

    #[test]
    fn archives_only_a_proposed_node_and_sets_archive_timestamp() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_ai_run(&database, &workspace.id, "archive-run");
        let proposed = repository
            .insert_proposed_node(
                &workspace.id,
                "archive-run",
                "Proposed",
                "Content",
                KnowledgeType::Insight,
            )
            .expect("insert proposed node");
        set_updated_at(&database, &proposed.id, "2026-06-14T00:00:00.000Z");

        let archived = repository
            .archive_proposed_node(&workspace.id, &proposed.id)
            .expect("archive proposed node");

        assert_eq!(archived.status, KnowledgeStatus::Archived);
        assert_ne!(archived.updated_at, "2026-06-14T00:00:00.000Z");
        assert_eq!(
            archived.archived_at.as_deref(),
            Some(archived.updated_at.as_str())
        );
        assert!(matches!(
            repository.archive_proposed_node(&workspace.id, &proposed.id),
            Err(crate::error::AppError::Conflict(_))
        ));
        assert!(matches!(
            repository.accept_proposed_node(&workspace.id, &proposed.id),
            Err(crate::error::AppError::Conflict(_))
        ));
    }

    #[test]
    fn rejects_missing_cross_workspace_and_manual_accepted_nodes() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "other-workspace", "Other");
        seed_ai_run(&database, "other-workspace", "other-run");
        let other_node = repository
            .insert_proposed_node(
                "other-workspace",
                "other-run",
                "Other",
                "Other content",
                KnowledgeType::Insight,
            )
            .expect("insert other workspace node");
        let manual_node = repository
            .insert_manual_node(
                &workspace.id,
                "Manual",
                "Manual content",
                KnowledgeType::Concept,
            )
            .expect("insert manual node");

        assert!(matches!(
            repository.accept_proposed_node(&workspace.id, "missing"),
            Err(crate::error::AppError::NotFound(_))
        ));
        assert!(matches!(
            repository.archive_proposed_node(&workspace.id, &other_node.id),
            Err(crate::error::AppError::NotFound(_))
        ));
        assert!(matches!(
            repository.accept_proposed_node(&workspace.id, &manual_node.id),
            Err(crate::error::AppError::Conflict(_))
        ));
    }

    #[test]
    fn transitions_only_the_target_node() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_ai_run(&database, &workspace.id, "target-run");
        seed_ai_run_with_source(
            &database,
            &workspace.id,
            "untouched-source",
            "untouched-run",
        );
        let target = repository
            .insert_proposed_node(
                &workspace.id,
                "target-run",
                "Target",
                "Target content",
                KnowledgeType::Insight,
            )
            .expect("insert target node");
        let untouched = repository
            .insert_proposed_node(
                &workspace.id,
                "untouched-run",
                "Untouched",
                "Untouched content",
                KnowledgeType::Insight,
            )
            .expect("insert untouched node");

        repository
            .accept_proposed_node(&workspace.id, &target.id)
            .expect("accept target node");
        let nodes = repository
            .list_nodes(&workspace.id, None, None, None, 50)
            .expect("list knowledge nodes");
        let untouched_after = nodes
            .into_iter()
            .find(|node| node.id == untouched.id)
            .expect("find untouched node");

        assert_eq!(untouched_after.status, KnowledgeStatus::Proposed);
        assert!(untouched_after.archived_at.is_none());
    }

    #[test]
    fn isolates_nodes_by_workspace() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let default_workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "other-workspace", "Other");
        repository
            .insert_manual_node(
                &default_workspace.id,
                "Default node",
                "Default content",
                KnowledgeType::Concept,
            )
            .expect("insert default workspace node");
        repository
            .insert_manual_node(
                "other-workspace",
                "Other node",
                "Other content",
                KnowledgeType::Insight,
            )
            .expect("insert other workspace node");

        let nodes = repository
            .list_nodes(&default_workspace.id, None, None, None, 50)
            .expect("list default workspace nodes");

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].title, "Default node");
    }

    #[test]
    fn lists_nodes_by_created_at_descending_and_honors_limit() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let older = repository
            .insert_manual_node(
                &workspace.id,
                "Older",
                "Older content",
                KnowledgeType::Resource,
            )
            .expect("insert older node");
        let newer = repository
            .insert_manual_node(
                &workspace.id,
                "Newer",
                "Newer content",
                KnowledgeType::Solution,
            )
            .expect("insert newer node");
        database
            .with_connection(|connection| {
                connection.execute(
                    "
                    UPDATE knowledge_nodes
                    SET created_at = ?1, updated_at = ?1
                    WHERE id = ?2
                    ",
                    params!["2026-06-14T01:00:00.000Z", older.id],
                )?;
                connection.execute(
                    "
                    UPDATE knowledge_nodes
                    SET created_at = ?1, updated_at = ?1
                    WHERE id = ?2
                    ",
                    params!["2026-06-14T02:00:00.000Z", newer.id],
                )?;
                Ok(())
            })
            .expect("set deterministic timestamps");

        let all_nodes = repository
            .list_nodes(&workspace.id, None, None, None, 50)
            .expect("list all nodes");
        let limited_nodes = repository
            .list_nodes(&workspace.id, None, None, None, 1)
            .expect("list limited nodes");

        assert_eq!(
            all_nodes
                .iter()
                .map(|node| node.title.as_str())
                .collect::<Vec<_>>(),
            vec!["Newer", "Older"]
        );
        assert_eq!(limited_nodes.len(), 1);
        assert_eq!(limited_nodes[0].title, "Newer");
    }

    #[test]
    fn filters_nodes_by_status_type_and_combination() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let accepted_concept = repository
            .insert_manual_node(
                &workspace.id,
                "Accepted concept",
                "Accepted concept content",
                KnowledgeType::Concept,
            )
            .expect("insert accepted concept");
        let accepted_tool = repository
            .insert_manual_node(
                &workspace.id,
                "Accepted tool",
                "Accepted tool content",
                KnowledgeType::Tool,
            )
            .expect("insert accepted tool");
        seed_ai_run_with_source(&database, &workspace.id, "proposed-source", "proposed-run");
        let proposed = repository
            .insert_proposed_node(
                &workspace.id,
                "proposed-run",
                "Proposed insight",
                "Proposed content",
                KnowledgeType::Insight,
            )
            .expect("insert proposed insight");
        seed_ai_run_with_source(&database, &workspace.id, "archived-source", "archived-run");
        let archived = repository
            .insert_proposed_node(
                &workspace.id,
                "archived-run",
                "Archived insight",
                "Archived content",
                KnowledgeType::Insight,
            )
            .expect("insert proposed node to archive");
        repository
            .archive_proposed_node(&workspace.id, &archived.id)
            .expect("archive proposed node");

        let all = repository
            .list_nodes(&workspace.id, None, None, None, 50)
            .expect("list all nodes");
        let proposed_nodes = repository
            .list_nodes(
                &workspace.id,
                Some(KnowledgeStatus::Proposed),
                None,
                None,
                50,
            )
            .expect("filter proposed nodes");
        let accepted_nodes = repository
            .list_nodes(
                &workspace.id,
                Some(KnowledgeStatus::Accepted),
                None,
                None,
                50,
            )
            .expect("filter accepted nodes");
        let archived_nodes = repository
            .list_nodes(
                &workspace.id,
                Some(KnowledgeStatus::Archived),
                None,
                None,
                50,
            )
            .expect("filter archived nodes");
        let insight_nodes = repository
            .list_nodes(&workspace.id, None, Some(KnowledgeType::Insight), None, 50)
            .expect("filter insight nodes");
        let accepted_concepts = repository
            .list_nodes(
                &workspace.id,
                Some(KnowledgeStatus::Accepted),
                Some(KnowledgeType::Concept),
                None,
                50,
            )
            .expect("filter accepted concepts");

        assert_eq!(all.len(), 4);
        assert_eq!(proposed_nodes, vec![proposed]);
        assert_eq!(accepted_nodes.len(), 2);
        assert!(accepted_nodes
            .iter()
            .any(|node| node.id == accepted_concept.id));
        assert!(accepted_nodes
            .iter()
            .any(|node| node.id == accepted_tool.id));
        assert_eq!(archived_nodes.len(), 1);
        assert_eq!(archived_nodes[0].id, archived.id);
        assert_eq!(insight_nodes.len(), 2);
        assert_eq!(accepted_concepts, vec![accepted_concept]);
    }

    #[test]
    fn filtered_lists_preserve_workspace_order_and_limit() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "other-filter-workspace", "Other filter");
        let older = repository
            .insert_manual_node(
                &workspace.id,
                "Older concept",
                "Older",
                KnowledgeType::Concept,
            )
            .expect("insert older concept");
        let newer = repository
            .insert_manual_node(
                &workspace.id,
                "First tied concept",
                "First tied",
                KnowledgeType::Concept,
            )
            .expect("insert first tied concept");
        let newest_by_rowid = repository
            .insert_manual_node(
                &workspace.id,
                "Second tied concept",
                "Second tied",
                KnowledgeType::Concept,
            )
            .expect("insert second tied concept");
        repository
            .insert_manual_node(
                "other-filter-workspace",
                "Other concept",
                "Other",
                KnowledgeType::Concept,
            )
            .expect("insert other workspace concept");
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE knowledge_nodes SET created_at = ?1 WHERE id = ?2",
                    params!["2026-06-14T01:00:00.000Z", older.id],
                )?;
                connection.execute(
                    "UPDATE knowledge_nodes SET created_at = ?1 WHERE id = ?2",
                    params!["2026-06-14T02:00:00.000Z", newer.id],
                )?;
                connection.execute(
                    "UPDATE knowledge_nodes SET created_at = ?1 WHERE id = ?2",
                    params!["2026-06-14T02:00:00.000Z", newest_by_rowid.id],
                )?;
                Ok(())
            })
            .expect("set deterministic creation timestamps");

        let filtered = repository
            .list_nodes(
                &workspace.id,
                Some(KnowledgeStatus::Accepted),
                Some(KnowledgeType::Concept),
                None,
                2,
            )
            .expect("list filtered nodes");

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, newest_by_rowid.id);
        assert_eq!(filtered[1].id, newer.id);
    }

    #[test]
    fn searches_title_and_content_case_insensitively_and_escapes_wildcards() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        let title_match = repository
            .insert_manual_node(
                &workspace.id,
                "Local First Architecture",
                "Stores data nearby.",
                KnowledgeType::Concept,
            )
            .expect("insert title match");
        let content_match = repository
            .insert_manual_node(
                &workspace.id,
                "Embedded database",
                "SQLite keeps the knowledge local.",
                KnowledgeType::Tool,
            )
            .expect("insert content match");
        let percent_match = repository
            .insert_manual_node(
                &workspace.id,
                "100% local",
                "Literal percent.",
                KnowledgeType::Insight,
            )
            .expect("insert percent match");
        repository
            .insert_manual_node(
                &workspace.id,
                "100X local",
                "Would match an unescaped percent.",
                KnowledgeType::Insight,
            )
            .expect("insert percent decoy");
        let underscore_match = repository
            .insert_manual_node(
                &workspace.id,
                "snake_case",
                "Literal underscore.",
                KnowledgeType::Resource,
            )
            .expect("insert underscore match");
        repository
            .insert_manual_node(
                &workspace.id,
                "snakeXcase",
                "Would match an unescaped underscore.",
                KnowledgeType::Resource,
            )
            .expect("insert underscore decoy");

        assert_eq!(
            repository
                .list_nodes(&workspace.id, None, None, Some("LOCAL FIRST"), 50)
                .expect("search title"),
            vec![title_match]
        );
        assert_eq!(
            repository
                .list_nodes(&workspace.id, None, None, Some("SQLITE"), 50)
                .expect("search content"),
            vec![content_match]
        );
        assert_eq!(
            repository
                .list_nodes(&workspace.id, None, None, Some("100%"), 50)
                .expect("search literal percent"),
            vec![percent_match]
        );
        assert_eq!(
            repository
                .list_nodes(&workspace.id, None, None, Some("snake_case"), 50)
                .expect("search literal underscore"),
            vec![underscore_match]
        );
        assert!(repository
            .list_nodes(&workspace.id, None, None, Some("missing"), 50)
            .expect("search missing term")
            .is_empty());
    }

    #[test]
    fn combines_search_with_status_type_workspace_order_and_limit() {
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let repository = SqliteKnowledgeRepository::new(&database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");
        seed_workspace(&database, "search-other-workspace", "Search other");
        let older = repository
            .insert_manual_node(
                &workspace.id,
                "Search needle older",
                "Matching content",
                KnowledgeType::Concept,
            )
            .expect("insert older accepted concept");
        let newer = repository
            .insert_manual_node(
                &workspace.id,
                "Search needle newer",
                "Matching content",
                KnowledgeType::Concept,
            )
            .expect("insert newer accepted concept");
        repository
            .insert_manual_node(
                &workspace.id,
                "Search needle tool",
                "Matching content",
                KnowledgeType::Tool,
            )
            .expect("insert accepted tool");
        seed_ai_run_with_source(
            &database,
            &workspace.id,
            "search-proposed-source",
            "search-proposed-run",
        );
        let proposed = repository
            .insert_proposed_node(
                &workspace.id,
                "search-proposed-run",
                "Search needle proposed",
                "Matching content",
                KnowledgeType::Insight,
            )
            .expect("insert proposed insight");
        repository
            .insert_manual_node(
                "search-other-workspace",
                "Search needle elsewhere",
                "Matching content",
                KnowledgeType::Concept,
            )
            .expect("insert cross-workspace match");
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE knowledge_nodes SET created_at = ?1 WHERE id = ?2",
                    params!["2026-06-14T01:00:00.000Z", older.id],
                )?;
                connection.execute(
                    "UPDATE knowledge_nodes SET created_at = ?1 WHERE id = ?2",
                    params!["2026-06-14T02:00:00.000Z", newer.id],
                )?;
                Ok(())
            })
            .expect("set search ordering timestamps");
        let older_before = repository
            .find_node(&workspace.id, &older.id)
            .expect("read node before search");

        let accepted = repository
            .list_nodes(
                &workspace.id,
                Some(KnowledgeStatus::Accepted),
                None,
                Some("needle"),
                50,
            )
            .expect("combine status and search");
        let concepts = repository
            .list_nodes(
                &workspace.id,
                None,
                Some(KnowledgeType::Concept),
                Some("needle"),
                50,
            )
            .expect("combine type and search");
        let proposed_insights = repository
            .list_nodes(
                &workspace.id,
                Some(KnowledgeStatus::Proposed),
                Some(KnowledgeType::Insight),
                Some("needle"),
                50,
            )
            .expect("combine status type and search");
        let limited = repository
            .list_nodes(
                &workspace.id,
                Some(KnowledgeStatus::Accepted),
                Some(KnowledgeType::Concept),
                Some("needle"),
                1,
            )
            .expect("limit searched results");
        let older_after = repository
            .find_node(&workspace.id, &older.id)
            .expect("read node after search");

        assert_eq!(accepted.len(), 3);
        assert_eq!(
            concepts
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec![newer.id.as_str(), older.id.as_str()]
        );
        assert_eq!(proposed_insights, vec![proposed]);
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, newer.id);
        assert_eq!(older_before, older_after);
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

    fn seed_ai_run(database: &Database, workspace_id: &str, ai_run_id: &str) {
        seed_ai_run_with_source(database, workspace_id, "draft-source", ai_run_id);
    }

    fn seed_ai_run_with_source(
        database: &Database,
        workspace_id: &str,
        source_id: &str,
        ai_run_id: &str,
    ) {
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
                        ?1,
                        ?2,
                        'text',
                        'Source',
                        ?3,
                        '{}',
                        'unprocessed',
                        ?4,
                        NULL,
                        ?4,
                        ?4,
                        NULL
                    )
                    ",
                    params![
                        source_id,
                        workspace_id,
                        format!("hash-{source_id}"),
                        "2026-06-14T00:00:00.000Z"
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
                        ?1,
                        ?2,
                        'builtin-source-summary-v1',
                        'deepseek',
                        'deepseek-v4-flash',
                        'succeeded',
                        'Summary content',
                        NULL,
                        ?3,
                        ?3
                    )
                    ",
                    params![ai_run_id, source_id, "2026-06-14T00:00:01.000Z"],
                )?;
                Ok(())
            })
            .expect("seed AI run");
    }

    fn set_updated_at(database: &Database, knowledge_id: &str, updated_at: &str) {
        database
            .with_connection(|connection| {
                connection.execute(
                    "
                    UPDATE knowledge_nodes
                    SET updated_at = ?1
                    WHERE id = ?2
                    ",
                    params![updated_at, knowledge_id],
                )?;
                Ok(())
            })
            .expect("set deterministic updated timestamp");
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
