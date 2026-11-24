use tauri::State;

use crate::db::AppState;
use crate::error::AppError;
use crate::models::anime::Anime;
use crate::services::bangumi_service::{
    self, BangumiConnectionStatus, BangumiRefreshPreview, BangumiSearchResult, BangumiTokenStatus,
};

#[tauri::command]
pub async fn search_bangumi(
    state: State<'_, AppState>,
    keyword: String,
) -> Result<Vec<BangumiSearchResult>, AppError> {
    bangumi_service::search(&state.pool, &keyword).await
}

#[tauri::command]
pub async fn import_bangumi_subject(
    state: State<'_, AppState>,
    subject_id: i64,
) -> Result<Anime, AppError> {
    bangumi_service::import_subject(&state.pool, subject_id).await
}

#[tauri::command]
pub async fn preview_bangumi_refresh(
    state: State<'_, AppState>,
    anime_id: i64,
) -> Result<BangumiRefreshPreview, AppError> {
    bangumi_service::preview_refresh(&state.pool, anime_id).await
}

#[tauri::command]
pub async fn apply_bangumi_refresh(
    state: State<'_, AppState>,
    anime_id: i64,
) -> Result<Anime, AppError> {
    bangumi_service::apply_refresh(&state.pool, anime_id).await
}

#[tauri::command]
pub async fn get_bangumi_token_status(
    state: State<'_, AppState>,
) -> Result<BangumiTokenStatus, AppError> {
    bangumi_service::token_status(&state.pool).await
}

#[tauri::command]
pub async fn set_bangumi_token(
    state: State<'_, AppState>,
    token: Option<String>,
) -> Result<BangumiTokenStatus, AppError> {
    bangumi_service::save_token(&state.pool, token).await
}

#[tauri::command]
pub async fn check_bangumi_connection(
    state: State<'_, AppState>,
) -> Result<BangumiConnectionStatus, AppError> {
    bangumi_service::test_connection(&state.pool).await
}
