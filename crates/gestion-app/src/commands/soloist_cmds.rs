use crate::state::AppState;
use gestion_core::{
    delete_soloist_certification as core_delete_soloist,
    generate_soloist_certification_from_work_cast as core_generate_soloist,
    get_soloist_certification as core_get_soloist,
    list_soloist_certifications as core_list_soloist,
    save_soloist_certification as core_save_soloist,
    SaveSoloistCertificationPayload, SerializableCoreError, SoloistCertificationDto,
};
use tauri::State;

#[tauri::command]
pub fn list_soloist_certifications(
    state: State<'_, AppState>,
) -> Result<Vec<SoloistCertificationDto>, SerializableCoreError> {
    let db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    core_list_soloist(db_guard.connection())
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn get_soloist_certification(
    state: State<'_, AppState>,
    id: String,
) -> Result<SoloistCertificationDto, SerializableCoreError> {
    let db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    core_get_soloist(db_guard.connection(), &id)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn save_soloist_certification(
    state: State<'_, AppState>,
    payload: SaveSoloistCertificationPayload,
) -> Result<SoloistCertificationDto, SerializableCoreError> {
    let mut db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    core_save_soloist(db_guard.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn delete_soloist_certification(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), SerializableCoreError> {
    let mut db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    core_delete_soloist(db_guard.connection_mut(), &id)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn generate_soloist_certification_from_work_cast(
    state: State<'_, AppState>,
    work_id: String,
    dates_label: String,
    performances_per_cast: i32,
) -> Result<SoloistCertificationDto, SerializableCoreError> {
    let mut db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    core_generate_soloist(
        db_guard.connection_mut(),
        &work_id,
        &dates_label,
        performances_per_cast,
    )
    .map_err(|e| SerializableCoreError::from(&e))
}
