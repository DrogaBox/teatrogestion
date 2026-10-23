use crate::state::AppState;
use gestion_core::db::works_repo;
use gestion_core::dto::{
    UpsertWorkCastingPlanPayload, UpsertWorkPassagePayload, UpsertWorkPayload,
    WorkCastingPlanDto, WorkPassageDto, WorkWithPassagesAndCastsDto,
};
use gestion_core::error::SerializableCoreError;
use tauri::State;

#[tauri::command]
pub fn list_works_with_casts(
    state: State<'_, AppState>,
) -> Result<Vec<WorkWithPassagesAndCastsDto>, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    works_repo::list_works_with_casts(db.connection())
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn upsert_work(
    state: State<'_, AppState>,
    payload: UpsertWorkPayload,
) -> Result<WorkWithPassagesAndCastsDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    works_repo::upsert_work(db.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn delete_work(
    state: State<'_, AppState>,
    work_id: String,
) -> Result<(), SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    works_repo::delete_work(db.connection_mut(), &work_id)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn upsert_work_passage(
    state: State<'_, AppState>,
    payload: UpsertWorkPassagePayload,
) -> Result<WorkPassageDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    works_repo::upsert_work_passage(db.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn delete_work_passage(
    state: State<'_, AppState>,
    passage_id: String,
) -> Result<(), SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    works_repo::delete_work_passage(db.connection_mut(), &passage_id)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn upsert_work_casting_plan(
    state: State<'_, AppState>,
    payload: UpsertWorkCastingPlanPayload,
) -> Result<WorkCastingPlanDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    works_repo::upsert_work_casting_plan(db.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn delete_work_casting_plan(
    state: State<'_, AppState>,
    plan_id: String,
) -> Result<(), SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    works_repo::delete_work_casting_plan(db.connection_mut(), &plan_id)
        .map_err(|e| SerializableCoreError::from(&e))
}
