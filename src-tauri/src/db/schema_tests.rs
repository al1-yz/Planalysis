//! Behavior of the schema itself: constraints, indexes and triggers reject bad data no matter
//! which code path writes it.

use rusqlite::{Connection, params};

use super::open_in_memory;

const NOW: &str = "2026-10-13T12:00:00.000Z";
const ACCOUNT_A: &str = "3f2b8c1e-5d4a-4e6f-9a1b-0c7d8e9f1a2b";
const ACCOUNT_B: &str = "9a8b7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d";
const CARD: &str = "11111111-2222-4333-8444-555555555555";
const CATEGORY: &str = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";
const TRANSFER: &str = "0f0f0f0f-1e1e-4d2d-8c3c-4b4b4b4b4b4b";

fn id(n: u32) -> String {
    format!("00000000-0000-4000-8000-{n:012}")
}

fn add_account(
    conn: &Connection,
    id: &str,
    key: &str,
    opening_date: &str,
) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO accounts VALUES (?1, ?2, ?3, 'checking', 1000, ?4, NULL, ?5, ?5)",
        params![id, key, key, opening_date, NOW],
    )
}

fn add_card(conn: &Connection, id: &str, key: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO cards VALUES (?1, ?2, ?3, 10, 17, NULL, NULL, ?4, ?4)",
        params![id, key, key, NOW],
    )
}

fn add_category(conn: &Connection, id: &str, key: &str, kind: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO categories VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?5)",
        params![id, key, key, kind, NOW],
    )
}

/// A transaction row to insert; start from `on_account` and override fields.
#[derive(Clone)]
struct Tx<'a> {
    id: String,
    account: Option<&'a str>,
    card: Option<&'a str>,
    category: Option<&'a str>,
    transfer: Option<&'a str>,
    date: &'a str,
    amount: i64,
}

impl<'a> Tx<'a> {
    fn on_account(n: u32, account: &'a str) -> Self {
        Self {
            id: id(n),
            account: Some(account),
            card: None,
            category: None,
            transfer: None,
            date: "2026-02-01",
            amount: -500,
        }
    }

    fn insert(&self, conn: &Connection) -> rusqlite::Result<usize> {
        conn.execute(
            "INSERT INTO transactions VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, '', ?8, ?8)",
            params![
                self.id,
                self.account,
                self.card,
                self.category,
                self.transfer,
                self.date,
                self.amount,
                NOW
            ],
        )
    }
}

fn failure(result: rusqlite::Result<usize>) -> String {
    result
        .expect_err("the statement should have been rejected")
        .to_string()
}

fn seeded() -> Connection {
    let conn = open_in_memory().unwrap();
    add_account(&conn, ACCOUNT_A, "conta a", "2026-01-01").unwrap();
    add_account(&conn, ACCOUNT_B, "conta b", "2026-01-01").unwrap();
    add_card(&conn, CARD, "nubank").unwrap();
    add_category(&conn, CATEGORY, "mercado", "expense").unwrap();
    conn
}

#[test]
fn active_account_names_are_unique_but_archived_names_can_be_reused() {
    let conn = seeded();
    assert!(failure(add_account(&conn, &id(1), "conta a", "2026-01-01")).contains("UNIQUE"));

    conn.execute(
        "UPDATE accounts SET archived_at = ?1 WHERE id = ?2",
        params![NOW, ACCOUNT_A],
    )
    .unwrap();
    add_account(&conn, &id(1), "conta a", "2026-01-01").unwrap();
}

#[test]
fn category_names_are_unique_per_kind() {
    let conn = seeded();
    add_category(&conn, &id(2), "mercado", "income").unwrap();
    assert!(failure(add_category(&conn, &id(3), "mercado", "expense")).contains("UNIQUE"));
}

#[test]
fn active_card_names_are_unique() {
    let conn = seeded();
    assert!(failure(add_card(&conn, &id(4), "nubank")).contains("UNIQUE"));
}

#[test]
fn invalid_dates_money_and_enums_are_rejected() {
    let conn = seeded();
    for bad_date in ["2026-02-30", "2026-3-5", "0000-01-01", "26-03-05", ""] {
        let message = failure(add_account(&conn, &id(5), "x", bad_date));
        assert!(message.contains("CHECK"), "{bad_date} should fail a CHECK");
    }
    let unsafe_balance = conn.execute(
        "INSERT INTO accounts VALUES (?1, 'x', 'x', 'cash', 9007199254740992, '2026-01-01', NULL, ?2, ?2)",
        params![id(6), NOW],
    );
    assert!(failure(unsafe_balance).contains("CHECK"));
    let bad_type = conn.execute(
        "INSERT INTO accounts VALUES (?1, 'x', 'x', 'investment', 0, '2026-01-01', NULL, ?2, ?2)",
        params![id(7), NOW],
    );
    assert!(failure(bad_type).contains("CHECK"));
    let text_in_integer = conn.execute(
        "INSERT INTO accounts VALUES (?1, 'x', 'x', 'cash', 'abc', '2026-01-01', NULL, ?2, ?2)",
        params![id(8), NOW],
    );
    assert!(text_in_integer.is_err());
}

#[test]
fn card_days_must_be_between_1_and_31() {
    let conn = seeded();
    for (closing, due) in [(0, 10), (32, 10), (10, 0), (10, 32)] {
        let result = conn.execute(
            "INSERT INTO cards VALUES (?1, 'c', 'c', ?2, ?3, NULL, NULL, ?4, ?4)",
            params![id(9), closing, due, NOW],
        );
        assert!(failure(result).contains("CHECK"));
    }
}

#[test]
fn a_transaction_belongs_to_exactly_one_account_or_card() {
    let conn = seeded();
    Tx::on_account(10, ACCOUNT_A).insert(&conn).unwrap();
    Tx {
        account: None,
        card: Some(CARD),
        ..Tx::on_account(11, ACCOUNT_A)
    }
    .insert(&conn)
    .unwrap();

    let both = Tx {
        card: Some(CARD),
        ..Tx::on_account(12, ACCOUNT_A)
    };
    assert!(failure(both.insert(&conn)).contains("CHECK"));
    let neither = Tx {
        account: None,
        ..Tx::on_account(13, ACCOUNT_A)
    };
    assert!(failure(neither.insert(&conn)).contains("CHECK"));
}

#[test]
fn amounts_must_be_non_zero_safe_integers() {
    let conn = seeded();
    for amount in [0, 9_007_199_254_740_992, -9_007_199_254_740_992] {
        let tx = Tx {
            amount,
            ..Tx::on_account(14, ACCOUNT_A)
        };
        assert!(failure(tx.insert(&conn)).contains("CHECK"), "{amount}");
    }
    Tx {
        amount: 9_007_199_254_740_991,
        ..Tx::on_account(15, ACCOUNT_A)
    }
    .insert(&conn)
    .unwrap();
}

#[test]
fn references_must_exist_and_cannot_be_deleted_while_used() {
    let conn = seeded();
    let orphan = Tx::on_account(16, "99999999-9999-4999-8999-999999999999");
    assert!(failure(orphan.insert(&conn)).contains("FOREIGN KEY"));

    Tx {
        category: Some(CATEGORY),
        ..Tx::on_account(17, ACCOUNT_A)
    }
    .insert(&conn)
    .unwrap();
    for (table, key) in [("accounts", ACCOUNT_A), ("categories", CATEGORY)] {
        let result = conn.execute(&format!("DELETE FROM {table} WHERE id = ?1"), params![key]);
        assert!(failure(result).contains("FOREIGN KEY"), "{table}");
    }
    Tx {
        account: None,
        card: Some(CARD),
        ..Tx::on_account(18, ACCOUNT_A)
    }
    .insert(&conn)
    .unwrap();
    let card_delete = conn.execute("DELETE FROM cards WHERE id = ?1", params![CARD]);
    assert!(failure(card_delete).contains("FOREIGN KEY"));
}

#[test]
fn transactions_cannot_precede_the_account_opening_date() {
    let conn = seeded();
    let too_early = Tx {
        date: "2025-12-31",
        ..Tx::on_account(19, ACCOUNT_A)
    };
    assert!(failure(too_early.insert(&conn)).contains("before_opening_date"));
    Tx {
        date: "2026-01-01",
        ..Tx::on_account(20, ACCOUNT_A)
    }
    .insert(&conn)
    .unwrap();

    let moved = conn.execute(
        "UPDATE transactions SET occurred_on = '2025-01-01' WHERE id = ?1",
        params![id(20)],
    );
    assert!(failure(moved).contains("before_opening_date"));

    conn.execute(
        "UPDATE accounts SET opening_date = '2026-06-01' WHERE id = ?1",
        params![ACCOUNT_B],
    )
    .unwrap();
    let rehomed = conn.execute(
        "UPDATE transactions SET account_id = ?1 WHERE id = ?2",
        params![ACCOUNT_B, id(20)],
    );
    assert!(failure(rehomed).contains("before_opening_date"));

    conn.execute(
        "UPDATE transactions SET description = 'ok' WHERE id = ?1",
        params![id(20)],
    )
    .unwrap();
}

#[test]
fn the_opening_date_can_move_earlier_but_not_past_a_transaction() {
    let conn = seeded();
    Tx {
        date: "2026-03-10",
        ..Tx::on_account(21, ACCOUNT_A)
    }
    .insert(&conn)
    .unwrap();

    let past = conn.execute(
        "UPDATE accounts SET opening_date = '2026-03-11' WHERE id = ?1",
        params![ACCOUNT_A],
    );
    assert!(failure(past).contains("opening_date_after_transactions"));
    for date in ["2026-03-10", "2025-01-01"] {
        conn.execute(
            "UPDATE accounts SET opening_date = ?1 WHERE id = ?2",
            params![date, ACCOUNT_A],
        )
        .unwrap();
    }
}

#[test]
fn transfer_legs_live_on_accounts_without_categories_and_one_per_account() {
    let conn = seeded();
    let leg = |n: u32, account: &'static str| Tx {
        transfer: Some(TRANSFER),
        ..Tx::on_account(n, account)
    };

    let with_category = Tx {
        category: Some(CATEGORY),
        ..leg(22, ACCOUNT_A)
    };
    assert!(failure(with_category.insert(&conn)).contains("CHECK"));
    let on_card = Tx {
        account: None,
        card: Some(CARD),
        ..leg(23, ACCOUNT_A)
    };
    assert!(failure(on_card.insert(&conn)).contains("CHECK"));

    leg(24, ACCOUNT_A).insert(&conn).unwrap();
    assert!(failure(leg(25, ACCOUNT_A).insert(&conn)).contains("UNIQUE"));
    Tx {
        amount: 500,
        ..leg(26, ACCOUNT_B)
    }
    .insert(&conn)
    .unwrap();
}
