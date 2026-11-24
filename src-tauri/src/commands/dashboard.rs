use tauri::State;

use crate::db::AppState;
use crate::error::AppError;
use crate::models::dashboard::DashboardData;
use crate::services::dashboard_service;

#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardData, AppError> {
    dashboard_service::get(&state.pool, &state.paths).await
}
