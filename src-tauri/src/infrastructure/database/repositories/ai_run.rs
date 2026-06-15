use std::io;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, types::Type, OptionalExtension};
use uuid::Uuid;

use crate::{
    domain::{ports::AiRunRepository, AiRun, AiRunStatus, ProviderType},
    error::AppError,
    infrastructure::database::Database,
};

pub struct SqliteAiRunRepository<'database> {
    database: &'database Database,
}

impl<'database> SqliteAiRunRepository<'database> {
    pub const fn new(database: &'database Database) -> Self {
        Self { database }
    }

    fn insert(
        &self,
        source_id: &str,
        prompt_version_id: Option<&str>,
        provider_type: Option<ProviderType>,
        model: Option<&str>,
        status: AiRunStatus,
        output_text: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<AiRun, AppError> {
        if provider_type.is_some() != model.is_some() {
            return Err(AppError::Validation(
                "AI run provider and model must either both be present or both be absent"
                    .to_owned(),
            ));
        }

        let value = match status {
            AiRunStatus::Succeeded => output_text,
            AiRunStatus::Failed => error_message,
        }
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Validation(format!(
                "{} AI run must contain a non-empty {}",
                status,
                if status == AiRunStatus::Succeeded {
                    "output"
                } else {
                    "error message"
                }
            ))
        })?;

        let id = Uuid::new_v4().to_string();
        let completed_at = current_timestamp();
        let output_text = (status == AiRunStatus::Succeeded).then_some(value);
        let error_message = (status == AiRunStatus::Failed).then_some(value);

        self.database.with_connection(|connection| {
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
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
                ",
                params![
                    id,
                    source_id,
                    prompt_version_id,
                    provider_type.map(ProviderType::as_str),
                    model,
                    status.as_str(),
                    output_text,
                    error_message,
                    completed_at,
                ],
            )?;

            find_ai_run(connection, &id)?.ok_or_else(|| {
                AppError::State(format!("AI run {id} was inserted but could not be loaded"))
            })
        })
    }
}

impl AiRunRepository for SqliteAiRunRepository<'_> {
    fn insert_success(
        &self,
        source_id: &str,
        prompt_version_id: &str,
        provider_type: ProviderType,
        model: &str,
        output_text: &str,
    ) -> Result<AiRun, AppError> {
        self.insert(
            source_id,
            Some(prompt_version_id),
            Some(provider_type),
            Some(model),
            AiRunStatus::Succeeded,
            Some(output_text),
            None,
        )
    }

    fn insert_failure(
        &self,
        source_id: &str,
        prompt_version_id: Option<&str>,
        provider_type: Option<ProviderType>,
        model: Option<&str>,
        error_message: &str,
    ) -> Result<AiRun, AppError> {
        self.insert(
            source_id,
            prompt_version_id,
            provider_type,
            model,
            AiRunStatus::Failed,
            None,
            Some(error_message),
        )
    }

    fn find_latest_for_source(&self, source_id: &str) -> Result<Option<AiRun>, AppError> {
        self.database
            .with_connection(|connection| find_latest_ai_run(connection, source_id, None))
    }

    fn find_latest_succeeded_for_source(&self, source_id: &str) -> Result<Option<AiRun>, AppError> {
        self.database.with_connection(|connection| {
            find_latest_ai_run(connection, source_id, Some(AiRunStatus::Succeeded))
        })
    }
}

fn find_latest_ai_run(
    connection: &rusqlite::Connection,
    source_id: &str,
    status: Option<AiRunStatus>,
) -> Result<Option<AiRun>, AppError> {
    connection
        .query_row(
            "
            SELECT
                run.id,
                run.source_id,
                run.prompt_version_id,
                version.version,
                run.provider_type,
                run.model,
                run.status,
                run.output_text,
                run.error_message,
                run.created_at,
                run.completed_at
            FROM ai_runs AS run
            LEFT JOIN prompt_versions AS version
              ON version.id = run.prompt_version_id
            WHERE run.source_id = ?1
              AND (?2 IS NULL OR run.status = ?2)
            ORDER BY run.created_at DESC, run.rowid DESC
            LIMIT 1
            ",
            params![source_id, status.map(AiRunStatus::as_str)],
            map_ai_run,
        )
        .optional()
        .map_err(AppError::from)
}

fn find_ai_run(
    connection: &rusqlite::Connection,
    ai_run_id: &str,
) -> Result<Option<AiRun>, AppError> {
    connection
        .query_row(
            "
            SELECT
                run.id,
                run.source_id,
                run.prompt_version_id,
                version.version,
                run.provider_type,
                run.model,
                run.status,
                run.output_text,
                run.error_message,
                run.created_at,
                run.completed_at
            FROM ai_runs AS run
            LEFT JOIN prompt_versions AS version
              ON version.id = run.prompt_version_id
            WHERE run.id = ?1
            ",
            [ai_run_id],
            map_ai_run,
        )
        .optional()
        .map_err(AppError::from)
}

fn map_ai_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<AiRun> {
    let provider_type = row
        .get::<_, Option<String>>(4)?
        .map(|value| {
            ProviderType::try_from(value.as_str()).map_err(|message| invalid_enum_value(4, message))
        })
        .transpose()?;
    let model = row.get::<_, Option<String>>(5)?;
    let status = row.get::<_, String>(6)?;

    Ok(AiRun {
        id: row.get(0)?,
        source_id: row.get(1)?,
        prompt_version_id: row.get(2)?,
        prompt_version: row.get(3)?,
        provider_type,
        model,
        status: AiRunStatus::try_from(status.as_str())
            .map_err(|message| invalid_enum_value(6, message))?,
        output_text: row.get(7)?,
        error_message: row.get(8)?,
        created_at: row.get(9)?,
        completed_at: row.get(10)?,
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
    use rusqlite::Connection;

    use super::SqliteAiRunRepository;
    use crate::{
        domain::{
            ports::{AiRunRepository, SourceRepository, WorkspaceRepository},
            AiRunStatus, ProviderType,
        },
        error::AppError,
        infrastructure::database::{
            repositories::{SqliteSourceRepository, SqliteWorkspaceRepository},
            Database,
        },
    };

    #[test]
    fn writes_succeeded_and_failed_runs() {
        let database = test_database();
        let source_id = seed_source(&database);
        let repository = SqliteAiRunRepository::new(&database);

        let succeeded = repository
            .insert_success(
                &source_id,
                "builtin-source-summary-v1",
                ProviderType::DeepSeek,
                "deepseek-v4-flash",
                "Summary output",
            )
            .expect("insert succeeded run");
        let failed = repository
            .insert_failure(
                &source_id,
                Some("builtin-source-summary-v1"),
                Some(ProviderType::DeepSeek),
                Some("deepseek-v4-pro"),
                "Provider failed",
            )
            .expect("insert failed run");

        assert_eq!(succeeded.status, AiRunStatus::Succeeded);
        assert_eq!(succeeded.output_text.as_deref(), Some("Summary output"));
        assert!(succeeded.error_message.is_none());
        assert_eq!(failed.status, AiRunStatus::Failed);
        assert!(failed.output_text.is_none());
        assert_eq!(failed.error_message.as_deref(), Some("Provider failed"));
    }

    #[test]
    fn returns_the_latest_run_for_a_source() {
        let database = test_database();
        let source_id = seed_source(&database);
        let repository = SqliteAiRunRepository::new(&database);
        repository
            .insert_success(
                &source_id,
                "builtin-source-summary-v1",
                ProviderType::DeepSeek,
                "deepseek-v4-flash",
                "First summary",
            )
            .expect("insert first run");
        let latest = repository
            .insert_failure(
                &source_id,
                Some("builtin-source-summary-v1"),
                Some(ProviderType::DeepSeek),
                Some("deepseek-v4-flash"),
                "Latest failure",
            )
            .expect("insert latest run");

        let found = repository
            .find_latest_for_source(&source_id)
            .expect("find latest run")
            .expect("latest run should exist");

        assert_eq!(found.id, latest.id);
        assert_eq!(found.prompt_version, Some(1));
        assert_eq!(found.error_message.as_deref(), Some("Latest failure"));
    }

    #[test]
    fn returns_none_when_a_source_has_no_runs() {
        let database = test_database();
        let source_id = seed_source(&database);
        let repository = SqliteAiRunRepository::new(&database);

        assert!(repository
            .find_latest_for_source(&source_id)
            .expect("query latest run")
            .is_none());
    }

    #[test]
    fn latest_succeeded_run_ignores_a_newer_failed_run() {
        let database = test_database();
        let source_id = seed_source(&database);
        let repository = SqliteAiRunRepository::new(&database);
        let succeeded = repository
            .insert_success(
                &source_id,
                "builtin-source-summary-v1",
                ProviderType::DeepSeek,
                "deepseek-v4-flash",
                "Successful summary",
            )
            .expect("insert succeeded run");
        repository
            .insert_failure(
                &source_id,
                Some("builtin-source-summary-v1"),
                Some(ProviderType::DeepSeek),
                Some("deepseek-v4-flash"),
                "Newer failure",
            )
            .expect("insert newer failed run");

        let found = repository
            .find_latest_succeeded_for_source(&source_id)
            .expect("find latest succeeded run")
            .expect("succeeded run should exist");

        assert_eq!(found.id, succeeded.id);
        assert_eq!(found.status, AiRunStatus::Succeeded);
        assert_eq!(found.output_text.as_deref(), Some("Successful summary"));
    }

    #[test]
    fn rejects_empty_success_output_and_failure_messages() {
        let database = test_database();
        let source_id = seed_source(&database);
        let repository = SqliteAiRunRepository::new(&database);

        assert!(matches!(
            repository.insert_success(
                &source_id,
                "builtin-source-summary-v1",
                ProviderType::DeepSeek,
                "deepseek-v4-flash",
                "   ",
            ),
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            repository.insert_failure(&source_id, None, None, None, "\n"),
            Err(AppError::Validation(_))
        ));
    }

    fn seed_source(database: &Database) -> String {
        let workspace_repository = SqliteWorkspaceRepository::new(database);
        let source_repository = SqliteSourceRepository::new(database);
        let workspace = workspace_repository
            .ensure_default_workspace()
            .expect("create default workspace");

        source_repository
            .insert_text_source(&workspace.id, "Source to summarize", None)
            .expect("insert source")
            .id
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
