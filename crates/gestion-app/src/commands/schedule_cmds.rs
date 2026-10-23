use crate::state::AppState;
use gestion_core::db::schedule_repo;
use gestion_core::domain::WorkRegime;
use gestion_core::dto::{
    DailyScheduleSummaryDto, DailyScheduleWithAuditDto, UpdateScheduleRulesPayload,
    UpsertMorningClassPayload, UpsertRehearsalBlockPayload,
};
use gestion_core::error::SerializableCoreError;
use tauri::State;

#[tauri::command]
pub fn list_daily_schedules(
    state: State<'_, AppState>,
) -> Result<Vec<DailyScheduleSummaryDto>, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    schedule_repo::list_daily_schedules(db.connection())
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn get_or_create_daily_schedule(
    state: State<'_, AppState>,
    schedule_date: String,
) -> Result<DailyScheduleWithAuditDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    schedule_repo::get_or_create_daily_schedule(db.connection_mut(), &schedule_date)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn set_schedule_regime(
    state: State<'_, AppState>,
    schedule_id: String,
    regime: WorkRegime,
) -> Result<DailyScheduleWithAuditDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    schedule_repo::set_schedule_regime(db.connection_mut(), &schedule_id, regime)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn update_schedule_rules(
    state: State<'_, AppState>,
    payload: UpdateScheduleRulesPayload,
) -> Result<DailyScheduleWithAuditDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    schedule_repo::update_schedule_rules(db.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn upsert_morning_class(
    state: State<'_, AppState>,
    payload: UpsertMorningClassPayload,
) -> Result<DailyScheduleWithAuditDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    schedule_repo::upsert_morning_class(db.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn delete_morning_class(
    state: State<'_, AppState>,
    class_id: String,
) -> Result<Option<DailyScheduleWithAuditDto>, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    schedule_repo::delete_morning_class(db.connection_mut(), &class_id)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn upsert_rehearsal_block(
    state: State<'_, AppState>,
    payload: UpsertRehearsalBlockPayload,
) -> Result<DailyScheduleWithAuditDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    schedule_repo::upsert_rehearsal_block(db.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn delete_rehearsal_block(
    state: State<'_, AppState>,
    block_id: String,
) -> Result<DailyScheduleWithAuditDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    schedule_repo::delete_rehearsal_block(db.connection_mut(), &block_id)
        .map_err(|e| SerializableCoreError::from(&e))
}
