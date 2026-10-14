use super::fixtures::{NOW, account, card, card_tx, category, db, id, leg, tx};
use super::{accounts, cards, categories, transactions};
use crate::dto::TransactionFilter;
use crate::error::ErrorCode;

fn seeded() -> rusqlite::Connection {
    let mut conn = db();
    accounts::create(&mut conn, &account(1, "Conta")).unwrap();
    accounts::create(&mut conn, &account(2, "Reserva")).unwrap();
    cards::create(&mut conn, &card(3, "Nubank")).unwrap();
    categories::create(&mut conn, &category(4, "Mercado", "expense")).unwrap();
    conn
}

fn code(result: Result<(), crate::error::AppError>) -> ErrorCode {
    result
        .expect_err("the operation should have been refused")
        .code
}

#[test]
fn account_and_card_transactions_round_trip() {
    let mut conn = seeded();
    let mut on_account = tx(10, &id(1), "2026-02-01", -4590);
    on_account.category_id = Some(id(4));
    let on_card = card_tx(11, &id(3), "2026-02-02", -12000);
    transactions::create(&mut conn, &on_account).unwrap();
    transactions::create(&mut conn, &on_card).unwrap();

    assert_eq!(transactions::get(&conn, &id(10)).unwrap(), Some(on_account));
    assert_eq!(transactions::get(&conn, &id(11)).unwrap(), Some(on_card));
    assert_eq!(transactions::get(&conn, &id(99)).unwrap(), None);
}

#[test]
fn list_is_newest_first_and_honours_every_filter() {
    let mut conn = seeded();
    let mut groceries = tx(10, &id(1), "2026-02-10", -100);
    groceries.category_id = Some(id(4));
    for t in [
        tx(11, &id(1), "2026-02-01", -200),
        groceries,
        tx(12, &id(2), "2026-02-20", 300),
        card_tx(13, &id(3), "2026-02-15", -400),
    ] {
        transactions::create(&mut conn, &t).unwrap();
    }
    let ids = |filter: TransactionFilter| -> Vec<String> {
        transactions::list(&conn, &filter)
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect()
    };

    assert_eq!(
        ids(TransactionFilter::default()),
        [id(12), id(13), id(10), id(11)]
    );
    let by_account = TransactionFilter {
        account_id: Some(id(1)),
        ..Default::default()
    };
    assert_eq!(ids(by_account), [id(10), id(11)]);
    let by_card = TransactionFilter {
        card_id: Some(id(3)),
        ..Default::default()
    };
    assert_eq!(ids(by_card), [id(13)]);
    let by_category = TransactionFilter {
        category_id: Some(id(4)),
        ..Default::default()
    };
    assert_eq!(ids(by_category), [id(10)]);
    let window = TransactionFilter {
        from: Some("2026-02-10".to_string()),
        to: Some("2026-02-15".to_string()),
        ..Default::default()
    };
    assert_eq!(ids(window), [id(13), id(10)], "bounds are inclusive");
    let account_window = TransactionFilter {
        account_id: Some(id(1)),
        to: Some("2026-02-05".to_string()),
        ..Default::default()
    };
    assert_eq!(ids(account_window), [id(11)]);
}

#[test]
fn invalid_transactions_are_refused_with_a_specific_reason() {
    let mut conn = seeded();
    assert_eq!(
        code(transactions::create(
            &mut conn,
            &tx(10, &id(99), "2026-02-01", -1)
        )),
        ErrorCode::NotFound
    );
    assert_eq!(
        code(transactions::create(
            &mut conn,
            &tx(10, &id(1), "2025-12-31", -1)
        )),
        ErrorCode::BeforeOpeningDate
    );
    assert_eq!(
        code(transactions::create(
            &mut conn,
            &tx(10, &id(1), "2026-02-01", 0)
        )),
        ErrorCode::InvalidData
    );
    assert_eq!(
        code(transactions::create(
            &mut conn,
            &tx(10, &id(1), "2026-02-30", -1)
        )),
        ErrorCode::InvalidData
    );
    let mut both = tx(10, &id(1), "2026-02-01", -1);
    both.card_id = Some(id(3));
    assert_eq!(
        code(transactions::create(&mut conn, &both)),
        ErrorCode::InvalidData
    );
    let mut bad_category = tx(10, &id(1), "2026-02-01", -1);
    bad_category.category_id = Some(id(98));
    assert_eq!(
        code(transactions::create(&mut conn, &bad_category)),
        ErrorCode::NotFound
    );
    assert!(
        transactions::list(&conn, &TransactionFilter::default())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn archived_accounts_cards_and_categories_cannot_receive_new_transactions() {
    let mut conn = seeded();
    let mut archived_account = account(1, "Conta");
    archived_account.archived_at = Some(NOW.to_string());
    accounts::update(&mut conn, &archived_account).unwrap();
    let mut archived_card = card(3, "Nubank");
    archived_card.archived_at = Some(NOW.to_string());
    cards::update(&mut conn, &archived_card).unwrap();
    let mut archived_category = category(4, "Mercado", "expense");
    archived_category.archived_at = Some(NOW.to_string());
    categories::update(&mut conn, &archived_category).unwrap();

    assert_eq!(
        code(transactions::create(
            &mut conn,
            &tx(10, &id(1), "2026-02-01", -1)
        )),
        ErrorCode::ArchivedTarget
    );
    assert_eq!(
        code(transactions::create(
            &mut conn,
            &card_tx(11, &id(3), "2026-02-01", -1)
        )),
        ErrorCode::ArchivedTarget
    );
    let mut categorized = tx(12, &id(2), "2026-02-01", -1);
    categorized.category_id = Some(id(4));
    assert_eq!(
        code(transactions::create(&mut conn, &categorized)),
        ErrorCode::ArchivedTarget
    );
}

#[test]
fn history_on_an_archived_account_stays_editable_but_cannot_move_to_one() {
    let mut conn = seeded();
    let mut categorized = tx(10, &id(1), "2026-02-01", -500);
    categorized.category_id = Some(id(4));
    transactions::create(&mut conn, &categorized).unwrap();
    transactions::create(&mut conn, &tx(11, &id(2), "2026-02-02", -700)).unwrap();

    let mut archived_account = account(1, "Conta");
    archived_account.archived_at = Some(NOW.to_string());
    accounts::update(&mut conn, &archived_account).unwrap();
    let mut archived_category = category(4, "Mercado", "expense");
    archived_category.archived_at = Some(NOW.to_string());
    categories::update(&mut conn, &archived_category).unwrap();

    let mut corrected = categorized.clone();
    corrected.description = "corrigido".to_string();
    corrected.amount = -550;
    transactions::update(&mut conn, &corrected).unwrap();
    assert_eq!(transactions::get(&conn, &id(10)).unwrap(), Some(corrected));

    let mut moved = tx(11, &id(1), "2026-02-02", -700);
    moved.account_id = Some(id(1));
    assert_eq!(
        code(transactions::update(&mut conn, &moved)),
        ErrorCode::ArchivedTarget
    );
}

#[test]
fn update_and_delete_report_missing_records_and_respect_the_opening_date() {
    let mut conn = seeded();
    transactions::create(&mut conn, &tx(10, &id(1), "2026-02-01", -500)).unwrap();

    assert_eq!(
        code(transactions::update(
            &mut conn,
            &tx(99, &id(1), "2026-02-01", -1)
        )),
        ErrorCode::NotFound
    );
    assert_eq!(
        code(transactions::update(
            &mut conn,
            &tx(10, &id(1), "2025-01-01", -500)
        )),
        ErrorCode::BeforeOpeningDate
    );
    let mut moved = tx(10, &id(1), "2026-02-01", -500);
    moved.account_id = None;
    moved.card_id = Some(id(3));
    transactions::update(&mut conn, &moved).unwrap();
    assert_eq!(
        transactions::get(&conn, &id(10)).unwrap().unwrap().card_id,
        Some(id(3))
    );

    transactions::delete(&mut conn, &id(10)).unwrap();
    assert_eq!(
        code(transactions::delete(&mut conn, &id(10))),
        ErrorCode::NotFound
    );
}

#[test]
fn transfer_legs_cannot_be_changed_through_the_single_transaction_operations() {
    let mut conn = seeded();
    let legs = [
        leg(10, &id(50), &id(1), "2026-02-01", -500),
        leg(11, &id(50), &id(2), "2026-02-01", 500),
    ];
    assert_eq!(
        code(transactions::create(&mut conn, &legs[0])),
        ErrorCode::TransferLeg
    );
    super::transfers::create(&mut conn, &legs).unwrap();

    let mut edited = legs[0].clone();
    edited.description = "x".to_string();
    assert_eq!(
        code(transactions::update(&mut conn, &edited)),
        ErrorCode::TransferLeg
    );
    assert_eq!(
        code(transactions::delete(&mut conn, &legs[0].id)),
        ErrorCode::TransferLeg
    );
    let mut detached = tx(10, &id(1), "2026-02-01", -500);
    detached.transfer_id = None;
    assert_eq!(
        code(transactions::update(&mut conn, &detached)),
        ErrorCode::TransferLeg
    );
    assert_eq!(super::fixtures::count(&conn, "transactions"), 2);
}
