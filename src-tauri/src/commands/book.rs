use tauri::State;

use crate::db::AppState;
use crate::error::AppError;
use crate::models::book::{Book, BookFileStatus, BookFilter, BookImportResult, BookUpdateInput};
use crate::services::book_service;

#[tauri::command]
pub async fn list_books(
    state: State<'_, AppState>,
    filter: Option<BookFilter>,
) -> Result<Vec<Book>, AppError> {
    book_service::list(&state.pool, &state.paths, &filter.unwrap_or_default()).await
}

#[tauri::command]
pub async fn get_book(state: State<'_, AppState>, id: i64) -> Result<Book, AppError> {
    book_service::get(&state.pool, &state.paths, id).await
}

#[tauri::command]
pub async fn import_book(
    state: State<'_, AppState>,
    path: String,
    copy_to_library: bool,
) -> Result<BookImportResult, AppError> {
    book_service::import_book(&state.pool, &state.paths, path, copy_to_library).await
}

#[tauri::command]
pub async fn update_book(
    state: State<'_, AppState>,
    id: i64,
    input: BookUpdateInput,
) -> Result<Book, AppError> {
    book_service::update(&state.pool, &state.paths, id, input).await
}

#[tauri::command]
pub async fn delete_book(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    book_service::delete(&state.pool, &state.paths, id).await
}

#[tauri::command]
pub async fn save_generated_cover(
    state: State<'_, AppState>,
    id: i64,
    data_url: String,
) -> Result<Book, AppError> {
    book_service::save_generated_cover(&state.pool, &state.paths, id, &data_url).await
}

#[tauri::command]
pub async fn set_book_favorite(
    state: State<'_, AppState>,
    id: i64,
    favorite: bool,
) -> Result<Book, AppError> {
    book_service::set_favorite(&state.pool, &state.paths, id, favorite).await
}

#[tauri::command]
pub async fn check_book_file(
    state: State<'_, AppState>,
    id: i64,
) -> Result<BookFileStatus, AppError> {
    book_service::check_file(&state.pool, id).await
}

#[tauri::command]
pub async fn relocate_book(
    state: State<'_, AppState>,
    id: i64,
    new_path: String,
) -> Result<Book, AppError> {
    book_service::relocate(&state.pool, &state.paths, id, new_path).await
}
