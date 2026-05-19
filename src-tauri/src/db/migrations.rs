use rusqlite::Connection;

use super::DbError;

struct Migration {
    version: i64,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("migrations/system-prompts.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("migrations/chat-history.sql"),
    },
];

/// Runs all pending migrations, tracked via `PRAGMA user_version`.
///
/// On first launch with a database previously managed by `tauri-plugin-sql`
/// (which records its state in `_sqlx_migrations` instead of `user_version`),
/// we detect that table and stamp `user_version` to the highest schema
/// version we know about. The schema is already there; we just need the
/// counter to match so subsequent runs do nothing.
pub fn run_migrations(conn: &mut Connection) -> Result<(), DbError> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    if user_version(conn)? == 0 && has_sqlx_migrations(conn)? {
        let latest = MIGRATIONS.last().map(|m| m.version).unwrap_or(0);
        tracing::info!(
            target = "pluely::db",
            "detected tauri-plugin-sql legacy schema; stamping user_version={}",
            latest,
        );
        set_user_version(conn, latest)?;
        return Ok(());
    }

    let current = user_version(conn)?;
    for m in MIGRATIONS {
        if m.version > current {
            let tx = conn.transaction()?;
            tx.execute_batch(m.sql)?;
            tx.execute_batch(&format!("PRAGMA user_version = {};", m.version))?;
            tx.commit()?;
        }
    }
    Ok(())
}

fn user_version(conn: &Connection) -> Result<i64, DbError> {
    let v: i64 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;
    Ok(v)
}

fn set_user_version(conn: &Connection, v: i64) -> Result<(), DbError> {
    conn.execute_batch(&format!("PRAGMA user_version = {};", v))?;
    Ok(())
}

fn has_sqlx_migrations(conn: &Connection) -> Result<bool, DbError> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations';",
        [],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}
