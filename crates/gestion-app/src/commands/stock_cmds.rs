use crate::state::AppState;
use gestion_core::db::stock_repo;
use gestion_core::domain::GaynorParsedSpec;
use gestion_core::dto::{
    ExecuteVoucherPayload, StockFilterPayload, StockItemAvailabilityDto, UpsertStockItemPayload,
    VoucherReceiptDto,
};
use gestion_core::error::SerializableCoreError;
use gestion_core::services::gaynor_parser::parse_gaynor_code;
use tauri::State;

#[tauri::command]
pub fn parse_gaynor_code_preview(
    raw_code: String,
) -> Result<GaynorParsedSpec, SerializableCoreError> {
    parse_gaynor_code(&raw_code).map_err(|e| SerializableCoreError {
        code: "GAYNOR_PARSE_ERROR",
        message: e.to_string(),
    })
}

#[tauri::command]
pub fn list_stock_catalog(
    state: State<'_, AppState>,
    filter: StockFilterPayload,
) -> Result<Vec<StockItemAvailabilityDto>, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    stock_repo::list_stock_catalog(db.connection(), filter)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn upsert_stock_item(
    state: State<'_, AppState>,
    payload: UpsertStockItemPayload,
) -> Result<StockItemAvailabilityDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    stock_repo::upsert_stock_item(db.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn execute_voucher_delivery(
    state: State<'_, AppState>,
    payload: ExecuteVoucherPayload,
) -> Result<VoucherReceiptDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Fallo al adquirir bloqueo de base de datos".to_string(),
    })?;
    stock_repo::execute_voucher_delivery(db.connection_mut(), payload)
        .map_err(|e| SerializableCoreError::from(&e))
}
