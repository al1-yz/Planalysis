//! Persistence operations: plain functions over a SQLite connection, one transaction per write.
//! No Tauri types appear here, so everything is testable against an in-memory database.

pub mod accounts;
pub mod cards;
pub mod categories;
pub mod transactions;
pub mod transfers;

#[cfg(test)]
mod accounts_tests;
#[cfg(test)]
mod cards_categories_tests;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod transactions_tests;
#[cfg(test)]
mod transfers_tests;

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior};

use crate::error::{AppError, ErrorCode};
use crate::names::is_normalized_key;

/// Starts a write transaction that takes the database write lock immediately, so a check made
/// inside it cannot be invalidated by another writer before it commits.
pub(crate) fn begin(conn: &mut Connection) -> Result<Transaction<'_>, AppError> {
    Ok(conn.transaction_with_behavior(TransactionBehavior::Immediate)?)
}

/// Fails unless the record exists and is not archived. Archived accounts, cards and categories
/// are hidden from new use but keep their history.
pub(crate) fn require_active(
    conn: &Connection,
    table: &'static str,
    id: &str,
) -> Result<(), AppError> {
    let archived_at = conn
        .query_row(
            &format!("SELECT archived_at FROM {table} WHERE id = ?1"),
            [id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?;
    match archived_at {
        None => Err(AppError::new(
            ErrorCode::NotFound,
            format!("no record {id} in {table}"),
        )),
        Some(Some(_)) => Err(AppError::new(
            ErrorCode::ArchivedTarget,
            format!("record {id} in {table} is archived"),
        )),
        Some(None) => Ok(()),
    }
}

/// Refuses a name key that was not normalized, so it cannot defeat the unique-name indexes.
pub(crate) fn require_normalized_key(name_key: &str) -> Result<(), AppError> {
    if is_normalized_key(name_key) {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::InvalidData,
            format!("name key {name_key:?} is not normalized"),
        ))
    }
}

/// An update or delete that touched no row means the record does not exist.
pub(crate) fn require_changed(changed: usize, what: &str, id: &str) -> Result<(), AppError> {
    if changed == 0 {
        Err(AppError::new(
            ErrorCode::NotFound,
            format!("no {what} {id}"),
        ))
    } else {
        Ok(())
    }
}
