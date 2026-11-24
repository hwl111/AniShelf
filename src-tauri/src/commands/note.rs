use tauri::State;

use crate::db::AppState;
use crate::error::AppError;
use crate::models::note::{
    Note, NoteDeleteImpact, NoteFilter, NoteInput, NoteLinkCandidate, NoteRelationCandidate,
    NoteRelationImpact, NoteSummary,
};
use crate::services::note_service;

#[tauri::command]
pub async fn list_notes(
    state: State<'_, AppState>,
    filter: Option<NoteFilter>,
) -> Result<Vec<NoteSummary>, AppError> {
    note_service::list(&state.pool, &filter.unwrap_or_default()).await
}

#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, id: i64) -> Result<Note, AppError> {
    note_service::get(&state.pool, id).await
}

#[tauri::command]
pub async fn create_note(state: State<'_, AppState>, input: NoteInput) -> Result<Note, AppError> {
    note_service::create(&state.pool, input).await
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, AppState>,
    id: i64,
    input: NoteInput,
) -> Result<Note, AppError> {
    note_service::update(&state.pool, id, input).await
}

#[tauri::command]
pub async fn delete_note(state: State<'_, AppState>, id: i64, force: bool) -> Result<(), AppError> {
    note_service::delete(&state.pool, id, force).await
}

#[tauri::command]
pub async fn set_note_favorite(
    state: State<'_, AppState>,
    id: i64,
    favorite: bool,
) -> Result<Note, AppError> {
    note_service::set_favorite(&state.pool, id, favorite).await
}

#[tauri::command]
pub async fn get_note_delete_impact(
    state: State<'_, AppState>,
    id: i64,
) -> Result<NoteDeleteImpact, AppError> {
    note_service::delete_impact(&state.pool, id).await
}

#[tauri::command]
pub async fn get_note_relation_impact(
    state: State<'_, AppState>,
    target_type: String,
    target_id: i64,
) -> Result<NoteRelationImpact, AppError> {
    note_service::relation_impact(&state.pool, &target_type, target_id).await
}

#[tauri::command]
pub async fn list_note_link_candidates(
    state: State<'_, AppState>,
    note_id: i64,
    query: Option<String>,
) -> Result<Vec<NoteLinkCandidate>, AppError> {
    note_service::list_link_candidates(&state.pool, note_id, &query.unwrap_or_default()).await
}

#[tauri::command]
pub async fn add_note_link(
    state: State<'_, AppState>,
    source_note_id: i64,
    target_note_id: i64,
) -> Result<(), AppError> {
    note_service::add_link(&state.pool, source_note_id, target_note_id).await
}

#[tauri::command]
pub async fn remove_note_link(
    state: State<'_, AppState>,
    source_note_id: i64,
    target_note_id: i64,
) -> Result<(), AppError> {
    note_service::remove_link(&state.pool, source_note_id, target_note_id).await
}

#[tauri::command]
pub async fn list_note_relation_candidates(
    state: State<'_, AppState>,
    target_type: String,
    query: Option<String>,
) -> Result<Vec<NoteRelationCandidate>, AppError> {
    note_service::list_relation_candidates(&state.pool, &target_type, &query.unwrap_or_default())
        .await
}

#[tauri::command]
pub async fn add_note_relation(
    state: State<'_, AppState>,
    note_id: i64,
    target_type: String,
    target_id: i64,
) -> Result<(), AppError> {
    note_service::add_relation(&state.pool, note_id, &target_type, target_id).await
}

#[tauri::command]
pub async fn remove_note_relation(
    state: State<'_, AppState>,
    note_id: i64,
    relation_id: i64,
) -> Result<(), AppError> {
    note_service::remove_relation(&state.pool, note_id, relation_id).await
}
