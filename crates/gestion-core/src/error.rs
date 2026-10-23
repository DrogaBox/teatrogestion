use crate::services::gaynor_parser::GaynorParseError;
use serde::Serialize;
use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Error de persistencia SQLite: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Fallo de integridad en migración #{version} ({name}): checksum esperado {expected}, encontrado {found}")]
    MigrationChecksumMismatch {
        version: u32,
        name: String,
        expected: String,
        found: String,
    },

    #[error("Error aplicando migración #{version} ({name}): {reason}")]
    MigrationExecutionFailed {
        version: u32,
        name: String,
        reason: String,
    },

    #[error("Error en código de fábrica Gaynor Minden: {0}")]
    GaynorParse(#[from] GaynorParseError),

    #[error("Stock libre insuficiente para '{sku_code}': solicitado {requested}, disponible libre {available_free}")]
    InsufficientFreeStock {
        sku_code: String,
        requested: u32,
        available_free: u32,
    },

    #[error("Cupo anual excedido para {dancer_name} ({personnel_id}) en rubro {quota_bucket}: saldo restante {remaining}, solicitado {requested}. Requiere autorización de excepción.")]
    AnnualQuotaExceeded {
        personnel_id: String,
        dancer_name: String,
        quota_bucket: String,
        remaining: i32,
        requested: u32,
    },

    #[error("Las entregas que exceden el cupo anual requieren un motivo de excepción documentado (mínimo 5 caracteres).")]
    QuotaExceptionReasonRequired,

    #[error("Se detectaron {conflict_count} cruces o alertas reglamentarias en el Parte Diario: {summary}")]
    ScheduleConflictDetected {
        conflict_count: usize,
        summary: String,
    },

    #[error("Rango horario inválido: '{start}' a '{end}'")]
    InvalidTimeRange { start: String, end: String },

    #[error("Violación del estatuto de Clase Técnica Matutina (09:00 a 10:15 h): {0}")]
    MorningClassInvariantViolation(String),

    #[error("Entidad '{entity}' con identificador '{id}' no encontrada")]
    EntityNotFound { entity: &'static str, id: String },

    #[error("Error de serialización JSON: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Error de entrada/salida: {0}")]
    Io(#[from] std::io::Error),

    #[error("Violación de integridad operativa: {0}")]
    IntegrityViolation(String),

    #[error("Error de validación: {0}")]
    ValidationFailed(String),
}

/// Representación serializable de error para transporte sobre Tauri IPC
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializableCoreError {
    pub code: &'static str,
    pub message: String,
}

impl From<&CoreError> for SerializableCoreError {
    fn from(err: &CoreError) -> Self {
        let code = match err {
            CoreError::Database(_) => "DB_ERROR",
            CoreError::MigrationChecksumMismatch { .. } => "MIGRATION_CHECKSUM_MISMATCH",
            CoreError::MigrationExecutionFailed { .. } => "MIGRATION_FAILED",
            CoreError::GaynorParse(_) => "GAYNOR_PARSE_ERROR",
            CoreError::InsufficientFreeStock { .. } => "INSUFFICIENT_FREE_STOCK",
            CoreError::AnnualQuotaExceeded { .. } => "ANNUAL_QUOTA_EXCEEDED",
            CoreError::QuotaExceptionReasonRequired => "QUOTA_EXCEPTION_REASON_REQUIRED",
            CoreError::ScheduleConflictDetected { .. } => "SCHEDULE_CONFLICT_DETECTED",
            CoreError::InvalidTimeRange { .. } => "INVALID_TIME_RANGE",
            CoreError::MorningClassInvariantViolation(_) => "MORNING_CLASS_INVARIANT_VIOLATION",
            CoreError::EntityNotFound { .. } => "ENTITY_NOT_FOUND",
            CoreError::Serialization(_) => "SERIALIZATION_ERROR",
            CoreError::Io(_) => "IO_ERROR",
            CoreError::IntegrityViolation(_) => "INTEGRITY_VIOLATION",
            CoreError::ValidationFailed(_) => "VALIDATION_FAILED",
        };
        Self {
            code,
            message: err.to_string(),
        }
    }
}
