//! The IPC surface: thin wrappers that hand each operation to a blocking thread. SQLite is
//! synchronous, so work never runs on the UI thread or ties up the async runtime.

use std::sync::Arc;

use rusqlite::Connection;
use tauri::State;

use crate::database::{Database, DatabaseStatus};
use crate::dto::{Account, Card, Category, Transaction, TransactionFilter};
use crate::error::{AppError, ErrorCode};
use crate::store::{accounts, cards, categories, transactions, transfers};

type Db<'a> = State<'a, Arc<Database>>;

/// Runs `work` on a blocking thread with exclusive use of the connection.
async fn run<T, F>(database: Db<'_>, work: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce(&mut Connection) -> Result<T, AppError> + Send + 'static,
{
    let database = Arc::clone(&database);
    tauri::async_runtime::spawn_blocking(move || database.with(work))
        .await
        .map_err(|error| AppError::new(ErrorCode::Internal, error.to_string()))?
}

#[tauri::command]
pub fn database_status(database: Db<'_>) -> DatabaseStatus {
    database.status()
}

#[tauri::command]
pub async fn account_list(database: Db<'_>) -> Result<Vec<Account>, AppError> {
    run(database, |conn| accounts::list(conn)).await
}

#[tauri::command]
pub async fn account_get(database: Db<'_>, id: String) -> Result<Option<Account>, AppError> {
    run(database, move |conn| accounts::get(conn, &id)).await
}

#[tauri::command]
pub async fn account_create(database: Db<'_>, account: Account) -> Result<(), AppError> {
    run(database, move |conn| accounts::create(conn, &account)).await
}

#[tauri::command]
pub async fn account_update(database: Db<'_>, account: Account) -> Result<(), AppError> {
    run(database, move |conn| accounts::update(conn, &account)).await
}

#[tauri::command]
pub async fn account_delete(database: Db<'_>, id: String) -> Result<(), AppError> {
    run(database, move |conn| accounts::delete(conn, &id)).await
}

#[tauri::command]
pub async fn category_list(database: Db<'_>) -> Result<Vec<Category>, AppError> {
    run(database, |conn| categories::list(conn)).await
}

#[tauri::command]
pub async fn category_get(database: Db<'_>, id: String) -> Result<Option<Category>, AppError> {
    run(database, move |conn| categories::get(conn, &id)).await
}

#[tauri::command]
pub async fn category_create(database: Db<'_>, category: Category) -> Result<(), AppError> {
    run(database, move |conn| categories::create(conn, &category)).await
}

#[tauri::command]
pub async fn category_update(database: Db<'_>, category: Category) -> Result<(), AppError> {
    run(database, move |conn| categories::update(conn, &category)).await
}

#[tauri::command]
pub async fn category_delete(database: Db<'_>, id: String) -> Result<(), AppError> {
    run(database, move |conn| categories::delete(conn, &id)).await
}

#[tauri::command]
pub async fn card_list(database: Db<'_>) -> Result<Vec<Card>, AppError> {
    run(database, |conn| cards::list(conn)).await
}

#[tauri::command]
pub async fn card_get(database: Db<'_>, id: String) -> Result<Option<Card>, AppError> {
    run(database, move |conn| cards::get(conn, &id)).await
}

#[tauri::command]
pub async fn card_create(database: Db<'_>, card: Card) -> Result<(), AppError> {
    run(database, move |conn| cards::create(conn, &card)).await
}

#[tauri::command]
pub async fn card_update(database: Db<'_>, card: Card) -> Result<(), AppError> {
    run(database, move |conn| cards::update(conn, &card)).await
}

#[tauri::command]
pub async fn card_delete(database: Db<'_>, id: String) -> Result<(), AppError> {
    run(database, move |conn| cards::delete(conn, &id)).await
}

#[tauri::command]
pub async fn transaction_list(
    database: Db<'_>,
    filter: TransactionFilter,
) -> Result<Vec<Transaction>, AppError> {
    run(database, move |conn| transactions::list(conn, &filter)).await
}

#[tauri::command]
pub async fn transaction_get(
    database: Db<'_>,
    id: String,
) -> Result<Option<Transaction>, AppError> {
    run(database, move |conn| transactions::get(conn, &id)).await
}

#[tauri::command]
pub async fn transaction_create(
    database: Db<'_>,
    transaction: Transaction,
) -> Result<(), AppError> {
    run(database, move |conn| {
        transactions::create(conn, &transaction)
    })
    .await
}

#[tauri::command]
pub async fn transaction_update(
    database: Db<'_>,
    transaction: Transaction,
) -> Result<(), AppError> {
    run(database, move |conn| {
        transactions::update(conn, &transaction)
    })
    .await
}

#[tauri::command]
pub async fn transaction_delete(database: Db<'_>, id: String) -> Result<(), AppError> {
    run(database, move |conn| transactions::delete(conn, &id)).await
}

#[tauri::command]
pub async fn transfer_get(
    database: Db<'_>,
    transfer_id: String,
) -> Result<Option<Vec<Transaction>>, AppError> {
    run(database, move |conn| transfers::get(conn, &transfer_id)).await
}

#[tauri::command]
pub async fn transfer_create(database: Db<'_>, legs: Vec<Transaction>) -> Result<(), AppError> {
    run(database, move |conn| transfers::create(conn, &legs)).await
}

#[tauri::command]
pub async fn transfer_update(database: Db<'_>, legs: Vec<Transaction>) -> Result<(), AppError> {
    run(database, move |conn| transfers::update(conn, &legs)).await
}

#[tauri::command]
pub async fn transfer_delete(database: Db<'_>, transfer_id: String) -> Result<(), AppError> {
    run(database, move |conn| transfers::delete(conn, &transfer_id)).await
}
