use rusqlite::{Connection, OptionalExtension, Row, params};

use super::{require_changed, require_normalized_key};
use crate::dto::Card;
use crate::error::{AppError, ErrorCode};

const COLUMNS: &str = "id, name, name_key, closing_day, due_day, credit_limit_cents, \
                       archived_at, created_at, updated_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Card> {
    Ok(Card {
        id: row.get(0)?,
        name: row.get(1)?,
        name_key: row.get(2)?,
        closing_day: row.get(3)?,
        due_day: row.get(4)?,
        credit_limit: row.get(5)?,
        archived_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub fn create(conn: &mut Connection, card: &Card) -> Result<(), AppError> {
    require_normalized_key(&card.name_key)?;
    conn.execute(
        &format!("INSERT INTO cards ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"),
        params![
            card.id,
            card.name,
            card.name_key,
            card.closing_day,
            card.due_day,
            card.credit_limit,
            card.archived_at,
            card.created_at,
            card.updated_at
        ],
    )?;
    Ok(())
}

/// Replaces the card's mutable fields. `created_at` never changes.
pub fn update(conn: &mut Connection, card: &Card) -> Result<(), AppError> {
    require_normalized_key(&card.name_key)?;
    let changed = conn.execute(
        "UPDATE cards SET name = ?2, name_key = ?3, closing_day = ?4, due_day = ?5, \
         credit_limit_cents = ?6, archived_at = ?7, updated_at = ?8 WHERE id = ?1",
        params![
            card.id,
            card.name,
            card.name_key,
            card.closing_day,
            card.due_day,
            card.credit_limit,
            card.archived_at,
            card.updated_at
        ],
    )?;
    require_changed(changed, "card", &card.id)
}

/// Only succeeds while nothing refers to the card.
pub fn delete(conn: &mut Connection, id: &str) -> Result<(), AppError> {
    let changed = conn
        .execute("DELETE FROM cards WHERE id = ?1", [id])
        .map_err(|error| AppError::from_sqlite(error, ErrorCode::InUse))?;
    require_changed(changed, "card", id)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Card>, AppError> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM cards WHERE id = ?1"),
            [id],
            from_row,
        )
        .optional()?)
}

pub fn list(conn: &Connection) -> Result<Vec<Card>, AppError> {
    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM cards ORDER BY name_key, id"
    ))?;
    let rows = statement.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}
