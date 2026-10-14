use rusqlite::types::ToSql;
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

use super::{begin, require_active, require_changed};
use crate::dto::{Transaction, TransactionFilter};
use crate::error::{AppError, ErrorCode};

pub(super) const COLUMNS: &str = "id, account_id, card_id, category_id, transfer_id, occurred_on, \
                                  amount_cents, description, created_at, updated_at";

pub(super) fn from_row(row: &Row<'_>) -> rusqlite::Result<Transaction> {
    Ok(Transaction {
        id: row.get(0)?,
        account_id: row.get(1)?,
        card_id: row.get(2)?,
        category_id: row.get(3)?,
        transfer_id: row.get(4)?,
        date: row.get(5)?,
        amount: row.get(6)?,
        description: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

pub(super) fn insert(conn: &Connection, t: &Transaction) -> Result<(), AppError> {
    conn.execute(
        &format!(
            "INSERT INTO transactions ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        ),
        params![
            t.id,
            t.account_id,
            t.card_id,
            t.category_id,
            t.transfer_id,
            t.date,
            t.amount,
            t.description,
            t.created_at,
            t.updated_at
        ],
    )?;
    Ok(())
}

pub(super) fn find(conn: &Connection, id: &str) -> Result<Option<Transaction>, AppError> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM transactions WHERE id = ?1"),
            [id],
            from_row,
        )
        .optional()?)
}

fn leg_error(id: &str) -> AppError {
    AppError::new(
        ErrorCode::TransferLeg,
        format!("transaction {id} is a transfer leg; use the transfer operations"),
    )
}

/// Every account, card and category a new reference points at must be active.
fn require_new_references_active(
    conn: &Connection,
    new: &Transaction,
    old: Option<&Transaction>,
) -> Result<(), AppError> {
    let references = [
        ("accounts", &new.account_id, old.map(|o| &o.account_id)),
        ("cards", &new.card_id, old.map(|o| &o.card_id)),
        ("categories", &new.category_id, old.map(|o| &o.category_id)),
    ];
    for (table, wanted, previous) in references {
        if let Some(id) = wanted
            && previous != Some(wanted)
        {
            require_active(conn, table, id)?;
        }
    }
    Ok(())
}

pub fn create(conn: &mut Connection, transaction: &Transaction) -> Result<(), AppError> {
    if transaction.transfer_id.is_some() {
        return Err(leg_error(&transaction.id));
    }
    let tx = begin(conn)?;
    require_new_references_active(&tx, transaction, None)?;
    insert(&tx, transaction)?;
    tx.commit()?;
    Ok(())
}

/// Replaces the transaction's mutable fields. A reference that stays the same may point at an
/// archived record, so history can still be corrected; a new reference must be active.
pub fn update(conn: &mut Connection, transaction: &Transaction) -> Result<(), AppError> {
    let tx = begin(conn)?;
    let old = find(&tx, &transaction.id)?.ok_or_else(|| {
        AppError::new(
            ErrorCode::NotFound,
            format!("no transaction {}", transaction.id),
        )
    })?;
    if old.transfer_id.is_some() || transaction.transfer_id.is_some() {
        return Err(leg_error(&transaction.id));
    }
    require_new_references_active(&tx, transaction, Some(&old))?;
    tx.execute(
        "UPDATE transactions SET account_id = ?2, card_id = ?3, category_id = ?4, \
         occurred_on = ?5, amount_cents = ?6, description = ?7, updated_at = ?8 WHERE id = ?1",
        params![
            transaction.id,
            transaction.account_id,
            transaction.card_id,
            transaction.category_id,
            transaction.date,
            transaction.amount,
            transaction.description,
            transaction.updated_at
        ],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn delete(conn: &mut Connection, id: &str) -> Result<(), AppError> {
    let tx = begin(conn)?;
    let existing = find(&tx, id)?;
    match existing {
        None => return require_changed(0, "transaction", id),
        Some(found) if found.transfer_id.is_some() => return Err(leg_error(id)),
        Some(_) => {}
    }
    let changed = tx.execute("DELETE FROM transactions WHERE id = ?1", [id])?;
    require_changed(changed, "transaction", id)?;
    tx.commit()?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Transaction>, AppError> {
    find(conn, id)
}

/// Newest first. Only the filters that are set apply.
pub fn list(conn: &Connection, filter: &TransactionFilter) -> Result<Vec<Transaction>, AppError> {
    let mut clauses: Vec<&str> = Vec::new();
    let mut values: Vec<&dyn ToSql> = Vec::new();
    let filters = [
        ("account_id = ?", &filter.account_id),
        ("card_id = ?", &filter.card_id),
        ("category_id = ?", &filter.category_id),
        ("occurred_on >= ?", &filter.from),
        ("occurred_on <= ?", &filter.to),
    ];
    for (clause, value) in filters {
        if let Some(value) = value {
            clauses.push(clause);
            values.push(value);
        }
    }
    let condition = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };
    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM transactions{condition} \
         ORDER BY occurred_on DESC, created_at DESC, id DESC"
    ))?;
    let rows = statement.query_map(params_from_iter(values), from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}
