use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

use crate::error::AppError;

struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
    checksum: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_schema",
        sql: include_str!("../../../migrations/0001_initial_schema.sql"),
        checksum: "058bcd44071d7681dd014440b1467dac863b4f9c2a6225183163a91b53224c3e",
    },
    Migration {
        version: 2,
        name: "ai_provider_settings",
        sql: include_str!("../../../migrations/0002_ai_provider_settings.sql"),
        checksum: "24a4e48ed1d6a06acb7a8eed98616a8fad8ed8fb6ea812a31fb5510f2726c4c5",
    },
    Migration {
        version: 3,
        name: "ai_provider_default_model",
        sql: include_str!("../../../migrations/0003_ai_provider_default_model.sql"),
        checksum: "ed97ee70c9663154021a9685be87e3afdfaf4be611571eaca20836540b2fd1c4",
    },
    Migration {
        version: 4,
        name: "prompt_registry",
        sql: include_str!("../../../migrations/0004_prompt_registry.sql"),
        checksum: "07ad4f5c133637e9c1c705e3676b870c73b63a543459fd61d87a72ee5cdee9ef",
    },
    Migration {
        version: 5,
        name: "ai_runs",
        sql: include_str!("../../../migrations/0005_ai_runs.sql"),
        checksum: "5fad4d119fe5f445ec03dd509fca49f49afc546b5f3adc33cbcee7e686eb3fde",
    },
    Migration {
        version: 6,
        name: "knowledge_nodes",
        sql: include_str!("../../../migrations/0006_knowledge_nodes.sql"),
        checksum: "794ccab3ea170259c9ae431ee6a0f60d1959be4b2ba79bc995729230e6830ac1",
    },
    Migration {
        version: 7,
        name: "knowledge_node_ai_run_origin",
        sql: include_str!("../../../migrations/0007_knowledge_node_ai_run_origin.sql"),
        checksum: "c5fa9ff84dda0e85d6b964c086c9fb3d1c54687ea2e544af7553b6f2d285ddf4",
    },
];

pub fn run(connection: &mut Connection) -> Result<(), AppError> {
    run_migrations(connection, MIGRATIONS)
}

fn run_migrations(connection: &mut Connection, migrations: &[Migration]) -> Result<(), AppError> {
    validate_migrations(migrations)?;

    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS _schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );
        ",
    )?;

    reject_unknown_versions(&transaction, migrations)?;

    for migration in migrations {
        apply_migration(&transaction, migration)?;
    }

    transaction.commit()?;
    Ok(())
}

fn validate_migrations(migrations: &[Migration]) -> Result<(), AppError> {
    let mut previous_version = None;

    for migration in migrations {
        if migration.name.trim().is_empty() {
            return Err(AppError::Migration(format!(
                "migration {} has an empty name",
                migration.version
            )));
        }

        if migration.checksum.trim().is_empty() {
            return Err(AppError::Migration(format!(
                "migration {} has an empty checksum",
                migration.version
            )));
        }

        let actual_checksum = checksum(migration.sql);
        if actual_checksum != migration.checksum {
            return Err(AppError::Migration(format!(
                "migration {} checksum does not match its SQL",
                migration.version
            )));
        }

        if let Some(previous_version) = previous_version {
            if migration.version <= previous_version {
                return Err(AppError::Migration(format!(
                    "migration versions must be strictly increasing: {} follows {}",
                    migration.version, previous_version
                )));
            }
        }

        previous_version = Some(migration.version);
    }

    Ok(())
}

fn reject_unknown_versions(
    transaction: &Transaction<'_>,
    migrations: &[Migration],
) -> Result<(), AppError> {
    let latest_known_version = migrations
        .last()
        .map(|migration| migration.version)
        .unwrap_or_default();
    let mut statement =
        transaction.prepare("SELECT version FROM _schema_migrations ORDER BY version")?;
    let applied_versions = statement
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for version in applied_versions {
        if migrations
            .iter()
            .all(|migration| migration.version != version)
        {
            let message = if version > latest_known_version {
                format!(
                    "database migration version {version} is higher than the current application version {latest_known_version}"
                )
            } else {
                format!(
                    "database contains unknown migration version {version}; the database may be incompatible with this application"
                )
            };

            return Err(AppError::Migration(message));
        }
    }

    Ok(())
}

fn apply_migration(transaction: &Transaction<'_>, migration: &Migration) -> Result<(), AppError> {
    let applied = transaction
        .query_row(
            "
            SELECT name, checksum
            FROM _schema_migrations
            WHERE version = ?1
            ",
            [migration.version],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;

    if let Some((applied_name, applied_checksum)) = applied {
        if applied_name != migration.name || applied_checksum != migration.checksum {
            return Err(AppError::Migration(format!(
                "migration {} does not match its applied record",
                migration.version
            )));
        }

        return Ok(());
    }

    transaction.execute_batch(migration.sql)?;
    transaction.execute(
        "
        INSERT INTO _schema_migrations (version, name, checksum, applied_at)
        VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        ",
        params![migration.version, migration.name, migration.checksum],
    )?;

    Ok(())
}

fn checksum(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.as_bytes()))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::{Arc, Barrier},
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use rusqlite::{Connection, OptionalExtension};

    use super::{checksum, run, run_migrations, Migration, MIGRATIONS};
    use crate::error::AppError;

    #[test]
    fn migrates_a_new_database() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");

        run(&mut connection).expect("run initial migration");

        assert_eq!(
            table_exists(&connection, "workspaces"),
            Some("workspaces".into())
        );
        assert_eq!(table_exists(&connection, "sources"), Some("sources".into()));
        assert_eq!(
            table_exists(&connection, "ai_provider_settings"),
            Some("ai_provider_settings".into())
        );
        assert_eq!(table_exists(&connection, "prompts"), Some("prompts".into()));
        assert_eq!(
            table_exists(&connection, "prompt_versions"),
            Some("prompt_versions".into())
        );
        assert_eq!(table_exists(&connection, "ai_runs"), Some("ai_runs".into()));
        assert_eq!(
            table_exists(&connection, "knowledge_nodes"),
            Some("knowledge_nodes".into())
        );
        assert_eq!(
            column_exists(&connection, "knowledge_nodes", "ai_run_id"),
            Some("ai_run_id".into())
        );
        assert_eq!(migration_count(&connection), 7);
    }

    #[test]
    fn repeated_runs_are_idempotent() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");

        run(&mut connection).expect("run initial migration");
        run(&mut connection).expect("run migrations again");

        assert_eq!(
            table_exists(&connection, "workspaces"),
            Some("workspaces".into())
        );
        assert_eq!(table_exists(&connection, "sources"), Some("sources".into()));
        assert_eq!(
            table_exists(&connection, "ai_provider_settings"),
            Some("ai_provider_settings".into())
        );
        assert_eq!(table_exists(&connection, "prompts"), Some("prompts".into()));
        assert_eq!(
            table_exists(&connection, "prompt_versions"),
            Some("prompt_versions".into())
        );
        assert_eq!(table_exists(&connection, "ai_runs"), Some("ai_runs".into()));
        assert_eq!(
            table_exists(&connection, "knowledge_nodes"),
            Some("knowledge_nodes".into())
        );
        assert_eq!(
            column_exists(&connection, "knowledge_nodes", "ai_run_id"),
            Some("ai_run_id".into())
        );
        assert_eq!(migration_count(&connection), 7);
    }

    #[test]
    fn applies_ai_run_origin_to_an_existing_version_six_database() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");
        run_migrations(&mut connection, &MIGRATIONS[..6]).expect("run through migration 6");
        connection
            .execute(
                "
                INSERT INTO workspaces (
                    id,
                    name,
                    description,
                    created_at,
                    updated_at,
                    archived_at
                )
                VALUES (
                    'migration-workspace',
                    'Migration',
                    NULL,
                    '2026-06-14T00:00:00.000Z',
                    '2026-06-14T00:00:00.000Z',
                    NULL
                )
                ",
                [],
            )
            .expect("seed workspace before migration 7");
        connection
            .execute(
                "
                INSERT INTO knowledge_nodes (
                    id,
                    workspace_id,
                    title,
                    content,
                    knowledge_type,
                    status,
                    created_at,
                    updated_at,
                    archived_at
                )
                VALUES (
                    'manual-node-before-origin',
                    'migration-workspace',
                    'Manual node',
                    'Manual content',
                    'concept',
                    'accepted',
                    '2026-06-14T00:00:00.000Z',
                    '2026-06-14T00:00:00.000Z',
                    NULL
                )
                ",
                [],
            )
            .expect("seed manual node before migration 7");
        assert_eq!(
            column_exists(&connection, "knowledge_nodes", "ai_run_id"),
            None
        );

        run(&mut connection).expect("apply knowledge AI run origin migration");

        assert_eq!(
            column_exists(&connection, "knowledge_nodes", "ai_run_id"),
            Some("ai_run_id".into())
        );
        let existing_origin = connection
            .query_row(
                "
                SELECT ai_run_id
                FROM knowledge_nodes
                WHERE id = 'manual-node-before-origin'
                ",
                [],
                |row| row.get::<_, Option<String>>(0),
            )
            .expect("read migrated manual node origin");
        assert!(existing_origin.is_none());
        assert_eq!(migration_count(&connection), 7);
    }

    #[test]
    fn applies_knowledge_nodes_to_an_existing_version_five_database() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");
        run_migrations(&mut connection, &MIGRATIONS[..5]).expect("run through migration 5");
        assert_eq!(table_exists(&connection, "knowledge_nodes"), None);

        run(&mut connection).expect("apply knowledge nodes migration");

        assert_eq!(
            table_exists(&connection, "knowledge_nodes"),
            Some("knowledge_nodes".into())
        );
        assert_eq!(migration_count(&connection), 7);
    }

    #[test]
    fn applies_ai_runs_to_an_existing_version_four_database() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");
        run_migrations(&mut connection, &MIGRATIONS[..4]).expect("run through migration 4");
        assert_eq!(table_exists(&connection, "ai_runs"), None);

        run(&mut connection).expect("apply AI runs migration");

        assert_eq!(table_exists(&connection, "ai_runs"), Some("ai_runs".into()));
        assert_eq!(migration_count(&connection), 7);
    }

    #[test]
    fn initializes_the_default_source_summary_prompt() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");

        run(&mut connection).expect("run migrations");

        let (prompt_key, active_version_id, version, prompt_content) = connection
            .query_row(
                "
                SELECT
                    prompt.prompt_key,
                    prompt.active_version_id,
                    version.version,
                    version.prompt_content
                FROM prompts AS prompt
                JOIN prompt_versions AS version
                  ON version.id = prompt.active_version_id
                 AND version.prompt_id = prompt.id
                WHERE prompt.prompt_key = 'source_summary'
                ",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .expect("read default prompt");

        assert_eq!(prompt_key, "source_summary");
        assert_eq!(active_version_id, "builtin-source-summary-v1");
        assert_eq!(version, 1);
        assert!(!prompt_content.trim().is_empty());
    }

    #[test]
    fn applies_the_prompt_registry_to_an_existing_version_three_database() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");
        run_migrations(&mut connection, &MIGRATIONS[..3]).expect("run through migration 3");
        assert_eq!(table_exists(&connection, "prompts"), None);
        assert_eq!(table_exists(&connection, "prompt_versions"), None);

        run(&mut connection).expect("apply prompt registry migration");

        assert_eq!(table_exists(&connection, "prompts"), Some("prompts".into()));
        assert_eq!(
            table_exists(&connection, "prompt_versions"),
            Some("prompt_versions".into())
        );
        assert_eq!(migration_count(&connection), 7);
    }

    #[test]
    fn upgrades_existing_provider_settings_with_the_default_model() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");
        run_migrations(&mut connection, &MIGRATIONS[..2]).expect("run through migration 2");
        connection
            .execute(
                "
                INSERT INTO ai_provider_settings (
                    id,
                    provider_type,
                    created_at,
                    updated_at
                )
                VALUES (
                    'default',
                    'deepseek',
                    '2026-06-14T00:00:00.000Z',
                    '2026-06-14T00:00:00.000Z'
                )
                ",
                [],
            )
            .expect("seed existing provider settings");

        run(&mut connection).expect("apply default model migration");

        let default_model = connection
            .query_row(
                "SELECT default_model FROM ai_provider_settings WHERE id = 'default'",
                [],
                |row| row.get::<_, String>(0),
            )
            .expect("read migrated default model");
        assert_eq!(default_model, "deepseek-v4-flash");
        assert_eq!(migration_count(&connection), 7);
    }

    #[test]
    fn rejects_an_applied_checksum_mismatch() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");
        run(&mut connection).expect("run initial migration");
        connection
            .execute(
                "UPDATE _schema_migrations SET checksum = 'changed' WHERE version = 1",
                [],
            )
            .expect("change applied checksum");

        let error = run(&mut connection).expect_err("reject checksum mismatch");

        assert_migration_error_contains(error, "does not match its applied record");
    }

    #[test]
    fn rolls_back_a_failed_migration() {
        let sql = "
            CREATE TABLE should_be_rolled_back (id INTEGER PRIMARY KEY);
            THIS IS NOT VALID SQL;
        ";
        let migrations = [test_migration(1, "failing_migration", sql)];
        let mut connection = Connection::open_in_memory().expect("open in-memory database");

        run_migrations(&mut connection, &migrations).expect_err("migration should fail");

        assert_eq!(table_exists(&connection, "should_be_rolled_back"), None);
        assert_eq!(table_exists(&connection, "_schema_migrations"), None);
    }

    #[test]
    fn rejects_an_unknown_higher_database_version() {
        let mut connection = Connection::open_in_memory().expect("open in-memory database");
        run(&mut connection).expect("run initial migration");
        connection
            .execute(
                "
                INSERT INTO _schema_migrations (version, name, checksum, applied_at)
                VALUES (8, 'future_migration', 'future-checksum', '2026-06-14T00:00:00Z')
                ",
                [],
            )
            .expect("insert future migration");

        let error = run(&mut connection).expect_err("reject future database");

        assert_migration_error_contains(error, "higher than the current application version");
    }

    #[test]
    fn rejects_duplicate_or_out_of_order_migration_versions() {
        let duplicate = [
            test_migration(1, "first", "SELECT 1;"),
            test_migration(1, "duplicate", "SELECT 2;"),
        ];
        let out_of_order = [
            test_migration(2, "second", "SELECT 2;"),
            test_migration(1, "first", "SELECT 1;"),
        ];

        for migrations in [&duplicate[..], &out_of_order[..]] {
            let mut connection = Connection::open_in_memory().expect("open in-memory database");
            let error =
                run_migrations(&mut connection, migrations).expect_err("reject invalid order");

            assert_migration_error_contains(error, "strictly increasing");
            assert_eq!(table_exists(&connection, "_schema_migrations"), None);
        }
    }

    #[test]
    fn validates_required_migration_metadata() {
        let empty_name = [test_migration(1, " ", "SELECT 1;")];
        let empty_checksum = [Migration {
            version: 1,
            name: "missing_checksum",
            sql: "SELECT 1;",
            checksum: "",
        }];

        for (migrations, expected_message) in [
            (&empty_name[..], "empty name"),
            (&empty_checksum[..], "empty checksum"),
        ] {
            let mut connection = Connection::open_in_memory().expect("open in-memory database");
            let error =
                run_migrations(&mut connection, migrations).expect_err("reject empty metadata");

            assert_migration_error_contains(error, expected_message);
            assert_eq!(table_exists(&connection, "_schema_migrations"), None);
        }
    }

    #[test]
    fn concurrent_first_runs_are_serialized() {
        let database_path = temporary_database_path();
        let barrier = Arc::new(Barrier::new(2));
        let handles = (0..2)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let database_path = database_path.clone();

                thread::spawn(move || {
                    let mut connection =
                        Connection::open(database_path).expect("open temporary database");
                    connection
                        .busy_timeout(Duration::from_secs(5))
                        .expect("set busy timeout");
                    barrier.wait();
                    run(&mut connection)
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle
                .join()
                .expect("migration thread should not panic")
                .expect("concurrent migration should succeed");
        }

        let connection = Connection::open(&database_path).expect("reopen temporary database");
        assert_eq!(migration_count(&connection), 7);
        assert_eq!(
            table_exists(&connection, "workspaces"),
            Some("workspaces".into())
        );
        assert_eq!(table_exists(&connection, "sources"), Some("sources".into()));
        assert_eq!(
            table_exists(&connection, "ai_provider_settings"),
            Some("ai_provider_settings".into())
        );
        assert_eq!(table_exists(&connection, "prompts"), Some("prompts".into()));
        assert_eq!(
            table_exists(&connection, "prompt_versions"),
            Some("prompt_versions".into())
        );
        assert_eq!(table_exists(&connection, "ai_runs"), Some("ai_runs".into()));
        assert_eq!(
            table_exists(&connection, "knowledge_nodes"),
            Some("knowledge_nodes".into())
        );
        assert_eq!(
            column_exists(&connection, "knowledge_nodes", "ai_run_id"),
            Some("ai_run_id".into())
        );

        drop(connection);
        let _ = fs::remove_file(&database_path);
        let _ = fs::remove_file(database_path.with_extension("sqlite3-shm"));
        let _ = fs::remove_file(database_path.with_extension("sqlite3-wal"));
    }

    fn test_migration(version: i64, name: &'static str, sql: &'static str) -> Migration {
        Migration {
            version,
            name,
            sql,
            checksum: Box::leak(checksum(sql).into_boxed_str()),
        }
    }

    fn table_exists(connection: &Connection, table: &str) -> Option<String> {
        connection
            .query_row(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .optional()
            .expect("query sqlite schema")
    }

    fn column_exists(connection: &Connection, table: &str, column: &str) -> Option<String> {
        let mut statement = connection
            .prepare(&format!("PRAGMA table_info({table})"))
            .expect("prepare table info query");

        statement
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table info")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect table columns")
            .into_iter()
            .find(|name| name == column)
    }

    fn migration_count(connection: &Connection) -> i64 {
        connection
            .query_row("SELECT COUNT(*) FROM _schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("count migrations")
    }

    fn assert_migration_error_contains(error: AppError, expected: &str) {
        match error {
            AppError::Migration(message) => assert!(
                message.contains(expected),
                "expected migration error to contain {expected:?}, got {message:?}"
            ),
            other => panic!("expected migration error, got {other}"),
        }
    }

    fn temporary_database_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "second-brain-os-migration-test-{}-{nonce}.sqlite3",
            std::process::id()
        ))
    }
}
