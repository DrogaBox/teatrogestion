import { invoke } from '@tauri-apps/api/core';

// =============================================================================
// MODELOS DE DOMINIO Y TIPOS COMPARTIDOS
// =============================================================================

export type PersonnelRole = 'DANCER' | 'MASTER' | 'PIANIST' | 'INSPECTOR' | 'SOUND_TECH';
export type GenderGroup = 'FEMALE' | 'MALE' | 'NON_BINARY' | 'NOT_APPLICABLE';
export type WorkRegime = 'ORDINARY' | 'PRE_FUNCTION' | 'EXTRAORDINARY';
export type RoomCode =
  | 'SALA_1'
  | 'SALA_2'
  | 'SALA_3'
  | 'ESCENARIO'
  | 'GIRA_TRASLADO'
  | 'GIRA_ESCENARIO'
  | 'GIRA_SALA'
  | 'SALA_AUX';
export type ActivityType =
  | 'STUDIO_REHEARSAL'
  | 'STAGE_TECH_REHEARSAL'
  | 'STAGE_LIGHTING_REHEARSAL'
  | 'STAGE_ORCHESTRA_REHEARSAL'
  | 'DRESS_REHEARSAL'
  | 'PERFORMANCE'
  | 'TOUR_TRAVEL'
  | 'TOUR_STAGE_REHEARSAL'
  | 'CLASS_MORNING'
  | 'COMPANY_BREAK';

export type ConfirmationState = 'CONFIRMED' | 'REHEARSAL_TO_CONFIRM' | 'CAST_TO_CONFIRM';
export type CastAssignmentCondition = 'TITULAR' | 'ALTERNATE_SLASH' | 'REINFORCEMENT' | 'COVER' | 'TO_CONFIRM';

export type QuotaBucket = 'POINTE' | 'FLAT' | 'TIGHTS' | 'SUPPORTER' | 'BRIEFS' | 'EXEMPT';
export type StockSubcategory =
  | 'MEDIA_PUNTA_VARON'
  | 'MEDIA_PUNTA_MUJER_UNISEX'
  | 'PUNTA_GAYNOR_MINDEN'
  | 'PUNTA_BLOCH'
  | 'PUNTA_SANSHA_DUVAL'
  | 'SUSPENSOR_MASCULINO'
  | 'MEDIA_MALLA'
  | 'CANCAN'
  | 'BOMBACHA_ESCENA';

export type Brand =
  | 'Sansha'
  | 'Capezio'
  | 'HD'
  | 'Bloch'
  | 'GaynorMinden'
  | 'SanshaDuval'
  | 'Fouette'
  | 'Silvana';

export type PropDyeState = 'ORIGINAL' | 'TENIDA';
export type GaynorManufacturingLine = 'EUROPEAN_TRIANGLE' | 'LYRA_CIRCLE' | 'AMERICAN_STAR';
export type GaynorFit = 'SCULPTED' | 'CLASSIC' | 'SLEEK';
export type GaynorWidth = 'NARROW' | 'MEDIUM' | 'WIDE';
export type GaynorBox = 2 | 3 | 4 | 5;
export type GaynorShank = 'EXTRAFLEX' | 'SUPPLE' | 'FEATHER' | 'PIANISSIMO' | 'HARD';
export type GaynorVamp = 'LOW' | 'DEEP' | 'DEEP_PLUS_6MM' | 'DEEP_PLUS_HALF_INCH';
export type GaynorHeel = 'LOW' | 'HIGH';

export interface SerializableCoreError {
  code: string;
  message: string;
}

export interface PersonnelWithQuotaDto {
  id: string;
  legajo: string | null;
  first_name: string;
  last_name: string;
  nickname: string | null;
  role: PersonnelRole;
  gender_group: GenderGroup;
  category_id: string;
  category_name: string;
  is_active: boolean;
  default_pointe_raw_code: string | null;
  default_flat_size: string | null;
  effective_pointe_quota: number;
  consumed_pointe_qty: number;
  remaining_pointe_qty: number;
  effective_flat_quota: number;
  consumed_flat_qty: number;
  remaining_flat_qty: number;
}

export interface MorningClassSlot {
  id: string;
  room: RoomCode;
  gender_division: GenderGroup;
  start_time: string;
  end_time: string;
  master_id: string;
  master_name: string;
  pianist_id: string;
  pianist_name: string;
}

export interface CastMemberEntry {
  id: string;
  personnel_id: string;
  display_name: string;
  character_role: string | null;
  partner_personnel_id: string | null;
  partner_display_name: string | null;
  condition: CastAssignmentCondition;
}

export interface RehearsalCastGroup {
  id: string;
  group_order: number;
  group_label: string | null;
  formatted_line: string;
  is_to_confirm: boolean;
  members: CastMemberEntry[];
}

export interface RehearsalBlock {
  id: string;
  schedule_id: string;
  room: RoomCode;
  start_time: string;
  end_time: string;
  activity_type: ActivityType;
  work_id: string | null;
  work_title: string | null;
  passage_id: string | null;
  passage_name: string;
  confirmation_state: ConfirmationState;
  sound_requirement: string;
  teatrocue_cue_ref: string | null;
  master_id: string | null;
  master_name: string | null;
  pianist_id: string | null;
  pianist_name: string | null;
  allow_shared_room: boolean;
  cast_groups: RehearsalCastGroup[];
}

export interface DailyScheduleAggregate {
  id: string;
  schedule_date: string;
  regime: WorkRegime;
  mandatory_checkin_time: string;
  nominal_end_time: string;
  minimum_intershift_rest_hours: number;
  class_to_rehearsal_rest_minutes: number;
  is_compensatory_rest_day: boolean;
  compensatory_rest_days?: number;
  compensatory_rest_policy: string;
  footer_legal_clause: string;
  morning_classes: MorningClassSlot[];
  rehearsal_blocks: RehearsalBlock[];
}

export interface DailyScheduleSummaryDto {
  id: string;
  schedule_date: string;
  regime: WorkRegime;
  mandatory_checkin_time: string;
  nominal_end_time: string;
  rehearsal_blocks_count: number;
  morning_classes_count: number;
  general_notes: string | null;
  status: string;
  updated_at: string;
}

export interface UpdateScheduleRulesPayload {
  schedule_id: string;
  mandatory_checkin_time: string;
  nominal_start_time: string;
  nominal_end_time: string;
  minimum_intershift_rest_hours: number;
  class_to_rehearsal_rest_minutes?: number;
  is_compensatory_rest_day?: boolean;
  compensatory_rest_days?: number;
  compensatory_rest_policy?: string;
  footer_legal_clause?: string | null;
  general_notes?: string | null;
}

export interface ScheduleConflict {
  conflict_type:
    | 'PERSON_DOUBLE_BOOKED'
    | 'ROOM_DOUBLE_BOOKED'
    | 'SHIFT_LIMIT_EXCEEDED'
    | 'MINIMUM_INTERSHIFT_REST_VIOLATION'
    | 'CLASS_TO_REHEARSAL_REST_VIOLATION'
    | 'COMPENSATORY_REST_DAY_VIOLATION';
  person_name?: string;
  role?: PersonnelRole;
  room?: RoomCode;
  first_room?: RoomCode;
  second_room?: RoomCode;
  first_passage?: string;
  second_passage?: string;
  overlap_start?: string;
  overlap_end?: string;
  passage_name?: string;
  block_end_time?: string;
  allowed_limit_time?: string;
  suggestion?: string;
  previous_date?: string;
  previous_end_time?: string;
  current_date?: string;
  current_checkin_time?: string;
  actual_rest_minutes?: number;
  required_rest_minutes?: number;
  class_room?: RoomCode;
  class_end_time?: string;
  rehearsal_block_id?: string;
  rehearsal_passage?: string;
  rehearsal_start_time?: string;
  actual_gap_minutes?: number;
  required_gap_minutes?: number;
  date?: string;
  rehearsal_blocks_count?: number;
}

export interface DailyScheduleWithAuditDto {
  schedule: DailyScheduleAggregate;
  conflicts: ScheduleConflict[];
}

export interface UpsertCastMemberPayload {
  personnel_id: string;
  character_role?: string | null;
  partner_personnel_id?: string | null;
  condition: CastAssignmentCondition;
}

export interface UpsertCastGroupPayload {
  id?: string | null;
  group_order: number;
  group_label?: string | null;
  is_to_confirm: boolean;
  members: UpsertCastMemberPayload[];
}

export interface UpsertMorningClassPayload {
  id?: string | null;
  schedule_id: string;
  room: RoomCode;
  gender_division: GenderGroup;
  start_time: string;
  end_time: string;
  master_id: string;
  pianist_id: string;
}

export interface UpsertRehearsalBlockPayload {
  id?: string | null;
  schedule_id: string;
  room: RoomCode;
  start_time: string;
  end_time: string;
  activity_type: ActivityType;
  work_id?: string | null;
  passage_id?: string | null;
  custom_passage_label?: string | null;
  confirmation_state: ConfirmationState;
  sound_requirement: string;
  teatrocue_cue_ref?: string | null;
  master_id?: string | null;
  pianist_id?: string | null;
  allow_shared_room: boolean;
  notes?: string | null;
  cast_groups: UpsertCastGroupPayload[];
}

export interface MonthlyDailyEntryDto {
  date: string;
  day_number: number;
  day_of_week: string;
  is_weekend: boolean;
  expected_checkin_time: string;
  actual_checkin_time: string | null;
  minutes_late: number;
  status: AttendanceStatus;
  is_in_class: boolean;
  remarks: string | null;
  medical_diagnosis: string | null;
  has_certificate: boolean;
}

export interface MonthlyAttendanceSheetDto {
  personnel_id: string;
  legajo: string | null;
  first_name: string;
  last_name: string;
  role: string;
  category_name: string;
  year: number;
  month: number;
  month_name: string;
  total_working_days: number;
  days_present: number;
  days_late: number;
  total_late_minutes: number;
  days_medical_leave: number;
  days_physiotherapy: number;
  days_not_in_class: number;
  days_absent: number;
  presence_percentage: number;
  entries: MonthlyDailyEntryDto[];
}

export interface MonthlyAgentSummaryDto {
  personnel_id: string;
  legajo: string | null;
  dancer_name: string;
  role: string;
  days_present: number;
  days_late: number;
  total_late_minutes: number;
  days_medical_leave: number;
  days_physio: number;
  days_absent: number;
  presence_percentage: number;
}

export interface GaynorParsedSpec {
  raw_code: string;
  fit: GaynorFit;
  size: string;
  width: GaynorWidth;
  box_shape: GaynorBox;
  shank: GaynorShank;
  vamp: GaynorVamp;
  heel: GaynorHeel;
}

export interface StockFilterPayload {
  query?: string | null;
  subcategory?: StockSubcategory | null;
  subcategory_str?: string | null;
  brand?: Brand | null;
  brand_str?: string | null;
  color?: string | null;
  size_value?: string | null;
  width_value?: string | null;
  gm_line?: GaynorManufacturingLine | null;
  prop_dye_state?: PropDyeState | null;
  low_stock_only?: boolean;
}

export interface StockItemAvailabilityDto {
  id: string;
  sku_code: string;
  family: string;
  subcategory: string;
  brand: string;
  model_name: string;
  raw_factory_code: string | null;
  color: string;
  material: string;
  prop_dye_state: string;
  size_system: string;
  size_value: string;
  width_value: string | null;
  gm_line: string | null;
  gm_fit: string | null;
  gm_box: number | null;
  gm_shank: string | null;
  gm_vamp: string | null;
  gm_heel: string | null;
  duval_hardness: string | null;
  physical_qty: number;
  reserved_qty: number;
  free_available_qty: number;
  storage_bin_location?: string | null;
  is_low_stock: boolean;
}

export interface UpsertStockItemPayload {
  id?: string | null;
  sku_code?: string | null;
  family: string;
  subcategory: string;
  quota_bucket: QuotaBucket;
  brand: string;
  model_name: string;
  raw_factory_code?: string | null;
  color: string;
  material: string;
  size_system: string;
  size_value: string;
  width_value?: string | null;
  physical_qty: number;
  min_alert_qty: number;
  storage_bin_location?: string | null;
}

export interface UpsertPersonnelPayload {
  id?: string | null;
  legajo?: string | null;
  first_name: string;
  last_name: string;
  nickname?: string | null;
  role?: PersonnelRole | null;
  gender_group?: GenderGroup | null;
  category_id?: string | null;
  default_pointe_raw_code?: string | null;
  default_flat_size?: string | null;
  is_active?: boolean | null;
  is_retired?: boolean | null;
  retirement_notes?: string | null;
  specific_position?: string | null;
  notes?: string | null;
}

export interface ImportPersonnelResult {
  inserted_count: number;
  updated_count: number;
  total_processed: number;
  errors: string[];
}

export interface ExecuteVoucherItemPayload {
  stock_item_id: string;
  reservation_id?: string | null;
  quantity: number;
  quota_bucket: QuotaBucket;
}

export interface ExecuteVoucherPayload {
  personnel_id: string;
  season_year?: number | null;
  delivered_by_inspector_id: string;
  device_id: string;
  signature_svg_strokes?: string | null;
  is_quota_exception: boolean;
  exception_reason?: string | null;
  notes?: string | null;
  items: ExecuteVoucherItemPayload[];
}

export interface VoucherReceiptDto {
  voucher_id: string;
  voucher_number: string;
  delivery_timestamp: string;
  recipient_name: string;
  total_items_count: number;
  is_quota_exception: boolean;
  message: string;
}

export type AttendanceStatus =
  | 'PENDING'
  | 'PRESENT'
  | 'NOT_IN_CLASS'
  | 'IN_PHYSIOTHERAPY'
  | 'LATE'
  | 'MEDICAL_LEAVE'
  | 'ABSENT'
  | 'NO_PART'
  | 'TASK_ADJUSTMENT'
  | 'JUSTIFIED_ABSENCE'
  | 'RETIRED_EXEMPT';

export type ActivityRestriction =
  | 'NORMAL'
  | 'LIGHT_BARRE_ONLY'
  | 'NO_JUMPS'
  | 'NO_POINTE'
  | 'TOTAL_REST';

export interface AttendanceLogDto {
  id: string;
  schedule_id: string;
  schedule_date: string;
  personnel_id: string;
  dancer_name: string;
  legajo: string | null;
  role: string;
  category_name: string;
  expected_checkin_time: string;
  actual_checkin_time: string | null;
  attendance_status: AttendanceStatus;
  is_in_class: boolean;
  minutes_late: number;
  remarks: string | null;
  certificate_file_name: string | null;
  certificate_data: string | null;
  medical_diagnosis: string | null;
  doctor_details: string | null;
  leave_end_date: string | null;
  physio_notes: string | null;
  current_physio_diagnosis: string | null;
  physio_tracking_notes: string | null;
  recorded_at: string;
}

export interface RecordAttendancePayload {
  schedule_date: string;
  personnel_id: string;
  attendance_status: AttendanceStatus;
  actual_checkin_time?: string | null;
  minutes_late?: number | null;
  remarks?: string | null;
  certificate_file_name?: string | null;
  certificate_data?: string | null;
  medical_diagnosis?: string | null;
  doctor_details?: string | null;
  leave_end_date?: string | null;
  physio_notes?: string | null;
  update_permanent_physio_tracking?: string | null;
}

export interface PhysiotherapySessionDto {
  id: string;
  personnel_id: string;
  dancer_name: string;
  session_date: string;
  session_time: string;
  doctor_name: string;
  diagnosis: string | null;
  treatment_applied: string;
  evolution_notes: string | null;
  activity_restriction: ActivityRestriction;
  attachment_name: string | null;
  attachment_data: string | null;
  created_at: string;
}

export interface CreatePhysioSessionPayload {
  personnel_id: string;
  session_date: string;
  session_time?: string | null;
  doctor_name?: string | null;
  diagnosis?: string | null;
  treatment_applied: string;
  evolution_notes?: string | null;
  activity_restriction: ActivityRestriction;
  attachment_name?: string | null;
  attachment_data?: string | null;
  sync_attendance_status: boolean;
}

// =============================================================================
// CLIENTE IPC FUERTEMENTE TIPADO
// =============================================================================

export const api = {
  listPersonnel: (roleFilter?: PersonnelRole | null, activeOnly = true): Promise<PersonnelWithQuotaDto[]> =>
    invoke('list_personnel', { roleFilter: roleFilter ?? null, activeOnly }),

  searchDancerQuick: (query: string): Promise<PersonnelWithQuotaDto[]> =>
    invoke('search_dancer_quick', { query }),

  deletePersonnel: (personnelId: string): Promise<void> =>
    invoke('delete_personnel', { personnelId }),

  listDailySchedules: (): Promise<DailyScheduleSummaryDto[]> =>
    invoke('list_daily_schedules'),

  getOrCreateDailySchedule: (scheduleDate: string): Promise<DailyScheduleWithAuditDto> =>
    invoke('get_or_create_daily_schedule', { scheduleDate }),

  setScheduleRegime: (scheduleId: string, regime: WorkRegime): Promise<DailyScheduleWithAuditDto> =>
    invoke('set_schedule_regime', { scheduleId, regime }),

  updateScheduleRules: (payload: UpdateScheduleRulesPayload): Promise<DailyScheduleWithAuditDto> =>
    invoke('update_schedule_rules', { payload }),

  upsertMorningClass: (payload: UpsertMorningClassPayload): Promise<DailyScheduleWithAuditDto> =>
    invoke('upsert_morning_class', { payload }),

  deleteMorningClass: (classId: string): Promise<DailyScheduleWithAuditDto | null> =>
    invoke('delete_morning_class', { classId }),

  upsertRehearsalBlock: (payload: UpsertRehearsalBlockPayload): Promise<DailyScheduleWithAuditDto> =>
    invoke('upsert_rehearsal_block', { payload }),

  deleteRehearsalBlock: (blockId: string): Promise<DailyScheduleWithAuditDto> =>
    invoke('delete_rehearsal_block', { blockId }),

  parseGaynorCodePreview: (rawCode: string): Promise<GaynorParsedSpec> =>
    invoke('parse_gaynor_code_preview', { rawCode }),

  listStockCatalog: (filter?: StockFilterPayload): Promise<StockItemAvailabilityDto[]> =>
    invoke('list_stock_catalog', { filter: filter ?? {} }),

  upsertStockItem: (payload: UpsertStockItemPayload): Promise<StockItemAvailabilityDto> =>
    invoke('upsert_stock_item', { payload }),

  upsertPersonnel: (payload: UpsertPersonnelPayload): Promise<PersonnelWithQuotaDto> =>
    invoke('upsert_personnel', { payload }),

  importPersonnelBatch: (batch: UpsertPersonnelPayload[]): Promise<ImportPersonnelResult> =>
    invoke('import_personnel_batch', { batch }),

  executeVoucherDelivery: (payload: ExecuteVoucherPayload): Promise<VoucherReceiptDto> =>
    invoke('execute_voucher_delivery', { payload }),

  listAttendanceForDate: (scheduleDate: string): Promise<AttendanceLogDto[]> =>
    invoke('list_attendance_for_date', { scheduleDate }),

  recordAttendance: (payload: RecordAttendancePayload): Promise<AttendanceLogDto> =>
    invoke('record_attendance', { payload }),

  getAgentMonthlyAttendanceSheet: (personnelId: string, year: number, month: number): Promise<MonthlyAttendanceSheetDto> =>
    invoke('get_agent_monthly_attendance_sheet', { personnelId, year, month }),

  listMonthlyAgentsSummary: (year: number, month: number): Promise<MonthlyAgentSummaryDto[]> =>
    invoke('list_monthly_agents_summary', { year, month }),

  createPhysiotherapySession: (payload: CreatePhysioSessionPayload): Promise<PhysiotherapySessionDto> =>
    invoke('create_physiotherapy_session', { payload }),

  listPhysiotherapySessions: (dateFilter?: string | null, personnelFilter?: string | null): Promise<PhysiotherapySessionDto[]> =>
    invoke('list_physiotherapy_sessions', { dateFilter: dateFilter ?? null, personnelFilter: personnelFilter ?? null }),

  getLanSyncInfo: (): Promise<LanSyncInfo> =>
    invoke('get_lan_sync_info'),

  getLanSyncKey: (): Promise<string> =>
    invoke('get_lan_sync_key'),

  setLanSyncKey: (key: string): Promise<string> =>
    invoke('set_lan_sync_key', { key }),

  syncWithLanPeer: (peerIp: string, peerPort?: number | null): Promise<SyncReportDto> =>
    invoke('sync_with_lan_peer', { peerIp, peerPort: peerPort ?? null }),

  exportLocalSyncBundle: (): Promise<any> =>
    invoke('export_local_sync_bundle'),

  importRemoteSyncBundle: (bundle: any): Promise<SyncReportDto> =>
    invoke('import_remote_sync_bundle', { bundle }),

  listWorksWithCasts: (): Promise<WorkWithPassagesAndCastsDto[]> =>
    invoke('list_works_with_casts'),

  upsertWork: (payload: UpsertWorkPayload): Promise<WorkWithPassagesAndCastsDto> =>
    invoke('upsert_work', { payload }),

  deleteWork: (workId: string): Promise<void> =>
    invoke('delete_work', { workId }),

  upsertWorkPassage: (payload: UpsertWorkPassagePayload): Promise<WorkPassageDto> =>
    invoke('upsert_work_passage', { payload }),

  deleteWorkPassage: (passageId: string): Promise<void> =>
    invoke('delete_work_passage', { passageId }),

  upsertWorkCastingPlan: (payload: UpsertWorkCastingPlanPayload): Promise<WorkCastingPlanDto> =>
    invoke('upsert_work_casting_plan', { payload }),

  deleteWorkCastingPlan: (planId: string): Promise<void> =>
    invoke('delete_work_casting_plan', { planId }),

  getUpdateStatus: (): Promise<UpdateStatus> =>
    invoke('get_update_status'),

  setUpdateToken: (token: string): Promise<void> =>
    invoke('set_update_token', { token }),

  clearUpdateToken: (): Promise<void> =>
    invoke('clear_update_token'),

  checkForUpdate: (): Promise<UpdateInfo | null> =>
    invoke('check_for_update'),

  installUpdate: (info: UpdateInfo): Promise<InstallOutcome> =>
    invoke('install_update', { info }),

  getLicenseInfo: (): Promise<LicenseInfo> =>
    invoke('get_license_info'),

  applyEmergencyActivation: (key: string): Promise<LicenseInfo> =>
    invoke('apply_emergency_activation', { key }),

  getMachineId: (): Promise<string> =>
    invoke('get_machine_id'),

  // Certificaciones de Solísticos (Plus por Rol)
  listSoloistCertifications: (): Promise<SoloistCertificationDto[]> =>
    invoke('list_soloist_certifications'),

  getSoloistCertification: (id: string): Promise<SoloistCertificationDto> =>
    invoke('get_soloist_certification', { id }),

  saveSoloistCertification: (payload: SaveSoloistCertificationPayload): Promise<SoloistCertificationDto> =>
    invoke('save_soloist_certification', { payload }),

  deleteSoloistCertification: (id: string): Promise<void> =>
    invoke('delete_soloist_certification', { id }),

  generateSoloistCertificationFromWorkCast: (workId: string, datesLabel: string, performancesPerCast: number): Promise<SoloistCertificationDto> =>
    invoke('generate_soloist_certification_from_work_cast', { workId, datesLabel, performancesPerCast }),

  // Planilla Matriz Mensual de Ausentismo BOP (Excel de Carina)
  getMonthlyBopMatrix: (year: number, month: number): Promise<MonthlyBopMatrixDto> =>
    invoke('get_monthly_bop_matrix', { year, month }),

  updateBopMatrixCell: (payload: UpdateBopMatrixCellPayload): Promise<void> =>
    invoke('update_bop_matrix_cell', { payload }),

  setDayScheduleAnnotation: (payload: SetDayScheduleAnnotationPayload): Promise<void> =>
    invoke('set_day_schedule_annotation', { payload }),

  // Bóveda Documental de Inspección (Carina Lion)
  listVaultDocuments: (filter: InspectionVaultFilterPayload): Promise<InspectionVaultDocumentDto[]> =>
    invoke('list_vault_documents', { filter }),

  createVaultDocument: (payload: CreateInspectionDocumentPayload): Promise<InspectionVaultDocumentDto> =>
    invoke('create_vault_document', { payload }),

  deleteVaultDocument: (id: string): Promise<void> =>
    invoke('delete_vault_document', { id }),

  getVaultMonthlyTotals: (month: number, year: number): Promise<VaultMonthlyTotalsDto> =>
    invoke('get_vault_monthly_totals', { month, year }),

  getVaultSecurityStatus: (): Promise<VaultSecurityStatusDto> =>
    invoke('get_vault_security_status'),

  verifyVaultPassword: (payload: VerifyVaultPasswordPayload): Promise<boolean> =>
    invoke('verify_vault_password', { payload }),

  setVaultPassword: (payload: SetVaultPasswordPayload): Promise<void> =>
    invoke('set_vault_password', { payload }),

  adminResetVaultPassword: (payload: AdminResetVaultPasswordPayload): Promise<void> =>
    invoke('admin_reset_vault_password', { payload }),
};

export interface WorkWithPassagesAndCastsDto {
  id: string;
  title: string;
  composer: string | null;
  choreographer: string | null;
  is_active: boolean;
  passages: WorkPassageDto[];
  casting_plans: WorkCastingPlanDto[];
}

export interface WorkPassageDto {
  id: string;
  work_id: string;
  name: string;
  act_scene: string | null;
  default_duration_min: number;
  default_sound_cue: string | null;
}

export interface WorkCastingPlanDto {
  id: string;
  work_id: string;
  work_title: string;
  passage_id: string | null;
  passage_name: string | null;
  tier_name: string;
  cast_label: string;
  default_sound_cue: string | null;
  notes: string | null;
  members: WorkCastPlanMemberDto[];
}

export interface WorkCastPlanMemberDto {
  id: string;
  plan_id: string;
  personnel_id: string;
  dancer_name: string;
  character_role: string | null;
  partner_personnel_id: string | null;
  partner_name: string | null;
  condition: CastAssignmentCondition;
  sort_order: number;
}

export interface UpsertWorkPayload {
  id?: string | null;
  title: string;
  composer?: string | null;
  choreographer?: string | null;
  is_active: boolean;
}

export interface UpsertWorkPassagePayload {
  id?: string | null;
  work_id: string;
  name: string;
  act_scene?: string | null;
  default_duration_min?: number | null;
  default_sound_cue?: string | null;
}

export interface UpsertWorkCastingPlanPayload {
  id?: string | null;
  work_id: string;
  passage_id?: string | null;
  tier_name: string;
  cast_label: string;
  default_sound_cue?: string | null;
  notes?: string | null;
  members: UpsertWorkCastPlanMemberPayload[];
}

export interface UpsertWorkCastPlanMemberPayload {
  personnel_id: string;
  character_role?: string | null;
  partner_personnel_id?: string | null;
  condition: CastAssignmentCondition;
}

export interface LanSyncInfo {
  local_ip: string;
  port: number;
  is_server_active: boolean;
  device_name: string;
}

export interface SyncReportDto {
  success: boolean;
  remote_device: string;
  records_received: number;
  records_applied: number;
  synced_at_iso: string;
  message: string;
}

export interface UpdateInfo {
  version: string;
  name: string;
  notes: string;
  published_at: string;
  asset_id: number;
  asset_name: string;
  asset_size: number;
  asset_digest: string | null;
}

export interface UpdateStatus {
  version: string;
  platform: 'windows' | 'android' | null;
  has_token: boolean;
}

export interface InstallOutcome {
  mode: 'windows' | 'android';
  path: string;
}

export interface LicenseInfo {
  is_valid: boolean;
  is_locked: boolean;
  state: 'Valid' | 'Invalid' | 'NotEnforced';
  reason?: string | null;
  licensee?: string | null;
  machine_id?: string | null;
  issued_date?: string | null;
  expires_date?: string | null;
  days_left?: number | null;
  warning?: string | null;
  reactivable: boolean;
  challenge_code?: string | null;
}

// =============================================================================
// CERTIFICACIÓN OFICIAL DE SOLÍSTICOS (PLUS POR PRESTACIÓN DE ROL)
// =============================================================================

export interface SoloistCertificationItemDto {
  id: string;
  certification_id: string;
  personnel_id: string | null;
  dancer_name: string;
  work_role: string;
  performance_dates: string;
  performance_count: number;
  sort_order: number;
}

export interface SoloistCertificationDto {
  id: string;
  program_title: string;
  hall_name: string;
  document_date: string;
  recipient_name: string;
  recipient_title: string;
  intro_body: string;
  closing_text: string;
  status: string;
  notes: string | null;
  total_performances: number;
  items: SoloistCertificationItemDto[];
  created_at: string;
  updated_at: string;
}

export interface SaveSoloistItemPayload {
  id?: string | null;
  personnel_id?: string | null;
  dancer_name: string;
  work_role: string;
  performance_dates: string;
  performance_count: number;
  sort_order: number;
}

export interface SaveSoloistCertificationPayload {
  id?: string | null;
  program_title: string;
  hall_name: string;
  document_date: string;
  recipient_name: string;
  recipient_title: string;
  intro_body: string;
  closing_text: string;
  status: string;
  notes?: string | null;
  items: SaveSoloistItemPayload[];
}

// =============================================================================
// PLANILLA MATRIZ MENSUAL DE AUSENTISMO BOP (EXCEL DE CARINA LION)
// =============================================================================

export interface BopMatrixCellDto {
  day_number: number;
  date: string;
  is_weekend: boolean;
  code: string; // "P", "EI", "SP", "LC", "ART", "LA", "IJ", "AT", "ES", ""
  minutes_late: number;
  remarks: string | null;
}

export interface BopMatrixRowDto {
  nro: number;
  personnel_id: string;
  dancer_name: string;
  legajo: string | null;
  role: string;
  cells: BopMatrixCellDto[];
  total_p: number;
  total_ei: number;
  total_sp: number;
  total_other: number;
}

export interface BopMatrixDayInfo {
  day_number: number;
  date: string;
  day_name: string;
  is_weekend: boolean;
  day_annotation: string | null;
}

export interface MonthlyBopMatrixDto {
  year: number;
  month: number;
  month_name: string;
  days_in_month: number;
  days_header: BopMatrixDayInfo[];
  rows: BopMatrixRowDto[];
}

export interface UpdateBopMatrixCellPayload {
  date: string;
  personnel_id: string;
  code: string;
  remarks?: string | null;
}

export interface SetDayScheduleAnnotationPayload {
  date: string;
  annotation?: string | null;
}

// =============================================================================
// BÓVEDA DOCUMENTAL DE INSPECCIÓN (CARINA LION)
// =============================================================================

export type VaultCategory =
  | 'CERTIFICADO_MEDICO'
  | 'RECIBO_COMPRA'
  | 'MEMO_DISPOSICION'
  | 'ACUERDO_DIRECCION'
  | 'SOLISTICO_FIRMA'
  | 'OTRO';

export interface InspectionVaultDocumentDto {
  id: string;
  title: string;
  category: VaultCategory | string;
  document_date: string;
  personnel_id: string | null;
  personnel_name: string | null;
  file_name: string;
  file_type: string;
  file_size_bytes: number;
  file_data: string;
  amount_ars: number | null;
  supplier_vendor: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateInspectionDocumentPayload {
  id?: string;
  title: string;
  category: string;
  document_date: string;
  personnel_id?: string | null;
  file_name: string;
  file_type: string;
  file_size_bytes?: number;
  file_data: string;
  amount_ars?: number | null;
  supplier_vendor?: string | null;
  notes?: string | null;
}

export interface InspectionVaultFilterPayload {
  category?: string;
  personnel_id?: string;
  query?: string;
  start_date?: string;
  end_date?: string;
}

export interface VaultSecurityStatusDto {
  has_password: boolean;
  lock_timeout_minutes: number;
}

export interface SetVaultPasswordPayload {
  old_password?: string | null;
  new_password: string;
  lock_timeout_minutes?: number | null;
}

export interface VerifyVaultPasswordPayload {
  password: string;
}

export interface AdminResetVaultPasswordPayload {
  new_password: string;
}

export interface VaultMonthlyTotalsDto {
  month: number;
  year: number;
  total_amount_ars: number;
  receipts_count: number;
  certificates_count: number;
  documents_total_count: number;
}


