pub mod commands;
pub mod database;
pub mod db;
pub mod dto;
pub mod error;
pub mod names;
pub mod store;

use std::sync::Arc;

use tauri::Manager;

use crate::database::Database;

/// Registers every IPC command. The `Arc<Database>` state they use is added by `run` (or by a
/// test) once the application exists.
pub fn register_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        commands::database_status,
        commands::account_list,
        commands::account_get,
        commands::account_create,
        commands::account_update,
        commands::account_delete,
        commands::category_list,
        commands::category_get,
        commands::category_create,
        commands::category_update,
        commands::category_delete,
        commands::card_list,
        commands::card_get,
        commands::card_create,
        commands::card_update,
        commands::card_delete,
        commands::transaction_list,
        commands::transaction_get,
        commands::transaction_create,
        commands::transaction_update,
        commands::transaction_delete,
        commands::transfer_get,
        commands::transfer_create,
        commands::transfer_update,
        commands::transfer_delete,
    ])
}

/// Development builds use their own file so running `tauri dev` can never touch real data.
const DATABASE_FILE: &str = if cfg!(debug_assertions) {
    "planalysis-dev.db"
} else {
    "planalysis.db"
};

/// Builds and runs the desktop application.
pub fn run() {
    register_commands(tauri::Builder::default())
        // Must be registered first: a second launch focuses the running window instead of
        // opening another process on the same database.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let directory = app.path().app_local_data_dir()?;
            app.manage(Arc::new(Database::open(&directory.join(DATABASE_FILE))));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Planalysis application");
}
