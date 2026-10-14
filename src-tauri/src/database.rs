//! The one SQLite connection, shared by every command, and the status the interface shows when
//! it could not be opened.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use serde::Serialize;

use crate::db::{self, OpenError};
use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupErrorCode {
    DatabaseTooNew,
    NotAPlanalysisDatabase,
    MigrationFailed,
    DatabaseUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StartupError {
    pub code: StartupErrorCode,
    /// Technical detail; the interface shows its own pt-BR text for the code.
    pub message: String,
}

impl From<&OpenError> for StartupError {
    fn from(error: &OpenError) -> Self {
        let code = match error {
            OpenError::TooNew { .. } => StartupErrorCode::DatabaseTooNew,
            OpenError::ForeignDatabase => StartupErrorCode::NotAPlanalysisDatabase,
            OpenError::Migration { .. } | OpenError::ForeignKeyViolation { .. } => {
                StartupErrorCode::MigrationFailed
            }
            OpenError::Io(_) | OpenError::Sqlite(_) | OpenError::ForeignKeysNotEnforced => {
                StartupErrorCode::DatabaseUnavailable
            }
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatus {
    pub ready: bool,
    pub schema_version: Option<u32>,
    pub app_version: String,
    pub path: String,
    pub error: Option<StartupError>,
}

/// The database connection. Opening never panics and never resets anything: when the file cannot
/// be used, the reason is kept and every operation reports the database as unavailable.
pub struct Database {
    connection: Mutex<Option<Connection>>,
    status: DatabaseStatus,
}

impl Database {
    pub fn open(path: &Path) -> Self {
        let location = path.display().to_string();
        match db::open(path) {
            Ok(connection) => Self::ready(connection, location),
            Err(error) => Self::unavailable(&error, location),
        }
    }

    /// A private in-memory database, for tests.
    pub fn in_memory() -> Self {
        match db::open_in_memory() {
            Ok(connection) => Self::ready(connection, ":memory:".to_string()),
            Err(error) => Self::unavailable(&error, ":memory:".to_string()),
        }
    }

    fn ready(connection: Connection, path: String) -> Self {
        let schema_version = connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
            .ok();
        Self {
            connection: Mutex::new(Some(connection)),
            status: DatabaseStatus {
                ready: true,
                schema_version,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                path,
                error: None,
            },
        }
    }

    fn unavailable(error: &OpenError, path: String) -> Self {
        Self {
            connection: Mutex::new(None),
            status: DatabaseStatus {
                ready: false,
                schema_version: None,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                path,
                error: Some(StartupError::from(error)),
            },
        }
    }

    pub fn status(&self) -> DatabaseStatus {
        self.status.clone()
    }

    /// Runs `work` with exclusive use of the connection.
    pub fn with<T>(
        &self,
        work: impl FnOnce(&mut Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        // A panic while holding the lock leaves SQLite rolled back, so the data is still sound.
        let mut guard = self
            .connection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match guard.as_mut() {
            Some(connection) => work(connection),
            None => Err(AppError::new(
                ErrorCode::DatabaseUnavailable,
                "the database could not be opened",
            )),
        }
    }
}
