use crate::domain::{
    ActivityRestriction, AttendanceLogDto, AttendanceStatus, CreatePhysioSessionPayload,
    PhysiotherapySessionDto, RecordAttendancePayload,
};
use crate::dto::{
    BopMatrixCellDto, BopMatrixDayInfo, BopMatrixRowDto, MonthlyAgentSummaryDto,
    MonthlyAttendanceSheetDto, MonthlyBopMatrixDto, MonthlyDailyEntryDto,
};
use crate::error::{CoreError, CoreResult};
use chrono::{Datelike, NaiveDate, Weekday};
use rusqlite::{params, Connection};

/// Minutos desde medianoche para "08:45"
const CHECKIN_BENCHMARK_MINUTES: i32 = 8 * 60 + 45; // 525 min

fn parse_hhmm_to_min(hhmm: &str) -> Option<i32> {
    let (h_str, m_str) = hhmm.trim().split_once(':')?;
    let h: i32 = h_str.parse().ok()?;
    let m: i32 = m_str.parse().ok()?;
    Some(h * 60 + m)
}

pub fn list_attendance_for_date(
    conn: &Connection,
    schedule_date: &str,
) -> CoreResult<Vec<AttendanceLogDto>> {
    // 1. Obtener o crear schedule_id para la fecha
    let schedule_id: String = match conn.query_row(
        "SELECT id FROM daily_schedules WHERE schedule_date = ?1",
        params![schedule_date],
        |r| r.get(0),
    ) {
        Ok(id) => id,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let new_id = format!("SCHED_{}", schedule_date.replace('-', "_"));
            conn.execute(
                "INSERT OR IGNORE INTO daily_schedules (
                    id, schedule_date, day_of_week, regime_type, mandatory_checkin_time, nominal_start_time, nominal_end_time, footer_legal_clause
                ) VALUES (?1, ?2, 1, 'ORDINARY', '08:45', '09:00', '13:45', 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.')",
                params![new_id, schedule_date],
            )?;
            new_id
        }
        Err(e) => return Err(CoreError::Database(e)),
    };

    // 2. Traer todo el personal activo con su registro de asistencia si existe
    let mut stmt = conn.prepare(
        "SELECT 
            p.id,
            p.first_name,
            p.last_name,
            p.nickname,
            p.legajo,
            p.role,
            c.name as category_name,
            a.id as log_id,
            COALESCE(a.expected_checkin_time, '08:45'),
            a.actual_checkin_time,
            COALESCE(
                a.operational_status,
                CASE 
                    WHEN ml.id IS NOT NULL THEN 'MEDICAL_LEAVE'
                    WHEN p.is_retired = 1 THEN 'RETIRED_EXEMPT'
                    ELSE COALESCE(a.attendance_status, 'PENDING')
                END
            ) as op_status,
            COALESCE(a.is_in_class, CASE WHEN p.is_retired = 1 OR ml.id IS NOT NULL THEN 0 ELSE 1 END),
            COALESCE(a.minutes_late, 0),
            COALESCE(a.remarks, ml.remarks),
            COALESCE(a.certificate_file_name, ml.certificate_file_name),
            COALESCE(a.certificate_data, ml.certificate_data),
            COALESCE(a.medical_diagnosis, ml.diagnosis),
            COALESCE(a.doctor_details, ml.doctor_name),
            COALESCE(a.leave_start_date, ml.start_date),
            COALESCE(a.leave_end_date, ml.end_date),
            a.physio_notes,
            p.current_physio_diagnosis,
            p.physio_tracking_notes,
            COALESCE(a.recorded_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            COALESCE(p.is_retired, 0) as is_retired,
            p.retirement_notes,
            p.specific_position
         FROM personnel p
         JOIN personnel_categories c ON p.category_id = c.id
         LEFT JOIN attendance_logs a ON a.personnel_id = p.id AND a.schedule_id = ?1
         LEFT JOIN medical_leave_records ml ON ml.personnel_id = p.id 
             AND ml.start_date <= ?2 AND ml.end_date >= ?2
         WHERE p.is_active = 1
         ORDER BY 
            CASE 
                WHEN p.is_retired = 1 THEN 5
                WHEN p.role = 'INSPECTOR' THEN 0
                WHEN p.role = 'MASTER' THEN 1
                WHEN p.role = 'PIANIST' THEN 2
                WHEN p.role = 'DANCER' THEN 3
                ELSE 4 
            END,
            p.last_name ASC,
            p.first_name ASC",
    )?;

    let rows = stmt.query_map(params![schedule_id, schedule_date], |r| {
        let personnel_id: String = r.get(0)?;
        let first_name: String = r.get(1)?;
        let last_name: String = r.get(2)?;
        let nickname: Option<String> = r.get(3)?;
        let legajo: Option<String> = r.get(4)?;
        let role: String = r.get(5)?;
        let category_name: String = r.get(6)?;
        let log_id: Option<String> = r.get(7)?;
        let expected_checkin_time: String = r.get(8)?;
        let actual_checkin_time: Option<String> = r.get(9)?;
        let op_status_str: String = r.get(10)?;
        let is_in_class_raw: i32 = r.get(11)?;
        let minutes_late: i32 = r.get(12)?;
        let remarks: Option<String> = r.get(13)?;
        let certificate_file_name: Option<String> = r.get(14)?;
        let certificate_data: Option<String> = r.get(15)?;
        let medical_diagnosis: Option<String> = r.get(16)?;
        let doctor_details: Option<String> = r.get(17)?;
        let leave_start_date: Option<String> = r.get(18)?;
        let leave_end_date: Option<String> = r.get(19)?;
        let physio_notes: Option<String> = r.get(20)?;
        let current_physio_diagnosis: Option<String> = r.get(21)?;
        let physio_tracking_notes: Option<String> = r.get(22)?;
        let recorded_at: String = r.get(23)?;
        let is_retired_raw: i32 = r.get(24)?;
        let retirement_notes: Option<String> = r.get(25)?;
        let _specific_position: Option<String> = r.get(26)?;

        let dancer_name = match nickname {
            Some(ref n) if !n.trim().is_empty() => format!("{} ({})", last_name, n),
            _ => format!("{}, {}", last_name, first_name),
        };

        Ok(AttendanceLogDto {
            id: log_id.unwrap_or_else(|| format!("ATT_DRAFT_{}_{}", schedule_id, personnel_id)),
            schedule_id: schedule_id.clone(),
            schedule_date: schedule_date.to_string(),
            personnel_id,
            dancer_name,
            legajo,
            role,
            category_name,
            expected_checkin_time,
            actual_checkin_time,
            attendance_status: AttendanceStatus::from_operational_str(&op_status_str),
            is_in_class: is_in_class_raw != 0,
            minutes_late,
            remarks,
            certificate_file_name,
            certificate_data,
            medical_diagnosis,
            doctor_details,
            leave_start_date,
            leave_end_date,
            is_retired: is_retired_raw == 1,
            retirement_notes,
            physio_notes,
            current_physio_diagnosis,
            physio_tracking_notes,
            recorded_at,
        })
    })?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(row?);
    }
    Ok(logs)
}

pub fn record_attendance(
    conn: &Connection,
    payload: RecordAttendancePayload,
) -> CoreResult<AttendanceLogDto> {
    // 1. Obtener o crear schedule_id
    let schedule_id: String = match conn.query_row(
        "SELECT id FROM daily_schedules WHERE schedule_date = ?1",
        params![payload.schedule_date],
        |r| r.get(0),
    ) {
        Ok(id) => id,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let new_id = format!("SCHED_{}", payload.schedule_date.replace('-', "_"));
            conn.execute(
                "INSERT OR IGNORE INTO daily_schedules (
                    id, schedule_date, day_of_week, regime_type, mandatory_checkin_time, nominal_start_time, nominal_end_time, footer_legal_clause
                ) VALUES (?1, ?2, 1, 'ORDINARY', '08:45', '09:00', '13:45', 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.')",
                params![new_id, payload.schedule_date],
            )?;
            new_id
        }
        Err(e) => return Err(CoreError::Database(e)),
    };

    // 2. Calcular llegada tarde
    let minutes_late = match payload.attendance_status {
        AttendanceStatus::Late => {
            if let Some(explicit) = payload.minutes_late {
                explicit.max(1)
            } else if let Some(ref checkin) = payload.actual_checkin_time {
                if let Some(min) = parse_hhmm_to_min(checkin) {
                    (min - CHECKIN_BENCHMARK_MINUTES).max(1)
                } else {
                    10
                }
            } else {
                10
            }
        }
        _ => 0,
    };

    let actual_checkin = match payload.attendance_status {
        AttendanceStatus::Present | AttendanceStatus::NotInClass | AttendanceStatus::InPhysiotherapy | AttendanceStatus::TaskAdjustment => {
            payload.actual_checkin_time.or(Some("08:45".to_string()))
        }
        AttendanceStatus::Late => payload.actual_checkin_time.or(Some("08:55".to_string())),
        _ => payload.actual_checkin_time,
    };

    // Regla: inasistencias justificadas (IJ) con tope por mes y por año
    if payload.attendance_status == AttendanceStatus::JustifiedAbsence {
        check_ij_limits(conn, &payload.schedule_date, &payload.personnel_id)?;
    }

    let is_in_class_val = if payload.attendance_status.is_in_class() { 1 } else { 0 };
    let log_id = format!("ATT_{}_{}", schedule_id, payload.personnel_id);

    conn.execute(
        "INSERT INTO attendance_logs (
            id, schedule_id, personnel_id, expected_checkin_time, actual_checkin_time,
            attendance_status, operational_status, is_in_class, minutes_late, remarks,
            certificate_file_name, certificate_data, medical_diagnosis, doctor_details,
            leave_start_date, leave_end_date, physio_notes, recorded_at
        ) VALUES (
            ?1, ?2, ?3, '08:45', ?4,
            ?5, ?6, ?7, ?8, ?9,
            ?10, ?11, ?12, ?13,
            ?14, ?15, ?16, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        )
        ON CONFLICT(schedule_id, personnel_id) DO UPDATE SET
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
            recorded_at = excluded.recorded_at",
        params![
            log_id,
            schedule_id,
            payload.personnel_id,
            actual_checkin,
            payload.attendance_status.to_db_attendance_status(),
            payload.attendance_status.as_operational_str(),
            is_in_class_val,
            minutes_late,
            payload.remarks,
            payload.certificate_file_name,
            payload.certificate_data,
            payload.medical_diagnosis,
            payload.doctor_details,
            payload.leave_start_date,
            payload.leave_end_date,
            payload.physio_notes,
        ],
    )?;

    // Si es licencia médica, guardar en la tabla permanente medical_leave_records
    if payload.attendance_status == AttendanceStatus::MedicalLeave {
        let start_date = payload
            .leave_start_date
            .as_deref()
            .unwrap_or(&payload.schedule_date);
        let end_date = payload
            .leave_end_date
            .as_deref()
            .unwrap_or(start_date);
        let rec_id = format!(
            "MED_LEAVE_{}_{}",
            payload.personnel_id,
            start_date.replace('-', "_")
        );
        conn.execute(
            "INSERT INTO medical_leave_records (
                id, personnel_id, start_date, end_date, diagnosis, doctor_name, certificate_file_name, certificate_data, remarks
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                end_date = excluded.end_date,
                diagnosis = excluded.diagnosis,
                doctor_name = excluded.doctor_name,
                certificate_file_name = COALESCE(excluded.certificate_file_name, medical_leave_records.certificate_file_name),
                certificate_data = COALESCE(excluded.certificate_data, medical_leave_records.certificate_data),
                remarks = excluded.remarks",
            params![
                rec_id,
                payload.personnel_id,
                start_date,
                end_date,
                payload.medical_diagnosis,
                payload.doctor_details,
                payload.certificate_file_name,
                payload.certificate_data,
                payload.remarks,
            ],
        )?;
    }

    // Si se marcó formalmente en estado jubilatorio
    if payload.attendance_status == AttendanceStatus::RetiredExempt {
        conn.execute(
            "UPDATE personnel SET is_retired = 1 WHERE id = ?1",
            params![payload.personnel_id],
        )?;
    }

    // Si se especificó actualización de la ficha permanente de fisioterapia
    if let Some(ref note) = payload.update_permanent_physio_tracking {
        conn.execute(
            "UPDATE personnel SET 
                physio_tracking_notes = CASE 
                    WHEN physio_tracking_notes IS NULL OR physio_tracking_notes = '' THEN ?1 
                    ELSE physio_tracking_notes || '\n[' || strftime('%Y-%m-%d', 'now') || '] ' || ?1 
                END
             WHERE id = ?2",
            params![note, payload.personnel_id],
        )?;
    }

    let list = list_attendance_for_date(conn, &payload.schedule_date)?;
    list.into_iter()
        .find(|i| i.personnel_id == payload.personnel_id)
        .ok_or_else(|| CoreError::EntityNotFound {
            entity: "personnel",
            id: payload.personnel_id,
        })
}

pub fn create_physiotherapy_session(
    conn: &Connection,
    payload: CreatePhysioSessionPayload,
) -> CoreResult<PhysiotherapySessionDto> {
    let session_id = format!("PHYS_{}_{}", payload.session_date.replace('-', ""), payload.personnel_id);
    let session_time = payload.session_time.unwrap_or_else(|| "09:15".to_string());
    let doctor_name = payload.doctor_name.unwrap_or_else(|| "Dra. de Fisioterapia".to_string());

    conn.execute(
        "INSERT INTO physiotherapy_sessions (
            id, personnel_id, session_date, session_time, doctor_name, diagnosis,
            treatment_applied, evolution_notes, activity_restriction, attachment_name, attachment_data,
            created_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6,
            ?7, ?8, ?9, ?10, ?11,
            strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        )
        ON CONFLICT(id) DO UPDATE SET
            session_time = excluded.session_time,
            doctor_name = excluded.doctor_name,
            diagnosis = excluded.diagnosis,
            treatment_applied = excluded.treatment_applied,
            evolution_notes = excluded.evolution_notes,
            activity_restriction = excluded.activity_restriction,
            attachment_name = COALESCE(excluded.attachment_name, physiotherapy_sessions.attachment_name),
            attachment_data = COALESCE(excluded.attachment_data, physiotherapy_sessions.attachment_data)",
        params![
            session_id,
            payload.personnel_id,
            payload.session_date,
            session_time,
            doctor_name,
            payload.diagnosis,
            payload.treatment_applied,
            payload.evolution_notes,
            payload.activity_restriction.as_db_str(),
            payload.attachment_name,
            payload.attachment_data,
        ],
    )?;

    // Actualizar ficha permanente del bailarín
    if let Some(ref diag) = payload.diagnosis {
        conn.execute(
            "UPDATE personnel SET current_physio_diagnosis = ?1 WHERE id = ?2",
            params![diag, payload.personnel_id],
        )?;
    }

    if let Some(ref evol) = payload.evolution_notes {
        let entry = format!("[{}] {}", payload.session_date, evol);
        conn.execute(
            "UPDATE personnel SET 
                physio_tracking_notes = CASE 
                    WHEN physio_tracking_notes IS NULL OR physio_tracking_notes = '' THEN ?1 
                    ELSE physio_tracking_notes || '\n' || ?1 
                END
             WHERE id = ?2",
            params![entry, payload.personnel_id],
        )?;
    }

    // Si se solicitó sincronizar la asistencia del día
    if payload.sync_attendance_status {
        let att_payload = RecordAttendancePayload {
            schedule_date: payload.session_date.clone(),
            personnel_id: payload.personnel_id.clone(),
            attendance_status: AttendanceStatus::InPhysiotherapy,
            actual_checkin_time: Some(session_time.clone()),
            minutes_late: None,
            remarks: Some(format!("Fisioterapia: {}", payload.treatment_applied)),
            certificate_file_name: payload.attachment_name.clone(),
            certificate_data: payload.attachment_data.clone(),
            medical_diagnosis: payload.diagnosis.clone(),
            doctor_details: Some(doctor_name.clone()),
            leave_start_date: None,
            leave_end_date: None,
            physio_notes: payload.evolution_notes.clone(),
            update_permanent_physio_tracking: None,
        };
        let _ = record_attendance(conn, att_payload);
    }

    // Traer el DTO insertado
    let dancer_name: String = conn.query_row(
        "SELECT CASE 
            WHEN nickname IS NOT NULL AND nickname != '' THEN last_name || ' (' || nickname || ')' 
            ELSE last_name || ', ' || first_name 
         END FROM personnel WHERE id = ?1",
        params![payload.personnel_id],
        |r| r.get(0),
    )?;

    Ok(PhysiotherapySessionDto {
        id: session_id,
        personnel_id: payload.personnel_id,
        dancer_name,
        session_date: payload.session_date,
        session_time,
        doctor_name,
        diagnosis: payload.diagnosis,
        treatment_applied: payload.treatment_applied,
        evolution_notes: payload.evolution_notes,
        activity_restriction: payload.activity_restriction,
        attachment_name: payload.attachment_name,
        attachment_data: payload.attachment_data,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn list_physiotherapy_sessions(
    conn: &Connection,
    date_filter: Option<&str>,
    personnel_filter: Option<&str>,
) -> CoreResult<Vec<PhysiotherapySessionDto>> {
    let mut sql = String::from(
        "SELECT 
            s.id,
            s.personnel_id,
            CASE 
                WHEN p.nickname IS NOT NULL AND p.nickname != '' THEN p.last_name || ' (' || p.nickname || ')' 
                ELSE p.last_name || ', ' || p.first_name 
            END as dancer_name,
            s.session_date,
            s.session_time,
            s.doctor_name,
            s.diagnosis,
            s.treatment_applied,
            s.evolution_notes,
            s.activity_restriction,
            s.attachment_name,
            s.attachment_data,
            s.created_at
         FROM physiotherapy_sessions s
         JOIN personnel p ON s.personnel_id = p.id
         WHERE 1=1",
    );

    let mut param_vals: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(date) = date_filter {
        sql.push_str(" AND s.session_date = ?");
        param_vals.push(Box::new(date.to_string()));
    }
    if let Some(p_id) = personnel_filter {
        sql.push_str(" AND s.personnel_id = ?");
        param_vals.push(Box::new(p_id.to_string()));
    }
    sql.push_str(" ORDER BY s.session_date DESC, s.session_time DESC");

    let params_refs: Vec<&dyn rusqlite::ToSql> = param_vals.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_refs.as_slice(), |r| {
        let restr_str: String = r.get(9)?;
        Ok(PhysiotherapySessionDto {
            id: r.get(0)?,
            personnel_id: r.get(1)?,
            dancer_name: r.get(2)?,
            session_date: r.get(3)?,
            session_time: r.get(4)?,
            doctor_name: r.get(5)?,
            diagnosis: r.get(6)?,
            treatment_applied: r.get(7)?,
            evolution_notes: r.get(8)?,
            activity_restriction: ActivityRestriction::from_db_str(&restr_str),
            attachment_name: r.get(10)?,
            attachment_data: r.get(11)?,
            created_at: r.get(12)?,
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn get_agent_monthly_attendance_sheet(
    conn: &Connection,
    personnel_id: &str,
    year: i32,
    month: u32,
) -> CoreResult<MonthlyAttendanceSheetDto> {
    let (legajo, first_name, last_name, role_str, category_name): (
        Option<String>,
        String,
        String,
        String,
        String,
    ) = conn
        .query_row(
            "SELECT p.legajo, p.first_name, p.last_name, p.role, c.name
             FROM personnel p
             JOIN personnel_categories c ON c.id = p.category_id
             WHERE p.id = ?1;",
            [personnel_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|_| CoreError::EntityNotFound {
            entity: "personnel",
            id: personnel_id.to_string(),
        })?;

    let month_name = match month {
        1 => "ENERO",
        2 => "FEBRERO",
        3 => "MARZO",
        4 => "ABRIL",
        5 => "MAYO",
        6 => "JUNIO",
        7 => "JULIO",
        8 => "AGOSTO",
        9 => "SEPTIEMBRE",
        10 => "OCTUBRE",
        11 => "NOVIEMBRE",
        12 => "DICIEMBRE",
        _ => "MES",
    }
    .to_string();

    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if is_leap {
                29
            } else {
                28
            }
        }
        _ => 30,
    };

    let month_prefix = format!("{:04}-{:02}-%", year, month);
    let mut log_stmt = conn.prepare(
        "SELECT ds.schedule_date, a.expected_checkin_time, a.actual_checkin_time,
                a.operational_status, a.is_in_class, a.minutes_late, a.remarks,
                a.medical_diagnosis, a.certificate_file_name
         FROM attendance_logs a
         JOIN daily_schedules ds ON ds.id = a.schedule_id
         WHERE a.personnel_id = ?1 AND ds.schedule_date LIKE ?2;",
    )?;

    struct LogRecord {
        expected_checkin_time: String,
        actual_checkin_time: Option<String>,
        status: AttendanceStatus,
        is_in_class: bool,
        minutes_late: i32,
        remarks: Option<String>,
        medical_diagnosis: Option<String>,
        has_cert: bool,
    }

    let mut logs_map: std::collections::HashMap<String, LogRecord> =
        std::collections::HashMap::new();
    let log_rows = log_stmt.query_map(params![personnel_id, month_prefix], |r| {
        let date: String = r.get(0)?;
        let expected: String = r.get(1)?;
        let actual: Option<String> = r.get(2)?;
        let st_str: String = r.get(3)?;
        let in_class: i32 = r.get(4)?;
        let late_mins: i32 = r.get(5)?;
        let remarks: Option<String> = r.get(6)?;
        let diag: Option<String> = r.get(7)?;
        let cert: Option<String> = r.get(8)?;

        Ok((
            date,
            LogRecord {
                expected_checkin_time: expected,
                actual_checkin_time: actual,
                status: AttendanceStatus::from_operational_str(&st_str),
                is_in_class: in_class == 1,
                minutes_late: late_mins,
                remarks,
                medical_diagnosis: diag,
                has_cert: cert.is_some(),
            },
        ))
    })?;

    for row in log_rows {
        let (dt, rec) = row?;
        logs_map.insert(dt, rec);
    }

    let start_of_month = format!("{:04}-{:02}-01", year, month);
    let end_of_month = format!("{:04}-{:02}-{:02}", year, month, days_in_month);
    let mut ml_stmt = conn.prepare(
        "SELECT start_date, end_date, diagnosis, certificate_file_name, remarks
         FROM medical_leave_records
         WHERE personnel_id = ?1 AND start_date <= ?2 AND end_date >= ?3;",
    )?;

    struct MLRecord {
        start_date: String,
        end_date: String,
        diagnosis: Option<String>,
        has_cert: bool,
        remarks: Option<String>,
    }

    let ml_rows = ml_stmt.query_map(params![personnel_id, end_of_month, start_of_month], |r| {
        let s: String = r.get(0)?;
        let e: String = r.get(1)?;
        let d: Option<String> = r.get(2)?;
        let c: Option<String> = r.get(3)?;
        let rem: Option<String> = r.get(4)?;
        Ok(MLRecord {
            start_date: s,
            end_date: e,
            diagnosis: d,
            has_cert: c.is_some(),
            remarks: rem,
        })
    })?;

    let mut ml_list = Vec::new();
    for row in ml_rows {
        ml_list.push(row?);
    }

    let mut entries = Vec::new();
    let mut total_working_days = 0;
    let mut days_present = 0;
    let mut days_late = 0;
    let mut total_late_minutes = 0;
    let mut days_medical_leave = 0;
    let mut days_physiotherapy = 0;
    let mut days_not_in_class = 0;
    let mut days_absent = 0;
    let mut days_not_summoned = 0;

    for day in 1..=days_in_month {
        let date_str = format!("{:04}-{:02}-{:02}", year, month, day);
        let naive_date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let weekday = naive_date.weekday();
        let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
        let day_of_week = match weekday {
            Weekday::Mon => "LUNES",
            Weekday::Tue => "MARTES",
            Weekday::Wed => "MIÉRCOLES",
            Weekday::Thu => "JUEVES",
            Weekday::Fri => "VIERNES",
            Weekday::Sat => "SÁBADO",
            Weekday::Sun => "DOMINGO",
        }
        .to_string();

        if !is_weekend {
            total_working_days += 1;
        }

        if let Some(log) = logs_map.get(&date_str) {
            match log.status {
                AttendanceStatus::Present => days_present += 1,
                AttendanceStatus::Late => {
                    days_present += 1;
                    days_late += 1;
                    total_late_minutes += log.minutes_late;
                }
                AttendanceStatus::NotInClass => {
                    days_present += 1;
                    days_not_in_class += 1;
                }
                AttendanceStatus::InPhysiotherapy => {
                    days_present += 1;
                    days_physiotherapy += 1;
                }
                // Adecuación de tareas: cumple jornada en otra área, cuenta como presente
                AttendanceStatus::TaskAdjustment => days_present += 1,
                // Sin parte (no convocado): no es falta ni se cuenta en el presentismo
                AttendanceStatus::NoPart => days_not_summoned += 1,
                AttendanceStatus::MedicalLeave => days_medical_leave += 1,
                AttendanceStatus::Absent => days_absent += 1,
                _ => {}
            }

            entries.push(MonthlyDailyEntryDto {
                date: date_str,
                day_number: day,
                day_of_week,
                is_weekend,
                expected_checkin_time: log.expected_checkin_time.clone(),
                actual_checkin_time: log.actual_checkin_time.clone(),
                minutes_late: log.minutes_late,
                status: log.status,
                is_in_class: log.is_in_class,
                remarks: log.remarks.clone(),
                medical_diagnosis: log.medical_diagnosis.clone(),
                has_certificate: log.has_cert,
            });
        } else if let Some(ml) = ml_list
            .iter()
            .find(|m| m.start_date <= date_str && m.end_date >= date_str)
        {
            if !is_weekend {
                days_medical_leave += 1;
            }
            entries.push(MonthlyDailyEntryDto {
                date: date_str,
                day_number: day,
                day_of_week,
                is_weekend,
                expected_checkin_time: "08:45".to_string(),
                actual_checkin_time: None,
                minutes_late: 0,
                status: AttendanceStatus::MedicalLeave,
                is_in_class: false,
                remarks: ml.remarks.clone(),
                medical_diagnosis: ml.diagnosis.clone(),
                has_certificate: ml.has_cert,
            });
        } else {
            entries.push(MonthlyDailyEntryDto {
                date: date_str,
                day_number: day,
                day_of_week,
                is_weekend,
                expected_checkin_time: "08:45".to_string(),
                actual_checkin_time: None,
                minutes_late: 0,
                status: AttendanceStatus::Pending,
                is_in_class: false,
                remarks: if is_weekend {
                    Some("Franco / Descanso".to_string())
                } else {
                    None
                },
                medical_diagnosis: None,
                has_certificate: false,
            });
        }
    }

    let expected_days = total_working_days - days_not_summoned;
    let presence_percentage = if expected_days > 0 {
        ((days_present as f32) / (expected_days as f32)) * 100.0
    } else {
        100.0
    };

    Ok(MonthlyAttendanceSheetDto {
        personnel_id: personnel_id.to_string(),
        legajo,
        first_name,
        last_name,
        role: role_str,
        category_name,
        year,
        month,
        month_name,
        total_working_days,
        days_present,
        days_late,
        total_late_minutes,
        days_medical_leave,
        days_physiotherapy,
        days_not_in_class,
        days_absent,
        presence_percentage,
        entries,
    })
}

pub fn list_monthly_agents_summary(
    conn: &Connection,
    year: i32,
    month: u32,
) -> CoreResult<Vec<MonthlyAgentSummaryDto>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM personnel WHERE is_active = 1 ORDER BY last_name ASC, first_name ASC;",
    )?;
    let p_rows = stmt.query_map([], |r| r.get::<_, String>(0))?;

    let mut out = Vec::new();
    for p_id in p_rows {
        let id = p_id?;
        if let Ok(sheet) = get_agent_monthly_attendance_sheet(conn, &id, year, month) {
            out.push(MonthlyAgentSummaryDto {
                personnel_id: sheet.personnel_id,
                legajo: sheet.legajo,
                dancer_name: format!("{}, {}", sheet.last_name, sheet.first_name),
                role: sheet.role,
                days_present: sheet.days_present,
                days_late: sheet.days_late,
                total_late_minutes: sheet.total_late_minutes,
                days_medical_leave: sheet.days_medical_leave,
                days_physio: sheet.days_physiotherapy,
                days_absent: sheet.days_absent,
                presence_percentage: sheet.presence_percentage,
            });
        }
    }
    Ok(out)
}

// =============================================================================
// PLANILLA MATRIZ MENSUAL DE AUSENTISMO BOP (EXCEL DE CARINA LION)
// =============================================================================

pub fn get_monthly_bop_matrix(
    conn: &Connection,
    year: i32,
    month: u32,
) -> CoreResult<MonthlyBopMatrixDto> {
    let month_name = match month {
        1 => "ENERO",
        2 => "FEBRERO",
        3 => "MARZO",
        4 => "ABRIL",
        5 => "MAYO",
        6 => "JUNIO",
        7 => "JULIO",
        8 => "AGOSTO",
        9 => "SEPTIEMBRE",
        10 => "OCTUBRE",
        11 => "NOVIEMBRE",
        12 => "DICIEMBRE",
        _ => "MES",
    }
    .to_string();

    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if is_leap {
                29
            } else {
                28
            }
        }
        _ => 30,
    };

    // 1. Obtener anotaciones de días (ej. ENSAYO SUSPENDIDO, FERIADO)
    let month_prefix = format!("{:04}-{:02}-%", year, month);
    let mut sched_stmt = conn.prepare(
        "SELECT schedule_date, day_annotation FROM daily_schedules WHERE schedule_date LIKE ?1;",
    )?;
    let mut day_annotations_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let sched_rows = sched_stmt.query_map([&month_prefix], |r| {
        let date: String = r.get(0)?;
        let annot: Option<String> = r.get(1)?;
        Ok((date, annot))
    })?;
    for row in sched_rows {
        let (dt, annot_opt) = row?;
        if let Some(a) = annot_opt {
            if !a.trim().is_empty() {
                day_annotations_map.insert(dt, a);
            }
        }
    }

    // 2. Construir cabecera de días
    let mut days_header = Vec::new();
    for day in 1..=days_in_month {
        let date_str = format!("{:04}-{:02}-{:02}", year, month, day);
        let naive_date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let weekday = naive_date.weekday();
        let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
        let day_name = match weekday {
            Weekday::Mon => "L",
            Weekday::Tue => "M",
            Weekday::Wed => "M",
            Weekday::Thu => "J",
            Weekday::Fri => "V",
            Weekday::Sat => "S",
            Weekday::Sun => "D",
        }
        .to_string();

        let day_annot = day_annotations_map.get(&date_str).cloned();

        days_header.push(BopMatrixDayInfo {
            day_number: day,
            date: date_str,
            day_name,
            is_weekend,
            day_annotation: day_annot,
        });
    }

    // 3. Obtener logs de asistencia del mes
    let mut att_stmt = conn.prepare(
        "SELECT a.personnel_id, ds.schedule_date, a.operational_status, a.minutes_late, a.remarks
         FROM attendance_logs a
         JOIN daily_schedules ds ON ds.id = a.schedule_id
         WHERE ds.schedule_date LIKE ?1;",
    )?;

    struct CellData {
        op_status: String,
        minutes_late: i32,
        remarks: Option<String>,
    }

    let mut att_map: std::collections::HashMap<(String, String), CellData> =
        std::collections::HashMap::new();
    let att_rows = att_stmt.query_map([&month_prefix], |r| {
        let p_id: String = r.get(0)?;
        let date: String = r.get(1)?;
        let op_st: String = r.get(2)?;
        let mins: i32 = r.get(3)?;
        let rem: Option<String> = r.get(4)?;
        Ok(((p_id, date), CellData { op_status: op_st, minutes_late: mins, remarks: rem }))
    })?;
    for row in att_rows {
        let (key, val) = row?;
        att_map.insert(key, val);
    }

    // 4. Obtener licencias médicas del mes
    let start_of_month = format!("{:04}-{:02}-01", year, month);
    let end_of_month = format!("{:04}-{:02}-{:02}", year, month, days_in_month);
    let mut ml_stmt = conn.prepare(
        "SELECT personnel_id, start_date, end_date, remarks
         FROM medical_leave_records
         WHERE start_date <= ?1 AND end_date >= ?2;",
    )?;
    struct MlSpan {
        personnel_id: String,
        start_date: String,
        end_date: String,
        remarks: Option<String>,
    }
    let ml_rows = ml_stmt.query_map([&end_of_month, &start_of_month], |r| {
        Ok(MlSpan {
            personnel_id: r.get(0)?,
            start_date: r.get(1)?,
            end_date: r.get(2)?,
            remarks: r.get(3)?,
        })
    })?;
    let mut ml_list = Vec::new();
    for row in ml_rows {
        ml_list.push(row?);
    }

    // 5. Obtener lista de personal activo (orden alfabético por Apellido, Nombre)
    let mut p_stmt = conn.prepare(
        "SELECT id, legajo, first_name, last_name, nickname, role
         FROM personnel
         WHERE is_active = 1
         ORDER BY 
            CASE 
                WHEN role = 'DANCER' THEN 0
                WHEN role = 'MASTER' THEN 1
                WHEN role = 'PIANIST' THEN 2
                WHEN role = 'INSPECTOR' THEN 3
                ELSE 4
            END,
            last_name ASC, first_name ASC;",
    )?;

    let p_rows = p_stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, String>(5)?,
        ))
    })?;

    let mut rows = Vec::new();
    let mut nro = 1;
    for p_res in p_rows {
        let (p_id, legajo, first_name, last_name, _nickname, role) = p_res?;
        let dancer_name = format!("{}, {}", last_name, first_name);

        let mut cells = Vec::new();
        let mut total_p = 0;
        let mut total_ei = 0;
        let mut total_sp = 0;
        let mut total_other = 0;

        for day in 1..=days_in_month {
            let date_str = format!("{:04}-{:02}-{:02}", year, month, day);
            let naive_date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            let weekday = naive_date.weekday();
            let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;

            let (code, mins_late, remarks) = if let Some(cell) = att_map.get(&(p_id.clone(), date_str.clone())) {
                let c = match cell.op_status.as_str() {
                    "PRESENT" | "ON_TIME" => "P",
                    "EI" => "EI",
                    "SP" => "SP",
                    "S/P" | "NO_PART" | "SIN_PARTE" => "S/P",
                    "LC" => "LC",
                    "ART" => "ART",
                    "LA" => "LA",
                    "ES" => "ES",
                    "IJ" => "IJ",
                    "MEDICAL_LEAVE" => "EI",
                    "TASK_ADJUSTMENT" | "AT" => "AT",
                    "JUSTIFIED_ABSENCE" => "IJ",
                    "NOT_IN_CLASS" | "IN_PHYSIOTHERAPY" => "P",
                    "LATE" => "P",
                    "ABSENT" | "ABSENT_UNJUSTIFIED" => "SP",
                    "ABSENT_JUSTIFIED" => "LC",
                    _ => "",
                };
                (c.to_string(), cell.minutes_late, cell.remarks.clone())
            } else if let Some(ml) = ml_list.iter().find(|m| m.personnel_id == p_id && m.start_date <= date_str && m.end_date >= date_str) {
                ("EI".to_string(), 0, ml.remarks.clone())
            } else {
                ("".to_string(), 0, None)
            };

            match code.as_str() {
                "P" => total_p += 1,
                "EI" => total_ei += 1,
                "SP" => total_sp += 1,
                "" | "S/P" => {}
                _ => total_other += 1,
            }

            cells.push(BopMatrixCellDto {
                day_number: day,
                date: date_str,
                is_weekend,
                code,
                minutes_late: mins_late,
                remarks,
            });
        }

        rows.push(BopMatrixRowDto {
            nro,
            personnel_id: p_id,
            dancer_name,
            legajo,
            role,
            cells,
            total_p,
            total_ei,
            total_sp,
            total_other,
        });
        nro += 1;
    }

    Ok(MonthlyBopMatrixDto {
        year,
        month,
        month_name,
        days_in_month,
        days_header,
        rows,
    })
}

/// Tope de inasistencias justificadas (IJ) por agente.
pub const IJ_MAX_PER_MONTH: i64 = 2;
pub const IJ_MAX_PER_YEAR: i64 = 10;

/// Cuenta las IJ de un agente en un período (`prefix` = "YYYY-MM" o "YYYY"),
/// sin contar la propia fecha `except_date`.
pub fn count_ij(conn: &Connection, personnel_id: &str, prefix: &str, except_date: &str) -> CoreResult<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM attendance_logs a
         JOIN daily_schedules d ON d.id = a.schedule_id
         WHERE a.personnel_id = ?1 AND a.operational_status IN ('IJ', 'JUSTIFIED_ABSENCE')
           AND substr(d.schedule_date, 1, ?2) = ?3 AND d.schedule_date <> ?4",
        params![personnel_id, prefix.len() as i64, prefix, except_date],
        |r| r.get(0),
    )?)
}

fn check_ij_limits(conn: &Connection, date: &str, personnel_id: &str) -> CoreResult<()> {
    let month = count_ij(conn, personnel_id, &date[..7], date)?;
    if month >= IJ_MAX_PER_MONTH {
        return Err(CoreError::ValidationFailed(format!(
            "Tope de IJ alcanzado: ya tiene {month} inasistencias justificadas en el mes (máximo {IJ_MAX_PER_MONTH} por mes)."
        )));
    }
    let year = count_ij(conn, personnel_id, &date[..4], date)?;
    if year >= IJ_MAX_PER_YEAR {
        return Err(CoreError::ValidationFailed(format!(
            "Tope de IJ alcanzado: ya tiene {year} inasistencias justificadas en el año (máximo {IJ_MAX_PER_YEAR} por año)."
        )));
    }
    Ok(())
}

pub fn update_bop_matrix_cell(
    conn: &Connection,
    date: &str,
    personnel_id: &str,
    code: &str,
    remarks: Option<&str>,
) -> CoreResult<()> {
    // 1. Obtener o crear schedule_id
    let schedule_id: String = match conn.query_row(
        "SELECT id FROM daily_schedules WHERE schedule_date = ?1",
        [date],
        |r| r.get(0),
    ) {
        Ok(id) => id,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let new_id = format!("SCHED_{}", date.replace('-', "_"));
            conn.execute(
                "INSERT OR IGNORE INTO daily_schedules (
                    id, schedule_date, day_of_week, regime_type, mandatory_checkin_time, nominal_start_time, nominal_end_time, footer_legal_clause
                ) VALUES (?1, ?2, 1, 'ORDINARY', '08:45', '09:00', '13:45', 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.')",
                params![new_id, date],
            )?;
            new_id
        }
        Err(e) => return Err(CoreError::Database(e)),
    };

    let log_id = format!("ATT_{}_{}", schedule_id, personnel_id);

    if code.trim().is_empty() {
        conn.execute(
            "DELETE FROM attendance_logs WHERE schedule_id = ?1 AND personnel_id = ?2;",
            params![schedule_id, personnel_id],
        )?;
        return Ok(());
    }

    let code_upper = code.trim().to_uppercase();
    let (op_status, att_status, checkin, in_class) = match code_upper.as_str() {
        "P" => ("PRESENT", "ON_TIME", Some("08:45"), 1),
        "EI" => ("EI", "MEDICAL_LEAVE", None, 0),
        "SP" => ("SP", "ABSENT_UNJUSTIFIED", None, 0),
        "S/P" | "NO_PART" | "SIN_PARTE" => ("NO_PART", "ABSENT_JUSTIFIED", None, 0),
        "LC" => ("LC", "ABSENT_JUSTIFIED", None, 0),
        "ART" => ("ART", "MEDICAL_LEAVE", None, 0),
        "LA" => ("LA", "ABSENT_JUSTIFIED", None, 0),
        "ES" => ("ES", "ABSENT_JUSTIFIED", None, 0),
        "AT" | "ADECUACION" => ("TASK_ADJUSTMENT", "ON_TIME", Some("08:45"), 0),
        "IJ" => {
            check_ij_limits(conn, date, personnel_id)?;
            ("JUSTIFIED_ABSENCE", "ABSENT_JUSTIFIED", None, 0)
        }
        other => (other, "PENDING", None, 1),
    };

    conn.execute(
        "INSERT INTO attendance_logs (
            id, schedule_id, personnel_id, expected_checkin_time, actual_checkin_time,
            attendance_status, operational_status, is_in_class, minutes_late, remarks, recorded_at
        ) VALUES (
            ?1, ?2, ?3, '08:45', ?4,
            ?5, ?6, ?7, 0, ?8, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        )
        ON CONFLICT(schedule_id, personnel_id) DO UPDATE SET
            actual_checkin_time = excluded.actual_checkin_time,
            attendance_status = excluded.attendance_status,
            operational_status = excluded.operational_status,
            is_in_class = excluded.is_in_class,
            remarks = COALESCE(excluded.remarks, attendance_logs.remarks),
            recorded_at = excluded.recorded_at;",
        params![
            log_id,
            schedule_id,
            personnel_id,
            checkin,
            att_status,
            op_status,
            in_class,
            remarks,
        ],
    )?;

    Ok(())
}

pub fn set_day_schedule_annotation(
    conn: &Connection,
    date: &str,
    annotation: Option<&str>,
) -> CoreResult<()> {
    let schedule_id: String = match conn.query_row(
        "SELECT id FROM daily_schedules WHERE schedule_date = ?1",
        [date],
        |r| r.get(0),
    ) {
        Ok(id) => id,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let new_id = format!("SCHED_{}", date.replace('-', "_"));
            conn.execute(
                "INSERT OR IGNORE INTO daily_schedules (
                    id, schedule_date, day_of_week, regime_type, mandatory_checkin_time, nominal_start_time, nominal_end_time, footer_legal_clause
                ) VALUES (?1, ?2, 1, 'ORDINARY', '08:45', '09:00', '13:45', 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.')",
                params![new_id, date],
            )?;
            new_id
        }
        Err(e) => return Err(CoreError::Database(e)),
    };

    conn.execute(
        "UPDATE daily_schedules SET day_annotation = ?1 WHERE id = ?2;",
        params![annotation, schedule_id],
    )?;

    Ok(())
}

