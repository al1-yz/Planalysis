//! The wire format shared with the TypeScript side. The same JSON files in `contract/` are
//! checked there, so a field renamed on one side fails a test on the other.

use planalysis_lib::database::{DatabaseStatus, StartupError, StartupErrorCode};
use planalysis_lib::dto::{Account, Card, Category, Transaction, TransactionFilter};
use planalysis_lib::error::{AppError, ErrorCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

fn fixture(json: &str) -> Value {
    serde_json::from_str(json).expect("the fixture is valid JSON")
}

/// Parsing a fixture and serializing it again must give back exactly the same JSON.
fn assert_round_trips<T: Serialize + DeserializeOwned>(json: &str) {
    let expected = fixture(json);
    let parsed: T = serde_json::from_value(expected.clone()).expect("the fixture matches the type");
    assert_eq!(
        serde_json::to_value(parsed).expect("the value serializes"),
        expected
    );
}

#[test]
fn entities_round_trip_through_the_shared_fixtures() {
    assert_round_trips::<Account>(include_str!("../../contract/account.json"));
    assert_round_trips::<Category>(include_str!("../../contract/category.json"));
    assert_round_trips::<Card>(include_str!("../../contract/card.json"));
    assert_round_trips::<Transaction>(include_str!("../../contract/transaction.json"));
    assert_round_trips::<Vec<Transaction>>(include_str!("../../contract/transfer-legs.json"));
    assert_round_trips::<TransactionFilter>(include_str!("../../contract/transaction-filter.json"));
}

#[test]
fn an_unexpected_field_is_rejected_instead_of_ignored() {
    let mut account = fixture(include_str!("../../contract/account.json"));
    account["surprise"] = Value::Bool(true);
    assert!(serde_json::from_value::<Account>(account).is_err());
}

#[test]
fn a_filter_may_omit_every_field() {
    let filter: TransactionFilter = serde_json::from_str("{}").unwrap();
    assert_eq!(filter, TransactionFilter::default());
}

#[test]
fn errors_serialize_as_a_code_and_a_message() {
    let error = AppError::new(
        ErrorCode::DuplicateName,
        "UNIQUE constraint failed: accounts.name_key",
    );
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        fixture(include_str!("../../contract/app-error.json"))
    );
}

#[test]
fn database_status_serializes_as_the_shared_fixtures() {
    let ready = DatabaseStatus {
        ready: true,
        schema_version: Some(1),
        app_version: "0.1.0".to_string(),
        path: "/data/planalysis.db".to_string(),
        error: None,
    };
    assert_eq!(
        serde_json::to_value(ready).unwrap(),
        fixture(include_str!("../../contract/database-status-ready.json"))
    );
    let failed = DatabaseStatus {
        ready: false,
        schema_version: None,
        app_version: "0.1.0".to_string(),
        path: "/data/planalysis.db".to_string(),
        error: Some(StartupError {
            code: StartupErrorCode::DatabaseTooNew,
            message: "the database uses schema version 99, but this build supports up to 1"
                .to_string(),
        }),
    };
    assert_eq!(
        serde_json::to_value(failed).unwrap(),
        fixture(include_str!("../../contract/database-status-failed.json"))
    );
}
