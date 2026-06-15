use std::{fs, path::Path};

use crate::{
    domain::{
        ports::{ObsidianSettingsRepository, WorkspaceRepository},
        ObsidianSettings,
    },
    error::AppError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsidianSettingsSummary {
    pub settings: ObsidianSettings,
    pub has_obsidian_directory: bool,
}

pub trait ObsidianSettingsService: Send + Sync {
    fn get_obsidian_settings(&self) -> Result<Option<ObsidianSettingsSummary>, AppError>;

    fn save_obsidian_settings(
        &self,
        vault_path: String,
    ) -> Result<ObsidianSettingsSummary, AppError>;
}

pub struct DefaultObsidianSettingsService<'service, WorkspaceRepo, SettingsRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SettingsRepo: ObsidianSettingsRepository + ?Sized,
{
    workspace_repository: &'service WorkspaceRepo,
    settings_repository: &'service SettingsRepo,
}

impl<'service, WorkspaceRepo, SettingsRepo>
    DefaultObsidianSettingsService<'service, WorkspaceRepo, SettingsRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SettingsRepo: ObsidianSettingsRepository + ?Sized,
{
    pub const fn new(
        workspace_repository: &'service WorkspaceRepo,
        settings_repository: &'service SettingsRepo,
    ) -> Self {
        Self {
            workspace_repository,
            settings_repository,
        }
    }
}

impl<WorkspaceRepo, SettingsRepo> ObsidianSettingsService
    for DefaultObsidianSettingsService<'_, WorkspaceRepo, SettingsRepo>
where
    WorkspaceRepo: WorkspaceRepository + ?Sized,
    SettingsRepo: ObsidianSettingsRepository + ?Sized,
{
    fn get_obsidian_settings(&self) -> Result<Option<ObsidianSettingsSummary>, AppError> {
        let workspace = self.workspace_repository.ensure_default_workspace()?;
        let Some(settings) = self.settings_repository.find_by_workspace(&workspace.id)? else {
            return Ok(None);
        };
        let has_obsidian_directory = has_obsidian_directory(Path::new(&settings.vault_path));

        Ok(Some(ObsidianSettingsSummary {
            settings,
            has_obsidian_directory,
        }))
    }

    fn save_obsidian_settings(
        &self,
        vault_path: String,
    ) -> Result<ObsidianSettingsSummary, AppError> {
        let vault_path = vault_path.trim();
        if vault_path.is_empty() {
            return Err(AppError::Validation(
                "Obsidian vault path must not be empty".to_owned(),
            ));
        }

        let path = Path::new(vault_path);
        let metadata = fs::metadata(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                AppError::Validation(format!("Obsidian vault path does not exist: {vault_path}"))
            } else {
                AppError::Validation(format!(
                    "Obsidian vault path could not be accessed: {vault_path}"
                ))
            }
        })?;
        if !metadata.is_dir() {
            return Err(AppError::Validation(format!(
                "Obsidian vault path is not a directory: {vault_path}"
            )));
        }

        let workspace = self.workspace_repository.ensure_default_workspace()?;
        let settings = self.settings_repository.upsert(&workspace.id, vault_path)?;

        Ok(ObsidianSettingsSummary {
            settings,
            has_obsidian_directory: has_obsidian_directory(path),
        })
    }
}

fn has_obsidian_directory(vault_path: &Path) -> bool {
    fs::metadata(vault_path.join(".obsidian"))
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use rusqlite::Connection;

    use super::{DefaultObsidianSettingsService, ObsidianSettingsService};
    use crate::{
        error::AppError,
        infrastructure::database::{
            repositories::{SqliteObsidianSettingsRepository, SqliteWorkspaceRepository},
            Database,
        },
    };

    #[test]
    fn saves_and_reloads_a_valid_vault_path() {
        let fixture = VaultFixture::new(true);
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let settings_repository = SqliteObsidianSettingsRepository::new(&database);
        let service =
            DefaultObsidianSettingsService::new(&workspace_repository, &settings_repository);

        let saved = service
            .save_obsidian_settings(fixture.path_string())
            .expect("save Obsidian settings");
        let reloaded = service
            .get_obsidian_settings()
            .expect("reload Obsidian settings")
            .expect("settings should exist");

        assert_eq!(saved.settings, reloaded.settings);
        assert!(saved.has_obsidian_directory);
        assert!(reloaded.has_obsidian_directory);
    }

    #[test]
    fn allows_a_vault_without_an_obsidian_directory() {
        let fixture = VaultFixture::new(false);
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let settings_repository = SqliteObsidianSettingsRepository::new(&database);
        let service =
            DefaultObsidianSettingsService::new(&workspace_repository, &settings_repository);

        let saved = service
            .save_obsidian_settings(fixture.path_string())
            .expect("save vault without .obsidian");

        assert!(!saved.has_obsidian_directory);
    }

    #[test]
    fn rejects_empty_missing_and_file_paths() {
        let fixture = VaultFixture::new(false);
        let file_path = fixture.path.join("not-a-directory.txt");
        fs::write(&file_path, "file").expect("create ordinary file");
        let missing_path = fixture.path.join("missing");
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let settings_repository = SqliteObsidianSettingsRepository::new(&database);
        let service =
            DefaultObsidianSettingsService::new(&workspace_repository, &settings_repository);

        for result in [
            service.save_obsidian_settings(" \n".to_owned()),
            service.save_obsidian_settings(missing_path.to_string_lossy().into_owned()),
            service.save_obsidian_settings(file_path.to_string_lossy().into_owned()),
        ] {
            assert!(matches!(result, Err(AppError::Validation(_))));
        }
    }

    #[test]
    fn reads_saved_settings_when_the_vault_becomes_unavailable() {
        let fixture = VaultFixture::new(true);
        let path = fixture.path_string();
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let settings_repository = SqliteObsidianSettingsRepository::new(&database);
        let service =
            DefaultObsidianSettingsService::new(&workspace_repository, &settings_repository);
        service
            .save_obsidian_settings(path.clone())
            .expect("save available vault");
        fixture.remove();

        let settings = service
            .get_obsidian_settings()
            .expect("read unavailable vault settings")
            .expect("settings should still exist");

        assert_eq!(settings.settings.vault_path, path);
        assert!(!settings.has_obsidian_directory);
    }

    #[test]
    fn saving_settings_does_not_modify_vault_contents() {
        let fixture = VaultFixture::new(true);
        let marker_path = fixture.path.join("marker.md");
        fs::write(&marker_path, "unchanged").expect("create marker file");
        let before = directory_entries(&fixture.path);
        let database = test_database();
        let workspace_repository = SqliteWorkspaceRepository::new(&database);
        let settings_repository = SqliteObsidianSettingsRepository::new(&database);
        let service =
            DefaultObsidianSettingsService::new(&workspace_repository, &settings_repository);

        service
            .save_obsidian_settings(fixture.path_string())
            .expect("save Obsidian settings");

        assert_eq!(directory_entries(&fixture.path), before);
        assert_eq!(
            fs::read_to_string(marker_path).expect("read marker file"),
            "unchanged"
        );
    }

    fn directory_entries(path: &Path) -> Vec<String> {
        let mut entries = fs::read_dir(path)
            .expect("read fixture directory")
            .map(|entry| {
                entry
                    .expect("read fixture entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }

    struct VaultFixture {
        path: PathBuf,
    }

    impl VaultFixture {
        fn new(with_obsidian_directory: bool) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should follow Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "second-brain-os-obsidian-settings-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create vault fixture");
            if with_obsidian_directory {
                fs::create_dir(path.join(".obsidian")).expect("create .obsidian fixture");
            }
            Self { path }
        }

        fn path_string(&self) -> String {
            self.path.to_string_lossy().into_owned()
        }

        fn remove(&self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    impl Drop for VaultFixture {
        fn drop(&mut self) {
            self.remove();
        }
    }

    fn test_database() -> Database {
        Database::from_connection(Connection::open_in_memory().expect("open in-memory database"))
            .expect("initialize test database")
    }
}
