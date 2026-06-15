use std::io;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, types::Type, OptionalExtension, TransactionBehavior};

use crate::{
    domain::{
        ports::ProviderSettingsRepository, validate_model_id, ProviderSettings, ProviderType,
    },
    error::AppError,
    infrastructure::database::Database,
};

const SETTINGS_ID: &str = "default";

pub struct SqliteProviderSettingsRepository<'database> {
    database: &'database Database,
}

impl<'database> SqliteProviderSettingsRepository<'database> {
    pub const fn new(database: &'database Database) -> Self {
        Self { database }
    }
}

impl ProviderSettingsRepository for SqliteProviderSettingsRepository<'_> {
    fn get_provider_settings(&self) -> Result<Option<ProviderSettings>, AppError> {
        self.database.with_connection(|connection| {
            find_provider_settings(connection).map_err(AppError::from)
        })
    }

    fn save_provider_settings(
        &self,
        provider_type: ProviderType,
        default_model: String,
    ) -> Result<ProviderSettings, AppError> {
        let default_model = validate_model_id(&default_model)?;

        self.database.with_connection(|connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let now = current_timestamp();

            transaction.execute(
                "
                INSERT INTO ai_provider_settings (
                    id,
                    provider_type,
                    default_model,
                    created_at,
                    updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?4)
                ON CONFLICT(id) DO UPDATE SET
                    provider_type = excluded.provider_type,
                    default_model = excluded.default_model,
                    updated_at = excluded.updated_at
                ",
                params![SETTINGS_ID, provider_type.as_str(), default_model, now],
            )?;

            let settings = find_provider_settings(&transaction)?.ok_or_else(|| {
                AppError::State(
                    "AI provider settings were saved but could not be loaded".to_owned(),
                )
            })?;

            transaction.commit()?;
            Ok(settings)
        })
    }
}

fn find_provider_settings(
    connection: &rusqlite::Connection,
) -> rusqlite::Result<Option<ProviderSettings>> {
    connection
        .query_row(
            "
            SELECT provider_type, default_model, created_at, updated_at
            FROM ai_provider_settings
            WHERE id = ?1
            ",
            [SETTINGS_ID],
            |row| {
                let provider_type: String = row.get(0)?;
                let default_model: String = row.get(1)?;

                Ok(ProviderSettings {
                    provider_type: ProviderType::try_from(provider_type.as_str())
                        .map_err(|message| invalid_setting_value(0, message))?,
                    default_model,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        )
        .optional()
}

fn invalid_setting_value(column: usize, message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        Type::Text,
        Box::new(io::Error::new(io::ErrorKind::InvalidData, message)),
    )
}

fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
