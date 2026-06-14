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

    fn list_nodes(&self, workspace_id: &str, limit: usize) -> Result<Vec<KnowledgeNode>, AppError> {
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
                ORDER BY created_at DESC, rowid DESC
                LIMIT ?2
                ",
            )?;
            let nodes = statement
                .query_map(params![workspace_id, limit], map_knowledge_node)?
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
            .list_nodes(&default_workspace.id, 50)
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
            .list_nodes(&workspace.id, 50)
            .expect("list all nodes");
        let limited_nodes = repository
            .list_nodes(&workspace.id, 1)
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
                        'draft-source',
                        ?1,
                        'text',
                        'Source',
                        'hash',
                        '{}',
                        'unprocessed',
                        ?2,
                        NULL,
                        ?2,
                        ?2,
                        NULL
                    )
                    ",
                    params![workspace_id, "2026-06-14T00:00:00.000Z"],
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
                        'draft-source',
                        'builtin-source-summary-v1',
                        'deepseek',
                        'deepseek-v4-flash',
                        'succeeded',
                        'Summary content',
                        NULL,
                        ?2,
                        ?2
                    )
                    ",
                    params![ai_run_id, "2026-06-14T00:00:01.000Z"],
                )?;
                Ok(())
            })
            .expect("seed AI run");
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
