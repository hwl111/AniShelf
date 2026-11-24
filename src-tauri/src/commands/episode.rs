use tauri::State;

use crate::db::AppState;
use crate::error::AppError;
use crate::models::episode::{Episode, EpisodeInput, WatchRecord, WatchRecordInput};
use crate::services::episode_service;

#[tauri::command]
pub async fn list_episodes(
    state: State<'_, AppState>,
    anime_id: i64,
) -> Result<Vec<Episode>, AppError> {
    episode_service::list_episodes(&state.pool, anime_id).await
}

#[tauri::command]
pub async fn create_episode(
    state: State<'_, AppState>,
    anime_id: i64,
    input: EpisodeInput,
) -> Result<Episode, AppError> {
    episode_service::create_episode(&state.pool, anime_id, input).await
}

#[tauri::command]
pub async fn update_episode(
    state: State<'_, AppState>,
    id: i64,
    input: EpisodeInput,
) -> Result<Episode, AppError> {
    episode_service::update_episode(&state.pool, id, input).await
}

#[tauri::command]
pub async fn delete_episode(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    episode_service::delete_episode(&state.pool, id).await
}

#[tauri::command]
pub async fn set_episode_watched(
    state: State<'_, AppState>,
    id: i64,
    watched: bool,
    watched_at: Option<i64>,
    comment: Option<String>,
) -> Result<Episode, AppError> {
    episode_service::set_episode_watched(&state.pool, id, watched, watched_at, comment).await
}

#[tauri::command]
pub async fn list_watch_records(
    state: State<'_, AppState>,
    anime_id: i64,
) -> Result<Vec<WatchRecord>, AppError> {
    episode_service::list_watch_records(&state.pool, anime_id).await
}

#[tauri::command]
pub async fn add_watch_record(
    state: State<'_, AppState>,
    anime_id: i64,
    input: WatchRecordInput,
) -> Result<WatchRecord, AppError> {
    episode_service::add_watch_record(&state.pool, anime_id, input).await
}

#[tauri::command]
pub async fn update_watch_record(
    state: State<'_, AppState>,
    id: i64,
    watched_at: Option<i64>,
    comment: String,
) -> Result<WatchRecord, AppError> {
    episode_service::update_watch_record(&state.pool, id, watched_at, comment).await
}

#[tauri::command]
pub async fn delete_watch_record(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    episode_service::delete_watch_record(&state.pool, id).await
}
