use tauri::State;

use crate::db::AppState;
use crate::error::AppError;
use crate::services::search::{self, SearchResult};

#[tauri::command]
pub async fn search_all(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, AppError> {
    search::search(&state.pool, &query, 20).await
}
