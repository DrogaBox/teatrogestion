use crate::dto::{SyncBundle, SyncReportDto};
use crate::error::CoreResult;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use serde_json::Value;

pub fn export_sync_bundle(
    conn: &Connection,
    device_name: &str,
    node_id: &str,
) -> CoreResult<SyncBundle> {
    let now_iso = Utc::now().to_rfc3339();

    // 1. Personnel
    let mut stmt = conn.prepare(
        "SELECT id, legajo, first_name, last_name, nickname, role, gender_group, category_id,
                phone_whatsapp, default_pointe_raw_code, default_flat_size, is_active,
                notes, created_at, updated_at, is_retired, retirement_notes, specific_position,
                current_physio_diagnosis, physio_tracking_notes,
                (SELECT name FROM personnel_categories c WHERE c.id = personnel.category_id)
         FROM personnel",
    )?;
    let personnel = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "legajo": row.get::<_, Option<String>>(1)?,
                "first_name": row.get::<_, String>(2)?,
                "last_name": row.get::<_, String>(3)?,
                "nickname": row.get::<_, Option<String>>(4)?,
                "role": row.get::<_, String>(5)?,
                "gender_group": row.get::<_, String>(6)?,
                "category_id": row.get::<_, String>(7)?,
                "phone_whatsapp": row.get::<_, Option<String>>(8)?,
                "default_pointe_raw_code": row.get::<_, Option<String>>(9)?,
                "default_flat_size": row.get::<_, Option<String>>(10)?,
                "is_active": row.get::<_, i32>(11)? == 1,
                "notes": row.get::<_, Option<String>>(12)?,
                "created_at": row.get::<_, String>(13)?,
                "updated_at": row.get::<_, String>(14)?,
                "is_retired": row.get::<_, i32>(15)? == 1,
                "retirement_notes": row.get::<_, Option<String>>(16)?,
                "specific_position": row.get::<_, Option<String>>(17)?,
                "current_physio_diagnosis": row.get::<_, Option<String>>(18)?,
                "physio_tracking_notes": row.get::<_, Option<String>>(19)?,
                "category_name": row.get::<_, Option<String>>(20)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 2. Stock items
    let mut stmt = conn.prepare(
        "SELECT id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name,
                raw_factory_code, color, material, prop_dye_state, dyed_for_work_id,
                size_system, size_value, width_value, gm_line, gm_fit, gm_box, gm_shank,
                gm_vamp, gm_heel, duval_hardness, physical_qty, reserved_qty, min_alert_qty,
                storage_bin_location, updated_at, is_active
         FROM stock_items",
    )?;
    let stock_items = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "sku_code": row.get::<_, String>(1)?,
                "barcode": row.get::<_, Option<String>>(2)?,
                "family": row.get::<_, String>(3)?,
                "subcategory": row.get::<_, String>(4)?,
                "quota_bucket": row.get::<_, String>(5)?,
                "brand": row.get::<_, String>(6)?,
                "model_name": row.get::<_, String>(7)?,
                "raw_factory_code": row.get::<_, Option<String>>(8)?,
                "color": row.get::<_, String>(9)?,
                "material": row.get::<_, String>(10)?,
                "prop_dye_state": row.get::<_, String>(11)?,
                "dyed_for_work_id": row.get::<_, Option<String>>(12)?,
                "size_system": row.get::<_, String>(13)?,
                "size_value": row.get::<_, String>(14)?,
                "width_value": row.get::<_, Option<String>>(15)?,
                "gm_line": row.get::<_, Option<String>>(16)?,
                "gm_fit": row.get::<_, Option<String>>(17)?,
                "gm_box": row.get::<_, Option<i32>>(18)?,
                "gm_shank": row.get::<_, Option<String>>(19)?,
                "gm_vamp": row.get::<_, Option<String>>(20)?,
                "gm_heel": row.get::<_, Option<String>>(21)?,
                "duval_hardness": row.get::<_, Option<String>>(22)?,
                "physical_qty": row.get::<_, i32>(23)?,
                "reserved_qty": row.get::<_, i32>(24)?,
                "min_alert_qty": row.get::<_, i32>(25)?,
                "storage_bin_location": row.get::<_, Option<String>>(26)?,
                "updated_at": row.get::<_, String>(27)?,
                "is_active": row.get::<_, i32>(28)? == 1,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 3. Stock Ledger
    let mut stmt = conn.prepare(
        "SELECT id, stock_item_id, movement_type, delta_physical, delta_reserved,
                balance_physical_after, balance_reserved_after, reference_id, performed_by, notes, created_at
         FROM stock_ledger",
    )?;
    let stock_ledger = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "stock_item_id": row.get::<_, String>(1)?,
                "movement_type": row.get::<_, String>(2)?,
                "delta_physical": row.get::<_, i32>(3)?,
                "delta_reserved": row.get::<_, i32>(4)?,
                "balance_physical_after": row.get::<_, i32>(5)?,
                "balance_reserved_after": row.get::<_, i32>(6)?,
                "reference_id": row.get::<_, Option<String>>(7)?,
                "performed_by": row.get::<_, Option<String>>(8)?,
                "notes": row.get::<_, Option<String>>(9)?,
                "created_at": row.get::<_, String>(10)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 4. Stock Reservations
    let mut stmt = conn.prepare(
        "SELECT id, stock_item_id, personnel_id, reserved_for_label, quantity, status, work_id, notes, created_by, created_at, resolved_at
         FROM stock_reservations",
    )?;
    let stock_reservations = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "stock_item_id": row.get::<_, String>(1)?,
                "personnel_id": row.get::<_, String>(2)?,
                "reserved_for_label": row.get::<_, String>(3)?,
                "quantity": row.get::<_, i32>(4)?,
                "status": row.get::<_, String>(5)?,
                "work_id": row.get::<_, Option<String>>(6)?,
                "notes": row.get::<_, Option<String>>(7)?,
                "created_by": row.get::<_, Option<String>>(8)?,
                "created_at": row.get::<_, String>(9)?,
                "resolved_at": row.get::<_, Option<String>>(10)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 5. Voucher Deliveries
    let mut stmt = conn.prepare(
        "SELECT id, voucher_number, personnel_id, season_year, delivered_by_inspector_id,
                delivery_timestamp, is_quota_exception, exception_reason, device_id, signature_svg_strokes, notes
         FROM voucher_deliveries",
    )?;
    let voucher_deliveries = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "voucher_number": row.get::<_, String>(1)?,
                "personnel_id": row.get::<_, String>(2)?,
                "season_year": row.get::<_, i32>(3)?,
                "delivered_by_inspector_id": row.get::<_, String>(4)?,
                "delivery_timestamp": row.get::<_, String>(5)?,
                "is_quota_exception": row.get::<_, i32>(6)?,
                "exception_reason": row.get::<_, Option<String>>(7)?,
                "device_id": row.get::<_, String>(8)?,
                "signature_svg_strokes": row.get::<_, Option<String>>(9)?,
                "notes": row.get::<_, Option<String>>(10)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 5b. Voucher Delivery Items
    let mut stmt = conn.prepare(
        "SELECT id, voucher_id, stock_item_id, reservation_id, quantity, quota_bucket
         FROM voucher_delivery_items",
    )?;
    let voucher_items = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "voucher_id": row.get::<_, String>(1)?,
                "stock_item_id": row.get::<_, String>(2)?,
                "reservation_id": row.get::<_, Option<String>>(3)?,
                "quantity": row.get::<_, i32>(4)?,
                "quota_bucket": row.get::<_, String>(5)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 6. Attendance Logs (with schedule_date from daily_schedules)
    let mut stmt = conn.prepare(
        "SELECT a.id, a.schedule_id, ds.schedule_date, a.personnel_id, a.expected_checkin_time,
                a.actual_checkin_time, a.attendance_status, a.operational_status, a.is_in_class,
                a.minutes_late, a.remarks, a.certificate_file_name, a.certificate_data,
                a.medical_diagnosis, a.doctor_details, a.leave_start_date, a.leave_end_date,
                a.physio_notes, a.recorded_at
         FROM attendance_logs a
         JOIN daily_schedules ds ON ds.id = a.schedule_id",
    )?;
    let attendance_logs = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "schedule_id": row.get::<_, String>(1)?,
                "schedule_date": row.get::<_, String>(2)?,
                "personnel_id": row.get::<_, String>(3)?,
                "expected_checkin_time": row.get::<_, Option<String>>(4)?,
                "actual_checkin_time": row.get::<_, Option<String>>(5)?,
                "attendance_status": row.get::<_, String>(6)?,
                "operational_status": row.get::<_, Option<String>>(7)?,
                "is_in_class": row.get::<_, i32>(8)? == 1,
                "minutes_late": row.get::<_, i32>(9)?,
                "remarks": row.get::<_, Option<String>>(10)?,
                "certificate_file_name": row.get::<_, Option<String>>(11)?,
                "certificate_data": row.get::<_, Option<String>>(12)?,
                "medical_diagnosis": row.get::<_, Option<String>>(13)?,
                "doctor_details": row.get::<_, Option<String>>(14)?,
                "leave_start_date": row.get::<_, Option<String>>(15)?,
                "leave_end_date": row.get::<_, Option<String>>(16)?,
                "physio_notes": row.get::<_, Option<String>>(17)?,
                "recorded_at": row.get::<_, String>(18)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 7. Physio Sessions
    let mut stmt = conn.prepare(
        "SELECT id, personnel_id, session_date, session_time, doctor_name, diagnosis,
                treatment_applied, evolution_notes, activity_restriction, attachment_name,
                attachment_data, created_at
         FROM physiotherapy_sessions",
    )?;
    let physio_sessions = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "personnel_id": row.get::<_, String>(1)?,
                "session_date": row.get::<_, String>(2)?,
                "session_time": row.get::<_, String>(3)?,
                "doctor_name": row.get::<_, String>(4)?,
                "diagnosis": row.get::<_, Option<String>>(5)?,
                "treatment_applied": row.get::<_, String>(6)?,
                "evolution_notes": row.get::<_, Option<String>>(7)?,
                "activity_restriction": row.get::<_, String>(8)?,
                "attachment_name": row.get::<_, Option<String>>(9)?,
                "attachment_data": row.get::<_, Option<String>>(10)?,
                "created_at": row.get::<_, String>(11)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 8. Daily Schedules (with all columns from migrations 0001, 0009, 0011, 0012)
    let mut stmt = conn.prepare(
        "SELECT id, schedule_date, day_of_week, regime_type, mandatory_checkin_time,
                nominal_start_time, nominal_end_time, minimum_intershift_rest_hours,
                footer_legal_clause, status, general_notes, created_at, updated_at,
                day_annotation, class_to_rehearsal_rest_minutes, is_compensatory_rest_day,
                compensatory_rest_policy, compensatory_rest_days
         FROM daily_schedules",
    )?;
    let daily_schedules = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "schedule_date": row.get::<_, String>(1)?,
                "day_of_week": row.get::<_, i32>(2)?,
                "regime_type": row.get::<_, String>(3)?,
                "mandatory_checkin_time": row.get::<_, String>(4)?,
                "nominal_start_time": row.get::<_, String>(5)?,
                "nominal_end_time": row.get::<_, String>(6)?,
                "minimum_intershift_rest_hours": row.get::<_, i32>(7)?,
                "footer_legal_clause": row.get::<_, String>(8)?,
                "status": row.get::<_, String>(9)?,
                "general_notes": row.get::<_, Option<String>>(10)?,
                "created_at": row.get::<_, String>(11)?,
                "updated_at": row.get::<_, String>(12)?,
                "day_annotation": row.get::<_, Option<String>>(13)?,
                "class_to_rehearsal_rest_minutes": row.get::<_, i32>(14)?,
                "is_compensatory_rest_day": row.get::<_, i32>(15)? == 1,
                "compensatory_rest_policy": row.get::<_, String>(16)?,
                "compensatory_rest_days": row.get::<_, i32>(17)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 9. Rehearsal Blocks
    let mut stmt = conn.prepare(
        "SELECT id, schedule_id, room_id, start_time, end_time, activity_type,
                work_id, passage_id, custom_passage_label, confirmation_state,
                sound_requirement, teatrocue_cue_ref, master_id, pianist_id,
                allow_shared_room, notes, sort_order, sound_tech_id
         FROM rehearsal_blocks",
    )?;
    let rehearsal_blocks = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "schedule_id": row.get::<_, String>(1)?,
                "room_id": row.get::<_, String>(2)?,
                "start_time": row.get::<_, String>(3)?,
                "end_time": row.get::<_, String>(4)?,
                "activity_type": row.get::<_, String>(5)?,
                "work_id": row.get::<_, Option<String>>(6)?,
                "passage_id": row.get::<_, Option<String>>(7)?,
                "custom_passage_label": row.get::<_, Option<String>>(8)?,
                "confirmation_state": row.get::<_, String>(9)?,
                "sound_requirement": row.get::<_, String>(10)?,
                "teatrocue_cue_ref": row.get::<_, Option<String>>(11)?,
                "master_id": row.get::<_, Option<String>>(12)?,
                "pianist_id": row.get::<_, Option<String>>(13)?,
                "allow_shared_room": row.get::<_, i32>(14)? == 1,
                "notes": row.get::<_, Option<String>>(15)?,
                "sort_order": row.get::<_, i32>(16)?,
                "sound_tech_id": row.get::<_, Option<String>>(17)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 10. Morning classes
    let mut stmt = conn.prepare(
        "SELECT id, schedule_id, room_id, gender_division, start_time, end_time,
                master_id, pianist_id, notes, sound_tech_id
         FROM morning_classes",
    )?;
    let morning_classes = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "schedule_id": row.get::<_, String>(1)?,
                "room_id": row.get::<_, String>(2)?,
                "gender_division": row.get::<_, String>(3)?,
                "start_time": row.get::<_, String>(4)?,
                "end_time": row.get::<_, String>(5)?,
                "master_id": row.get::<_, String>(6)?,
                "pianist_id": row.get::<_, String>(7)?,
                "notes": row.get::<_, Option<String>>(8)?,
                "sound_tech_id": row.get::<_, Option<String>>(9)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 11. Medical Leave Records
    let mut stmt = conn.prepare(
        "SELECT id, personnel_id, start_date, end_date, diagnosis, doctor_name,
                certificate_file_name, certificate_data, remarks, created_at, leave_type
         FROM medical_leave_records",
    )?;
    let medical_leave_records = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "personnel_id": row.get::<_, String>(1)?,
                "start_date": row.get::<_, String>(2)?,
                "end_date": row.get::<_, String>(3)?,
                "diagnosis": row.get::<_, Option<String>>(4)?,
                "doctor_name": row.get::<_, Option<String>>(5)?,
                "certificate_file_name": row.get::<_, Option<String>>(6)?,
                "certificate_data": row.get::<_, Option<String>>(7)?,
                "remarks": row.get::<_, Option<String>>(8)?,
                "created_at": row.get::<_, String>(9)?,
                "leave_type": row.get::<_, String>(10)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 12. Works
    let mut stmt = conn.prepare(
        "SELECT id, title, composer, choreographer, is_active FROM works",
    )?;
    let works = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "composer": row.get::<_, Option<String>>(2)?,
                "choreographer": row.get::<_, Option<String>>(3)?,
                "is_active": row.get::<_, i32>(4)? == 1,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 13. Work Passages
    let mut stmt = conn.prepare(
        "SELECT id, work_id, name, act_scene, default_duration_min, default_sound_cue
         FROM work_passages",
    )?;
    let work_passages = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "work_id": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "act_scene": row.get::<_, Option<String>>(3)?,
                "default_duration_min": row.get::<_, i32>(4)?,
                "default_sound_cue": row.get::<_, Option<String>>(5)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 14. Work Cast Plans
    let mut stmt = conn.prepare(
        "SELECT id, work_id, passage_id, tier_name, cast_label, default_sound_cue, notes, created_at, updated_at
         FROM work_cast_plans",
    )?;
    let work_cast_plans = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "work_id": row.get::<_, String>(1)?,
                "passage_id": row.get::<_, Option<String>>(2)?,
                "tier_name": row.get::<_, String>(3)?,
                "cast_label": row.get::<_, String>(4)?,
                "default_sound_cue": row.get::<_, Option<String>>(5)?,
                "notes": row.get::<_, Option<String>>(6)?,
                "created_at": row.get::<_, String>(7)?,
                "updated_at": row.get::<_, String>(8)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 15. Work Cast Plan Members
    let mut stmt = conn.prepare(
        "SELECT id, plan_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order
         FROM work_cast_plan_members",
    )?;
    let work_cast_plan_members = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "plan_id": row.get::<_, String>(1)?,
                "personnel_id": row.get::<_, String>(2)?,
                "character_role": row.get::<_, Option<String>>(3)?,
                "partner_personnel_id": row.get::<_, Option<String>>(4)?,
                "assignment_condition": row.get::<_, String>(5)?,
                "sort_order": row.get::<_, i32>(6)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 16. Rehearsal Cast Groups
    let mut stmt = conn.prepare(
        "SELECT id, rehearsal_block_id, group_order, group_label, display_expression, is_to_confirm
         FROM rehearsal_cast_groups",
    )?;
    let rehearsal_cast_groups = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "rehearsal_block_id": row.get::<_, String>(1)?,
                "group_order": row.get::<_, i32>(2)?,
                "group_label": row.get::<_, Option<String>>(3)?,
                "display_expression": row.get::<_, Option<String>>(4)?,
                "is_to_confirm": row.get::<_, i32>(5)? == 1,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 17. Rehearsal Cast Members
    let mut stmt = conn.prepare(
        "SELECT id, cast_group_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order
         FROM rehearsal_cast_members",
    )?;
    let rehearsal_cast_members = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "cast_group_id": row.get::<_, String>(1)?,
                "personnel_id": row.get::<_, String>(2)?,
                "character_role": row.get::<_, Option<String>>(3)?,
                "partner_personnel_id": row.get::<_, Option<String>>(4)?,
                "assignment_condition": row.get::<_, String>(5)?,
                "sort_order": row.get::<_, i32>(6)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 18. Soloist Certifications
    let mut stmt = conn.prepare(
        "SELECT id, program_title, hall_name, document_date, recipient_name, recipient_title,
                intro_body, closing_text, status, notes, created_at, updated_at
         FROM soloist_certifications",
    )?;
    let soloist_certifications = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "program_title": row.get::<_, String>(1)?,
                "hall_name": row.get::<_, String>(2)?,
                "document_date": row.get::<_, String>(3)?,
                "recipient_name": row.get::<_, String>(4)?,
                "recipient_title": row.get::<_, String>(5)?,
                "intro_body": row.get::<_, String>(6)?,
                "closing_text": row.get::<_, String>(7)?,
                "status": row.get::<_, String>(8)?,
                "notes": row.get::<_, Option<String>>(9)?,
                "created_at": row.get::<_, String>(10)?,
                "updated_at": row.get::<_, String>(11)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 19. Soloist Certification Items
    let mut stmt = conn.prepare(
        "SELECT id, certification_id, personnel_id, dancer_name, work_role,
                performance_dates, performance_count, sort_order
         FROM soloist_certification_items",
    )?;
    let soloist_certification_items = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "certification_id": row.get::<_, String>(1)?,
                "personnel_id": row.get::<_, Option<String>>(2)?,
                "dancer_name": row.get::<_, String>(3)?,
                "work_role": row.get::<_, String>(4)?,
                "performance_dates": row.get::<_, String>(5)?,
                "performance_count": row.get::<_, i32>(6)?,
                "sort_order": row.get::<_, i32>(7)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    // 20. Inspection Vault Documents
    let mut stmt = conn.prepare(
        "SELECT id, title, category, document_date, personnel_id, file_name, file_type,
                file_size_bytes, file_data, amount_ars, supplier_vendor, notes, created_at, updated_at
         FROM inspection_vault_documents",
    )?;
    let inspection_vault_documents = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "category": row.get::<_, String>(2)?,
                "document_date": row.get::<_, String>(3)?,
                "personnel_id": row.get::<_, Option<String>>(4)?,
                "file_name": row.get::<_, String>(5)?,
                "file_type": row.get::<_, String>(6)?,
                "file_size_bytes": row.get::<_, i64>(7)?,
                "file_data": row.get::<_, String>(8)?,
                "amount_ars": row.get::<_, Option<f64>>(9)?,
                "supplier_vendor": row.get::<_, Option<String>>(10)?,
                "notes": row.get::<_, Option<String>>(11)?,
                "created_at": row.get::<_, String>(12)?,
                "updated_at": row.get::<_, String>(13)?,
            }))
        })?
        .collect::<Result<Vec<Value>, _>>()?;

    Ok(SyncBundle {
        format_version: 2,
        exported_at_iso: now_iso,
        source_device_name: device_name.to_string(),
        source_node_id: node_id.to_string(),
        personnel,
        stock_items,
        stock_ledger,
        stock_reservations,
        voucher_deliveries,
        voucher_items,
        attendance_logs,
        physio_sessions,
        daily_schedules,
        rehearsal_blocks,
        morning_classes,
        medical_leave_records,
        works,
        work_passages,
        work_cast_plans,
        work_cast_plan_members,
        rehearsal_cast_groups,
        rehearsal_cast_members,
        soloist_certifications,
        soloist_certification_items,
        inspection_vault_documents,
    })
}

pub fn import_sync_bundle(
    conn: &mut Connection,
    bundle: &SyncBundle,
) -> CoreResult<SyncReportDto> {
    let mut total_received = 0;
    let mut total_applied = 0;

    let tx = conn.transaction()?;
    // Parte remoto -> parte local de la misma fecha: cada equipo crea su propio parte del día al tomar asistencia,
    // así que la misma fecha puede llegar con otro `id` y `schedule_date` es UNIQUE.
    let mut schedule_ids: HashMap<String, String> = HashMap::new();
    let mut work_ids: HashMap<String, String> = HashMap::new();
    let mut passage_ids: HashMap<String, String> = HashMap::new();

    // 1. Personnel
    for item in &bundle.personnel {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let legajo = item["legajo"].as_str();
        let first_name = item["first_name"].as_str().unwrap_or_default();
        let last_name = item["last_name"].as_str().unwrap_or_default();
        let nickname = item["nickname"].as_str();
        let role = item["role"].as_str().unwrap_or("DANCER");
        let gender_group = item["gender_group"].as_str().unwrap_or("UNISEX");
        let category_id = item["category_id"].as_str().unwrap_or("CORPS_DE_BALLET");
        let phone_whatsapp = item["phone_whatsapp"].as_str();
        let default_pointe = item["default_pointe_raw_code"].as_str();
        let default_flat = item["default_flat_size"].as_str();
        let is_active = if item["is_active"].as_bool().unwrap_or(true) { 1 } else { 0 };
        let notes = item["notes"].as_str();
        let created_at = item["created_at"].as_str().unwrap_or("");
        let updated_at = item["updated_at"].as_str().unwrap_or("");
        let is_retired = if item["is_retired"].as_bool().unwrap_or(false) { 1 } else { 0 };
        let retirement_notes = item["retirement_notes"].as_str();
        let specific_position = item["specific_position"].as_str();
        let current_physio_diagnosis = item["current_physio_diagnosis"].as_str();
        let physio_tracking_notes = item["physio_tracking_notes"].as_str();
        // Una categoría escrita a mano en el otro equipo puede no existir aquí todavía.
        tx.execute(
            "INSERT OR IGNORE INTO personnel_categories (id, code, name) VALUES (?1, ?1, ?2)",
            params![category_id, item["category_name"].as_str().unwrap_or(category_id)],
        )?;

        let changes = tx.execute(
            "INSERT INTO personnel (
                id, legajo, first_name, last_name, nickname, role, gender_group, category_id,
                phone_whatsapp, default_pointe_raw_code, default_flat_size, is_active, notes,
                created_at, updated_at, is_retired, retirement_notes, specific_position,
                current_physio_diagnosis, physio_tracking_notes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
             ON CONFLICT(id) DO UPDATE SET
                legajo = excluded.legajo,
                first_name = excluded.first_name,
                last_name = excluded.last_name,
                nickname = excluded.nickname,
                role = excluded.role,
                gender_group = excluded.gender_group,
                category_id = excluded.category_id,
                phone_whatsapp = excluded.phone_whatsapp,
                default_pointe_raw_code = excluded.default_pointe_raw_code,
                default_flat_size = excluded.default_flat_size,
                is_active = excluded.is_active,
                notes = excluded.notes,
                updated_at = excluded.updated_at,
                is_retired = excluded.is_retired,
                retirement_notes = excluded.retirement_notes,
                specific_position = excluded.specific_position,
                current_physio_diagnosis = COALESCE(excluded.current_physio_diagnosis, personnel.current_physio_diagnosis),
                physio_tracking_notes = COALESCE(excluded.physio_tracking_notes, personnel.physio_tracking_notes)
             WHERE excluded.updated_at >= personnel.updated_at",
            params![
                id, legajo, first_name, last_name, nickname, role, gender_group, category_id,
                phone_whatsapp, default_pointe, default_flat, is_active, notes,
                created_at, updated_at, is_retired, retirement_notes, specific_position,
                current_physio_diagnosis, physio_tracking_notes
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 1b. Works (antes de stock_items, work_passages, stock_reservations y rehearsal_blocks por FK)
    for item in &bundle.works {
        total_received += 1;
        let remote_id = item["id"].as_str().unwrap_or_default();
        let title = item["title"].as_str().unwrap_or_default();
        let composer = item["composer"].as_str();
        let choreographer = item["choreographer"].as_str();
        let is_active = if item["is_active"].as_bool().unwrap_or(true) { 1 } else { 0 };
        if remote_id.is_empty() || title.is_empty() {
            continue;
        }
        let local_id: String = tx
            .query_row("SELECT id FROM works WHERE title = ?1", params![title], |r| r.get(0))
            .optional()?
            .unwrap_or_else(|| remote_id.to_string());
        work_ids.insert(remote_id.to_string(), local_id.clone());

        let changes = tx.execute(
            "INSERT INTO works (id, title, composer, choreographer, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                composer = COALESCE(excluded.composer, works.composer),
                choreographer = COALESCE(excluded.choreographer, works.choreographer),
                is_active = excluded.is_active",
            params![local_id, title, composer, choreographer, is_active],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 1c. Work Passages
    for item in &bundle.work_passages {
        total_received += 1;
        let remote_id = item["id"].as_str().unwrap_or_default();
        let raw_work_id = item["work_id"].as_str().unwrap_or_default();
        let work_id = work_ids.get(raw_work_id).map_or(raw_work_id, String::as_str);
        let name = item["name"].as_str().unwrap_or_default();
        let act_scene = item["act_scene"].as_str();
        let default_duration = item["default_duration_min"].as_i64().unwrap_or(60);
        let default_cue = item["default_sound_cue"].as_str();
        if remote_id.is_empty() || work_id.is_empty() || name.is_empty() {
            continue;
        }
        let local_id: String = tx
            .query_row(
                "SELECT id FROM work_passages WHERE work_id = ?1 AND name = ?2",
                params![work_id, name],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or_else(|| remote_id.to_string());
        passage_ids.insert(remote_id.to_string(), local_id.clone());

        let changes = tx.execute(
            "INSERT INTO work_passages (id, work_id, name, act_scene, default_duration_min, default_sound_cue)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                act_scene = COALESCE(excluded.act_scene, work_passages.act_scene),
                default_duration_min = excluded.default_duration_min,
                default_sound_cue = COALESCE(excluded.default_sound_cue, work_passages.default_sound_cue)",
            params![local_id, work_id, name, act_scene, default_duration, default_cue],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 1d. Work Cast Plans
    for item in &bundle.work_cast_plans {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let raw_work_id = item["work_id"].as_str().unwrap_or_default();
        let work_id = work_ids.get(raw_work_id).map_or(raw_work_id, String::as_str);
        let passage_id = item["passage_id"]
            .as_str()
            .map(|p| passage_ids.get(p).map_or(p, String::as_str));
        let tier_name = item["tier_name"].as_str().unwrap_or("1° Elenco");
        let cast_label = item["cast_label"].as_str().unwrap_or_default();
        let default_cue = item["default_sound_cue"].as_str();
        let notes = item["notes"].as_str();
        let created_at = item["created_at"].as_str().unwrap_or("");
        let updated_at = item["updated_at"].as_str().unwrap_or("");
        if id.is_empty() || work_id.is_empty() {
            continue;
        }

        let changes = tx.execute(
            "INSERT INTO work_cast_plans (
                id, work_id, passage_id, tier_name, cast_label, default_sound_cue, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                work_id = excluded.work_id,
                passage_id = excluded.passage_id,
                tier_name = excluded.tier_name,
                cast_label = excluded.cast_label,
                default_sound_cue = excluded.default_sound_cue,
                notes = excluded.notes,
                updated_at = excluded.updated_at
             WHERE excluded.updated_at >= work_cast_plans.updated_at",
            params![id, work_id, passage_id, tier_name, cast_label, default_cue, notes, created_at, updated_at],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 1e. Work Cast Plan Members
    for item in &bundle.work_cast_plan_members {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let plan_id = item["plan_id"].as_str().unwrap_or_default();
        let personnel_id = item["personnel_id"].as_str().unwrap_or_default();
        let character_role = item["character_role"].as_str();
        let partner_id = item["partner_personnel_id"].as_str();
        let cond = item["assignment_condition"].as_str().unwrap_or("TITULAR");
        let sort_order = item["sort_order"].as_i64().unwrap_or(0);
        if id.is_empty() || plan_id.is_empty() || personnel_id.is_empty() {
            continue;
        }

        let result = tx.execute(
            "INSERT INTO work_cast_plan_members (
                id, plan_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                character_role = excluded.character_role,
                partner_personnel_id = excluded.partner_personnel_id,
                assignment_condition = excluded.assignment_condition,
                sort_order = excluded.sort_order",
            params![id, plan_id, personnel_id, character_role, partner_id, cond, sort_order],
        );
        match result {
            Ok(n) if n > 0 => total_applied += 1,
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == rusqlite::ErrorCode::ConstraintViolation => {}
            Err(e) => return Err(e.into()),
        }
    }

    // 2. Stock items
    for item in &bundle.stock_items {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let sku_code = item["sku_code"].as_str().unwrap_or_default();
        let barcode = item["barcode"].as_str();
        let family = item["family"].as_str().unwrap_or_default();
        let subcategory = item["subcategory"].as_str().unwrap_or_default();
        let quota_bucket = item["quota_bucket"].as_str().unwrap_or("EXEMPT");
        let brand = item["brand"].as_str().unwrap_or_default();
        let model_name = item["model_name"].as_str().unwrap_or_default();
        let raw_factory_code = item["raw_factory_code"].as_str();
        let color = item["color"].as_str().unwrap_or_default();
        let material = item["material"].as_str().unwrap_or_default();
        let prop_dye_state = item["prop_dye_state"].as_str().unwrap_or("ORIGINAL");
        let dyed_for_work_id = item["dyed_for_work_id"].as_str();
        let size_system = item["size_system"].as_str().unwrap_or("ALPHA");
        let size_value = item["size_value"].as_str().unwrap_or_default();
        let width_value = item["width_value"].as_str();
        let gm_line = item["gm_line"].as_str();
        let gm_fit = item["gm_fit"].as_str();
        let gm_box = item["gm_box"].as_i64();
        let gm_shank = item["gm_shank"].as_str();
        let gm_vamp = item["gm_vamp"].as_str();
        let gm_heel = item["gm_heel"].as_str();
        let duval_hardness = item["duval_hardness"].as_str();
        let physical_qty = item["physical_qty"].as_i64().unwrap_or(0);
        let reserved_qty = item["reserved_qty"].as_i64().unwrap_or(0);
        let min_alert = item["min_alert_qty"].as_i64().unwrap_or(2);
        let bin_loc = item["storage_bin_location"].as_str();
        let updated_at = item["updated_at"].as_str().unwrap_or("");
        // Un equipo sin la baja de artículos no manda el campo: no cambia lo que haya acá.
        let is_active = item["is_active"].as_bool().map(i32::from);

        let changes = tx.execute(
            "INSERT INTO stock_items (
                id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name,
                raw_factory_code, color, material, prop_dye_state, dyed_for_work_id,
                size_system, size_value, width_value, gm_line, gm_fit, gm_box, gm_shank,
                gm_vamp, gm_heel, duval_hardness, physical_qty, reserved_qty, min_alert_qty,
                storage_bin_location, updated_at, is_active
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, COALESCE(?29, 1))
             ON CONFLICT(id) DO UPDATE SET
                sku_code = excluded.sku_code,
                barcode = excluded.barcode,
                family = excluded.family,
                subcategory = excluded.subcategory,
                quota_bucket = excluded.quota_bucket,
                brand = excluded.brand,
                model_name = excluded.model_name,
                raw_factory_code = excluded.raw_factory_code,
                color = excluded.color,
                material = excluded.material,
                prop_dye_state = excluded.prop_dye_state,
                dyed_for_work_id = excluded.dyed_for_work_id,
                size_system = excluded.size_system,
                size_value = excluded.size_value,
                width_value = excluded.width_value,
                gm_line = excluded.gm_line,
                gm_fit = excluded.gm_fit,
                gm_box = excluded.gm_box,
                gm_shank = excluded.gm_shank,
                gm_vamp = excluded.gm_vamp,
                gm_heel = excluded.gm_heel,
                duval_hardness = excluded.duval_hardness,
                physical_qty = excluded.physical_qty,
                reserved_qty = excluded.reserved_qty,
                min_alert_qty = excluded.min_alert_qty,
                storage_bin_location = excluded.storage_bin_location,
                updated_at = excluded.updated_at,
                is_active = COALESCE(?29, stock_items.is_active)
             WHERE excluded.updated_at >= stock_items.updated_at",
            params![
                id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name,
                raw_factory_code, color, material, prop_dye_state, dyed_for_work_id,
                size_system, size_value, width_value, gm_line, gm_fit, gm_box, gm_shank,
                gm_vamp, gm_heel, duval_hardness, physical_qty, reserved_qty, min_alert,
                bin_loc, updated_at, is_active
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 3. Daily Schedules (Imported before attendance_logs and rehearsal_blocks for FK integrity)
    for item in &bundle.daily_schedules {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let sched_date = item["schedule_date"].as_str().unwrap_or_default();
        let day_of_week = item["day_of_week"].as_i64().unwrap_or(1);
        let regime_type = item["regime_type"].as_str().unwrap_or("ORDINARY");
        let mandatory_checkin = item["mandatory_checkin_time"].as_str().unwrap_or("08:45");
        let start_time = item["nominal_start_time"].as_str().unwrap_or("09:00");
        let end_time = item["nominal_end_time"].as_str().unwrap_or("13:45");
        let rest_hours = item["minimum_intershift_rest_hours"].as_i64().unwrap_or(12);
        let legal_clause = item["footer_legal_clause"].as_str().unwrap_or("★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★");
        let status = item["status"].as_str().unwrap_or("DRAFT");
        let notes = item["general_notes"].as_str();
        let created_at = item["created_at"].as_str().unwrap_or("");
        let updated_at = item["updated_at"].as_str().unwrap_or("");
        let day_annotation = item["day_annotation"].as_str();
        let class_rest_mins = item["class_to_rehearsal_rest_minutes"].as_i64().unwrap_or(15);
        let is_comp_day = if item["is_compensatory_rest_day"].as_bool().unwrap_or(false) { 1 } else { 0 };
        let comp_policy = item["compensatory_rest_policy"].as_str().unwrap_or("NEXT_DAY");
        let comp_days = item["compensatory_rest_days"].as_i64().unwrap_or(1);
        let id = local_schedule_id(&tx, &mut schedule_ids, id, sched_date)?;

        let changes = tx.execute(
            "INSERT INTO daily_schedules (
                id, schedule_date, day_of_week, regime_type, mandatory_checkin_time,
                nominal_start_time, nominal_end_time, minimum_intershift_rest_hours,
                footer_legal_clause, status, general_notes, created_at, updated_at,
                day_annotation, class_to_rehearsal_rest_minutes, is_compensatory_rest_day,
                compensatory_rest_policy, compensatory_rest_days
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(id) DO UPDATE SET
                regime_type = excluded.regime_type,
                mandatory_checkin_time = excluded.mandatory_checkin_time,
                nominal_start_time = excluded.nominal_start_time,
                nominal_end_time = excluded.nominal_end_time,
                minimum_intershift_rest_hours = excluded.minimum_intershift_rest_hours,
                footer_legal_clause = excluded.footer_legal_clause,
                status = excluded.status,
                general_notes = excluded.general_notes,
                day_annotation = COALESCE(excluded.day_annotation, daily_schedules.day_annotation),
                class_to_rehearsal_rest_minutes = excluded.class_to_rehearsal_rest_minutes,
                is_compensatory_rest_day = excluded.is_compensatory_rest_day,
                compensatory_rest_policy = excluded.compensatory_rest_policy,
                compensatory_rest_days = excluded.compensatory_rest_days,
                updated_at = excluded.updated_at
             WHERE excluded.updated_at >= daily_schedules.updated_at",
            params![
                id, sched_date, day_of_week, regime_type, mandatory_checkin,
                start_time, end_time, rest_hours, legal_clause, status, notes, created_at, updated_at,
                day_annotation, class_rest_mins, is_comp_day, comp_policy, comp_days
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 4. Stock Reservations
    for item in &bundle.stock_reservations {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let stock_item_id = item["stock_item_id"].as_str().unwrap_or_default();
        let personnel_id = item["personnel_id"].as_str().unwrap_or_default();
        let reserved_for_label = item["reserved_for_label"].as_str().unwrap_or_default();
        let quantity = item["quantity"].as_i64().unwrap_or(1);
        let status = item["status"].as_str().unwrap_or("ACTIVE");
        let work_id = item["work_id"].as_str();
        let notes = item["notes"].as_str();
        let created_by = item["created_by"].as_str();
        let created_at = item["created_at"].as_str().unwrap_or("");
        let resolved_at = item["resolved_at"].as_str();

        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM stock_reservations WHERE id = ?1)",
            params![id],
            |r| r.get(0),
        )?;

        let changes = if exists {
            tx.execute(
                "UPDATE stock_reservations SET
                    status = ?1,
                    notes = ?2,
                    resolved_at = ?3
                 WHERE id = ?4",
                params![status, notes, resolved_at, id],
            )?
        } else {
            tx.execute(
                "INSERT INTO stock_reservations (
                    id, stock_item_id, personnel_id, reserved_for_label, quantity, status,
                    work_id, notes, created_by, created_at, resolved_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    id, stock_item_id, personnel_id, reserved_for_label, quantity, status,
                    work_id, notes, created_by, created_at, resolved_at
                ],
            )?
        };
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 5. Voucher Deliveries
    for item in &bundle.voucher_deliveries {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let voucher_num = item["voucher_number"].as_str().unwrap_or_default();
        let personnel_id = item["personnel_id"].as_str().unwrap_or_default();
        let season_year = item["season_year"].as_i64().unwrap_or(2026);
        let inspector_id = item["delivered_by_inspector_id"].as_str().unwrap_or_default();
        let delivery_ts = item["delivery_timestamp"].as_str().unwrap_or("");
        let is_quota_exception = item["is_quota_exception"].as_i64().unwrap_or(0);
        let exception_reason = item["exception_reason"].as_str();
        let device_id = item["device_id"].as_str().unwrap_or("UNKNOWN");
        let signature = item["signature_svg_strokes"].as_str();
        let notes = item["notes"].as_str();

        // Evitar colisión UNIQUE(voucher_number) si dos equipos emitieron el mismo correlativo offline
        let final_voucher_num = {
            let collides: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM voucher_deliveries WHERE voucher_number = ?1 AND id != ?2)",
                params![voucher_num, id],
                |r| r.get(0),
            )?;
            if collides {
                let short_id: String = id.chars().rev().take(4).collect();
                format!("{}-{}", voucher_num, short_id.to_uppercase())
            } else {
                voucher_num.to_string()
            }
        };

        let changes = tx.execute(
            "INSERT INTO voucher_deliveries (
                id, voucher_number, personnel_id, season_year, delivered_by_inspector_id,
                delivery_timestamp, is_quota_exception, exception_reason, device_id,
                signature_svg_strokes, notes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO NOTHING",
            params![
                id, final_voucher_num, personnel_id, season_year, inspector_id,
                delivery_ts, is_quota_exception, exception_reason, device_id,
                signature, notes
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 5b. Voucher Delivery Items
    for item in &bundle.voucher_items {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let voucher_id = item["voucher_id"].as_str().unwrap_or_default();
        let stock_item_id = item["stock_item_id"].as_str().unwrap_or_default();
        let reservation_id = item["reservation_id"].as_str();
        let quantity = item["quantity"].as_i64().unwrap_or(1);
        let quota_bucket = item["quota_bucket"].as_str().unwrap_or("EXEMPT");

        let changes = tx.execute(
            "INSERT INTO voucher_delivery_items (
                id, voucher_id, stock_item_id, reservation_id, quantity, quota_bucket
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO NOTHING",
            params![id, voucher_id, stock_item_id, reservation_id, quantity, quota_bucket],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 6. Stock Ledger
    for item in &bundle.stock_ledger {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let stock_item_id = item["stock_item_id"].as_str().unwrap_or_default();
        let movement_type = item["movement_type"].as_str().unwrap_or("INITIAL_INVENTORY");
        let delta_physical = item["delta_physical"].as_i64().unwrap_or(0);
        let delta_reserved = item["delta_reserved"].as_i64().unwrap_or(0);
        let balance_physical_after = item["balance_physical_after"].as_i64().unwrap_or(0);
        let balance_reserved_after = item["balance_reserved_after"].as_i64().unwrap_or(0);
        let reference_id = item["reference_id"].as_str();
        let performed_by = item["performed_by"].as_str();
        let notes = item["notes"].as_str();
        let created_at = item["created_at"].as_str().unwrap_or("");

        let changes = tx.execute(
            "INSERT INTO stock_ledger (
                id, stock_item_id, movement_type, delta_physical, delta_reserved,
                balance_physical_after, balance_reserved_after, reference_id,
                performed_by, notes, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO NOTHING",
            params![
                id, stock_item_id, movement_type, delta_physical, delta_reserved,
                balance_physical_after, balance_reserved_after, reference_id,
                performed_by, notes, created_at
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 7. Attendance Logs
    for item in &bundle.attendance_logs {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let schedule_id = item["schedule_id"].as_str().unwrap_or_default();
        let schedule_date = item["schedule_date"].as_str().unwrap_or("");
        let personnel_id = item["personnel_id"].as_str().unwrap_or_default();
        let expected_checkin = item["expected_checkin_time"].as_str().unwrap_or("08:45");
        let actual_checkin = item["actual_checkin_time"].as_str();
        let att_status = item["attendance_status"].as_str().unwrap_or("PENDING");
        let op_status = item["operational_status"].as_str();
        let is_in_class = if item["is_in_class"].as_bool().unwrap_or(true) { 1 } else { 0 };
        let mins_late = item["minutes_late"].as_i64().unwrap_or(0);
        let remarks = item["remarks"].as_str();
        let cert_name = item["certificate_file_name"].as_str();
        let cert_data = item["certificate_data"].as_str();
        let diagnosis = item["medical_diagnosis"].as_str();
        let doctor = item["doctor_details"].as_str();
        let leave_start = item["leave_start_date"].as_str();
        let leave_end = item["leave_end_date"].as_str();
        let physio_notes = item["physio_notes"].as_str();
        let recorded_at = item["recorded_at"].as_str().unwrap_or("");

        let schedule_id = local_schedule_id(&tx, &mut schedule_ids, schedule_id, schedule_date)?;
        // Ensure parent schedule exists if not already present
        if !schedule_id.is_empty() && !schedule_date.is_empty() {
            let _ = tx.execute(
                "INSERT OR IGNORE INTO daily_schedules (id, schedule_date, day_of_week, regime_type)
                 VALUES (?1, ?2, 1, 'ORDINARY')",
                params![schedule_id, schedule_date],
            );
        }

        let changes = tx.execute(
            "INSERT INTO attendance_logs (
                id, schedule_id, personnel_id, expected_checkin_time, actual_checkin_time,
                attendance_status, operational_status, is_in_class, minutes_late, remarks,
                certificate_file_name, certificate_data, medical_diagnosis, doctor_details,
                leave_start_date, leave_end_date, physio_notes, recorded_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(schedule_id, personnel_id) DO UPDATE SET
                expected_checkin_time = excluded.expected_checkin_time,
                actual_checkin_time = excluded.actual_checkin_time,
                attendance_status = excluded.attendance_status,
                operational_status = excluded.operational_status,
                is_in_class = excluded.is_in_class,
                minutes_late = excluded.minutes_late,
                remarks = excluded.remarks,
                certificate_file_name = COALESCE(excluded.certificate_file_name, attendance_logs.certificate_file_name),
                certificate_data = COALESCE(excluded.certificate_data, attendance_logs.certificate_data),
                medical_diagnosis = COALESCE(excluded.medical_diagnosis, attendance_logs.medical_diagnosis),
                doctor_details = COALESCE(excluded.doctor_details, attendance_logs.doctor_details),
                leave_start_date = COALESCE(excluded.leave_start_date, attendance_logs.leave_start_date),
                leave_end_date = COALESCE(excluded.leave_end_date, attendance_logs.leave_end_date),
                physio_notes = COALESCE(excluded.physio_notes, attendance_logs.physio_notes),
                recorded_at = excluded.recorded_at
             WHERE excluded.recorded_at >= attendance_logs.recorded_at",
            params![
                id, schedule_id, personnel_id, expected_checkin, actual_checkin,
                att_status, op_status, is_in_class, mins_late, remarks,
                cert_name, cert_data, diagnosis, doctor, leave_start, leave_end,
                physio_notes, recorded_at
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 8. Physio Sessions
    for item in &bundle.physio_sessions {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let personnel_id = item["personnel_id"].as_str().unwrap_or_default();
        let session_date = item["session_date"].as_str().unwrap_or_default();
        let session_time = item["session_time"].as_str().unwrap_or("09:15");
        let doctor_name = item["doctor_name"].as_str().unwrap_or("Dra. de Fisioterapia");
        let diagnosis = item["diagnosis"].as_str();
        let treatment_applied = item["treatment_applied"].as_str().unwrap_or("");
        let evolution_notes = item["evolution_notes"].as_str();
        let activity_restriction = item["activity_restriction"].as_str().unwrap_or("NORMAL");
        let att_name = item["attachment_name"].as_str();
        let att_data = item["attachment_data"].as_str();
        let created_at = item["created_at"].as_str().unwrap_or("");

        let changes = tx.execute(
            "INSERT INTO physiotherapy_sessions (
                id, personnel_id, session_date, session_time, doctor_name, diagnosis,
                treatment_applied, evolution_notes, activity_restriction, attachment_name,
                attachment_data, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                doctor_name = excluded.doctor_name,
                diagnosis = excluded.diagnosis,
                treatment_applied = excluded.treatment_applied,
                evolution_notes = excluded.evolution_notes,
                activity_restriction = excluded.activity_restriction,
                attachment_name = COALESCE(excluded.attachment_name, physiotherapy_sessions.attachment_name),
                attachment_data = COALESCE(excluded.attachment_data, physiotherapy_sessions.attachment_data)",
            params![
                id, personnel_id, session_date, session_time, doctor_name, diagnosis,
                treatment_applied, evolution_notes, activity_restriction, att_name,
                att_data, created_at
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 9. Rehearsal Blocks
    for item in &bundle.rehearsal_blocks {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let sched_id = item["schedule_id"].as_str().unwrap_or_default();
        let sched_id = schedule_ids.get(sched_id).map_or(sched_id, String::as_str);
        let room_id = item["room_id"].as_str().unwrap_or("SALA_1");
        let start_time = item["start_time"].as_str().unwrap_or("10:30");
        let end_time = item["end_time"].as_str().unwrap_or("12:00");
        let activity_type = item["activity_type"].as_str().unwrap_or("STUDIO_REHEARSAL");
        let work_id = item["work_id"]
            .as_str()
            .map(|w| work_ids.get(w).map_or(w, String::as_str));
        let passage_id = item["passage_id"]
            .as_str()
            .map(|p| passage_ids.get(p).map_or(p, String::as_str));
        let custom_passage = item["custom_passage_label"].as_str();
        let confirmation = item["confirmation_state"].as_str().unwrap_or("CONFIRMED");
        let sound_req = item["sound_requirement"].as_str().unwrap_or("PIANO");
        let teatrocue_ref = item["teatrocue_cue_ref"].as_str();
        let master_id = item["master_id"].as_str();
        let pianist_id = item["pianist_id"].as_str();
        let allow_shared = if item["allow_shared_room"].as_bool().unwrap_or(false) { 1 } else { 0 };
        let notes = item["notes"].as_str();
        let sort_order = item["sort_order"].as_i64().unwrap_or(0);
        let sound_tech_id = item["sound_tech_id"].as_str();

        let changes = tx.execute(
            "INSERT INTO rehearsal_blocks (
                id, schedule_id, room_id, start_time, end_time, activity_type,
                work_id, passage_id, custom_passage_label, confirmation_state,
                sound_requirement, teatrocue_cue_ref, master_id, pianist_id,
                allow_shared_room, notes, sort_order, sound_tech_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(id) DO UPDATE SET
                room_id = excluded.room_id,
                start_time = excluded.start_time,
                end_time = excluded.end_time,
                activity_type = excluded.activity_type,
                work_id = excluded.work_id,
                passage_id = excluded.passage_id,
                custom_passage_label = excluded.custom_passage_label,
                confirmation_state = excluded.confirmation_state,
                sound_requirement = excluded.sound_requirement,
                teatrocue_cue_ref = excluded.teatrocue_cue_ref,
                master_id = excluded.master_id,
                pianist_id = excluded.pianist_id,
                allow_shared_room = excluded.allow_shared_room,
                notes = excluded.notes,
                sort_order = excluded.sort_order,
                sound_tech_id = excluded.sound_tech_id",
            params![
                id, sched_id, room_id, start_time, end_time, activity_type,
                work_id, passage_id, custom_passage, confirmation,
                sound_req, teatrocue_ref, master_id, pianist_id,
                allow_shared, notes, sort_order, sound_tech_id
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 10. Morning classes
    for item in &bundle.morning_classes {
        total_received += 1;
        let result = tx.execute(
            "INSERT INTO morning_classes (
                id, schedule_id, room_id, gender_division, start_time, end_time,
                master_id, pianist_id, notes, sound_tech_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                room_id = excluded.room_id,
                gender_division = excluded.gender_division,
                master_id = excluded.master_id,
                pianist_id = excluded.pianist_id,
                notes = excluded.notes,
                sound_tech_id = excluded.sound_tech_id",
            params![
                item["id"].as_str().unwrap_or_default(),
                item["schedule_id"].as_str().map(|s| schedule_ids.get(s).map_or(s, String::as_str)).unwrap_or_default(),
                item["room_id"].as_str().unwrap_or("SALA_1"),
                item["gender_division"].as_str().unwrap_or("FEMALE"),
                item["start_time"].as_str().unwrap_or("09:00"),
                item["end_time"].as_str().unwrap_or("10:15"),
                item["master_id"].as_str().unwrap_or_default(),
                item["pianist_id"].as_str().unwrap_or_default(),
                item["notes"].as_str(),
                item["sound_tech_id"].as_str(),
            ],
        );
        match result {
            Ok(n) if n > 0 => total_applied += 1,
            Ok(_) => {}
            // Otra clase ya ocupa esa sala o ese maestro ese día, o falta el parte del día: se conserva lo local.
            Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == rusqlite::ErrorCode::ConstraintViolation => {}
            Err(e) => return Err(e.into()),
        }
    }

    // 10b. Medical Leave Records
    for item in &bundle.medical_leave_records {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let personnel_id = item["personnel_id"].as_str().unwrap_or_default();
        let start_date = item["start_date"].as_str().unwrap_or_default();
        let end_date = item["end_date"].as_str().unwrap_or_default();
        let diagnosis = item["diagnosis"].as_str();
        let doctor_name = item["doctor_name"].as_str();
        let cert_name = item["certificate_file_name"].as_str();
        let cert_data = item["certificate_data"].as_str();
        let remarks = item["remarks"].as_str();
        let created_at = item["created_at"].as_str().unwrap_or("");
        let leave_type = item["leave_type"].as_str().unwrap_or("ENFERMEDAD");
        if id.is_empty() || personnel_id.is_empty() {
            continue;
        }
        let changes = tx.execute(
            "INSERT INTO medical_leave_records (
                id, personnel_id, start_date, end_date, diagnosis, doctor_name,
                certificate_file_name, certificate_data, remarks, created_at, leave_type
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                diagnosis = COALESCE(excluded.diagnosis, medical_leave_records.diagnosis),
                doctor_name = COALESCE(excluded.doctor_name, medical_leave_records.doctor_name),
                certificate_file_name = COALESCE(excluded.certificate_file_name, medical_leave_records.certificate_file_name),
                certificate_data = COALESCE(excluded.certificate_data, medical_leave_records.certificate_data),
                remarks = COALESCE(excluded.remarks, medical_leave_records.remarks),
                leave_type = excluded.leave_type",
            params![
                id, personnel_id, start_date, end_date, diagnosis, doctor_name,
                cert_name, cert_data, remarks, created_at, leave_type
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 10c. Rehearsal Cast Groups
    for item in &bundle.rehearsal_cast_groups {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let block_id = item["rehearsal_block_id"].as_str().unwrap_or_default();
        let group_order = item["group_order"].as_i64().unwrap_or(1);
        let group_label = item["group_label"].as_str();
        let display_expr = item["display_expression"].as_str();
        let is_to_confirm = if item["is_to_confirm"].as_bool().unwrap_or(false) { 1 } else { 0 };
        if id.is_empty() || block_id.is_empty() {
            continue;
        }
        let changes = tx.execute(
            "INSERT INTO rehearsal_cast_groups (
                id, rehearsal_block_id, group_order, group_label, display_expression, is_to_confirm
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                group_order = excluded.group_order,
                group_label = excluded.group_label,
                display_expression = excluded.display_expression,
                is_to_confirm = excluded.is_to_confirm",
            params![id, block_id, group_order, group_label, display_expr, is_to_confirm],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 10d. Rehearsal Cast Members
    for item in &bundle.rehearsal_cast_members {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let group_id = item["cast_group_id"].as_str().unwrap_or_default();
        let personnel_id = item["personnel_id"].as_str().unwrap_or_default();
        let character_role = item["character_role"].as_str();
        let partner_id = item["partner_personnel_id"].as_str();
        let cond = item["assignment_condition"].as_str().unwrap_or("TITULAR");
        let sort_order = item["sort_order"].as_i64().unwrap_or(0);
        if id.is_empty() || group_id.is_empty() || personnel_id.is_empty() {
            continue;
        }
        let result = tx.execute(
            "INSERT INTO rehearsal_cast_members (
                id, cast_group_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                character_role = excluded.character_role,
                partner_personnel_id = excluded.partner_personnel_id,
                assignment_condition = excluded.assignment_condition,
                sort_order = excluded.sort_order",
            params![id, group_id, personnel_id, character_role, partner_id, cond, sort_order],
        );
        match result {
            Ok(n) if n > 0 => total_applied += 1,
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == rusqlite::ErrorCode::ConstraintViolation => {}
            Err(e) => return Err(e.into()),
        }
    }

    // 10e. Soloist Certifications
    for item in &bundle.soloist_certifications {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let program_title = item["program_title"].as_str().unwrap_or_default();
        let hall_name = item["hall_name"].as_str().unwrap_or("Sala Mayor del Teatro del Libertador Gral. San Martín");
        let document_date = item["document_date"].as_str().unwrap_or_default();
        let recipient_name = item["recipient_name"].as_str().unwrap_or_default();
        let recipient_title = item["recipient_title"].as_str().unwrap_or_default();
        let intro_body = item["intro_body"].as_str().unwrap_or("");
        let closing_text = item["closing_text"].as_str().unwrap_or("Sin otro particular saludo a Ud. Atte.");
        let status = item["status"].as_str().unwrap_or("DRAFT");
        let notes = item["notes"].as_str();
        let created_at = item["created_at"].as_str().unwrap_or("");
        let updated_at = item["updated_at"].as_str().unwrap_or("");
        if id.is_empty() {
            continue;
        }
        let changes = tx.execute(
            "INSERT INTO soloist_certifications (
                id, program_title, hall_name, document_date, recipient_name, recipient_title,
                intro_body, closing_text, status, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                program_title = excluded.program_title,
                hall_name = excluded.hall_name,
                document_date = excluded.document_date,
                recipient_name = excluded.recipient_name,
                recipient_title = excluded.recipient_title,
                intro_body = excluded.intro_body,
                closing_text = excluded.closing_text,
                status = excluded.status,
                notes = excluded.notes,
                updated_at = excluded.updated_at
             WHERE excluded.updated_at >= soloist_certifications.updated_at",
            params![
                id, program_title, hall_name, document_date, recipient_name, recipient_title,
                intro_body, closing_text, status, notes, created_at, updated_at
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 10f. Soloist Certification Items
    for item in &bundle.soloist_certification_items {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let cert_id = item["certification_id"].as_str().unwrap_or_default();
        let raw_personnel_id = item["personnel_id"].as_str();
        let personnel_id = match raw_personnel_id {
            Some(pid) if !pid.is_empty() => {
                let exists: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM personnel WHERE id = ?1)",
                    [pid],
                    |r| r.get(0),
                )?;
                if exists { Some(pid) } else { None }
            }
            _ => None,
        };
        let dancer_name = item["dancer_name"].as_str().unwrap_or_default();
        let work_role = item["work_role"].as_str().unwrap_or_default();
        let perf_dates = item["performance_dates"].as_str().unwrap_or_default();
        let perf_count = item["performance_count"].as_i64().unwrap_or(1);
        let sort_order = item["sort_order"].as_i64().unwrap_or(0);
        if id.is_empty() || cert_id.is_empty() {
            continue;
        }
        let changes = tx.execute(
            "INSERT INTO soloist_certification_items (
                id, certification_id, personnel_id, dancer_name, work_role,
                performance_dates, performance_count, sort_order
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                personnel_id = excluded.personnel_id,
                dancer_name = excluded.dancer_name,
                work_role = excluded.work_role,
                performance_dates = excluded.performance_dates,
                performance_count = excluded.performance_count,
                sort_order = excluded.sort_order",
            params![id, cert_id, personnel_id, dancer_name, work_role, perf_dates, perf_count, sort_order],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    // 10g. Inspection Vault Documents
    for item in &bundle.inspection_vault_documents {
        total_received += 1;
        let id = item["id"].as_str().unwrap_or_default();
        let title = item["title"].as_str().unwrap_or_default();
        let category = item["category"].as_str().unwrap_or("OTRO");
        let document_date = item["document_date"].as_str().unwrap_or_default();
        let raw_personnel_id = item["personnel_id"].as_str();
        let personnel_id = match raw_personnel_id {
            Some(pid) if !pid.is_empty() => {
                let exists: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM personnel WHERE id = ?1)",
                    [pid],
                    |r| r.get(0),
                )?;
                if exists { Some(pid) } else { None }
            }
            _ => None,
        };
        let file_name = item["file_name"].as_str().unwrap_or_default();
        let file_type = item["file_type"].as_str().unwrap_or("application/octet-stream");
        let file_size_bytes = item["file_size_bytes"].as_i64().unwrap_or(0);
        let file_data = item["file_data"].as_str().unwrap_or("");
        let amount_ars = item["amount_ars"].as_f64();
        let supplier_vendor = item["supplier_vendor"].as_str();
        let notes = item["notes"].as_str();
        let created_at = item["created_at"].as_str().unwrap_or("");
        let updated_at = item["updated_at"].as_str().unwrap_or("");
        if id.is_empty() || file_data.is_empty() {
            continue;
        }
        let changes = tx.execute(
            "INSERT INTO inspection_vault_documents (
                id, title, category, document_date, personnel_id, file_name, file_type,
                file_size_bytes, file_data, amount_ars, supplier_vendor, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                category = excluded.category,
                document_date = excluded.document_date,
                personnel_id = excluded.personnel_id,
                file_name = excluded.file_name,
                file_type = excluded.file_type,
                file_size_bytes = excluded.file_size_bytes,
                file_data = excluded.file_data,
                amount_ars = excluded.amount_ars,
                supplier_vendor = excluded.supplier_vendor,
                notes = excluded.notes,
                updated_at = excluded.updated_at
             WHERE excluded.updated_at >= inspection_vault_documents.updated_at",
            params![
                id, title, category, document_date, personnel_id, file_name, file_type,
                file_size_bytes, file_data, amount_ars, supplier_vendor, notes, created_at, updated_at
            ],
        )?;
        if changes > 0 {
            total_applied += 1;
        }
    }

    tx.commit()?;

    Ok(SyncReportDto {
        success: true,
        remote_device: bundle.source_device_name.clone(),
        records_received: total_received,
        records_applied: total_applied,
        synced_at_iso: Utc::now().to_rfc3339(),
        message: format!(
            "Sincronización P2P exitosa con {}. {} registros recibidos, {} actualizados.",
            bundle.source_device_name, total_received, total_applied
        ),
    })
}

/// `id` del parte local con la fecha `date` si ya existe con otro `id`; si no, el mismo `id` recibido.
/// Queda anotado en `map` para traducir después los `schedule_id` de asistencia, ensayos y clases.
fn local_schedule_id(
    tx: &rusqlite::Transaction<'_>,
    map: &mut HashMap<String, String>,
    id: &str,
    date: &str,
) -> CoreResult<String> {
    if let Some(local) = map.get(id) {
        return Ok(local.clone());
    }
    if id.is_empty() || date.is_empty() {
        return Ok(id.to_string());
    }
    let local: Option<String> = tx
        .query_row("SELECT id FROM daily_schedules WHERE schedule_date = ?1", params![date], |r| r.get(0))
        .optional()?;
    let local = local.unwrap_or_else(|| id.to_string());
    map.insert(id.to_string(), local.clone());
    Ok(local)
}
