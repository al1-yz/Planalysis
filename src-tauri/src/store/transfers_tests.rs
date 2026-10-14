use super::fixtures::{NOW, account, count, db, id, leg, transfer, tx};
use super::{accounts, transactions, transfers};
use crate::error::ErrorCode;

const T: &str = "0f0f0f0f-1e1e-4d2d-8c3c-4b4b4b4b4b4b";

fn seeded() -> rusqlite::Connection {
    let mut conn = db();
    accounts::create(&mut conn, &account(1, "Conta")).unwrap();
    accounts::create(&mut conn, &account(2, "Reserva")).unwrap();
    accounts::create(&mut conn, &account(3, "Carteira")).unwrap();
    conn
}

fn code(result: Result<(), crate::error::AppError>) -> ErrorCode {
    result
        .expect_err("the operation should have been refused")
        .code
}

#[test]
fn a_transfer_writes_both_legs_and_reads_back_outgoing_first() {
    let mut conn = seeded();
    let legs = transfer(10, T, &id(1), &id(2), "2026-02-01", 25_000);
    transfers::create(&mut conn, &legs).unwrap();

    let stored = transfers::get(&conn, T).unwrap().unwrap();
    assert_eq!(stored, legs);
    assert_eq!(stored[0].amount + stored[1].amount, 0);
    assert_eq!(transfers::get(&conn, &id(77)).unwrap(), None);
    // Legs given in the opposite order are accepted and stored the same way.
    transfers::delete(&mut conn, T).unwrap();
    let reversed = vec![legs[1].clone(), legs[0].clone()];
    transfers::create(&mut conn, &reversed).unwrap();
    assert_eq!(transfers::get(&conn, T).unwrap().unwrap(), legs);
}

#[test]
fn a_pair_that_is_not_a_valid_transfer_is_refused_and_writes_nothing() {
    let mut conn = seeded();
    let good = transfer(10, T, &id(1), &id(2), "2026-02-01", 500);

    let mut one_leg = good.clone();
    one_leg.pop();
    let mut three_legs = good.clone();
    three_legs.push(leg(12, T, &id(3), "2026-02-01", 0));
    let mut not_opposite = good.clone();
    not_opposite[1].amount = 499;
    let mut both_negative = good.clone();
    both_negative[1].amount = -500;
    let mut different_dates = good.clone();
    different_dates[1].date = "2026-02-02".to_string();
    let mut same_account = good.clone();
    same_account[1].account_id = Some(id(1));
    let mut different_transfers = good.clone();
    different_transfers[1].transfer_id = Some(id(51));
    let mut no_transfer_id = good.clone();
    no_transfer_id[0].transfer_id = None;
    no_transfer_id[1].transfer_id = None;
    let mut with_category = good.clone();
    with_category[0].category_id = Some(id(60));
    let mut on_card = good.clone();
    on_card[0].account_id = None;
    on_card[0].card_id = Some(id(61));

    for (label, legs) in [
        ("one leg", one_leg),
        ("three legs", three_legs),
        ("amounts not opposite", not_opposite),
        ("both negative", both_negative),
        ("different dates", different_dates),
        ("same account", same_account),
        ("different transfer ids", different_transfers),
        ("no transfer id", no_transfer_id),
        ("category on a leg", with_category),
        ("card on a leg", on_card),
    ] {
        let error = transfers::create(&mut conn, &legs).expect_err(label);
        assert_eq!(error.code, ErrorCode::InvalidTransfer, "{label}");
    }
    assert_eq!(count(&conn, "transactions"), 0);
}

#[test]
fn creating_a_transfer_is_all_or_nothing() {
    let mut conn = seeded();
    // The second leg points at an account that does not exist, so its insert fails after the
    // first leg was already written inside the transaction.
    let legs = transfer(10, T, &id(1), &id(99), "2026-02-01", 500);
    assert_eq!(
        code(transfers::create(&mut conn, &legs)),
        ErrorCode::NotFound
    );
    assert_eq!(
        count(&conn, "transactions"),
        0,
        "the first leg must be rolled back"
    );
}

#[test]
fn a_transfer_cannot_predate_either_accounts_opening_date() {
    let mut conn = seeded();
    let mut later = account(2, "Reserva");
    later.opening_date = "2026-06-01".to_string();
    accounts::update(&mut conn, &later).unwrap();

    let legs = transfer(10, T, &id(1), &id(2), "2026-03-01", 500);
    assert_eq!(
        code(transfers::create(&mut conn, &legs)),
        ErrorCode::BeforeOpeningDate
    );
    assert_eq!(count(&conn, "transactions"), 0);
}

#[test]
fn a_transfer_id_cannot_be_written_twice() {
    let mut conn = seeded();
    transfers::create(
        &mut conn,
        &transfer(10, T, &id(1), &id(2), "2026-02-01", 500),
    )
    .unwrap();
    let again = transfer(20, T, &id(1), &id(3), "2026-02-01", 700);
    assert_eq!(
        code(transfers::create(&mut conn, &again)),
        ErrorCode::InvalidTransfer
    );
    assert_eq!(count(&conn, "transactions"), 2);
    let other_accounts = transfer(30, T, &id(3), &id(2), "2026-02-01", 700);
    assert_eq!(
        code(transfers::create(&mut conn, &other_accounts)),
        ErrorCode::InvalidTransfer
    );
    assert_eq!(count(&conn, "transactions"), 2);
}

#[test]
fn updating_a_transfer_changes_both_legs_together() {
    let mut conn = seeded();
    transfers::create(
        &mut conn,
        &transfer(10, T, &id(1), &id(2), "2026-02-01", 500),
    )
    .unwrap();

    let mut changed = transfer(10, T, &id(1), &id(3), "2026-02-05", 900);
    for leg in &mut changed {
        leg.description = "ajustado".to_string();
        leg.updated_at = "2026-11-01T09:00:00Z".to_string();
    }
    transfers::update(&mut conn, &changed).unwrap();

    assert_eq!(transfers::get(&conn, T).unwrap().unwrap(), changed);
    assert_eq!(count(&conn, "transactions"), 2);
    assert_eq!(
        code(transfers::update(
            &mut conn,
            &transfer(40, &id(52), &id(1), &id(2), "2026-02-01", 1)
        )),
        ErrorCode::NotFound
    );
}

#[test]
fn a_failed_transfer_update_leaves_the_original_untouched() {
    let mut conn = seeded();
    let original = transfer(10, T, &id(1), &id(2), "2026-02-01", 500);
    transfers::create(&mut conn, &original).unwrap();

    // A transfer cannot be redirected to an account that does not exist.
    let broken = transfer(10, T, &id(1), &id(99), "2026-02-05", 900);
    assert_eq!(
        code(transfers::update(&mut conn, &broken)),
        ErrorCode::NotFound
    );
    assert_eq!(
        transfers::get(&conn, T).unwrap().unwrap(),
        original,
        "the original must survive"
    );

    // This one only fails after the old legs were deleted and the first new leg was inserted
    // (the opening-date trigger fires on that insert), so the rollback has real work to undo.
    let too_early = transfer(10, T, &id(1), &id(2), "2025-01-01", 900);
    assert_eq!(
        code(transfers::update(&mut conn, &too_early)),
        ErrorCode::BeforeOpeningDate
    );
    assert_eq!(transfers::get(&conn, T).unwrap().unwrap(), original);
}

#[test]
fn deleting_a_transfer_removes_both_legs() {
    let mut conn = seeded();
    transfers::create(
        &mut conn,
        &transfer(10, T, &id(1), &id(2), "2026-02-01", 500),
    )
    .unwrap();
    transactions::create(&mut conn, &tx(20, &id(1), "2026-02-01", -100)).unwrap();

    transfers::delete(&mut conn, T).unwrap();
    assert_eq!(transfers::get(&conn, T).unwrap(), None);
    assert_eq!(
        count(&conn, "transactions"),
        1,
        "unrelated transactions are untouched"
    );
    assert_eq!(code(transfers::delete(&mut conn, T)), ErrorCode::NotFound);
}

#[test]
fn an_archived_account_blocks_new_transfers_but_not_corrections_to_existing_ones() {
    let mut conn = seeded();
    transfers::create(
        &mut conn,
        &transfer(10, T, &id(1), &id(2), "2026-02-01", 500),
    )
    .unwrap();

    let mut archived = account(2, "Reserva");
    archived.archived_at = Some(NOW.to_string());
    accounts::update(&mut conn, &archived).unwrap();

    // A brand-new transfer involving the archived account is refused.
    let fresh = transfer(30, &id(53), &id(1), &id(2), "2026-02-02", 100);
    assert_eq!(
        code(transfers::create(&mut conn, &fresh)),
        ErrorCode::ArchivedTarget
    );
    assert_eq!(count(&conn, "transactions"), 2);

    // Correcting the existing transfer is allowed because its account identities are unchanged.
    let mut corrected = transfer(10, T, &id(1), &id(2), "2026-02-01", 650);
    for leg in &mut corrected {
        leg.description = "corrigido".to_string();
    }
    transfers::update(&mut conn, &corrected).unwrap();
    assert_eq!(transfers::get(&conn, T).unwrap().unwrap(), corrected);

    // Pointing the transfer at an archived account it did not use before is refused.
    let mut archived_wallet = account(3, "Carteira");
    archived_wallet.archived_at = Some(NOW.to_string());
    accounts::update(&mut conn, &archived_wallet).unwrap();
    let redirected = transfer(10, T, &id(1), &id(3), "2026-02-01", 650);
    assert_eq!(
        code(transfers::update(&mut conn, &redirected)),
        ErrorCode::ArchivedTarget
    );
    assert_eq!(
        transfers::get(&conn, T).unwrap().unwrap(),
        corrected,
        "rolled back"
    );
}
