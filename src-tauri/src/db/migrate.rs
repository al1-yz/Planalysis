use std::fmt;

use rusqlite::{Connection, TransactionBehavior};

/// Identifies a file as Planalysis data (`PLAN` in ASCII), stored in the SQLite header.
pub const APPLICATION_ID: i32 = 0x504C_414E;

/// One forward-only schema change. Versions are consecutive, starting at 1.
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial_schema",
    sql: include_str!("../../migrations/0001_initial_schema.sql"),
}];

#[derive(Debug)]
pub enum OpenError {
    Io(String),
    Sqlite(rusqlite::Error),
    /// The file holds data, but not Planalysis's.
    ForeignDatabase,
    /// The file was written by a newer version of Planalysis.
    TooNew {
        found: i64,
        supported: u32,
    },
    Migration {
        version: u32,
        source: rusqlite::Error,
    },
    ForeignKeyViolation {
        version: u32,
    },
    ForeignKeysNotEnforced,
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(f, "could not prepare the database location: {message}"),
            Self::Sqlite(error) => write!(f, "database error: {error}"),
            Self::ForeignDatabase => write!(f, "the file is not a Planalysis database"),
            Self::TooNew { found, supported } => write!(
                f,
                "the database uses schema version {found}, but this build supports up to {supported}"
            ),
            Self::Migration { version, source } => {
                write!(f, "migration {version} failed: {source}")
            }
            Self::ForeignKeyViolation { version } => {
                write!(f, "migration {version} left foreign key violations")
            }
            Self::ForeignKeysNotEnforced => write!(f, "SQLite foreign key enforcement is off"),
        }
    }
}

impl std::error::Error for OpenError {}

impl From<rusqlite::Error> for OpenError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

fn header_value(conn: &Connection, pragma: &str) -> Result<i64, OpenError> {
    Ok(conn.pragma_query_value(None, pragma, |row| row.get(0))?)
}

/// Reads where the database stands without writing anything, and refuses databases this build
/// must not touch. Returns the current schema version (0 for a brand-new file).
pub fn current_version(conn: &Connection, migrations: &[Migration]) -> Result<u32, OpenError> {
    let application_id = header_value(conn, "application_id")?;
    let user_version = header_value(conn, "user_version")?;
    let objects: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'",
        [],
        |row| row.get(0),
    )?;
    if application_id == 0 && user_version == 0 && objects == 0 {
        return Ok(0);
    }
    if application_id != i64::from(APPLICATION_ID) {
        return Err(OpenError::ForeignDatabase);
    }
    let supported = migrations.last().map_or(0, |migration| migration.version);
    match u32::try_from(user_version) {
        Ok(version) if version <= supported => Ok(version),
        _ => Err(OpenError::TooNew {
            found: user_version,
            supported,
        }),
    }
}

/// Applies every migration newer than the database's version, each in its own transaction.
/// Returns the resulting version.
pub fn apply_pending(conn: &mut Connection, migrations: &[Migration]) -> Result<u32, OpenError> {
    let starting_version = current_version(conn, migrations)?;
    let mut version = starting_version;
    for migration in migrations.iter().filter(|m| m.version > starting_version) {
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute_batch(migration.sql)
            .map_err(|source| OpenError::Migration {
                version: migration.version,
                source,
            })?;
        tx.pragma_update(None, "application_id", APPLICATION_ID)?;
        tx.pragma_update(None, "user_version", migration.version)?;
        let violations: i64 =
            tx.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })?;
        if violations > 0 {
            return Err(OpenError::ForeignKeyViolation {
                version: migration.version,
            });
        }
        tx.commit()?;
        version = migration.version;
    }
    Ok(version)
}

/// Validates the database, then applies pending migrations. Returns the resulting version.
pub fn migrate(conn: &mut Connection, migrations: &[Migration]) -> Result<u32, OpenError> {
    current_version(conn, migrations)?;
    apply_pending(conn, migrations)
}
