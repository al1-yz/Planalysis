//! The commands exactly as the webview calls them: JSON in, JSON out, through Tauri's IPC layer
//! on a mock runtime (no window, no WebView).

use std::sync::Arc;

use planalysis_lib::database::Database;
use planalysis_lib::register_commands;
use serde_json::{Value, json};
use tauri::Manager;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{
    INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
};
use tauri::webview::InvokeRequest;

const FIRST: &str = "3f2b8c1e-5d4a-4e6f-9a1b-0c7d8e9f1a2b";
const SECOND: &str = "9a8b7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d";

struct Harness {
    webview: tauri::WebviewWindow<MockRuntime>,
    _app: tauri::App<MockRuntime>,
}

impl Harness {
    fn new(database: Database) -> Self {
        let app = register_commands(mock_builder())
            .build(mock_context(noop_assets()))
            .expect("the mock app builds");
        app.manage(Arc::new(database));
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("the mock window builds");
        Self { webview, _app: app }
    }

    /// Calls a command with JSON arguments and returns its JSON result or its JSON error.
    fn call(&self, cmd: &str, args: Value) -> Result<Value, Value> {
        let request = InvokeRequest {
            cmd: cmd.to_string(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "http://tauri.localhost".parse().expect("a valid URL"),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        };
        get_ipc_response(&self.webview, request)
            .map(|body| body.deserialize::<Value>().expect("a JSON response"))
    }
}

fn account(id: &str, name: &str) -> Value {
    let mut account: Value = serde_json::from_str(include_str!("../../contract/account.json"))
        .expect("the fixture is valid JSON");
    account["id"] = json!(id);
    account["name"] = json!(name);
    account["nameKey"] = json!(name.to_lowercase());
    account
}

fn transfer_legs() -> Value {
    serde_json::from_str(include_str!("../../contract/transfer-legs.json"))
        .expect("the fixture is valid JSON")
}

#[test]
fn database_status_reports_the_schema_version() {
    let harness = Harness::new(Database::in_memory());
    let status = harness.call("database_status", json!({})).unwrap();
    assert_eq!(status["ready"], json!(true));
    assert_eq!(status["schemaVersion"], json!(1));
    assert_eq!(status["error"], Value::Null);
}

#[test]
fn an_account_round_trips_over_ipc_with_camel_case_json() {
    let harness = Harness::new(Database::in_memory());
    let created = account(FIRST, "Conta Corrente");
    harness
        .call("account_create", json!({ "account": created }))
        .unwrap();

    assert_eq!(
        harness.call("account_get", json!({ "id": FIRST })).unwrap(),
        created
    );
    assert_eq!(
        harness
            .call("account_get", json!({ "id": SECOND }))
            .unwrap(),
        Value::Null
    );
    assert_eq!(
        harness.call("account_list", json!({})).unwrap(),
        json!([created])
    );
}

#[test]
fn failures_reach_the_webview_as_a_code_and_a_message() {
    let harness = Harness::new(Database::in_memory());
    harness
        .call(
            "account_create",
            json!({ "account": account(FIRST, "Conta") }),
        )
        .unwrap();

    let duplicate = harness
        .call(
            "account_create",
            json!({ "account": account(SECOND, "Conta") }),
        )
        .unwrap_err();
    assert_eq!(duplicate["code"], json!("duplicate_name"));
    assert!(duplicate["message"].as_str().is_some());

    let missing = harness
        .call("account_delete", json!({ "id": SECOND }))
        .unwrap_err();
    assert_eq!(missing["code"], json!("not_found"));
}

#[test]
fn unknown_fields_and_malformed_arguments_are_rejected() {
    let harness = Harness::new(Database::in_memory());
    let mut with_extra = account(FIRST, "Conta");
    with_extra["surprise"] = json!(true);
    assert!(
        harness
            .call("account_create", json!({ "account": with_extra }))
            .is_err()
    );
    assert!(harness.call("account_create", json!({})).is_err());
    let accounts = harness.call("account_list", json!({})).unwrap();
    assert!(accounts.as_array().unwrap().is_empty());
}

#[test]
fn a_transfer_is_written_read_and_removed_as_a_pair_over_ipc() {
    let harness = Harness::new(Database::in_memory());
    for (id, name) in [(FIRST, "Conta"), (SECOND, "Reserva")] {
        harness
            .call("account_create", json!({ "account": account(id, name) }))
            .unwrap();
    }
    let legs = transfer_legs();
    let transfer_id = legs[0]["transferId"].clone();

    harness
        .call("transfer_create", json!({ "legs": legs }))
        .unwrap();
    assert_eq!(
        harness
            .call("transfer_get", json!({ "transferId": transfer_id }))
            .unwrap(),
        legs
    );
    let listed = harness
        .call("transaction_list", json!({ "filter": {} }))
        .unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 2);

    let single = harness
        .call("transaction_delete", json!({ "id": legs[0]["id"] }))
        .unwrap_err();
    assert_eq!(single["code"], json!("transfer_leg"));

    harness
        .call("transfer_delete", json!({ "transferId": transfer_id }))
        .unwrap();
    assert_eq!(
        harness
            .call("transfer_get", json!({ "transferId": transfer_id }))
            .unwrap(),
        Value::Null
    );
}

#[test]
fn a_transfer_that_fails_leaves_nothing_behind() {
    let harness = Harness::new(Database::in_memory());
    harness
        .call(
            "account_create",
            json!({ "account": account(FIRST, "Conta") }),
        )
        .unwrap();
    // The second account was never created, so the second leg cannot be written.
    let error = harness
        .call("transfer_create", json!({ "legs": transfer_legs() }))
        .unwrap_err();
    assert_eq!(error["code"], json!("not_found"));
    let listed = harness
        .call("transaction_list", json!({ "filter": {} }))
        .unwrap();
    assert!(listed.as_array().unwrap().is_empty());
}

#[test]
fn an_unusable_database_explains_why_and_refuses_every_command() {
    let directory = std::env::temp_dir().join(format!("planalysis-ipc-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("newer.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.pragma_update(None, "application_id", 0x504C_414E_i32)
            .unwrap();
        conn.execute_batch("CREATE TABLE future (x INTEGER); PRAGMA user_version = 99;")
            .unwrap();
    }
    let before = std::fs::read(&path).unwrap();

    let harness = Harness::new(Database::open(&path));
    let status = harness.call("database_status", json!({})).unwrap();
    assert_eq!(status["ready"], json!(false));
    assert_eq!(status["error"]["code"], json!("database_too_new"));
    let refused = harness.call("account_list", json!({})).unwrap_err();
    assert_eq!(refused["code"], json!("database_unavailable"));

    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "an existing database is never modified"
    );
    let _ = std::fs::remove_dir_all(&directory);
}
