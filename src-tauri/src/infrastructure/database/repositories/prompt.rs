use chrono::{SecondsFormat, Utc};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use uuid::Uuid;

use crate::{
    domain::{ports::PromptRepository, Prompt, PromptVersion},
    error::AppError,
    infrastructure::database::Database,
};

pub struct SqlitePromptRepository<'database> {
    database: &'database Database,
}

impl<'database> SqlitePromptRepository<'database> {
    pub const fn new(database: &'database Database) -> Self {
        Self { database }
    }
}

impl PromptRepository for SqlitePromptRepository<'_> {
    fn find_by_key(&self, prompt_key: &str) -> Result<Option<Prompt>, AppError> {
        self.database.with_connection(|connection| {
            find_prompt_by_key(connection, prompt_key).map_err(AppError::from)
        })
    }

    fn get_active_version(&self, prompt_id: &str) -> Result<Option<PromptVersion>, AppError> {
        self.database.with_connection(|connection| {
            connection
                .query_row(
                    "
                    SELECT
                        version.id,
                        version.prompt_id,
                        version.version,
                        version.prompt_content,
                        version.created_at
                    FROM prompts AS prompt
                    JOIN prompt_versions AS version
                      ON version.id = prompt.active_version_id
                     AND version.prompt_id = prompt.id
                    WHERE prompt.id = ?1
                    ",
                    [prompt_id],
                    map_prompt_version,
                )
                .optional()
                .map_err(AppError::from)
        })
    }

    fn list_versions(&self, prompt_id: &str) -> Result<Vec<PromptVersion>, AppError> {
        self.database.with_connection(|connection| {
            let mut statement = connection.prepare(
                "
                SELECT id, prompt_id, version, prompt_content, created_at
                FROM prompt_versions
                WHERE prompt_id = ?1
                ORDER BY version DESC
                ",
            )?;
            let versions = statement
                .query_map([prompt_id], map_prompt_version)?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(versions)
        })
    }

    fn create_version(
        &self,
        prompt_id: &str,
        prompt_content: &str,
    ) -> Result<PromptVersion, AppError> {
        self.database.with_connection(|connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

            if find_prompt_by_id(&transaction, prompt_id)?.is_none() {
                return Err(AppError::NotFound(format!(
                    "prompt {prompt_id} does not exist"
                )));
            }

            let next_version = transaction.query_row(
                "
                SELECT COALESCE(MAX(version), 0) + 1
                FROM prompt_versions
                WHERE prompt_id = ?1
                ",
                [prompt_id],
                |row| row.get::<_, i64>(0),
            )?;
            let id = Uuid::new_v4().to_string();
            let created_at = current_timestamp();

            transaction.execute(
                "
                INSERT INTO prompt_versions (
                    id,
                    prompt_id,
                    version,
                    prompt_content,
                    created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5)
                ",
                params![id, prompt_id, next_version, prompt_content, created_at],
            )?;

            let version = find_prompt_version(&transaction, &id)?.ok_or_else(|| {
                AppError::State(format!(
                    "prompt version {id} was created but could not be loaded"
                ))
            })?;

            transaction.commit()?;
            Ok(version)
        })
    }

    fn set_active_version(&self, prompt_id: &str, version_id: &str) -> Result<Prompt, AppError> {
        self.database.with_connection(|connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

            if find_prompt_by_id(&transaction, prompt_id)?.is_none() {
                return Err(AppError::NotFound(format!(
                    "prompt {prompt_id} does not exist"
                )));
            }

            let version_prompt_id = transaction
                .query_row(
                    "SELECT prompt_id FROM prompt_versions WHERE id = ?1",
                    [version_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .ok_or_else(|| {
                    AppError::NotFound(format!("prompt version {version_id} does not exist"))
                })?;

            if version_prompt_id != prompt_id {
                return Err(AppError::Conflict(format!(
                    "prompt version {version_id} does not belong to prompt {prompt_id}"
                )));
            }

            transaction.execute(
                "
                UPDATE prompts
                SET active_version_id = ?1,
                    updated_at = ?2
                WHERE id = ?3
                ",
                params![version_id, current_timestamp(), prompt_id],
            )?;

            let prompt = find_prompt_by_id(&transaction, prompt_id)?.ok_or_else(|| {
                AppError::State(format!(
                    "prompt {prompt_id} was updated but could not be loaded"
                ))
            })?;

            transaction.commit()?;
            Ok(prompt)
        })
    }
}

fn find_prompt_by_key(
    connection: &rusqlite::Connection,
    prompt_key: &str,
) -> rusqlite::Result<Option<Prompt>> {
    connection
        .query_row(
            "
            SELECT
                id,
                prompt_key,
                name,
                description,
                active_version_id,
                created_at,
                updated_at
            FROM prompts
            WHERE prompt_key = ?1
            ",
            [prompt_key],
            map_prompt,
        )
        .optional()
}

fn find_prompt_by_id(
    connection: &rusqlite::Connection,
    prompt_id: &str,
) -> rusqlite::Result<Option<Prompt>> {
    connection
        .query_row(
            "
            SELECT
                id,
                prompt_key,
                name,
                description,
                active_version_id,
                created_at,
                updated_at
            FROM prompts
            WHERE id = ?1
            ",
            [prompt_id],
            map_prompt,
        )
        .optional()
}

fn find_prompt_version(
    connection: &rusqlite::Connection,
    version_id: &str,
) -> rusqlite::Result<Option<PromptVersion>> {
    connection
        .query_row(
            "
            SELECT id, prompt_id, version, prompt_content, created_at
            FROM prompt_versions
            WHERE id = ?1
            ",
            [version_id],
            map_prompt_version,
        )
        .optional()
}

fn map_prompt(row: &rusqlite::Row<'_>) -> rusqlite::Result<Prompt> {
    Ok(Prompt {
        id: row.get(0)?,
        prompt_key: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        active_version_id: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn map_prompt_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<PromptVersion> {
    Ok(PromptVersion {
        id: row.get(0)?,
        prompt_id: row.get(1)?,
        version: row.get(2)?,
        prompt_content: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
