use rusqlite::{Connection, OptionalExtension, Row, params};

use super::{require_changed, require_normalized_key};
use crate::dto::Account;
use crate::error::{AppError, ErrorCode};

const COLUMNS: &str = "id, name, name_key, type, opening_balance_cents, opening_date, \
                       archived_at, created_at, updated_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Account> {
    Ok(Account {
        id: row.get(0)?,
        name: row.get(1)?,
        name_key: row.get(2)?,
        account_type: row.get(3)?,
        opening_balance: row.get(4)?,
        opening_date: row.get(5)?,
        archived_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub fn create(conn: &mut Connection, account: &Account) -> Result<(), AppError> {
    require_normalized_key(&account.name_key)?;
    conn.execute(
        &format!("INSERT INTO accounts ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"),
        params![
            account.id,
            account.name,
            account.name_key,
            account.account_type,
            account.opening_balance,
            account.opening_date,
            account.archived_at,
            account.created_at,
            account.updated_at
        ],
    )?;
    Ok(())
}

/// Replaces the account's mutable fields. `created_at` never changes.
pub fn update(conn: &mut Connection, account: &Account) -> Result<(), AppError> {
    require_normalized_key(&account.name_key)?;
    let changed = conn.execute(
        "UPDATE accounts SET name = ?2, name_key = ?3, type = ?4, opening_balance_cents = ?5, \
         opening_date = ?6, archived_at = ?7, updated_at = ?8 WHERE id = ?1",
        params![
            account.id,
            account.name,
            account.name_key,
            account.account_type,
            account.opening_balance,
            account.opening_date,
            account.archived_at,
            account.updated_at
        ],
    )?;
    require_changed(changed, "account", &account.id)
}

/// Only succeeds while nothing refers to the account.
pub fn delete(conn: &mut Connection, id: &str) -> Result<(), AppError> {
    let changed = conn
        .execute("DELETE FROM accounts WHERE id = ?1", [id])
        .map_err(|error| AppError::from_sqlite(error, ErrorCode::InUse))?;
    require_changed(changed, "account", id)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Account>, AppError> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM accounts WHERE id = ?1"),
            [id],
            from_row,
        )
        .optional()?)
}

pub fn list(conn: &Connection) -> Result<Vec<Account>, AppError> {
    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM accounts ORDER BY name_key, id"
    ))?;
    let rows = statement.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}
