use crate::domain::{
    ActivityType, AttendanceStatus, Brand, CastAssignmentCondition, ConfirmationState,
    DailyScheduleAggregate, GaynorManufacturingLine, GenderGroup, PersonnelRole, PropDyeState,
    QuotaBucket, RoomCode, StockSubcategory, WorkRegime,
};
use crate::services::conflict_engine::ScheduleConflict;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonnelWithQuotaDto {
    pub id: String,
    pub legajo: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub nickname: Option<String>,
    pub role: PersonnelRole,
    pub gender_group: GenderGroup,
    pub category_id: String,
    pub category_name: String,
    pub is_active: bool,
    pub default_pointe_raw_code: Option<String>,
    pub default_flat_size: Option<String>,
    pub effective_pointe_quota: i32,
    pub consumed_pointe_qty: i32,
    pub remaining_pointe_qty: i32,
    pub effective_flat_quota: i32,
    pub consumed_flat_qty: i32,
    pub remaining_flat_qty: i32,
    pub is_retired: bool,
    pub retirement_notes: Option<String>,
    pub specific_position: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertPersonnelPayload {
    pub id: Option<String>,
    pub legajo: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub nickname: Option<String>,
    pub role: Option<PersonnelRole>,
    pub gender_group: Option<GenderGroup>,
    pub category_id: Option<String>,
    pub default_pointe_raw_code: Option<String>,
    pub default_flat_size: Option<String>,
    pub is_active: Option<bool>,
    pub is_retired: Option<bool>,
    pub retirement_notes: Option<String>,
    pub specific_position: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPersonnelResult {
    pub inserted_count: usize,
    pub updated_count: usize,
    pub total_processed: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyScheduleWithAuditDto {
    pub schedule: DailyScheduleAggregate,
    pub conflicts: Vec<ScheduleConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateScheduleRulesPayload {
    pub schedule_id: String,
    pub mandatory_checkin_time: String,
    pub nominal_start_time: String,
    pub nominal_end_time: String,
    pub minimum_intershift_rest_hours: i32,
    pub class_to_rehearsal_rest_minutes: Option<i32>,
    pub is_compensatory_rest_day: Option<bool>,
    pub compensatory_rest_days: Option<i32>,
    pub compensatory_rest_policy: Option<String>,
    pub footer_legal_clause: Option<String>,
    pub general_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyScheduleSummaryDto {
    pub id: String,
    pub schedule_date: String,
    pub regime: WorkRegime,
    pub mandatory_checkin_time: String,
    pub nominal_end_time: String,
    pub rehearsal_blocks_count: i32,
    pub morning_classes_count: i32,
    pub general_notes: Option<String>,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertMorningClassPayload {
    pub id: Option<String>,
    pub schedule_id: String,
    pub room: RoomCode,
    pub gender_division: GenderGroup,
    pub start_time: String,
    pub end_time: String,
    pub master_id: String,
    pub pianist_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertRehearsalBlockPayload {
    pub id: Option<String>,
    pub schedule_id: String,
    pub room: RoomCode,
    pub start_time: String,
    pub end_time: String,
    pub activity_type: ActivityType,
    pub work_id: Option<String>,
    pub passage_id: Option<String>,
    pub custom_passage_label: Option<String>,
    pub confirmation_state: ConfirmationState,
    pub sound_requirement: String,
    pub teatrocue_cue_ref: Option<String>,
    pub master_id: Option<String>,
    pub pianist_id: Option<String>,
    pub allow_shared_room: bool,
    pub notes: Option<String>,
    pub cast_groups: Vec<UpsertCastGroupPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertCastGroupPayload {
    pub id: Option<String>,
    pub group_order: i32,
    pub group_label: Option<String>,
    pub is_to_confirm: bool,
    pub members: Vec<UpsertCastMemberPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertCastMemberPayload {
    pub personnel_id: String,
    pub character_role: Option<String>,
    pub partner_personnel_id: Option<String>,
    pub condition: CastAssignmentCondition,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockFilterPayload {
    pub query: Option<String>,
    pub subcategory: Option<StockSubcategory>,
    pub subcategory_str: Option<String>,
    pub brand: Option<Brand>,
    pub brand_str: Option<String>,
    pub color: Option<String>,
    pub size_value: Option<String>,
    pub width_value: Option<String>,
    pub gm_line: Option<GaynorManufacturingLine>,
    pub prop_dye_state: Option<PropDyeState>,
    pub low_stock_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockItemAvailabilityDto {
    pub id: String,
    pub sku_code: String,
    pub family: String,
    pub subcategory: String,
    pub brand: String,
    pub model_name: String,
    pub raw_factory_code: Option<String>,
    pub color: String,
    pub material: String,
    pub prop_dye_state: String,
    pub size_system: String,
    pub size_value: String,
    pub width_value: Option<String>,
    pub gm_line: Option<String>,
    pub gm_fit: Option<String>,
    pub gm_box: Option<i64>,
    pub gm_shank: Option<String>,
    pub gm_vamp: Option<String>,
    pub gm_heel: Option<String>,
    pub duval_hardness: Option<String>,
    pub physical_qty: i64,
    pub reserved_qty: i64,
    pub free_available_qty: i64,
    pub storage_bin_location: Option<String>,
    pub is_low_stock: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertStockItemPayload {
    pub id: Option<String>,
    pub sku_code: Option<String>,
    pub family: String,
    pub subcategory: String,
    pub quota_bucket: QuotaBucket,
    pub brand: String,
    pub model_name: String,
    pub raw_factory_code: Option<String>,
    pub color: String,
    pub material: String,
    pub size_system: String,
    pub size_value: String,
    pub width_value: Option<String>,
    pub physical_qty: i32,
    pub min_alert_qty: i32,
    pub storage_bin_location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecuteVoucherPayload {
    pub personnel_id: String,
    pub season_year: Option<i32>,
    pub delivered_by_inspector_id: String,
    pub device_id: String,
    pub signature_svg_strokes: Option<String>,
    pub is_quota_exception: bool,
    pub exception_reason: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<ExecuteVoucherItemPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecuteVoucherItemPayload {
    pub stock_item_id: String,
    pub reservation_id: Option<String>,
    pub quantity: u32,
    pub quota_bucket: QuotaBucket,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoucherReceiptDto {
    pub voucher_id: String,
    pub voucher_number: String,
    pub delivery_timestamp: String,
    pub recipient_name: String,
    pub total_items_count: usize,
    pub is_quota_exception: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncPeerInfo {
    pub device_name: String,
    pub ip_address: String,
    pub port: u16,
    pub node_id: String,
    pub database_version: u32,
    pub last_seen_iso: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncBundle {
    pub format_version: u8,
    pub exported_at_iso: String,
    pub source_device_name: String,
    pub source_node_id: String,
    pub personnel: Vec<serde_json::Value>,
    pub stock_items: Vec<serde_json::Value>,
    pub stock_ledger: Vec<serde_json::Value>,
    pub stock_reservations: Vec<serde_json::Value>,
    pub voucher_deliveries: Vec<serde_json::Value>,
    pub voucher_items: Vec<serde_json::Value>,
    pub attendance_logs: Vec<serde_json::Value>,
    pub physio_sessions: Vec<serde_json::Value>,
    pub daily_schedules: Vec<serde_json::Value>,
    pub rehearsal_blocks: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncReportDto {
    pub success: bool,
    pub remote_device: String,
    pub records_received: usize,
    pub records_applied: usize,
    pub synced_at_iso: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonthlyDailyEntryDto {
    pub date: String,
    pub day_number: u32,
    pub day_of_week: String,
    pub is_weekend: bool,
    pub expected_checkin_time: String,
    pub actual_checkin_time: Option<String>,
    pub minutes_late: i32,
    pub status: AttendanceStatus,
    pub is_in_class: bool,
    pub remarks: Option<String>,
    pub medical_diagnosis: Option<String>,
    pub has_certificate: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonthlyAttendanceSheetDto {
    pub personnel_id: String,
    pub legajo: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub category_name: String,
    pub year: i32,
    pub month: u32,
    pub month_name: String,
    pub total_working_days: i32,
    pub days_present: i32,
    pub days_late: i32,
    pub total_late_minutes: i32,
    pub days_medical_leave: i32,
    pub days_physiotherapy: i32,
    pub days_not_in_class: i32,
    pub days_absent: i32,
    pub presence_percentage: f32,
    pub entries: Vec<MonthlyDailyEntryDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonthlyAgentSummaryDto {
    pub personnel_id: String,
    pub legajo: Option<String>,
    pub dancer_name: String,
    pub role: String,
    pub days_present: i32,
    pub days_late: i32,
    pub total_late_minutes: i32,
    pub days_medical_leave: i32,
    pub days_physio: i32,
    pub days_absent: i32,
    pub presence_percentage: f32,
}

// =============================================================================
// PLANIFICACIÓN DE REPARTOS PARA OBRAS (CASTING & ROLE MATRIX)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkWithPassagesAndCastsDto {
    pub id: String,
    pub title: String,
    pub composer: Option<String>,
    pub choreographer: Option<String>,
    pub is_active: bool,
    pub passages: Vec<WorkPassageDto>,
    pub casting_plans: Vec<WorkCastingPlanDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkPassageDto {
    pub id: String,
    pub work_id: String,
    pub name: String,
    pub act_scene: Option<String>,
    pub default_duration_min: i32,
    pub default_sound_cue: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkCastingPlanDto {
    pub id: String,
    pub work_id: String,
    pub work_title: String,
    pub passage_id: Option<String>,
    pub passage_name: Option<String>,
    pub tier_name: String,
    pub cast_label: String,
    pub default_sound_cue: Option<String>,
    pub notes: Option<String>,
    pub members: Vec<WorkCastPlanMemberDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkCastPlanMemberDto {
    pub id: String,
    pub plan_id: String,
    pub personnel_id: String,
    pub dancer_name: String,
    pub character_role: Option<String>,
    pub partner_personnel_id: Option<String>,
    pub partner_name: Option<String>,
    pub condition: CastAssignmentCondition,
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertWorkPayload {
    pub id: Option<String>,
    pub title: String,
    pub composer: Option<String>,
    pub choreographer: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertWorkPassagePayload {
    pub id: Option<String>,
    pub work_id: String,
    pub name: String,
    pub act_scene: Option<String>,
    pub default_duration_min: Option<i32>,
    pub default_sound_cue: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertWorkCastingPlanPayload {
    pub id: Option<String>,
    pub work_id: String,
    pub passage_id: Option<String>,
    pub tier_name: String,
    pub cast_label: String,
    pub default_sound_cue: Option<String>,
    pub notes: Option<String>,
    pub members: Vec<UpsertWorkCastPlanMemberPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertWorkCastPlanMemberPayload {
    pub personnel_id: String,
    pub character_role: Option<String>,
    pub partner_personnel_id: Option<String>,
    pub condition: CastAssignmentCondition,
}

// =============================================================================
// PLANILLA MATRIZ MENSUAL DE AUSENTISMO BOP (EXCEL HISTÓRICO DE CARINA)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BopMatrixCellDto {
    pub day_number: u32,
    pub date: String,
    pub is_weekend: bool,
    pub code: String, // "P", "EI", "SP", "LC", "ART", "LA", "ES", ""
    pub minutes_late: i32,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BopMatrixRowDto {
    pub nro: usize,
    pub personnel_id: String,
    pub dancer_name: String, // "Álvarez, Mercedes"
    pub legajo: Option<String>,
    pub role: String,
    pub cells: Vec<BopMatrixCellDto>,
    pub total_p: i32,
    pub total_ei: i32,
    pub total_sp: i32,
    pub total_other: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BopMatrixDayInfo {
    pub day_number: u32,
    pub date: String,
    pub day_name: String, // "L", "M", "M", "J", "V", "S", "D"
    pub is_weekend: bool,
    pub day_annotation: Option<String>, // e.g. "ENSAYO SUSPENDIDO"
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonthlyBopMatrixDto {
    pub year: i32,
    pub month: u32,
    pub month_name: String, // "SEPTIEMBRE"
    pub days_in_month: u32,
    pub days_header: Vec<BopMatrixDayInfo>,
    pub rows: Vec<BopMatrixRowDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateBopMatrixCellPayload {
    pub date: String,
    pub personnel_id: String,
    pub code: String, // "P", "EI", "SP", "LC", "ART", "LA", "ES", ""
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetDayScheduleAnnotationPayload {
    pub date: String,
    pub annotation: Option<String>, // e.g. "ENSAYO SUSPENDIDO", "FERIADO"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InspectionVaultDocumentDto {
    pub id: String,
    pub title: String,
    pub category: String, // "CERTIFICADO_MEDICO", "RECIBO_COMPRA", "MEMO_DISPOSICION", "ACUERDO_DIRECCION", "SOLISTICO_FIRMA", "OTRO"
    pub document_date: String,
    pub personnel_id: Option<String>,
    pub personnel_name: Option<String>,
    pub file_name: String,
    pub file_type: String,
    pub file_size_bytes: i64,
    pub file_data: String,
    pub amount_ars: Option<f64>,
    pub supplier_vendor: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateInspectionDocumentPayload {
    pub id: Option<String>,
    pub title: String,
    pub category: String,
    pub document_date: String,
    pub personnel_id: Option<String>,
    pub file_name: String,
    pub file_type: String,
    pub file_size_bytes: Option<i64>,
    pub file_data: String,
    pub amount_ars: Option<f64>,
    pub supplier_vendor: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionVaultFilterPayload {
    pub category: Option<String>,
    pub personnel_id: Option<String>,
    pub query: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultSecurityStatusDto {
    pub has_password: bool,
    pub lock_timeout_minutes: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetVaultPasswordPayload {
    pub old_password: Option<String>,
    pub new_password: String,
    pub lock_timeout_minutes: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyVaultPasswordPayload {
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminResetVaultPasswordPayload {
    pub new_password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultMonthlyTotalsDto {
    pub month: u32,
    pub year: i32,
    pub total_amount_ars: f64,
    pub receipts_count: usize,
    pub certificates_count: usize,
    pub documents_total_count: usize,
}



