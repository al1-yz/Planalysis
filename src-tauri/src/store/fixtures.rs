//! Builders for valid records. Tests override only the fields they care about.

use rusqlite::Connection;

use crate::db::open_in_memory;
use crate::dto::{Account, Card, Category, Transaction};

pub const NOW: &str = "2026-10-13T12:00:00.000Z";

pub fn id(n: u32) -> String {
    format!("00000000-0000-4000-8000-{n:012}")
}

pub fn db() -> Connection {
    open_in_memory().unwrap()
}

pub fn key(name: &str) -> String {
    name.to_lowercase()
}

pub fn account(n: u32, name: &str) -> Account {
    Account {
        id: id(n),
        name: name.to_string(),
        name_key: key(name),
        account_type: "checking".to_string(),
        opening_balance: 1000,
        opening_date: "2026-01-01".to_string(),
        archived_at: None,
        created_at: NOW.to_string(),
        updated_at: NOW.to_string(),
    }
}

pub fn category(n: u32, name: &str, kind: &str) -> Category {
    Category {
        id: id(n),
        name: name.to_string(),
        name_key: key(name),
        kind: kind.to_string(),
        archived_at: None,
        created_at: NOW.to_string(),
        updated_at: NOW.to_string(),
    }
}

pub fn card(n: u32, name: &str) -> Card {
    Card {
        id: id(n),
        name: name.to_string(),
        name_key: key(name),
        closing_day: 10,
        due_day: 17,
        credit_limit: Some(500_000),
        archived_at: None,
        created_at: NOW.to_string(),
        updated_at: NOW.to_string(),
    }
}

/// A transaction on an account.
pub fn tx(n: u32, account_id: &str, date: &str, amount: i64) -> Transaction {
    Transaction {
        id: id(n),
        account_id: Some(account_id.to_string()),
        card_id: None,
        category_id: None,
        transfer_id: None,
        date: date.to_string(),
        amount,
        description: "descrição".to_string(),
        created_at: NOW.to_string(),
        updated_at: NOW.to_string(),
    }
}

/// A purchase on a card.
pub fn card_tx(n: u32, card_id: &str, date: &str, amount: i64) -> Transaction {
    Transaction {
        account_id: None,
        card_id: Some(card_id.to_string()),
        ..tx(n, "unused", date, amount)
    }
}

/// One leg of a transfer.
pub fn leg(n: u32, transfer_id: &str, account_id: &str, date: &str, amount: i64) -> Transaction {
    Transaction {
        transfer_id: Some(transfer_id.to_string()),
        ..tx(n, account_id, date, amount)
    }
}

/// The two legs of a transfer of `amount` cents between two accounts.
pub fn transfer(
    n: u32,
    transfer_id: &str,
    from: &str,
    to: &str,
    date: &str,
    amount: i64,
) -> Vec<Transaction> {
    vec![
        leg(n, transfer_id, from, date, -amount),
        leg(n + 1, transfer_id, to, date, amount),
    ]
}

pub fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}
