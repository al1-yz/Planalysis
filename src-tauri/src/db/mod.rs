//! SQLite connection setup and schema migrations. No Tauri types appear here.

pub mod migrate;

#[cfg(test)]
mod open_tests;
#[cfg(test)]
mod schema_tests;
#[cfg(test)]
mod test_support;

use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;

pub use migrate::OpenError;
use migrate::{MIGRATIONS, apply_pending, current_version};

/// Connection-level settings. These never write to the database file.
fn configure_connection(conn: &Connection) -> Result<(), OpenError> {
    conn.pragma_update(None, "foreign_keys", true)?;
    let enforced: i64 = conn.pragma_query_value(None, "foreign_keys", |row| row.get(0))?;
    if enforced != 1 {
        return Err(OpenError::ForeignKeysNotEnforced);
    }
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(())
}

/// Durable writes for the database file: write-ahead logging, fsync on every commit.
fn configure_durability(conn: &Connection) -> Result<(), OpenError> {
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
    conn.pragma_update(None, "synchronous", "FULL")?;
    Ok(())
}

/// Opens (creating if needed) the database file, configures the connection and applies pending
/// migrations. Refuses, without writing, a database that is not Planalysis's or is newer than
/// this build understands.
pub fn open(path: &Path) -> Result<Connection, OpenError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| OpenError::Io(error.to_string()))?;
    }
    let mut conn = Connection::open(path)?;
    configure_connection(&conn)?;
    // Validate before switching to WAL, which rewrites the file header.
    current_version(&conn, MIGRATIONS)?;
    configure_durability(&conn)?;
    apply_pending(&mut conn, MIGRATIONS)?;
    Ok(conn)
}

/// An in-memory database with the same configuration and schema, for tests.
pub fn open_in_memory() -> Result<Connection, OpenError> {
    let mut conn = Connection::open_in_memory()?;
    configure_connection(&conn)?;
    apply_pending(&mut conn, MIGRATIONS)?;
    Ok(conn)
}
