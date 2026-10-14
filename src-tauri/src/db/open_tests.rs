//! Opening a database: configuration, migrations, and the cases it must refuse.

use rusqlite::Connection;

use super::migrate::{APPLICATION_ID, MIGRATIONS, Migration, OpenError, migrate};
use super::test_support::TempDir;
use super::{open, open_in_memory};

fn user_version(conn: &Connection) -> i64 {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap()
}

fn application_id(conn: &Connection) -> i64 {
    conn.pragma_query_value(None, "application_id", |row| row.get(0))
        .unwrap()
}

fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [name],
        |row| row.get::<_, i64>(0),
    )
    .unwrap()
        == 1
}

fn latest_version() -> u32 {
    MIGRATIONS.last().map(|m| m.version).unwrap()
}

#[test]
fn the_built_in_migrations_are_numbered_consecutively_from_one() {
    for (index, migration) in MIGRATIONS.iter().enumerate() {
        assert_eq!(migration.version as usize, index + 1, "{}", migration.name);
    }
}

#[test]
fn a_fresh_database_gets_the_latest_schema_and_the_planalysis_identity() {
    let conn = open_in_memory().unwrap();
    assert_eq!(user_version(&conn), i64::from(latest_version()));
    assert_eq!(application_id(&conn), i64::from(APPLICATION_ID));
    for table in ["accounts", "categories", "cards", "transactions"] {
        assert!(table_exists(&conn, table), "{table}");
    }
}

#[test]
fn foreign_key_enforcement_is_on_for_every_opened_connection() {
    let dir = TempDir::new();
    for conn in [open_in_memory().unwrap(), open(&dir.file("a.db")).unwrap()] {
        let enabled: i64 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert_eq!(enabled, 1);
    }
}

#[test]
fn a_file_database_uses_wal_and_full_synchronous_writes() {
    let dir = TempDir::new();
    let conn = open(&dir.file("planalysis.db")).unwrap();
    let mode: String = conn
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .unwrap();
    let synchronous: i64 = conn
        .pragma_query_value(None, "synchronous", |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
    assert_eq!(synchronous, 2, "FULL");
}

#[test]
fn reopening_keeps_the_data_and_changes_nothing() {
    let dir = TempDir::new();
    let path = dir.file("planalysis.db");
    {
        let conn = open(&path).unwrap();
        conn.execute(
            "INSERT INTO categories VALUES ('aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee', 'Mercado', 'mercado', 'expense', NULL, '2026-10-13T12:00:00Z', '2026-10-13T12:00:00Z')",
            [],
        )
        .unwrap();
    }
    let conn = open(&path).unwrap();
    assert_eq!(user_version(&conn), i64::from(latest_version()));
    let count: i64 = conn
        .query_row("SELECT count(*) FROM categories", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn a_database_from_a_newer_version_is_refused_and_left_untouched() {
    let dir = TempDir::new();
    let path = dir.file("newer.db");
    {
        let conn = Connection::open(&path).unwrap();
        conn.pragma_update(None, "application_id", APPLICATION_ID)
            .unwrap();
        conn.execute_batch("CREATE TABLE future (x INTEGER); PRAGMA user_version = 99;")
            .unwrap();
    }
    let before = std::fs::read(&path).unwrap();

    match open(&path) {
        Err(OpenError::TooNew { found: 99, .. }) => {}
        other => panic!("expected TooNew, got {other:?}"),
    }
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "the file must not be modified"
    );
}

#[test]
fn a_non_planalysis_database_is_refused_and_left_untouched() {
    let dir = TempDir::new();
    let path = dir.file("other.db");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE notes (text TEXT); INSERT INTO notes VALUES ('hi');")
            .unwrap();
    }
    let before = std::fs::read(&path).unwrap();

    match open(&path) {
        Err(OpenError::ForeignDatabase) => {}
        other => panic!("expected ForeignDatabase, got {other:?}"),
    }
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "the file must not be modified"
    );
}

const V1: Migration = Migration {
    version: 1,
    name: "create_notes",
    sql: "CREATE TABLE notes (id INTEGER PRIMARY KEY, text TEXT NOT NULL);",
};
const V2: Migration = Migration {
    version: 2,
    name: "add_pinned",
    sql: "ALTER TABLE notes ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;",
};
const V2_BROKEN: Migration = Migration {
    version: 2,
    name: "broken",
    sql: "CREATE TABLE half_done (x INTEGER); INSERT INTO table_that_does_not_exist VALUES (1);",
};

#[test]
fn an_upgrade_applies_only_pending_migrations_and_keeps_existing_data() {
    let mut conn = Connection::open_in_memory().unwrap();
    assert_eq!(migrate(&mut conn, &[V1]).unwrap(), 1);
    conn.execute("INSERT INTO notes (text) VALUES ('keep me')", [])
        .unwrap();

    assert_eq!(migrate(&mut conn, &[V1, V2]).unwrap(), 2);
    let (text, pinned): (String, i64) = conn
        .query_row("SELECT text, pinned FROM notes", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap();
    assert_eq!((text.as_str(), pinned), ("keep me", 0));
    assert_eq!(
        migrate(&mut conn, &[V1, V2]).unwrap(),
        2,
        "re-running is a no-op"
    );
}

#[test]
fn a_failing_migration_rolls_back_completely_and_keeps_the_previous_version() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate(&mut conn, &[V1]).unwrap();

    match migrate(&mut conn, &[V1, V2_BROKEN]) {
        Err(OpenError::Migration { version: 2, .. }) => {}
        other => panic!("expected a failed migration 2, got {other:?}"),
    }
    assert_eq!(user_version(&conn), 1);
    assert!(
        !table_exists(&conn, "half_done"),
        "partial changes must be rolled back"
    );
}
