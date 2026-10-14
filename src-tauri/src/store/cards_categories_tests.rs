use super::fixtures::{NOW, account, card, card_tx, category, db, id, tx};
use super::{accounts, cards, categories, transactions};
use crate::error::ErrorCode;

#[test]
fn a_category_round_trips_and_names_are_unique_per_kind() {
    let mut conn = db();
    categories::create(&mut conn, &category(1, "Mercado", "expense")).unwrap();
    categories::create(&mut conn, &category(2, "Mercado", "income")).unwrap();
    assert_eq!(
        categories::get(&conn, &id(1)).unwrap(),
        Some(category(1, "Mercado", "expense"))
    );
    let duplicate = categories::create(&mut conn, &category(3, "Mercado", "expense")).unwrap_err();
    assert_eq!(duplicate.code, ErrorCode::DuplicateName);
    assert_eq!(categories::list(&conn).unwrap().len(), 2);
}

#[test]
fn an_invalid_category_kind_is_refused() {
    let mut conn = db();
    let error = categories::create(&mut conn, &category(1, "X", "transfer")).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidData);
}

#[test]
fn a_category_can_be_archived_renamed_and_deleted_only_when_unused() {
    let mut conn = db();
    accounts::create(&mut conn, &account(1, "Conta")).unwrap();
    categories::create(&mut conn, &category(2, "Mercado", "expense")).unwrap();
    categories::create(&mut conn, &category(3, "Lazer", "expense")).unwrap();
    let mut used = tx(10, &id(1), "2026-02-01", -500);
    used.category_id = Some(id(2));
    transactions::create(&mut conn, &used).unwrap();

    let mut renamed = category(2, "Supermercado", "expense");
    renamed.updated_at = "2026-11-01T09:00:00Z".to_string();
    categories::update(&mut conn, &renamed).unwrap();
    assert_eq!(
        categories::get(&conn, &id(2)).unwrap().unwrap().name,
        "Supermercado"
    );

    assert_eq!(
        categories::delete(&mut conn, &id(2)).unwrap_err().code,
        ErrorCode::InUse
    );
    categories::delete(&mut conn, &id(3)).unwrap();
    assert_eq!(
        categories::delete(&mut conn, &id(3)).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert_eq!(
        categories::update(&mut conn, &category(9, "Fantasma", "income"))
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
}

#[test]
fn a_card_round_trips_with_and_without_a_credit_limit() {
    let mut conn = db();
    cards::create(&mut conn, &card(1, "Nubank")).unwrap();
    let mut unlimited = card(2, "Inter");
    unlimited.credit_limit = None;
    cards::create(&mut conn, &unlimited).unwrap();
    assert_eq!(cards::get(&conn, &id(1)).unwrap(), Some(card(1, "Nubank")));
    assert_eq!(cards::get(&conn, &id(2)).unwrap(), Some(unlimited));
    let names: Vec<String> = cards::list(&conn)
        .unwrap()
        .into_iter()
        .map(|c| c.name)
        .collect();
    assert_eq!(names, ["Inter", "Nubank"]);
}

#[test]
fn card_names_days_and_limits_are_validated() {
    let mut conn = db();
    cards::create(&mut conn, &card(1, "Nubank")).unwrap();
    assert_eq!(
        cards::create(&mut conn, &card(2, "Nubank"))
            .unwrap_err()
            .code,
        ErrorCode::DuplicateName
    );
    for (closing, due, limit) in [
        (0, 10, Some(0)),
        (32, 10, Some(0)),
        (10, 0, Some(0)),
        (10, 32, Some(0)),
        (10, 10, Some(-1)),
    ] {
        let mut bad = card(3, "Outro");
        (bad.closing_day, bad.due_day, bad.credit_limit) = (closing, due, limit);
        assert_eq!(
            cards::create(&mut conn, &bad).unwrap_err().code,
            ErrorCode::InvalidData
        );
    }
    let mut edge = card(4, "Limite zero");
    (edge.closing_day, edge.due_day, edge.credit_limit) = (1, 31, Some(0));
    cards::create(&mut conn, &edge).unwrap();
}

#[test]
fn a_card_can_be_archived_and_deleted_only_when_unused() {
    let mut conn = db();
    cards::create(&mut conn, &card(1, "Usado")).unwrap();
    cards::create(&mut conn, &card(2, "Livre")).unwrap();
    transactions::create(&mut conn, &card_tx(10, &id(1), "2026-02-01", -500)).unwrap();

    let mut archived = card(1, "Usado");
    archived.archived_at = Some(NOW.to_string());
    cards::update(&mut conn, &archived).unwrap();
    assert_eq!(
        cards::delete(&mut conn, &id(1)).unwrap_err().code,
        ErrorCode::InUse
    );
    cards::delete(&mut conn, &id(2)).unwrap();
    assert_eq!(
        cards::delete(&mut conn, &id(2)).unwrap_err().code,
        ErrorCode::NotFound
    );
    // The name of an archived card can be reused.
    cards::create(&mut conn, &card(3, "Usado")).unwrap();
}
