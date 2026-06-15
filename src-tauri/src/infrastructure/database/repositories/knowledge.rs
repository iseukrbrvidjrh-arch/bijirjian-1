use std::io;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, types::Type, OptionalExtension, TransactionBehavior};
use uuid::Uuid;

use crate::{
    domain::{ports::KnowledgeRepository, KnowledgeNode, KnowledgeStatus, KnowledgeType},
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
        limit: usize,
    ) -> Result<Vec<KnowledgeNode>, AppError> {
        let limit = i64::try_from(limit)
            .map_err(|_| AppError::Validation("knowledge limit is too large".to_owned()))?;

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
                ORDER BY created_at DESC, rowid DESC
                LIMIT ?4
                ",
            )?;
            let nodes = statement
                .query_map(
                    params![
                        workspace_id,
                        status.map(KnowledgeStatus::as_str),
                        knowledge_type.map(KnowledgeType::as_str),
                        limit
                    ],
                    map_knowledge_node,
                )?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(nodes)
        })
    }
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
            ports::{KnowledgeRepository, WorkspaceRepository},
            KnowledgeStatus, KnowledgeType,
        },
        infrastructure::database::{repositories::SqliteWorkspaceRepository, Database},
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
            .list_nodes(&workspace.id, None, None, 50)
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
            .list_nodes(&default_workspace.id, None, None, 50)
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
            .list_nodes(&workspace.id, None, None, 50)
            .expect("list all nodes");
        let limited_nodes = repository
            .list_nodes(&workspace.id, None, None, 1)
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
            .list_nodes(&workspace.id, None, None, 50)
            .expect("list all nodes");
        let proposed_nodes = repository
            .list_nodes(&workspace.id, Some(KnowledgeStatus::Proposed), None, 50)
            .expect("filter proposed nodes");
        let accepted_nodes = repository
            .list_nodes(&workspace.id, Some(KnowledgeStatus::Accepted), None, 50)
            .expect("filter accepted nodes");
        let archived_nodes = repository
            .list_nodes(&workspace.id, Some(KnowledgeStatus::Archived), None, 50)
            .expect("filter archived nodes");
        let insight_nodes = repository
            .list_nodes(&workspace.id, None, Some(KnowledgeType::Insight), 50)
            .expect("filter insight nodes");
        let accepted_concepts = repository
            .list_nodes(
                &workspace.id,
                Some(KnowledgeStatus::Accepted),
                Some(KnowledgeType::Concept),
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
                2,
            )
            .expect("list filtered nodes");

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, newest_by_rowid.id);
        assert_eq!(filtered[1].id, newer.id);
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
