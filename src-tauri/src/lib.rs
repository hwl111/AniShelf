mod commands;
mod db;
mod error;
mod integrations;
mod models;
mod repositories;
mod services;
mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            commands::anime::list_tags,
            commands::episode::list_episodes,
            commands::episode::create_episode,
            commands::episode::update_episode,
            commands::episode::delete_episode,
            commands::episode::set_episode_watched,
            commands::episode::list_watch_records,
            commands::episode::add_watch_record,
            commands::episode::update_watch_record,
            commands::episode::delete_watch_record,
            commands::bangumi::search_bangumi,
            commands::bangumi::import_bangumi_subject,
            commands::bangumi::preview_bangumi_refresh,
            commands::bangumi::apply_bangumi_refresh,
            commands::bangumi::get_bangumi_token_status,
            commands::bangumi::set_bangumi_token,
            commands::bangumi::check_bangumi_connection,
            commands::book::list_books,
            commands::book::get_book,
            commands::book::import_book,
            commands::book::update_book,
            commands::book::delete_book,
            commands::book::set_book_favorite,
            commands::book::check_book_file,
            commands::book::relocate_book,
            commands::reader::read_book_bytes,
            commands::reader::get_reading_progress,
            commands::reader::save_reading_progress,
            commands::reader::list_bookmarks,
            commands::reader::create_bookmark,
            commands::reader::delete_bookmark,
            commands::reader::get_reader_settings,
            commands::reader::save_reader_settings,
            commands::reader::start_reading_session,
            commands::reader::end_reading_session,
            commands::reader::open_external_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
