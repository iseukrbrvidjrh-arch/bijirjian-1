use crate::{
    domain::{ports::PromptRepository, Prompt, PromptVersion},
    error::AppError,
};

const DEFAULT_PROMPT_KEY: &str = "source_summary";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultPromptDetails {
    pub prompt: Prompt,
    pub active_version: PromptVersion,
}

pub trait PromptService: Send + Sync {
    fn get_default_prompt(&self) -> Result<DefaultPromptDetails, AppError>;

    fn list_default_prompt_versions(&self) -> Result<Vec<PromptVersion>, AppError>;

    fn create_default_prompt_version(
        &self,
        prompt_content: String,
    ) -> Result<PromptVersion, AppError>;

    fn set_default_prompt_active_version(
        &self,
        version_id: &str,
    ) -> Result<DefaultPromptDetails, AppError>;
}

pub struct DefaultPromptService<'service, Repository>
where
    Repository: PromptRepository + ?Sized,
{
    repository: &'service Repository,
}

impl<'service, Repository> DefaultPromptService<'service, Repository>
where
    Repository: PromptRepository + ?Sized,
{
    pub const fn new(repository: &'service Repository) -> Self {
        Self { repository }
    }

    fn find_default_prompt(&self) -> Result<Prompt, AppError> {
        self.repository
            .find_by_key(DEFAULT_PROMPT_KEY)?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "default prompt {DEFAULT_PROMPT_KEY} is not initialized"
                ))
            })
    }

    fn load_details(&self, prompt: Prompt) -> Result<DefaultPromptDetails, AppError> {
        let active_version_id = prompt.active_version_id.as_deref().ok_or_else(|| {
            AppError::State(format!(
                "default prompt {} does not have an active version",
                prompt.prompt_key
            ))
        })?;
        let active_version = self
            .repository
            .get_active_version(&prompt.id)?
            .ok_or_else(|| {
                AppError::State(format!(
                    "active prompt version {active_version_id} could not be loaded"
                ))
            })?;

        Ok(DefaultPromptDetails {
            prompt,
            active_version,
        })
    }
}

impl<Repository> PromptService for DefaultPromptService<'_, Repository>
where
    Repository: PromptRepository + ?Sized,
{
    fn get_default_prompt(&self) -> Result<DefaultPromptDetails, AppError> {
        let prompt = self.find_default_prompt()?;
        self.load_details(prompt)
    }

    fn list_default_prompt_versions(&self) -> Result<Vec<PromptVersion>, AppError> {
        let prompt = self.find_default_prompt()?;
        self.repository.list_versions(&prompt.id)
    }

    fn create_default_prompt_version(
        &self,
        prompt_content: String,
    ) -> Result<PromptVersion, AppError> {
        let prompt_content = prompt_content.trim();
        if prompt_content.is_empty() {
            return Err(AppError::Validation(
                "prompt content must not be empty".to_owned(),
            ));
        }

        let prompt = self.find_default_prompt()?;
        self.repository.create_version(&prompt.id, prompt_content)
    }

    fn set_default_prompt_active_version(
        &self,
        version_id: &str,
    ) -> Result<DefaultPromptDetails, AppError> {
        let prompt = self.find_default_prompt()?;
        let updated_prompt = self.repository.set_active_version(&prompt.id, version_id)?;

        self.load_details(updated_prompt)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{DefaultPromptService, PromptService};
    use crate::{
        domain::ports::PromptRepository,
        error::AppError,
        infrastructure::database::{repositories::SqlitePromptRepository, Database},
    };

    #[test]
    fn reads_the_default_prompt_and_active_version() {
        let database = test_database();
        let repository = SqlitePromptRepository::new(&database);
        let service = DefaultPromptService::new(&repository);

        let details = service
            .get_default_prompt()
            .expect("default prompt should be available");

        assert_eq!(details.prompt.prompt_key, "source_summary");
        assert_eq!(details.active_version.version, 1);
        assert_eq!(
            details.prompt.active_version_id.as_deref(),
            Some(details.active_version.id.as_str())
        );
    }

    #[test]
    fn lists_versions_in_descending_order() {
        let database = test_database();
        let repository = SqlitePromptRepository::new(&database);
        let service = DefaultPromptService::new(&repository);
        service
            .create_default_prompt_version("Version two".to_owned())
            .expect("create version 2");
        service
            .create_default_prompt_version("Version three".to_owned())
            .expect("create version 3");

        let versions = service
            .list_default_prompt_versions()
            .expect("list prompt versions");

        assert_eq!(
            versions
                .iter()
                .map(|version| version.version)
                .collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
    }

    #[test]
    fn rejects_empty_prompt_content() {
        let database = test_database();
        let repository = SqlitePromptRepository::new(&database);
        let service = DefaultPromptService::new(&repository);

        assert!(matches!(
            service.create_default_prompt_version("   \n".to_owned()),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn creates_sequential_immutable_versions_without_activating_them() {
        let database = test_database();
        let repository = SqlitePromptRepository::new(&database);
        let service = DefaultPromptService::new(&repository);
        let original = service.get_default_prompt().expect("read original prompt");

        let second = service
            .create_default_prompt_version("  Version two content  ".to_owned())
            .expect("create version 2");
        let third = service
            .create_default_prompt_version("Version three content".to_owned())
            .expect("create version 3");
        let after_create = service
            .get_default_prompt()
            .expect("read prompt after creating versions");
        let versions = service
            .list_default_prompt_versions()
            .expect("list versions");

        assert_eq!(second.version, 2);
        assert_eq!(second.prompt_content, "Version two content");
        assert_eq!(third.version, 3);
        assert_eq!(after_create.active_version.id, original.active_version.id);
        assert_eq!(
            versions
                .iter()
                .find(|version| version.version == 1)
                .expect("version 1 should remain")
                .prompt_content,
            original.active_version.prompt_content
        );
    }

    #[test]
    fn explicitly_sets_the_active_version() {
        let database = test_database();
        let repository = SqlitePromptRepository::new(&database);
        let service = DefaultPromptService::new(&repository);
        let second = service
            .create_default_prompt_version("Version two".to_owned())
            .expect("create version 2");

        let updated = service
            .set_default_prompt_active_version(&second.id)
            .expect("activate version 2");

        assert_eq!(updated.active_version.id, second.id);
        assert_eq!(updated.active_version.version, 2);
        assert_eq!(
            updated.prompt.active_version_id.as_deref(),
            Some(updated.active_version.id.as_str())
        );
    }

    #[test]
    fn rejects_a_missing_version() {
        let database = test_database();
        let repository = SqlitePromptRepository::new(&database);
        let service = DefaultPromptService::new(&repository);

        assert!(matches!(
            service.set_default_prompt_active_version("missing-version"),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn rejects_a_version_owned_by_another_prompt() {
        let database = test_database();
        seed_other_prompt(&database);
        let repository = SqlitePromptRepository::new(&database);
        let service = DefaultPromptService::new(&repository);

        assert!(matches!(
            service.set_default_prompt_active_version("other-prompt-v1"),
            Err(AppError::Conflict(_))
        ));
    }

    #[test]
    fn reports_a_missing_active_version_as_an_application_state_error() {
        let database = test_database();
        database
            .with_connection(|connection| {
                connection.execute(
                    "
                    UPDATE prompts
                    SET active_version_id = NULL
                    WHERE prompt_key = 'source_summary'
                    ",
                    [],
                )?;
                Ok(())
            })
            .expect("remove active version");
        let repository = SqlitePromptRepository::new(&database);
        let service = DefaultPromptService::new(&repository);

        assert!(matches!(
            service.get_default_prompt(),
            Err(AppError::State(_))
        ));
    }

    #[test]
    fn creating_a_new_version_does_not_modify_existing_content() {
        let database = test_database();
        let repository = SqlitePromptRepository::new(&database);
        let prompt = repository
            .find_by_key("source_summary")
            .expect("read prompt")
            .expect("default prompt should exist");
        let original = repository
            .get_active_version(&prompt.id)
            .expect("read active version")
            .expect("active version should exist");

        repository
            .create_version(&prompt.id, "A separate immutable version")
            .expect("create another version");
        let unchanged = repository
            .get_active_version(&prompt.id)
            .expect("read active version again")
            .expect("active version should still exist");

        assert_eq!(unchanged, original);
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }

    fn seed_other_prompt(database: &Database) {
        database
            .with_connection(|connection| {
                connection.execute(
                    "
                    INSERT INTO prompts (
                        id,
                        prompt_key,
                        name,
                        description,
                        active_version_id,
                        created_at,
                        updated_at
                    )
                    VALUES (?1, ?2, ?3, NULL, NULL, ?4, ?4)
                    ",
                    params![
                        "other-prompt",
                        "other_prompt",
                        "Other Prompt",
                        "2026-06-14T00:00:00.000Z"
                    ],
                )?;
                connection.execute(
                    "
                    INSERT INTO prompt_versions (
                        id,
                        prompt_id,
                        version,
                        prompt_content,
                        created_at
                    )
                    VALUES (?1, ?2, 1, ?3, ?4)
                    ",
                    params![
                        "other-prompt-v1",
                        "other-prompt",
                        "Other prompt content",
                        "2026-06-14T00:00:00.000Z"
                    ],
                )?;
                Ok(())
            })
            .expect("seed other prompt");
    }
}
