//! A transfer is two linked transactions on two owned accounts: one outgoing (negative), one
//! incoming (positive), equal and opposite, on the same date. Every operation here writes both
//! legs or neither, inside one transaction.

use rusqlite::Connection;

use super::transactions::{COLUMNS, from_row, insert};
use super::{begin, require_active};
use crate::dto::Transaction;
use crate::error::{AppError, ErrorCode};

fn invalid(message: &str) -> AppError {
    AppError::new(ErrorCode::InvalidTransfer, message)
}

/// Checks that exactly two legs form a valid transfer and returns them as (outgoing, incoming).
fn check_pair(legs: &[Transaction]) -> Result<(&Transaction, &Transaction), AppError> {
    let [first, second] = legs else {
        return Err(invalid("a transfer has exactly two legs"));
    };
    let (outgoing, incoming) = if first.amount < second.amount {
        (first, second)
    } else {
        (second, first)
    };
    if outgoing.amount >= 0 || incoming.amount <= 0 {
        return Err(invalid("a transfer has one outgoing and one incoming leg"));
    }
    if outgoing.amount.checked_add(incoming.amount) != Some(0) {
        return Err(invalid("the legs of a transfer must be equal and opposite"));
    }
    match (&outgoing.transfer_id, &incoming.transfer_id) {
        (Some(a), Some(b)) if a == b => {}
        _ => return Err(invalid("both legs must carry the same transfer id")),
    }
    match (&outgoing.account_id, &incoming.account_id) {
        (Some(a), Some(b)) if a != b => {}
        _ => return Err(invalid("a transfer needs two different accounts")),
    }
    if outgoing.card_id.is_some()
        || incoming.card_id.is_some()
        || outgoing.category_id.is_some()
        || incoming.category_id.is_some()
    {
        return Err(invalid("transfer legs have no card or category"));
    }
    if outgoing.date != incoming.date {
        return Err(invalid("both legs must have the same date"));
    }
    Ok((outgoing, incoming))
}

fn legs_of(conn: &Connection, transfer_id: &str) -> Result<Vec<Transaction>, AppError> {
    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM transactions WHERE transfer_id = ?1 ORDER BY amount_cents, id"
    ))?;
    let rows = statement.query_map([transfer_id], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// After writing, confirms what is stored for this transfer really is a valid pair.
fn verify_stored(conn: &Connection, transfer_id: &str) -> Result<(), AppError> {
    let (legs, total, dates, accounts): (i64, i64, i64, i64) = conn.query_row(
        "SELECT count(*), coalesce(sum(amount_cents), 0), count(DISTINCT occurred_on), \
         count(DISTINCT account_id) FROM transactions WHERE transfer_id = ?1",
        [transfer_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;
    if (legs, total, dates, accounts) == (2, 0, 1, 2) {
        Ok(())
    } else {
        Err(invalid("the stored legs do not form a valid transfer"))
    }
}

pub fn create(conn: &mut Connection, legs: &[Transaction]) -> Result<(), AppError> {
    let (outgoing, incoming) = check_pair(legs)?;
    let tx = begin(conn)?;
    for leg in [outgoing, incoming] {
        if let Some(account_id) = &leg.account_id {
            require_active(&tx, "accounts", account_id)?;
        }
    }
    insert(&tx, outgoing)?;
    insert(&tx, incoming)?;
    if let Some(transfer_id) = &outgoing.transfer_id {
        verify_stored(&tx, transfer_id)?;
    }
    tx.commit()?;
    Ok(())
}

/// Replaces both legs of the transfer they share. An account the transfer already used may be
/// archived by now (history stays correctable); a different account must be active.
pub fn update(conn: &mut Connection, legs: &[Transaction]) -> Result<(), AppError> {
    let (outgoing, incoming) = check_pair(legs)?;
    let Some(transfer_id) = outgoing.transfer_id.as_deref() else {
        return Err(invalid("both legs must carry the same transfer id"));
    };
    let tx = begin(conn)?;
    let existing = legs_of(&tx, transfer_id)?;
    if existing.is_empty() {
        return Err(AppError::new(
            ErrorCode::NotFound,
            format!("no transfer {transfer_id}"),
        ));
    }
    let previous_accounts: Vec<&str> = existing
        .iter()
        .filter_map(|leg| leg.account_id.as_deref())
        .collect();
    for leg in [outgoing, incoming] {
        if let Some(account_id) = leg.account_id.as_deref()
            && !previous_accounts.contains(&account_id)
        {
            require_active(&tx, "accounts", account_id)?;
        }
    }
    tx.execute(
        "DELETE FROM transactions WHERE transfer_id = ?1",
        [transfer_id],
    )?;
    insert(&tx, outgoing)?;
    insert(&tx, incoming)?;
    verify_stored(&tx, transfer_id)?;
    tx.commit()?;
    Ok(())
}

pub fn delete(conn: &mut Connection, transfer_id: &str) -> Result<(), AppError> {
    let tx = begin(conn)?;
    let removed = tx.execute(
        "DELETE FROM transactions WHERE transfer_id = ?1",
        [transfer_id],
    )?;
    if removed == 0 {
        return Err(AppError::new(
            ErrorCode::NotFound,
            format!("no transfer {transfer_id}"),
        ));
    }
    tx.commit()?;
    Ok(())
}

/// The legs of a transfer, outgoing (negative) first.
pub fn get(conn: &Connection, transfer_id: &str) -> Result<Option<Vec<Transaction>>, AppError> {
    let legs = legs_of(conn, transfer_id)?;
    Ok(if legs.is_empty() { None } else { Some(legs) })
}
