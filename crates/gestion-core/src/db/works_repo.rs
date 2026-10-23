use crate::dto::*;
use crate::error::{CoreError, CoreResult};
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn list_works_with_casts(conn: &Connection) -> CoreResult<Vec<WorkWithPassagesAndCastsDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, composer, choreographer, is_active
         FROM works
         ORDER BY title ASC;"
    )?;

    let work_rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, i32>(4)? == 1,
        ))
    })?;

    let mut works = Vec::new();

    for w in work_rows {
        let (work_id, title, composer, choreographer, is_active) = w?;

        // 1. Pasajes de la obra
        let mut pass_stmt = conn.prepare(
            "SELECT id, work_id, name, act_scene, default_duration_min, default_sound_cue
             FROM work_passages
             WHERE work_id = ?1
             ORDER BY act_scene ASC, name ASC;"
        )?;
        let pass_rows = pass_stmt.query_map([&work_id], |r| {
            Ok(WorkPassageDto {
                id: r.get(0)?,
                work_id: r.get(1)?,
                name: r.get(2)?,
                act_scene: r.get(3)?,
                default_duration_min: r.get(4)?,
                default_sound_cue: r.get(5)?,
            })
        })?;
        let mut passages = Vec::new();
        for p in pass_rows {
            passages.push(p?);
        }

        // 2. Planes de reparto de la obra
        let mut plans_stmt = conn.prepare(
            "SELECT p.id, p.work_id, w.title, p.passage_id, wp.name,
                    p.tier_name, p.cast_label, p.default_sound_cue, p.notes
             FROM work_cast_plans p
             JOIN works w ON w.id = p.work_id
             LEFT JOIN work_passages wp ON wp.id = p.passage_id
             WHERE p.work_id = ?1
             ORDER BY p.tier_name ASC, p.cast_label ASC;"
        )?;

        let plan_rows = plans_stmt.query_map([&work_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, Option<String>>(8)?,
            ))
        })?;

        let mut casting_plans = Vec::new();
        for pl in plan_rows {
            let (plan_id, w_id, w_title, pass_id, pass_name, tier_name, cast_label, sound_cue, notes) = pl?;

            // Miembros del plan
            let mut mem_stmt = conn.prepare(
                "SELECT m.id, m.plan_id, m.personnel_id,
                        (d.last_name || ', ' || d.first_name || COALESCE(' (\"' || d.nickname || '\")', '')) AS dancer_name,
                        m.character_role, m.partner_personnel_id,
                        (p.last_name || ', ' || p.first_name || COALESCE(' (\"' || p.nickname || '\")', '')) AS partner_name,
                        m.assignment_condition, m.sort_order
                 FROM work_cast_plan_members m
                 JOIN personnel d ON d.id = m.personnel_id
                 LEFT JOIN personnel p ON p.id = m.partner_personnel_id
                 WHERE m.plan_id = ?1
                 ORDER BY m.sort_order ASC, d.last_name ASC;"
            )?;

            let mem_rows = mem_stmt.query_map([&plan_id], |mr| {
                let cond_str: String = mr.get(7)?;
                let condition = match cond_str.as_str() {
                    "COVER" => crate::domain::schedule::CastAssignmentCondition::Cover,
                    "ALTERNATE_SLASH" => crate::domain::schedule::CastAssignmentCondition::AlternateSlash,
                    "REINFORCEMENT" => crate::domain::schedule::CastAssignmentCondition::Reinforcement,
                    "TO_CONFIRM" => crate::domain::schedule::CastAssignmentCondition::ToConfirm,
                    _ => crate::domain::schedule::CastAssignmentCondition::Titular,
                };

                Ok(WorkCastPlanMemberDto {
                    id: mr.get(0)?,
                    plan_id: mr.get(1)?,
                    personnel_id: mr.get(2)?,
                    dancer_name: mr.get(3)?,
                    character_role: mr.get(4)?,
                    partner_personnel_id: mr.get(5)?,
                    partner_name: mr.get(6)?,
                    condition,
                    sort_order: mr.get(8)?,
                })
            })?;

            let mut members = Vec::new();
            for m in mem_rows {
                members.push(m?);
            }

            casting_plans.push(WorkCastingPlanDto {
                id: plan_id,
                work_id: w_id,
                work_title: w_title,
                passage_id: pass_id,
                passage_name: pass_name,
                tier_name,
                cast_label,
                default_sound_cue: sound_cue,
                notes,
                members,
            });
        }

        works.push(WorkWithPassagesAndCastsDto {
            id: work_id,
            title,
            composer,
            choreographer,
            is_active,
            passages,
            casting_plans,
        });
    }

    Ok(works)
}

pub fn upsert_work(conn: &mut Connection, payload: UpsertWorkPayload) -> CoreResult<WorkWithPassagesAndCastsDto> {
    let final_id = payload.id.unwrap_or_else(|| format!("WORK_{}", Uuid::new_v4().simple()));
    let is_active_int = if payload.is_active { 1 } else { 0 };

    conn.execute(
        "INSERT INTO works (id, title, composer, choreographer, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            composer = excluded.composer,
            choreographer = excluded.choreographer,
            is_active = excluded.is_active;",
        params![final_id, payload.title.trim(), payload.composer, payload.choreographer, is_active_int],
    )?;

    let all_works = list_works_with_casts(conn)?;
    all_works
        .into_iter()
        .find(|w| w.id == final_id)
        .ok_or_else(|| CoreError::EntityNotFound {
            entity: "works",
            id: final_id,
        })
}

pub fn delete_work(conn: &mut Connection, work_id: &str) -> CoreResult<()> {
    conn.execute("DELETE FROM works WHERE id = ?1;", [work_id])?;
    Ok(())
}

pub fn upsert_work_passage(conn: &mut Connection, payload: UpsertWorkPassagePayload) -> CoreResult<WorkPassageDto> {
    let final_id = payload.id.unwrap_or_else(|| format!("PAS_{}", Uuid::new_v4().simple()));
    let duration = payload.default_duration_min.unwrap_or(60);

    conn.execute(
        "INSERT INTO work_passages (id, work_id, name, act_scene, default_duration_min, default_sound_cue)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            act_scene = excluded.act_scene,
            default_duration_min = excluded.default_duration_min,
            default_sound_cue = excluded.default_sound_cue;",
        params![
            final_id,
            payload.work_id,
            payload.name.trim(),
            payload.act_scene,
            duration,
            payload.default_sound_cue
        ],
    )?;

    conn.query_row(
        "SELECT id, work_id, name, act_scene, default_duration_min, default_sound_cue
         FROM work_passages WHERE id = ?1;",
        [&final_id],
        |r| {
            Ok(WorkPassageDto {
                id: r.get(0)?,
                work_id: r.get(1)?,
                name: r.get(2)?,
                act_scene: r.get(3)?,
                default_duration_min: r.get(4)?,
                default_sound_cue: r.get(5)?,
            })
        },
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CoreError::EntityNotFound {
            entity: "work_passages",
            id: final_id,
        },
        other => CoreError::Database(other),
    })
}

pub fn delete_work_passage(conn: &mut Connection, passage_id: &str) -> CoreResult<()> {
    conn.execute("DELETE FROM work_passages WHERE id = ?1;", [passage_id])?;
    Ok(())
}

pub fn upsert_work_casting_plan(
    conn: &mut Connection,
    payload: UpsertWorkCastingPlanPayload,
) -> CoreResult<WorkCastingPlanDto> {
    let final_id = payload.id.unwrap_or_else(|| format!("PLAN_{}", Uuid::new_v4().simple()));

    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO work_cast_plans (
            id, work_id, passage_id, tier_name, cast_label, default_sound_cue, notes, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(id) DO UPDATE SET
            passage_id = excluded.passage_id,
            tier_name = excluded.tier_name,
            cast_label = excluded.cast_label,
            default_sound_cue = excluded.default_sound_cue,
            notes = excluded.notes,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now');",
        params![
            final_id,
            payload.work_id,
            payload.passage_id,
            payload.tier_name.trim(),
            payload.cast_label.trim(),
            payload.default_sound_cue,
            payload.notes
        ],
    )?;

    // Reemplazar miembros
    tx.execute("DELETE FROM work_cast_plan_members WHERE plan_id = ?1;", [&final_id])?;

    for (idx, mem) in payload.members.iter().enumerate() {
        let member_id = format!("MEM_{}", Uuid::new_v4().simple());
        let cond_str = match mem.condition {
            crate::domain::schedule::CastAssignmentCondition::Cover => "COVER",
            crate::domain::schedule::CastAssignmentCondition::AlternateSlash => "ALTERNATE_SLASH",
            crate::domain::schedule::CastAssignmentCondition::Reinforcement => "REINFORCEMENT",
            crate::domain::schedule::CastAssignmentCondition::ToConfirm => "TO_CONFIRM",
            _ => "TITULAR",
        };

        tx.execute(
            "INSERT INTO work_cast_plan_members (
                id, plan_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            params![
                member_id,
                final_id,
                mem.personnel_id,
                mem.character_role,
                mem.partner_personnel_id,
                cond_str,
                idx as i32
            ],
        )?;
    }

    tx.commit()?;

    // Consultar el plan resultante
    let (w_id, w_title, pass_id, pass_name, tier_name, cast_label, sound_cue, notes): (
        String, String, Option<String>, Option<String>, String, String, Option<String>, Option<String>
    ) = conn.query_row(
        "SELECT p.work_id, w.title, p.passage_id, wp.name,
                p.tier_name, p.cast_label, p.default_sound_cue, p.notes
         FROM work_cast_plans p
         JOIN works w ON w.id = p.work_id
         LEFT JOIN work_passages wp ON wp.id = p.passage_id
         WHERE p.id = ?1;",
        [&final_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?)),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CoreError::EntityNotFound {
            entity: "work_cast_plans",
            id: final_id.clone(),
        },
        other => CoreError::Database(other),
    })?;

    let mut mem_stmt = conn.prepare(
        "SELECT m.id, m.plan_id, m.personnel_id,
                (d.last_name || ', ' || d.first_name || COALESCE(' (\"' || d.nickname || '\")', '')) AS dancer_name,
                m.character_role, m.partner_personnel_id,
                (p.last_name || ', ' || p.first_name || COALESCE(' (\"' || p.nickname || '\")', '')) AS partner_name,
                m.assignment_condition, m.sort_order
         FROM work_cast_plan_members m
         JOIN personnel d ON d.id = m.personnel_id
         LEFT JOIN personnel p ON p.id = m.partner_personnel_id
         WHERE m.plan_id = ?1
         ORDER BY m.sort_order ASC, d.last_name ASC;"
    )?;

    let mem_rows = mem_stmt.query_map([&final_id], |mr| {
        let cond_str: String = mr.get(7)?;
        let condition = match cond_str.as_str() {
            "COVER" => crate::domain::schedule::CastAssignmentCondition::Cover,
            "ALTERNATE_SLASH" => crate::domain::schedule::CastAssignmentCondition::AlternateSlash,
            "REINFORCEMENT" => crate::domain::schedule::CastAssignmentCondition::Reinforcement,
            "TO_CONFIRM" => crate::domain::schedule::CastAssignmentCondition::ToConfirm,
            _ => crate::domain::schedule::CastAssignmentCondition::Titular,
        };

        Ok(WorkCastPlanMemberDto {
            id: mr.get(0)?,
            plan_id: mr.get(1)?,
            personnel_id: mr.get(2)?,
            dancer_name: mr.get(3)?,
            character_role: mr.get(4)?,
            partner_personnel_id: mr.get(5)?,
            partner_name: mr.get(6)?,
            condition,
            sort_order: mr.get(8)?,
        })
    })?;

    let mut members = Vec::new();
    for m in mem_rows {
        members.push(m?);
    }

    Ok(WorkCastingPlanDto {
        id: final_id,
        work_id: w_id,
        work_title: w_title,
        passage_id: pass_id,
        passage_name: pass_name,
        tier_name,
        cast_label,
        default_sound_cue: sound_cue,
        notes,
        members,
    })
}

pub fn delete_work_casting_plan(conn: &mut Connection, plan_id: &str) -> CoreResult<()> {
    conn.execute("DELETE FROM work_cast_plans WHERE id = ?1;", [plan_id])?;
    Ok(())
}
