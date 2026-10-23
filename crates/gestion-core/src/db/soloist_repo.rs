use crate::domain::{
    SaveSoloistCertificationPayload, SoloistCertificationDto, SoloistCertificationItemDto,
};
use crate::error::{CoreError, CoreResult};
use rusqlite::{params, Connection};

pub fn list_soloist_certifications(conn: &Connection) -> CoreResult<Vec<SoloistCertificationDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, program_title, hall_name, document_date, recipient_name, recipient_title,
                intro_body, closing_text, status, notes, created_at, updated_at
         FROM soloist_certifications
         ORDER BY document_date DESC, created_at DESC;",
    )?;

    let cert_rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
            r.get::<_, String>(8)?,
            r.get::<_, Option<String>>(9)?,
            r.get::<_, String>(10)?,
            r.get::<_, String>(11)?,
        ))
    })?;

    let mut out = Vec::new();
    for row in cert_rows {
        let (id, prog, hall, date, rec_n, rec_t, intro, closing, st, notes, c_at, u_at) = row?;
        let items = get_certification_items(conn, &id)?;
        let total_perf: i32 = items.iter().map(|i| i.performance_count).sum();

        out.push(SoloistCertificationDto {
            id,
            program_title: prog,
            hall_name: hall,
            document_date: date,
            recipient_name: rec_n,
            recipient_title: rec_t,
            intro_body: intro,
            closing_text: closing,
            status: st,
            notes,
            total_performances: total_perf,
            items,
            created_at: c_at,
            updated_at: u_at,
        });
    }

    Ok(out)
}

pub fn get_soloist_certification(conn: &Connection, id: &str) -> CoreResult<SoloistCertificationDto> {
    let mut stmt = conn.prepare(
        "SELECT id, program_title, hall_name, document_date, recipient_name, recipient_title,
                intro_body, closing_text, status, notes, created_at, updated_at
         FROM soloist_certifications
         WHERE id = ?1;",
    )?;

    let mut rows = stmt.query_map([id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
            r.get::<_, String>(8)?,
            r.get::<_, Option<String>>(9)?,
            r.get::<_, String>(10)?,
            r.get::<_, String>(11)?,
        ))
    })?;

    if let Some(r) = rows.next() {
        let (cid, prog, hall, date, rec_n, rec_t, intro, closing, st, notes, c_at, u_at) = r?;
        let items = get_certification_items(conn, &cid)?;
        let total_perf: i32 = items.iter().map(|i| i.performance_count).sum();

        Ok(SoloistCertificationDto {
            id: cid,
            program_title: prog,
            hall_name: hall,
            document_date: date,
            recipient_name: rec_n,
            recipient_title: rec_t,
            intro_body: intro,
            closing_text: closing,
            status: st,
            notes,
            total_performances: total_perf,
            items,
            created_at: c_at,
            updated_at: u_at,
        })
    } else {
        Err(CoreError::EntityNotFound {
            entity: "soloist_certifications",
            id: id.to_string(),
        })
    }
}

fn get_certification_items(
    conn: &Connection,
    certification_id: &str,
) -> CoreResult<Vec<SoloistCertificationItemDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, certification_id, personnel_id, dancer_name, work_role,
                performance_dates, performance_count, sort_order
         FROM soloist_certification_items
         WHERE certification_id = ?1
         ORDER BY sort_order ASC, id ASC;",
    )?;

    let rows = stmt.query_map([certification_id], |r| {
        Ok(SoloistCertificationItemDto {
            id: r.get(0)?,
            certification_id: r.get(1)?,
            personnel_id: r.get(2)?,
            dancer_name: r.get(3)?,
            work_role: r.get(4)?,
            performance_dates: r.get(5)?,
            performance_count: r.get(6)?,
            sort_order: r.get(7)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

pub fn save_soloist_certification(
    conn: &Connection,
    payload: SaveSoloistCertificationPayload,
) -> CoreResult<SoloistCertificationDto> {
    let cert_id = payload
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            let sanitized = payload.program_title.to_uppercase().replace(|c: char| !c.is_alphanumeric(), "_");
            format!("CERT_SOLO_{}_{}", payload.document_date.replace('-', ""), sanitized)
        });

    let now_iso = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO soloist_certifications (
            id, program_title, hall_name, document_date, recipient_name, recipient_title,
            intro_body, closing_text, status, notes, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6,
            ?7, ?8, ?9, ?10, ?11, ?11
        )
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
            updated_at = excluded.updated_at;",
        params![
            cert_id,
            payload.program_title,
            payload.hall_name,
            payload.document_date,
            payload.recipient_name,
            payload.recipient_title,
            payload.intro_body,
            payload.closing_text,
            payload.status,
            payload.notes,
            now_iso,
        ],
    )?;

    // Eliminar ítems previos de esta certificación y reinsertar
    conn.execute(
        "DELETE FROM soloist_certification_items WHERE certification_id = ?1;",
        [&cert_id],
    )?;

    for (idx, item) in payload.items.iter().enumerate() {
        let item_id = item
            .id
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("ITEM_SOLO_{}_{:02}", cert_id, idx + 1));

        conn.execute(
            "INSERT INTO soloist_certification_items (
                id, certification_id, personnel_id, dancer_name, work_role,
                performance_dates, performance_count, sort_order
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
            params![
                item_id,
                cert_id,
                item.personnel_id,
                item.dancer_name,
                item.work_role,
                item.performance_dates,
                item.performance_count,
                item.sort_order.max(idx as i32 + 1),
            ],
        )?;
    }

    get_soloist_certification(conn, &cert_id)
}

pub fn delete_soloist_certification(conn: &Connection, id: &str) -> CoreResult<()> {
    conn.execute(
        "DELETE FROM soloist_certifications WHERE id = ?1;",
        [id],
    )?;
    Ok(())
}

pub fn generate_soloist_certification_from_work_cast(
    conn: &Connection,
    work_id: &str,
    dates_label: &str,
    performances_per_cast: i32,
) -> CoreResult<SoloistCertificationDto> {
    let (work_title,): (String,) = conn
        .query_row(
            "SELECT title FROM works WHERE id = ?1;",
            [work_id],
            |r| Ok((r.get(0)?,)),
        )
        .map_err(|_| CoreError::EntityNotFound {
            entity: "works",
            id: work_id.to_string(),
        })?;

    let mut stmt = conn.prepare(
        "SELECT m.personnel_id, 
                CASE 
                    WHEN p.nickname IS NOT NULL AND p.nickname != '' THEN p.last_name || ', ' || p.first_name || ' (' || p.nickname || ')'
                    ELSE p.last_name || ', ' || p.first_name
                END as dancer_name,
                COALESCE(m.character_role, wp.name, cp.cast_label) as role_label,
                cp.tier_name
         FROM work_cast_plan_members m
         JOIN work_cast_plans cp ON cp.id = m.plan_id
         JOIN personnel p ON p.id = m.personnel_id
         LEFT JOIN work_passages wp ON wp.id = cp.passage_id
         WHERE cp.work_id = ?1
         ORDER BY cp.tier_name ASC, m.sort_order ASC;",
    )?;

    let rows = stmt.query_map([work_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    })?;

    let mut items = Vec::new();
    let mut sort = 1;
    for row in rows {
        let (p_id, dancer, role, tier) = row?;
        let full_role = format!("{} [{}]", role, tier);
        items.push(crate::domain::SaveSoloistItemPayload {
            id: None,
            personnel_id: Some(p_id),
            dancer_name: dancer.to_uppercase(),
            work_role: full_role,
            performance_dates: dates_label.to_string(),
            performance_count: performances_per_cast.max(1),
            sort_order: sort,
        });
        sort += 1;
    }

    let today_iso = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let intro = format!(
        "Me dirijo a Ud. a los fines de informarle los solísticos correspondientes al Ballet Oficial de la Pcia. con respecto a las funciones \"{}\" programadas en la Sala Mayor del Teatro del Libertador Gral. San Martín los días {} se cumplió según el siguiente detalle:",
        work_title, dates_label
    );

    let payload = SaveSoloistCertificationPayload {
        id: None,
        program_title: work_title,
        hall_name: "Sala Mayor del Teatro del Libertador Gral. San Martín".to_string(),
        document_date: today_iso,
        recipient_name: "Hadrián Ávila Arzuza".to_string(),
        recipient_title: "Director del Teatro del Libertador".to_string(),
        intro_body: intro,
        closing_text: "Sin otro particular saludo a Ud. Atte.".to_string(),
        status: "DRAFT".to_string(),
        notes: Some(format!("Generado automáticamente desde reparto de obra {}", work_id)),
        items,
    };

    save_soloist_certification(conn, payload)
}
