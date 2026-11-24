use tauri::State;
use tauri_plugin_opener::OpenerExt;

use crate::db::AppState;
use crate::error::AppError;
use crate::models::reader::{
    Bookmark, BookmarkInput, ProgressInput, ReaderSettings, ReadingProgress, TextBookInfo,
    TextChunk,
};
use crate::services::reader_service;

#[tauri::command]
pub async fn read_book_bytes(
    state: State<'_, AppState>,
    book_id: i64,
) -> Result<tauri::ipc::Response, AppError> {
    let bytes = reader_service::read_book_bytes(&state.pool, book_id).await?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
pub async fn get_text_book_info(
    state: State<'_, AppState>,
    book_id: i64,
) -> Result<TextBookInfo, AppError> {
    reader_service::get_text_book_info(&state.pool, book_id).await
}

#[tauri::command]
pub async fn read_text_chunk(
    state: State<'_, AppState>,
    book_id: i64,
    offset: i64,
    max_bytes: i64,
    encoding: String,
) -> Result<TextChunk, AppError> {
    reader_service::read_text_chunk(&state.pool, book_id, offset, max_bytes, &encoding).await
}

#[tauri::command]
pub async fn get_reading_progress(
    state: State<'_, AppState>,
    book_id: i64,
) -> Result<Option<ReadingProgress>, AppError> {
    reader_service::get_progress(&state.pool, book_id).await
}

#[tauri::command]
pub async fn save_reading_progress(
    state: State<'_, AppState>,
    book_id: i64,
    input: ProgressInput,
) -> Result<ReadingProgress, AppError> {
    reader_service::save_progress(&state.pool, book_id, input).await
}

#[tauri::command]
pub async fn list_bookmarks(
    state: State<'_, AppState>,
    book_id: i64,
) -> Result<Vec<Bookmark>, AppError> {
    reader_service::list_bookmarks(&state.pool, book_id).await
}

#[tauri::command]
pub async fn create_bookmark(
    state: State<'_, AppState>,
    book_id: i64,
    input: BookmarkInput,
) -> Result<Bookmark, AppError> {
    reader_service::create_bookmark(&state.pool, book_id, input).await
}

#[tauri::command]
pub async fn delete_bookmark(state: State<'_, AppState>, bookmark_id: i64) -> Result<(), AppError> {
    reader_service::delete_bookmark(&state.pool, bookmark_id).await
}

#[tauri::command]
pub async fn get_reader_settings(
    state: State<'_, AppState>,
    scope: String,
) -> Result<Option<ReaderSettings>, AppError> {
    reader_service::get_settings(&state.pool, &scope).await
}

#[tauri::command]
pub async fn save_reader_settings(
    state: State<'_, AppState>,
    scope: String,
    preferences_json: String,
) -> Result<ReaderSettings, AppError> {
    reader_service::save_settings(&state.pool, &scope, &preferences_json).await
}

#[tauri::command]
pub async fn start_reading_session(
    state: State<'_, AppState>,
    book_id: i64,
) -> Result<i64, AppError> {
    reader_service::start_session(&state.pool, book_id).await
}

#[tauri::command]
pub async fn end_reading_session(
    state: State<'_, AppState>,
    session_id: i64,
    end_location_json: Option<String>,
) -> Result<(), AppError> {
    reader_service::end_session(&state.pool, session_id, end_location_json).await
}

#[tauri::command]
pub async fn open_external_url(app: tauri::AppHandle, url: String) -> Result<(), AppError> {
    let parsed = reader_service::validate_external_url(&url)?;
    app.opener()
        .open_url(parsed.to_string(), None::<&str>)
        .map_err(|error| AppError::External(format!("无法打开外部链接：{error}")))
}
