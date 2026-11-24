use tauri::State;

use crate::db::AppState;
use crate::error::AppError;
use crate::models::backup::{AppInfo, BackupInfo, RestoreResult};
use crate::services::backup_service;

#[tauri::command]
pub async fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupInfo>, AppError> {
    backup_service::list_backups(&state.paths).await
}

#[tauri::command]
pub async fn create_backup(
    state: State<'_, AppState>,
    destination: Option<String>,
) -> Result<BackupInfo, AppError> {
    backup_service::create_backup(&state.pool, &state.paths, destination).await
}

#[tauri::command]
pub async fn stage_restore(
    state: State<'_, AppState>,
    source_path: String,
) -> Result<RestoreResult, AppError> {
    backup_service::stage_restore(&state.pool, &state.paths, &source_path).await
}

#[tauri::command]
pub async fn cancel_pending_restore(state: State<'_, AppState>) -> Result<bool, AppError> {
    backup_service::cancel_pending_restore(&state.paths).await
}

#[tauri::command]
pub async fn get_app_info(state: State<'_, AppState>) -> Result<AppInfo, AppError> {
    backup_service::app_info(&state.pool, &state.paths).await
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) -> Result<(), AppError> {
    log::info!("用户请求重启应用以应用数据库恢复");
    app.restart()
}
