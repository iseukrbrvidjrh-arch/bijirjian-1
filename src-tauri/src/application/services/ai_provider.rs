use crate::{
    domain::{
        ports::{CredentialStore, ProviderRouter, ProviderSettingsRepository},
        ProviderModel, ProviderType,
    },
    error::AppError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiProviderSettingsSummary {
    pub provider_type: ProviderType,
    pub default_model: ProviderModel,
    pub has_api_key: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConnectionResult {
    pub provider_type: ProviderType,
    pub message: String,
}

pub trait AiProviderService: Send + Sync {
    fn get_settings(&self) -> Result<Option<AiProviderSettingsSummary>, AppError>;

    fn save_settings(
        &self,
        provider_type: ProviderType,
        default_model: ProviderModel,
        api_key: Option<String>,
    ) -> Result<AiProviderSettingsSummary, AppError>;

    fn test_connection(&self) -> Result<ProviderConnectionResult, AppError>;
}

pub struct DefaultAiProviderService<'service, SettingsRepo, Credentials, Router>
where
    SettingsRepo: ProviderSettingsRepository + ?Sized,
    Credentials: CredentialStore + ?Sized,
    Router: ProviderRouter + ?Sized,
{
    settings_repository: &'service SettingsRepo,
    credential_store: &'service Credentials,
    provider_router: &'service Router,
}

impl<'service, SettingsRepo, Credentials, Router>
    DefaultAiProviderService<'service, SettingsRepo, Credentials, Router>
where
    SettingsRepo: ProviderSettingsRepository + ?Sized,
    Credentials: CredentialStore + ?Sized,
    Router: ProviderRouter + ?Sized,
{
    pub const fn new(
        settings_repository: &'service SettingsRepo,
        credential_store: &'service Credentials,
        provider_router: &'service Router,
    ) -> Self {
        Self {
            settings_repository,
            credential_store,
            provider_router,
        }
    }
}

impl<SettingsRepo, Credentials, Router> AiProviderService
    for DefaultAiProviderService<'_, SettingsRepo, Credentials, Router>
where
    SettingsRepo: ProviderSettingsRepository + ?Sized,
    Credentials: CredentialStore + ?Sized,
    Router: ProviderRouter + ?Sized,
{
    fn get_settings(&self) -> Result<Option<AiProviderSettingsSummary>, AppError> {
        let Some(settings) = self.settings_repository.get_provider_settings()? else {
            return Ok(None);
        };
        let has_api_key = self
            .credential_store
            .get_api_key(settings.provider_type)?
            .is_some();

        Ok(Some(AiProviderSettingsSummary {
            provider_type: settings.provider_type,
            default_model: settings.default_model,
            has_api_key,
            updated_at: settings.updated_at,
        }))
    }

    fn save_settings(
        &self,
        provider_type: ProviderType,
        default_model: ProviderModel,
        api_key: Option<String>,
    ) -> Result<AiProviderSettingsSummary, AppError> {
        if default_model.provider_type() != provider_type {
            return Err(AppError::Validation(format!(
                "model {default_model} is not supported by provider {provider_type}"
            )));
        }

        let api_key = api_key
            .as_deref()
            .map(str::trim)
            .filter(|api_key| !api_key.is_empty());
        let has_existing_api_key = self.credential_store.get_api_key(provider_type)?.is_some();

        if api_key.is_none() && !has_existing_api_key {
            return Err(AppError::Validation(
                "an API key is required when configuring an AI provider for the first time"
                    .to_owned(),
            ));
        }

        if let Some(api_key) = api_key {
            self.credential_store.set_api_key(provider_type, api_key)?;
        }

        let settings = self
            .settings_repository
            .save_provider_settings(provider_type, default_model)?;

        Ok(AiProviderSettingsSummary {
            provider_type: settings.provider_type,
            default_model: settings.default_model,
            has_api_key: true,
            updated_at: settings.updated_at,
        })
    }

    fn test_connection(&self) -> Result<ProviderConnectionResult, AppError> {
        let settings = self
            .settings_repository
            .get_provider_settings()?
            .ok_or_else(|| {
                AppError::Validation(
                    "configure and save an AI provider before testing the connection".to_owned(),
                )
            })?;
        let api_key = self
            .credential_store
            .get_api_key(settings.provider_type)?
            .ok_or_else(|| {
                AppError::Validation(
                    "the configured AI provider does not have a saved API key".to_owned(),
                )
            })?;

        self.provider_router
            .test_connection(settings.provider_type, &api_key)?;

        Ok(ProviderConnectionResult {
            provider_type: settings.provider_type,
            message: format!(
                "{} connection succeeded",
                provider_display_name(settings.provider_type)
            ),
        })
    }
}

fn provider_display_name(provider_type: ProviderType) -> &'static str {
    match provider_type {
        ProviderType::DeepSeek => "DeepSeek",
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use rusqlite::Connection;

    use super::{AiProviderService, DefaultAiProviderService};
    use crate::{
        domain::{
            ports::{CredentialStore, ProviderRouter, ProviderSettingsRepository},
            ProviderModel, ProviderType,
        },
        error::AppError,
        infrastructure::database::{repositories::SqliteProviderSettingsRepository, Database},
    };

    #[test]
    fn saves_provider_settings_for_the_first_time() {
        let database = test_database();
        let repository = SqliteProviderSettingsRepository::new(&database);
        let credentials = FakeCredentialStore::default();
        let router = FakeProviderRouter;
        let service = DefaultAiProviderService::new(&repository, &credentials, &router);

        let saved = service
            .save_settings(
                ProviderType::DeepSeek,
                ProviderModel::DeepSeekV4Flash,
                Some("secret-api-key".to_owned()),
            )
            .expect("first provider save should succeed");

        assert_eq!(saved.provider_type, ProviderType::DeepSeek);
        assert_eq!(saved.default_model, ProviderModel::DeepSeekV4Flash);
        assert!(saved.has_api_key);
        assert_eq!(
            credentials
                .get_api_key(ProviderType::DeepSeek)
                .expect("read fake credential")
                .as_deref(),
            Some("secret-api-key")
        );
        let settings = repository
            .get_provider_settings()
            .expect("read provider settings")
            .expect("provider settings should exist");
        assert_eq!(settings.provider_type, ProviderType::DeepSeek);
        assert_eq!(settings.default_model, ProviderModel::DeepSeekV4Flash);
    }

    #[test]
    fn rejects_first_save_without_an_api_key() {
        let database = test_database();
        let repository = SqliteProviderSettingsRepository::new(&database);
        let credentials = FakeCredentialStore::default();
        let router = FakeProviderRouter;
        let service = DefaultAiProviderService::new(&repository, &credentials, &router);

        assert!(matches!(
            service.save_settings(ProviderType::DeepSeek, ProviderModel::DeepSeekV4Flash, None),
            Err(AppError::Validation(_))
        ));
        assert!(repository
            .get_provider_settings()
            .expect("read provider settings")
            .is_none());
    }

    #[test]
    fn keeps_an_existing_api_key_when_the_input_is_empty() {
        let database = test_database();
        let repository = SqliteProviderSettingsRepository::new(&database);
        let credentials = FakeCredentialStore::default();
        credentials
            .set_api_key(ProviderType::DeepSeek, "existing-key")
            .expect("seed fake credential");
        let router = FakeProviderRouter;
        let service = DefaultAiProviderService::new(&repository, &credentials, &router);

        let saved = service
            .save_settings(
                ProviderType::DeepSeek,
                ProviderModel::DeepSeekV4Pro,
                Some("   ".to_owned()),
            )
            .expect("empty input should retain existing credential");

        assert!(saved.has_api_key);
        assert_eq!(saved.default_model, ProviderModel::DeepSeekV4Pro);
        assert_eq!(
            credentials
                .get_api_key(ProviderType::DeepSeek)
                .expect("read fake credential")
                .as_deref(),
            Some("existing-key")
        );
    }

    #[test]
    fn never_stores_the_api_key_in_sqlite() {
        let database = test_database();
        let repository = SqliteProviderSettingsRepository::new(&database);
        let credentials = FakeCredentialStore::default();
        let router = FakeProviderRouter;
        let service = DefaultAiProviderService::new(&repository, &credentials, &router);

        let summary = service
            .save_settings(
                ProviderType::DeepSeek,
                ProviderModel::DeepSeekV4Flash,
                Some("sqlite-must-not-contain-this-key".to_owned()),
            )
            .expect("save provider settings");

        let schema_columns = database
            .with_connection(|connection| {
                let mut statement =
                    connection.prepare("PRAGMA table_info(ai_provider_settings)")?;
                let columns = statement
                    .query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(columns)
            })
            .expect("read provider settings schema");
        let stored_values = database
            .with_connection(|connection| {
                connection
                    .query_row(
                        "
                        SELECT id || provider_type || default_model || created_at || updated_at
                        FROM ai_provider_settings
                        ",
                        [],
                        |row| row.get::<_, String>(0),
                    )
                    .map_err(AppError::from)
            })
            .expect("read stored provider values");

        assert_eq!(summary.provider_type, ProviderType::DeepSeek);
        assert!(summary.has_api_key);
        assert!(!schema_columns.iter().any(|name| name.contains("key")));
        assert!(!stored_values.contains("sqlite-must-not-contain-this-key"));
    }

    #[test]
    fn rejects_connection_test_when_provider_is_not_configured() {
        let database = test_database();
        let repository = SqliteProviderSettingsRepository::new(&database);
        let credentials = FakeCredentialStore::default();
        let router = FakeProviderRouter;
        let service = DefaultAiProviderService::new(&repository, &credentials, &router);

        assert!(matches!(
            service.test_connection(),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn rejects_connection_test_when_api_key_is_missing() {
        let database = test_database();
        let repository = SqliteProviderSettingsRepository::new(&database);
        repository
            .save_provider_settings(ProviderType::DeepSeek, ProviderModel::DeepSeekV4Flash)
            .expect("seed provider settings");
        let credentials = FakeCredentialStore::default();
        let router = FakeProviderRouter;
        let service = DefaultAiProviderService::new(&repository, &credentials, &router);

        assert!(matches!(
            service.test_connection(),
            Err(AppError::Validation(_))
        ));
    }

    #[derive(Default)]
    struct FakeCredentialStore {
        api_keys: Mutex<HashMap<ProviderType, String>>,
    }

    impl CredentialStore for FakeCredentialStore {
        fn get_api_key(&self, provider_type: ProviderType) -> Result<Option<String>, AppError> {
            Ok(self
                .api_keys
                .lock()
                .expect("fake credential lock")
                .get(&provider_type)
                .cloned())
        }

        fn set_api_key(&self, provider_type: ProviderType, api_key: &str) -> Result<(), AppError> {
            self.api_keys
                .lock()
                .expect("fake credential lock")
                .insert(provider_type, api_key.to_owned());
            Ok(())
        }
    }

    struct FakeProviderRouter;

    impl ProviderRouter for FakeProviderRouter {
        fn test_connection(
            &self,
            _provider_type: ProviderType,
            _api_key: &str,
        ) -> Result<(), AppError> {
            Ok(())
        }

        fn generate_text(
            &self,
            _provider_type: ProviderType,
            _model: ProviderModel,
            _api_key: &str,
            _system_prompt: &str,
            _user_content: &str,
        ) -> Result<String, AppError> {
            Ok("unused".to_owned())
        }
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
