use crate::state::AppState;
use gestion_core::{
    create_physiotherapy_session as core_create_physio,
    list_attendance_for_date as core_list_attendance,
    list_physiotherapy_sessions as core_list_physio,
    record_attendance as core_record_attendance,
    AttendanceLogDto, CreatePhysioSessionPayload, PhysiotherapySessionDto,
    RecordAttendancePayload, SerializableCoreError,
};
use tauri::State;

#[tauri::command]
pub fn list_attendance_for_date(
    state: State<'_, AppState>,
    schedule_date: String,
) -> Result<Vec<AttendanceLogDto>, SerializableCoreError> {
    let db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    core_list_attendance(db_guard.connection(), &schedule_date)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn record_attendance(
    state: State<'_, AppState>,
    payload: RecordAttendancePayload,
) -> Result<AttendanceLogDto, SerializableCoreError> {
    let mut db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    core_record_attendance(db_guard.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn create_physiotherapy_session(
    state: State<'_, AppState>,
    payload: CreatePhysioSessionPayload,
) -> Result<PhysiotherapySessionDto, SerializableCoreError> {
    let mut db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    core_create_physio(db_guard.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn list_physiotherapy_sessions(
    state: State<'_, AppState>,
    date_filter: Option<String>,
    personnel_filter: Option<String>,
) -> Result<Vec<PhysiotherapySessionDto>, SerializableCoreError> {
    let db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    core_list_physio(
        db_guard.connection(),
        date_filter.as_deref(),
        personnel_filter.as_deref(),
    )
    .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn get_agent_monthly_attendance_sheet(
    state: State<'_, AppState>,
    personnel_id: String,
    year: i32,
    month: u32,
) -> Result<gestion_core::MonthlyAttendanceSheetDto, SerializableCoreError> {
    let db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    gestion_core::get_agent_monthly_attendance_sheet(db_guard.connection(), &personnel_id, year, month)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn list_monthly_agents_summary(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<Vec<gestion_core::MonthlyAgentSummaryDto>, SerializableCoreError> {
    let db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    gestion_core::list_monthly_agents_summary(db_guard.connection(), year, month)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn get_monthly_bop_matrix(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<gestion_core::MonthlyBopMatrixDto, SerializableCoreError> {
    let db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    gestion_core::get_monthly_bop_matrix(db_guard.connection(), year, month)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn update_bop_matrix_cell(
    state: State<'_, AppState>,
    payload: gestion_core::UpdateBopMatrixCellPayload,
) -> Result<(), SerializableCoreError> {
    let mut db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    gestion_core::update_bop_matrix_cell(
        db_guard.connection_mut(),
        &payload.date,
        &payload.personnel_id,
        &payload.code,
        payload.remarks.as_deref(),
    )
    .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn set_day_schedule_annotation(
    state: State<'_, AppState>,
    payload: gestion_core::SetDayScheduleAnnotationPayload,
) -> Result<(), SerializableCoreError> {
    let mut db_guard = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error adquiriendo candado de base de datos".to_string(),
    })?;

    gestion_core::set_day_schedule_annotation(
        db_guard.connection_mut(),
        &payload.date,
        payload.annotation.as_deref(),
    )
    .map_err(|e| SerializableCoreError::from(&e))
}

