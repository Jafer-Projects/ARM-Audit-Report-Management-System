mod db;
mod commands;
mod import_cost;

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
            commands::add_task_review_note,
            commands::disposition_task_review_note,
            commands::find_invoice_duplicates,
            commands::import_historical_batch,
            commands::list_historical_import_batches,
            commands::list_task_review_notes,
            commands::list_workspace_record_versions,
            commands::list_records,
            commands::save_record,
            commands::list_events,
            commands::calculate_import_cost_preview,
            commands::attach_source_document,
            commands::list_source_documents,
            commands::read_source_document,
            commands::list_workspace_notifications,
            commands::mark_workspace_notification_read,
            commands::reset_demo_data
        ])
        .run(tauri::generate_context!())
        .expect("ARM desktop runtime failed");
}
