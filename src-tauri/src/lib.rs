mod db;
mod commands;
mod import_cost;
mod sync;
mod credential_vault;

use db::Database;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState { pub database: Mutex<Database> }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().setup(|app| {
        let app_data = app.path().app_data_dir()?;
        std::fs::create_dir_all(&app_data)?;
        // Only development builds create and seed the synthetic workspace database.
        // Release builds do not open, migrate or seed the demo database.
        if cfg!(debug_assertions) {
            let database = Database::open(app_data.join("arm-demo.sqlite3"))
                .map_err(std::io::Error::other)?;
            app.manage(AppState { database: Mutex::new(database) });
        }
        Ok(())
    });

    // Production binaries expose only native credential-vault operations. Demo and
    // prototype synchronization commands are not registered in release builds.
    let builder = if cfg!(debug_assertions) {
        builder.invoke_handler(tauri::generate_handler![
            credential_vault::store_cloud_session,
            credential_vault::load_cloud_session,
            credential_vault::clear_cloud_session,
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
            commands::respond_task_review_note,
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
            commands::reset_demo_data,
            sync::sync_configure,
            sync::sync_outbox_batch,
            sync::sync_ack_outbox,
            sync::sync_apply_pull,
            sync::sync_status
        ])
    } else {
        builder.invoke_handler(tauri::generate_handler![
            credential_vault::store_cloud_session,
            credential_vault::load_cloud_session,
            credential_vault::clear_cloud_session
        ])
    };

    builder.run(tauri::generate_context!()).expect("ARM desktop runtime failed");
}
