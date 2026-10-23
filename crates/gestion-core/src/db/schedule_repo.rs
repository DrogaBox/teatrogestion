use crate::domain::{
    ActivityType, CastAssignmentCondition, CastMemberEntry, ConfirmationState,
    DailyScheduleAggregate, GenderGroup, MorningClassSlot, RehearsalBlock, RehearsalCastGroup,
    RoomCode, WorkRegime,
};
use crate::dto::{
    DailyScheduleSummaryDto, DailyScheduleWithAuditDto, UpsertMorningClassPayload,
    UpsertRehearsalBlockPayload,
};
use crate::error::{CoreError, CoreResult};
use crate::services::conflict_engine::audit_daily_schedule;
use chrono::{Datelike, NaiveDate};
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn get_or_create_daily_schedule(
    conn: &mut Connection,
    schedule_date: &str,
) -> CoreResult<DailyScheduleWithAuditDto> {
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM daily_schedules WHERE schedule_date = ?1;",
            [schedule_date],
            |r| r.get(0),
        )
        .ok();

    let schedule_id = match existing_id {
        Some(id) => id,
        None => {
            let id = format!("SCH_{}", Uuid::new_v4().simple());
            let parsed_date = NaiveDate::parse_from_str(schedule_date, "%Y-%m-%d")
                .map_err(|_| CoreError::IntegrityViolation(format!("Fecha inválida: {schedule_date}")))?;
            let day_of_week = parsed_date.weekday().number_from_monday();

            let tx = conn.transaction()?;

            tx.execute(
                "INSERT INTO daily_schedules (
                    id, schedule_date, day_of_week, regime_type, mandatory_checkin_time,
                    nominal_start_time, nominal_end_time, minimum_intershift_rest_hours,
                    footer_legal_clause, status
                 ) VALUES (?1, ?2, ?3, 'ORDINARY', '08:45', '09:00', '13:45', 12,
                    'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.', 'DRAFT');",
                params![id, schedule_date, day_of_week],
            )?;

            // Crear clases matutinas por defecto (Sala 1 y Sala 3)
            let mc1_id = format!("MC_{}", Uuid::new_v4().simple());
            let mc3_id = format!("MC_{}", Uuid::new_v4().simple());

            tx.execute(
                "INSERT INTO morning_classes (
                    id, schedule_id, room_id, gender_division, start_time, end_time, master_id, pianist_id
                 ) VALUES (?1, ?2, 'SALA_1', 'FEMALE', '09:00', '10:15', 'PER_MASTER_ANANIEV', 'PER_PIANIST_ROMERO');",
                params![mc1_id, id],
            )?;

            tx.execute(
                "INSERT INTO morning_classes (
                    id, schedule_id, room_id, gender_division, start_time, end_time, master_id, pianist_id
                 ) VALUES (?1, ?2, 'SALA_3', 'MALE', '09:00', '10:15', 'PER_MASTER_BENITES', 'PER_PIANIST_VILLEGAS');",
                params![mc3_id, id],
            )?;

            tx.commit()?;
            id
        }
    };

    load_daily_schedule_with_audit(conn, &schedule_id)
}

pub fn list_daily_schedules(conn: &Connection) -> CoreResult<Vec<DailyScheduleSummaryDto>> {
    let mut stmt = conn.prepare(
        "SELECT
            ds.id,
            ds.schedule_date,
            ds.regime_type,
            ds.mandatory_checkin_time,
            ds.nominal_end_time,
            COUNT(DISTINCT rb.id) AS blocks_count,
            COUNT(DISTINCT mc.id) AS classes_count,
            ds.general_notes,
            ds.status,
            ds.updated_at
         FROM daily_schedules ds
         LEFT JOIN rehearsal_blocks rb ON rb.schedule_id = ds.id
         LEFT JOIN morning_classes mc ON mc.schedule_id = ds.id
         GROUP BY ds.id
         ORDER BY ds.schedule_date DESC;",
    )?;

    let rows = stmt.query_map([], |r| {
        let regime_str: String = r.get(2)?;
        let regime = match regime_str.as_str() {
            "PRE_FUNCTION" => WorkRegime::PreFunction,
            "EXTRAORDINARY" => WorkRegime::Extraordinary,
            _ => WorkRegime::Ordinary,
        };

        Ok(DailyScheduleSummaryDto {
            id: r.get(0)?,
            schedule_date: r.get(1)?,
            regime,
            mandatory_checkin_time: r.get(3)?,
            nominal_end_time: r.get(4)?,
            rehearsal_blocks_count: r.get(5)?,
            morning_classes_count: r.get(6)?,
            general_notes: r.get(7)?,
            status: r.get(8)?,
            updated_at: r.get(9)?,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn set_schedule_regime(
    conn: &mut Connection,
    schedule_id: &str,
    regime: WorkRegime,
) -> CoreResult<DailyScheduleWithAuditDto> {
    let (regime_str, end_time) = match regime {
        WorkRegime::Ordinary => ("ORDINARY", "13:45"),
        WorkRegime::PreFunction => ("PRE_FUNCTION", "14:45"),
        WorkRegime::Extraordinary => ("EXTRAORDINARY", "23:59"),
    };

    let updated = conn.execute(
        "UPDATE daily_schedules
         SET regime_type = ?1, nominal_end_time = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?3;",
        params![regime_str, end_time, schedule_id],
    )?;

    if updated == 0 {
        return Err(CoreError::EntityNotFound {
            entity: "daily_schedules",
            id: schedule_id.to_string(),
        });
    }

    load_daily_schedule_with_audit(conn, schedule_id)
}

pub fn update_schedule_rules(
    conn: &mut Connection,
    payload: crate::dto::UpdateScheduleRulesPayload,
) -> CoreResult<DailyScheduleWithAuditDto> {
    let updated = conn.execute(
        "UPDATE daily_schedules
         SET mandatory_checkin_time = ?1,
             nominal_start_time = ?2,
             nominal_end_time = ?3,
             minimum_intershift_rest_hours = ?4,
             footer_legal_clause = COALESCE(?5, footer_legal_clause),
             general_notes = ?6,
             class_to_rehearsal_rest_minutes = COALESCE(?7, class_to_rehearsal_rest_minutes),
             is_compensatory_rest_day = COALESCE(?8, is_compensatory_rest_day),
             compensatory_rest_policy = COALESCE(?9, compensatory_rest_policy),
             compensatory_rest_days = COALESCE(?10, compensatory_rest_days),
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?11;",
        params![
            payload.mandatory_checkin_time,
            payload.nominal_start_time,
            payload.nominal_end_time,
            payload.minimum_intershift_rest_hours,
            payload.footer_legal_clause,
            payload.general_notes,
            payload.class_to_rehearsal_rest_minutes,
            payload.is_compensatory_rest_day.map(|b| if b { 1 } else { 0 }),
            payload.compensatory_rest_policy,
            payload.compensatory_rest_days,
            payload.schedule_id,
        ],
    )?;

    if updated == 0 {
        return Err(CoreError::EntityNotFound {
            entity: "daily_schedules",
            id: payload.schedule_id,
        });
    }

    load_daily_schedule_with_audit(conn, &payload.schedule_id)
}

pub fn upsert_rehearsal_block(
    conn: &mut Connection,
    payload: UpsertRehearsalBlockPayload,
) -> CoreResult<DailyScheduleWithAuditDto> {
    let tx = conn.transaction()?;

    let block_id = payload.id.unwrap_or_else(|| format!("BLK_{}", Uuid::new_v4().simple()));
    let room_str = payload.room.as_db_str();
    let activity_str = match payload.activity_type {
        ActivityType::StudioRehearsal => "STUDIO_REHEARSAL",
        ActivityType::StageTechRehearsal => "STAGE_TECH_REHEARSAL",
        ActivityType::StageLightingRehearsal => "STAGE_LIGHTING_REHEARSAL",
        ActivityType::StageOrchestraRehearsal => "STAGE_ORCHESTRA_REHEARSAL",
        ActivityType::DressRehearsal => "DRESS_REHEARSAL",
        ActivityType::Performance => "PERFORMANCE",
        ActivityType::TourTravel => "TOUR_TRAVEL",
        ActivityType::TourStageRehearsal => "TOUR_STAGE_REHEARSAL",
        ActivityType::ClassMorning => "CLASS_MORNING",
        ActivityType::CompanyBreak => "COMPANY_BREAK",
    };
    let conf_str = match payload.confirmation_state {
        ConfirmationState::Confirmed => "CONFIRMED",
        ConfirmationState::RehearsalToConfirm => "REHEARSAL_TO_CONFIRM",
        ConfirmationState::CastToConfirm => "CAST_TO_CONFIRM",
    };

    // Upsert block
    tx.execute(
        "INSERT INTO rehearsal_blocks (
            id, schedule_id, room_id, start_time, end_time, activity_type,
            work_id, passage_id, custom_passage_label, confirmation_state,
            sound_requirement, teatrocue_cue_ref, master_id, pianist_id, allow_shared_room, notes, sound_tech_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
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
            sound_tech_id = excluded.sound_tech_id;",
        params![
            block_id,
            payload.schedule_id,
            room_str,
            payload.start_time,
            payload.end_time,
            activity_str,
            payload.work_id,
            payload.passage_id,
            payload.custom_passage_label,
            conf_str,
            payload.sound_requirement,
            payload.teatrocue_cue_ref,
            payload.master_id,
            payload.pianist_id,
            if payload.allow_shared_room { 1 } else { 0 },
            payload.notes,
            payload.sound_tech_id,
        ],
    )?;

    // Limpiar grupos de elenco existentes para este bloque
    tx.execute(
        "DELETE FROM rehearsal_cast_groups WHERE rehearsal_block_id = ?1;",
        [&block_id],
    )?;

    for (g_idx, group) in payload.cast_groups.iter().enumerate() {
        let group_id = group.id.clone().unwrap_or_else(|| format!("RCG_{}", Uuid::new_v4().simple()));
        tx.execute(
            "INSERT INTO rehearsal_cast_groups (id, rehearsal_block_id, group_order, group_label, is_to_confirm)
             VALUES (?1, ?2, ?3, ?4, ?5);",
            params![
                group_id,
                block_id,
                group.group_order.max((g_idx + 1) as i32),
                group.group_label,
                if group.is_to_confirm { 1 } else { 0 },
            ],
        )?;

        for (m_idx, member) in group.members.iter().enumerate() {
            let member_id = format!("RCM_{}", Uuid::new_v4().simple());
            let cond_str = match member.condition {
                CastAssignmentCondition::Titular => "TITULAR",
                CastAssignmentCondition::AlternateSlash => "ALTERNATE_SLASH",
                CastAssignmentCondition::Reinforcement => "REINFORCEMENT",
                CastAssignmentCondition::Cover => "COVER",
                CastAssignmentCondition::ToConfirm => "TO_CONFIRM",
            };

            tx.execute(
                "INSERT INTO rehearsal_cast_members (
                    id, cast_group_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
                params![
                    member_id,
                    group_id,
                    member.personnel_id,
                    member.character_role,
                    member.partner_personnel_id,
                    cond_str,
                    m_idx as i32,
                ],
            )?;
        }
    }

    tx.commit()?;

    load_daily_schedule_with_audit(conn, &payload.schedule_id)
}

pub fn delete_rehearsal_block(
    conn: &mut Connection,
    block_id: &str,
) -> CoreResult<DailyScheduleWithAuditDto> {
    let schedule_id: String = conn.query_row(
        "SELECT schedule_id FROM rehearsal_blocks WHERE id = ?1;",
        [block_id],
        |r| r.get(0),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CoreError::EntityNotFound {
            entity: "rehearsal_blocks",
            id: block_id.to_string(),
        },
        other => CoreError::Database(other),
    })?;

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM rehearsal_cast_groups WHERE rehearsal_block_id = ?1;", [block_id])?;
    tx.execute("DELETE FROM rehearsal_blocks WHERE id = ?1;", [block_id])?;
    tx.commit()?;

    load_daily_schedule_with_audit(conn, &schedule_id)
}

pub fn load_daily_schedule_with_audit(
    conn: &Connection,
    schedule_id: &str,
) -> CoreResult<DailyScheduleWithAuditDto> {
    let aggregate = load_daily_schedule_aggregate(conn, schedule_id)?;
    let conflicts = audit_daily_schedule(&aggregate, None);
    Ok(DailyScheduleWithAuditDto {
        schedule: aggregate,
        conflicts,
    })
}

pub fn load_daily_schedule_aggregate(
    conn: &Connection,
    schedule_id: &str,
) -> CoreResult<DailyScheduleAggregate> {
    let (
        id,
        schedule_date,
        regime_str,
        mandatory_checkin,
        nominal_end,
        rest_hours,
        footer_clause,
        class_rest_mins,
        is_comp_day,
        comp_days,
        comp_policy,
    ): (
        String,
        String,
        String,
        String,
        String,
        u8,
        String,
        u16,
        bool,
        u8,
        String,
    ) = conn.query_row(
        "SELECT id, schedule_date, regime_type, mandatory_checkin_time, nominal_end_time,
                minimum_intershift_rest_hours, footer_legal_clause,
                COALESCE(class_to_rehearsal_rest_minutes, 15),
                COALESCE(is_compensatory_rest_day, 0),
                COALESCE(compensatory_rest_days, 1),
                COALESCE(compensatory_rest_policy, 'NEXT_DAY')
         FROM daily_schedules WHERE id = ?1;",
        [schedule_id],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get::<_, i64>(5)? as u8,
                r.get(6)?,
                r.get::<_, i64>(7)? as u16,
                r.get::<_, i64>(8)? == 1,
                r.get::<_, i64>(9)? as u8,
                r.get(10)?,
            ))
        },
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CoreError::EntityNotFound {
            entity: "daily_schedules",
            id: schedule_id.to_string(),
        },
        other => CoreError::Database(other),
    })?;

    let regime = match regime_str.as_str() {
        "PRE_FUNCTION" => WorkRegime::PreFunction,
        "EXTRAORDINARY" => WorkRegime::Extraordinary,
        _ => WorkRegime::Ordinary,
    };

    // Cargar MorningClasses
    let mut mc_stmt = conn.prepare(
        "SELECT mc.id, mc.room_id, mc.gender_division, mc.start_time, mc.end_time,
                mc.master_id, COALESCE(m.last_name || ', ' || m.first_name, mc.master_id),
                mc.pianist_id, COALESCE(p.last_name || ', ' || p.first_name, mc.pianist_id),
                mc.sound_tech_id, (s.last_name || ', ' || s.first_name)
         FROM morning_classes mc
         LEFT JOIN personnel m ON m.id = mc.master_id
         LEFT JOIN personnel p ON p.id = mc.pianist_id
         LEFT JOIN personnel s ON s.id = mc.sound_tech_id
         WHERE mc.schedule_id = ?1
         ORDER BY mc.start_time ASC, mc.room_id ASC;",
    )?;

    let mc_rows = mc_stmt.query_map([schedule_id], |r| {
        let room_code_str: String = r.get(1)?;
        let gender_str: String = r.get(2)?;
        let room = match room_code_str.as_str() {
            "SALA_2" => RoomCode::Sala2,
            "SALA_3" => RoomCode::Sala3,
            "ESCENARIO" => RoomCode::Escenario,
            "GIRA_TRASLADO" => RoomCode::GiraTraslado,
            "GIRA_ESCENARIO" => RoomCode::GiraEscenario,
            "GIRA_SALA" => RoomCode::GiraSala,
            "SALA_AUX" => RoomCode::SalaAux,
            _ => RoomCode::Sala1,
        };
        let gender = GenderGroup::from_db_str(&gender_str).unwrap_or(GenderGroup::Female);

        Ok(MorningClassSlot {
            id: r.get(0)?,
            room,
            gender_division: gender,
            start_time: r.get(3)?,
            end_time: r.get(4)?,
            master_id: r.get(5)?,
            master_name: r.get(6)?,
            pianist_id: r.get(7)?,
            pianist_name: r.get(8)?,
            sound_tech_id: r.get(9)?,
            sound_tech_name: r.get(10)?,
        })
    })?;

    let mut morning_classes = Vec::new();
    for mc in mc_rows {
        morning_classes.push(mc?);
    }

    // Cargar RehearsalBlocks
    let mut rb_stmt = conn.prepare(
        "SELECT rb.id, rb.schedule_id, rb.room_id, rb.start_time, rb.end_time, rb.activity_type,
                rb.work_id, w.title, rb.passage_id,
                COALESCE(rb.custom_passage_label, wp.name, 'Ensayo General') AS passage_label,
                rb.confirmation_state, rb.sound_requirement, rb.teatrocue_cue_ref,
                rb.master_id, (m.last_name || ', ' || m.first_name),
                rb.pianist_id, (p.last_name || ', ' || p.first_name),
                rb.allow_shared_room,
                rb.sound_tech_id, (s.last_name || ', ' || s.first_name)
         FROM rehearsal_blocks rb
         LEFT JOIN works w ON w.id = rb.work_id
         LEFT JOIN work_passages wp ON wp.id = rb.passage_id
         LEFT JOIN personnel m ON m.id = rb.master_id
         LEFT JOIN personnel p ON p.id = rb.pianist_id
         LEFT JOIN personnel s ON s.id = rb.sound_tech_id
         WHERE rb.schedule_id = ?1
         ORDER BY rb.start_time ASC, rb.room_id ASC;",
    )?;

    let rb_rows = rb_stmt.query_map([schedule_id], |r| {
        let room_code_str: String = r.get(2)?;
        let room = match room_code_str.as_str() {
            "SALA_2" => RoomCode::Sala2,
            "SALA_3" => RoomCode::Sala3,
            "ESCENARIO" => RoomCode::Escenario,
            "GIRA_TRASLADO" => RoomCode::GiraTraslado,
            "GIRA_ESCENARIO" => RoomCode::GiraEscenario,
            "GIRA_SALA" => RoomCode::GiraSala,
            "SALA_AUX" => RoomCode::SalaAux,
            _ => RoomCode::Sala1,
        };
        let act_str: String = r.get(5)?;
        let activity_type = match act_str.as_str() {
            "STAGE_TECH_REHEARSAL" => ActivityType::StageTechRehearsal,
            "STAGE_LIGHTING_REHEARSAL" => ActivityType::StageLightingRehearsal,
            "STAGE_ORCHESTRA_REHEARSAL" => ActivityType::StageOrchestraRehearsal,
            "DRESS_REHEARSAL" => ActivityType::DressRehearsal,
            "PERFORMANCE" => ActivityType::Performance,
            "TOUR_TRAVEL" => ActivityType::TourTravel,
            "TOUR_STAGE_REHEARSAL" => ActivityType::TourStageRehearsal,
            "CLASS_MORNING" => ActivityType::ClassMorning,
            "COMPANY_BREAK" => ActivityType::CompanyBreak,
            _ => ActivityType::StudioRehearsal,
        };
        let conf_str: String = r.get(10)?;
        let confirmation_state = match conf_str.as_str() {
            "REHEARSAL_TO_CONFIRM" => ConfirmationState::RehearsalToConfirm,
            "CAST_TO_CONFIRM" => ConfirmationState::CastToConfirm,
            _ => ConfirmationState::Confirmed,
        };

        Ok(RehearsalBlock {
            id: r.get(0)?,
            schedule_id: r.get(1)?,
            room,
            start_time: r.get(3)?,
            end_time: r.get(4)?,
            activity_type,
            work_id: r.get(6)?,
            work_title: r.get(7)?,
            passage_id: r.get(8)?,
            passage_name: r.get(9)?,
            confirmation_state,
            sound_requirement: r.get(11)?,
            teatrocue_cue_ref: r.get(12)?,
            master_id: r.get(13)?,
            master_name: r.get(14)?,
            pianist_id: r.get(15)?,
            pianist_name: r.get(16)?,
            allow_shared_room: r.get::<_, i32>(17)? == 1,
            sound_tech_id: r.get(18)?,
            sound_tech_name: r.get(19)?,
            cast_groups: Vec::new(),
        })
    })?;

    let mut rehearsal_blocks = Vec::new();
    for rb in rb_rows {
        let mut block = rb?;

        // Cargar cast_groups para este bloque
        let mut grp_stmt = conn.prepare(
            "SELECT id, group_order, group_label, is_to_confirm
             FROM rehearsal_cast_groups
             WHERE rehearsal_block_id = ?1
             ORDER BY group_order ASC;",
        )?;

        let grp_rows = grp_stmt.query_map([&block.id], |gr| {
            Ok(RehearsalCastGroup {
                id: gr.get(0)?,
                group_order: gr.get(1)?,
                group_label: gr.get(2)?,
                formatted_line: String::new(),
                is_to_confirm: gr.get::<_, i32>(3)? == 1,
                members: Vec::new(),
            })
        })?;

        let mut cast_groups = Vec::new();
        for grp in grp_rows {
            let mut group = grp?;

            let mut mem_stmt = conn.prepare(
                "SELECT rcm.id, rcm.personnel_id, (p.last_name || ', ' || p.first_name),
                        rcm.character_role, rcm.partner_personnel_id,
                        (part.last_name || ', ' || part.first_name),
                        rcm.assignment_condition
                 FROM rehearsal_cast_members rcm
                 JOIN personnel p ON p.id = rcm.personnel_id
                 LEFT JOIN personnel part ON part.id = rcm.partner_personnel_id
                 WHERE rcm.cast_group_id = ?1
                 ORDER BY rcm.sort_order ASC;",
            )?;

            let mem_rows = mem_stmt.query_map([&group.id], |mr| {
                let cond_str: String = mr.get(6)?;
                let condition = match cond_str.as_str() {
                    "ALTERNATE_SLASH" => CastAssignmentCondition::AlternateSlash,
                    "REINFORCEMENT" => CastAssignmentCondition::Reinforcement,
                    "COVER" => CastAssignmentCondition::Cover,
                    "TO_CONFIRM" => CastAssignmentCondition::ToConfirm,
                    _ => CastAssignmentCondition::Titular,
                };

                Ok(CastMemberEntry {
                    id: mr.get(0)?,
                    personnel_id: mr.get(1)?,
                    display_name: mr.get(2)?,
                    character_role: mr.get(3)?,
                    partner_personnel_id: mr.get(4)?,
                    partner_display_name: mr.get(5)?,
                    condition,
                })
            })?;

            for mem in mem_rows {
                group.members.push(mem?);
            }

            // Formatear línea visible de reparto
            let names: Vec<String> = group.members.iter().map(|m| {
                if let Some(partner) = &m.partner_display_name {
                    format!("{} / {}", m.display_name, partner)
                } else {
                    m.display_name.clone()
                }
            }).collect();
            group.formatted_line = names.join(" - ");

            cast_groups.push(group);
        }

        block.cast_groups = cast_groups;
        rehearsal_blocks.push(block);
    }

    Ok(DailyScheduleAggregate {
        id,
        schedule_date,
        regime,
        mandatory_checkin_time: mandatory_checkin,
        nominal_end_time: nominal_end,
        minimum_intershift_rest_hours: rest_hours,
        class_to_rehearsal_rest_minutes: class_rest_mins,
        is_compensatory_rest_day: is_comp_day,
        compensatory_rest_days: comp_days,
        compensatory_rest_policy: comp_policy,
        footer_legal_clause: footer_clause,
        morning_classes,
        rehearsal_blocks,
    })
}

pub fn upsert_morning_class(
    conn: &mut Connection,
    payload: UpsertMorningClassPayload,
) -> CoreResult<DailyScheduleWithAuditDto> {
    let class_id = payload.id.unwrap_or_else(|| format!("MC_{}", Uuid::new_v4().simple()));
    let room_str = payload.room.as_db_str();
    let gender_str = payload.gender_division.as_db_str();

    conn.execute(
        "INSERT INTO morning_classes (
            id, schedule_id, room_id, gender_division, start_time, end_time, master_id, pianist_id, sound_tech_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
            room_id = excluded.room_id,
            gender_division = excluded.gender_division,
            start_time = excluded.start_time,
            end_time = excluded.end_time,
            master_id = excluded.master_id,
            pianist_id = excluded.pianist_id,
            sound_tech_id = excluded.sound_tech_id;",
        params![
            class_id,
            payload.schedule_id,
            room_str,
            gender_str,
            payload.start_time,
            payload.end_time,
            payload.master_id,
            payload.pianist_id,
            payload.sound_tech_id,
        ],
    )?;

    load_daily_schedule_with_audit(conn, &payload.schedule_id)
}

pub fn delete_morning_class(
    conn: &mut Connection,
    class_id: &str,
) -> CoreResult<Option<DailyScheduleWithAuditDto>> {
    let schedule_id: Option<String> = conn
        .query_row("SELECT schedule_id FROM morning_classes WHERE id = ?1;", [class_id], |r| r.get(0))
        .ok();

    conn.execute("DELETE FROM morning_classes WHERE id = ?1;", [class_id])?;

    if let Some(sid) = schedule_id {
        Ok(Some(load_daily_schedule_with_audit(conn, &sid)?))
    } else {
        Ok(None)
    }
}

