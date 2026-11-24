use tauri::State;

use crate::db::{self, AppState, DatabaseStatus};
use crate::error::AppError;

#[tauri::command]
pub async fn get_database_status(state: State<'_, AppState>) -> Result<DatabaseStatus, AppError> {
    db::status(&state.pool, &state.paths.database_file).await
}
