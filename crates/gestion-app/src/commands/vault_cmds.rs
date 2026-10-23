use crate::state::AppState;
use gestion_core::dto::{
    AdminResetVaultPasswordPayload, CreateInspectionDocumentPayload, InspectionVaultDocumentDto,
    InspectionVaultFilterPayload, SetVaultPasswordPayload, VaultMonthlyTotalsDto,
    VaultSecurityStatusDto, VerifyVaultPasswordPayload,
};
use gestion_core::error::SerializableCoreError;
use tauri::State;

#[tauri::command]
pub fn list_vault_documents(
    state: State<AppState>,
    filter: InspectionVaultFilterPayload,
) -> Result<Vec<InspectionVaultDocumentDto>, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "MUTEX_LOCK_FAILED",
        message: "No se pudo adquirir el bloqueo de base de datos".into(),
    })?;

    gestion_core::db::list_vault_documents(db.connection(), filter).map_err(|e| (&e).into())
}

#[tauri::command]
pub fn create_vault_document(
    state: State<AppState>,
    payload: CreateInspectionDocumentPayload,
) -> Result<InspectionVaultDocumentDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "MUTEX_LOCK_FAILED",
        message: "No se pudo adquirir el bloqueo de base de datos".into(),
    })?;

    gestion_core::db::create_vault_document(db.connection_mut(), payload).map_err(|e| (&e).into())
}

#[tauri::command]
pub fn delete_vault_document(
    state: State<AppState>,
    id: String,
) -> Result<(), SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "MUTEX_LOCK_FAILED",
        message: "No se pudo adquirir el bloqueo de base de datos".into(),
    })?;

    gestion_core::db::delete_vault_document(db.connection_mut(), &id).map_err(|e| (&e).into())
}

#[tauri::command]
pub fn get_vault_monthly_totals(
    state: State<AppState>,
    month: u32,
    year: i32,
) -> Result<VaultMonthlyTotalsDto, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "MUTEX_LOCK_FAILED",
        message: "No se pudo adquirir el bloqueo de base de datos".into(),
    })?;

    gestion_core::db::get_vault_monthly_totals(db.connection(), month, year).map_err(|e| (&e).into())
}

#[tauri::command]
pub fn get_vault_security_status(
    state: State<AppState>,
) -> Result<VaultSecurityStatusDto, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "MUTEX_LOCK_FAILED",
        message: "No se pudo adquirir el bloqueo de base de datos".into(),
    })?;

    gestion_core::db::get_vault_security_status(db.connection()).map_err(|e| (&e).into())
}

#[tauri::command]
pub fn verify_vault_password(
    state: State<AppState>,
    payload: VerifyVaultPasswordPayload,
) -> Result<bool, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "MUTEX_LOCK_FAILED",
        message: "No se pudo adquirir el bloqueo de base de datos".into(),
    })?;

    gestion_core::db::verify_vault_password(db.connection(), &payload.password).map_err(|e| (&e).into())
}

#[tauri::command]
pub fn set_vault_password(
    state: State<AppState>,
    payload: SetVaultPasswordPayload,
) -> Result<(), SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "MUTEX_LOCK_FAILED",
        message: "No se pudo adquirir el bloqueo de base de datos".into(),
    })?;

    gestion_core::db::set_vault_password(
        db.connection_mut(),
        payload.old_password.as_deref(),
        &payload.new_password,
        payload.lock_timeout_minutes,
    )
    .map_err(|e| (&e).into())
}

#[tauri::command]
pub fn admin_reset_vault_password(
    state: State<AppState>,
    payload: AdminResetVaultPasswordPayload,
) -> Result<(), SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "MUTEX_LOCK_FAILED",
        message: "No se pudo adquirir el bloqueo de base de datos".into(),
    })?;

    gestion_core::db::admin_reset_vault_password(db.connection_mut(), &payload.new_password)
        .map_err(|e| (&e).into())
}
