use tauri::State;

use crate::db::AppState;
use crate::error::AppError;
use crate::models::anime::{Anime, AnimeFilter, AnimeInput};
use crate::models::tag::Tag;
use crate::services::anime_service;

#[tauri::command]
pub async fn list_anime(
    state: State<'_, AppState>,
    filter: Option<AnimeFilter>,
) -> Result<Vec<Anime>, AppError> {
    anime_service::list(&state.pool, &filter.unwrap_or_default()).await
}

#[tauri::command]
pub async fn get_anime(state: State<'_, AppState>, id: i64) -> Result<Anime, AppError> {
    anime_service::get(&state.pool, id).await
}

#[tauri::command]
pub async fn create_anime(
    state: State<'_, AppState>,
    input: AnimeInput,
) -> Result<Anime, AppError> {
    anime_service::create(&state.pool, input).await
}

#[tauri::command]
pub async fn update_anime(
    state: State<'_, AppState>,
    id: i64,
    input: AnimeInput,
) -> Result<Anime, AppError> {
    anime_service::update(&state.pool, id, input).await
}

#[tauri::command]
pub async fn delete_anime(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    anime_service::delete(&state.pool, id).await
}

#[tauri::command]
pub async fn set_anime_favorite(
    state: State<'_, AppState>,
    id: i64,
    favorite: bool,
) -> Result<Anime, AppError> {
    anime_service::set_favorite(&state.pool, id, favorite).await
}

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, AppError> {
    anime_service::list_tags(&state.pool).await
}
