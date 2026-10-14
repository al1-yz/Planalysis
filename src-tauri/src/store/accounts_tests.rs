use super::fixtures::{NOW, account, db, id, tx};
use super::{accounts, transactions};
use crate::error::ErrorCode;

#[test]
fn an_account_round_trips_and_lists_by_name() {
    let mut conn = db();
    accounts::create(&mut conn, &account(2, "Reserva")).unwrap();
    accounts::create(&mut conn, &account(1, "Carteira")).unwrap();

    assert_eq!(
        accounts::get(&conn, &id(1)).unwrap(),
        Some(account(1, "Carteira"))
    );
    assert_eq!(accounts::get(&conn, &id(99)).unwrap(), None);
    let names: Vec<String> = accounts::list(&conn)
        .unwrap()
        .into_iter()
        .map(|a| a.name)
        .collect();
    assert_eq!(names, ["Carteira", "Reserva"]);
}

#[test]
fn an_archived_flag_and_negative_opening_balance_are_stored() {
    let mut conn = db();
    let mut stored = account(1, "Conta");
    stored.opening_balance = -25_000;
    stored.archived_at = Some(NOW.to_string());
    accounts::create(&mut conn, &stored).unwrap();
    assert_eq!(accounts::get(&conn, &id(1)).unwrap(), Some(stored));
}

#[test]
fn active_names_are_unique_but_archived_names_are_free_to_reuse() {
    let mut conn = db();
    accounts::create(&mut conn, &account(1, "Conta")).unwrap();
    let duplicate = accounts::create(&mut conn, &account(2, "Conta")).unwrap_err();
    assert_eq!(duplicate.code, ErrorCode::DuplicateName);

    let mut archived = account(1, "Conta");
    archived.archived_at = Some(NOW.to_string());
    accounts::update(&mut conn, &archived).unwrap();
    accounts::create(&mut conn, &account(2, "Conta")).unwrap();

    let unarchive_clash = accounts::update(&mut conn, &account(1, "Conta")).unwrap_err();
    assert_eq!(unarchive_clash.code, ErrorCode::DuplicateName);
}

#[test]
fn a_name_key_that_was_not_normalized_is_refused() {
    let mut conn = db();
    let mut loud = account(1, "Conta");
    loud.name_key = "CONTA".to_string();
    assert_eq!(
        accounts::create(&mut conn, &loud).unwrap_err().code,
        ErrorCode::InvalidData
    );
    let mut spaced = account(2, "Conta");
    spaced.name_key = "conta  corrente".to_string();
    assert_eq!(
        accounts::create(&mut conn, &spaced).unwrap_err().code,
        ErrorCode::InvalidData
    );
}

#[test]
fn invalid_fields_are_refused() {
    let mut conn = db();
    let mut bad_type = account(1, "Conta");
    bad_type.account_type = "investment".to_string();
    assert_eq!(
        accounts::create(&mut conn, &bad_type).unwrap_err().code,
        ErrorCode::InvalidData
    );
    let mut bad_date = account(2, "Outra");
    bad_date.opening_date = "2026-02-30".to_string();
    assert_eq!(
        accounts::create(&mut conn, &bad_date).unwrap_err().code,
        ErrorCode::InvalidData
    );
    let repeated = accounts::create(&mut conn, &account(3, "Nova"))
        .and_then(|()| accounts::create(&mut conn, &account(3, "Nova2")));
    assert_eq!(
        repeated.unwrap_err().code,
        ErrorCode::InvalidData,
        "an id cannot be reused"
    );
}

#[test]
fn update_changes_fields_but_not_the_creation_time() {
    let mut conn = db();
    accounts::create(&mut conn, &account(1, "Conta")).unwrap();

    let mut changed = account(1, "Conta nova");
    changed.opening_balance = 5000;
    changed.created_at = "2030-01-01T00:00:00Z".to_string();
    changed.updated_at = "2026-11-01T09:00:00Z".to_string();
    accounts::update(&mut conn, &changed).unwrap();

    let stored = accounts::get(&conn, &id(1)).unwrap().unwrap();
    assert_eq!(
        (stored.name.as_str(), stored.opening_balance),
        ("Conta nova", 5000)
    );
    assert_eq!(stored.updated_at, "2026-11-01T09:00:00Z");
    assert_eq!(stored.created_at, NOW, "created_at never changes");

    let missing = accounts::update(&mut conn, &account(9, "Fantasma")).unwrap_err();
    assert_eq!(missing.code, ErrorCode::NotFound);
}

#[test]
fn an_unused_account_can_be_deleted_but_a_referenced_one_cannot() {
    let mut conn = db();
    accounts::create(&mut conn, &account(1, "Usada")).unwrap();
    accounts::create(&mut conn, &account(2, "Livre")).unwrap();
    transactions::create(&mut conn, &tx(10, &id(1), "2026-02-01", -500)).unwrap();

    assert_eq!(
        accounts::delete(&mut conn, &id(1)).unwrap_err().code,
        ErrorCode::InUse
    );
    accounts::delete(&mut conn, &id(2)).unwrap();
    assert_eq!(accounts::get(&conn, &id(2)).unwrap(), None);
    assert_eq!(
        accounts::delete(&mut conn, &id(2)).unwrap_err().code,
        ErrorCode::NotFound
    );
}

#[test]
fn the_opening_date_may_move_earlier_but_not_past_a_transaction() {
    let mut conn = db();
    accounts::create(&mut conn, &account(1, "Conta")).unwrap();
    transactions::create(&mut conn, &tx(10, &id(1), "2026-03-10", -500)).unwrap();

    let mut later = account(1, "Conta");
    later.opening_date = "2026-03-11".to_string();
    assert_eq!(
        accounts::update(&mut conn, &later).unwrap_err().code,
        ErrorCode::OpeningDateAfterTransactions
    );
    let mut earlier = account(1, "Conta");
    earlier.opening_date = "2025-06-01".to_string();
    accounts::update(&mut conn, &earlier).unwrap();
}
