use crate::state::AppState;
use gestion_core::db::personnel_repo;
use gestion_core::domain::PersonnelRole;
use gestion_core::dto::{ImportPersonnelResult, PersonnelWithQuotaDto, UpsertPersonnelPayload};
use gestion_core::error::SerializableCoreError;
use tauri::State;

#[tauri::command]
pub fn list_personnel(
    state: State<'_, AppState>,
    role_filter: Option<PersonnelRole>,
    active_only: bool,
) -> Result<Vec<PersonnelWithQuotaDto>, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    personnel_repo::list_personnel(db.connection(), role_filter, active_only)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn search_dancer_quick(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<PersonnelWithQuotaDto>, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    personnel_repo::search_dancer_quick(db.connection(), &query)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn upsert_personnel(
    state: State<'_, AppState>,
    payload: UpsertPersonnelPayload,
) -> Result<PersonnelWithQuotaDto, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    personnel_repo::upsert_personnel(db.connection(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn import_personnel_batch(
    state: State<'_, AppState>,
    items: Vec<UpsertPersonnelPayload>,
) -> Result<ImportPersonnelResult, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    personnel_repo::import_personnel_batch(db.connection_mut(), items)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn delete_personnel(
    state: State<'_, AppState>,
    personnel_id: String,
) -> Result<(), SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    personnel_repo::delete_personnel(db.connection_mut(), &personnel_id)
        .map_err(|e| SerializableCoreError::from(&e))
}

