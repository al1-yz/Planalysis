use std::fmt;

use rusqlite::ffi;
use serde::Serialize;

/// Machine-readable reason an operation was refused. The interface translates it into text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
    DuplicateName,
    InUse,
    BeforeOpeningDate,
    OpeningDateAfterTransactions,
    ArchivedTarget,
    InvalidTransfer,
    TransferLeg,
    InvalidData,
    DatabaseUnavailable,
    Internal,
}

/// The error every persistence operation reports, serialized as `{ "code", "message" }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppError {
    pub code: ErrorCode,
    /// Technical detail for diagnostics; not meant to be shown to the user.
    pub message: String,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Translates a SQLite failure. `on_foreign_key` says what a foreign key violation means for
    /// the calling operation: a missing reference when writing, a record in use when deleting.
    pub fn from_sqlite(error: rusqlite::Error, on_foreign_key: ErrorCode) -> Self {
        let code = match &error {
            rusqlite::Error::QueryReturnedNoRows => ErrorCode::NotFound,
            rusqlite::Error::SqliteFailure(failure, message)
                if failure.code == ffi::ErrorCode::ConstraintViolation =>
            {
                constraint_code(
                    failure.extended_code,
                    message.as_deref().unwrap_or(""),
                    on_foreign_key,
                )
            }
            _ => ErrorCode::Internal,
        };
        Self::new(code, error.to_string())
    }
}

fn constraint_code(extended_code: i32, message: &str, on_foreign_key: ErrorCode) -> ErrorCode {
    if message.contains("before_opening_date") {
        ErrorCode::BeforeOpeningDate
    } else if message.contains("opening_date_after_transactions") {
        ErrorCode::OpeningDateAfterTransactions
    } else if message.contains("FOREIGN KEY") {
        // A RESTRICT violation on delete reports extended code 1811, not SQLITE_CONSTRAINT_FOREIGNKEY,
        // so the message is the reliable signal.
        on_foreign_key
    } else if extended_code == ffi::SQLITE_CONSTRAINT_UNIQUE && message.contains("name_key") {
        ErrorCode::DuplicateName
    } else if extended_code == ffi::SQLITE_CONSTRAINT_UNIQUE && message.contains("transfer_id") {
        ErrorCode::InvalidTransfer
    } else {
        ErrorCode::InvalidData
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self::from_sqlite(error, ErrorCode::NotFound)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}
