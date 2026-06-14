use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use crate::error::AppError;

struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial_schema",
    sql: include_str!("../../../migrations/0001_initial_schema.sql"),
}];

pub fn run(connection: &mut Connection) -> Result<(), AppError> {
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS _schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );
        ",
    )?;

    for migration in MIGRATIONS {
        apply_migration(connection, migration)?;
    }

    Ok(())
}

fn apply_migration(connection: &mut Connection, migration: &Migration) -> Result<(), AppError> {
    let checksum = checksum(migration.sql);
    let applied = connection
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
        if applied_name != migration.name || applied_checksum != checksum {
            return Err(AppError::Migration(format!(
                "migration {} does not match its applied record",
                migration.version
            )));
        }

        return Ok(());
    }

    let transaction = connection.transaction()?;
    transaction.execute_batch(migration.sql)?;
    transaction.execute(
        "
        INSERT INTO _schema_migrations (version, name, checksum, applied_at)
        VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        ",
        params![migration.version, migration.name, checksum],
    )?;
    transaction.commit()?;

    Ok(())
}

fn checksum(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.as_bytes()))
}
