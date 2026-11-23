mod db;
mod commands;

use db::Database;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub database: Mutex<Database>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data)?;
            let database = Database::open(app_data.join("arm-demo.sqlite3"))
                .map_err(std::io::Error::other)?;
            app.manage(AppState {
                database: Mutex::new(database),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::initialize_workspace,
            commands::list_clients,
            commands::save_client,
            commands::find_client_duplicates,
            commands::archive_client,
            commands::list_client_changes,
            commands::activate_client,
            commands::restore_client,
            commands::list_notes,
            commands::save_note,
            commands::set_demo_persona,
            commands::find_invoice_duplicates,
            commands::list_records,
            commands::save_record,
            commands::list_events,
            commands::reset_demo_data
        ])
        .run(tauri::generate_context!())
        .expect("ARM desktop runtime failed");
}
