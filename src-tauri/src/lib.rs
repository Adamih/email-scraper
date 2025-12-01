// Prevent additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod email_extractor;
mod models;

use models::AppState;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize database
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            // Create app data directory if it doesn't exist
            std::fs::create_dir_all(&app_dir).expect("failed to create app data directory");

            let db_path = app_dir.join("email_scraper.db");

            // Initialize database synchronously by blocking on the async operation
            let db = tauri::async_runtime::block_on(async {
                db::init_db(db_path.as_path())
                    .await
                    .expect("failed to initialize database")
            });

            // Create app state
            let state = AppState { db };

            // Manage state
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::search::execute_google_search,
            commands::search::get_searches,
            commands::search::get_domains_for_search,
            commands::search::delete_search,
            commands::crawler::start_crawler,
            commands::crawler::start_all_crawlers,
            commands::crawler::delete_domain,
            commands::crawler::delete_all_pending_domains,
            commands::crawler::get_pending_domains,
            commands::emails::get_emails,
            commands::emails::get_emails_by_query,
            commands::emails::get_emails_by_domain,
            commands::emails::get_domain_statistics,
            commands::emails::export_to_csv,
            commands::config::get_config,
            commands::config::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
