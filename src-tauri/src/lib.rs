mod commands;
mod db;
mod error;
mod models;
mod repositories;
mod services;
mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = tauri::async_runtime::block_on(db::init(app.handle()))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::get_database_status,
            commands::search::search_all,
            commands::anime::list_anime,
            commands::anime::get_anime,
            commands::anime::create_anime,
            commands::anime::update_anime,
            commands::anime::delete_anime,
            commands::anime::set_anime_favorite,
            commands::anime::list_tags
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
