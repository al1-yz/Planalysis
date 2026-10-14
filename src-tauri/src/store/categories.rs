use rusqlite::{Connection, OptionalExtension, Row, params};

use super::{require_changed, require_normalized_key};
use crate::dto::Category;
use crate::error::{AppError, ErrorCode};

const COLUMNS: &str = "id, name, name_key, kind, archived_at, created_at, updated_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Category> {
    Ok(Category {
        id: row.get(0)?,
        name: row.get(1)?,
        name_key: row.get(2)?,
        kind: row.get(3)?,
        archived_at: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

pub fn create(conn: &mut Connection, category: &Category) -> Result<(), AppError> {
    require_normalized_key(&category.name_key)?;
    conn.execute(
        &format!("INSERT INTO categories ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"),
        params![
            category.id,
            category.name,
            category.name_key,
            category.kind,
            category.archived_at,
            category.created_at,
            category.updated_at
        ],
    )?;
    Ok(())
}

/// Replaces the category's mutable fields. `created_at` never changes.
pub fn update(conn: &mut Connection, category: &Category) -> Result<(), AppError> {
    require_normalized_key(&category.name_key)?;
    let changed = conn.execute(
        "UPDATE categories SET name = ?2, name_key = ?3, kind = ?4, archived_at = ?5, \
         updated_at = ?6 WHERE id = ?1",
        params![
            category.id,
            category.name,
            category.name_key,
            category.kind,
            category.archived_at,
            category.updated_at
        ],
    )?;
    require_changed(changed, "category", &category.id)
}

/// Only succeeds while nothing refers to the category.
pub fn delete(conn: &mut Connection, id: &str) -> Result<(), AppError> {
    let changed = conn
        .execute("DELETE FROM categories WHERE id = ?1", [id])
        .map_err(|error| AppError::from_sqlite(error, ErrorCode::InUse))?;
    require_changed(changed, "category", id)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Category>, AppError> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM categories WHERE id = ?1"),
            [id],
            from_row,
        )
        .optional()?)
}

pub fn list(conn: &Connection) -> Result<Vec<Category>, AppError> {
    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM categories ORDER BY kind, name_key, id"
    ))?;
    let rows = statement.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}
