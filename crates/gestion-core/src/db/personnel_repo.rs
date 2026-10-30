use crate::domain::{GenderGroup, PersonnelRole};
use crate::dto::{ImportPersonnelResult, PersonnelWithQuotaDto, UpsertPersonnelPayload};
use crate::error::CoreResult;
use rusqlite::Connection;

pub fn list_personnel(
    conn: &Connection,
    role_filter: Option<PersonnelRole>,
    active_only: bool,
) -> CoreResult<Vec<PersonnelWithQuotaDto>> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.legajo,
            p.first_name,
            p.last_name,
            p.nickname,
            p.role,
            p.gender_group,
            p.category_id,
            c.name AS category_name,
            p.is_active,
            p.default_pointe_raw_code,
            p.default_flat_size,
            COALESCE(q.effective_pointe_quota, c.annual_pointe_quota, 0) AS effective_pointe_quota,
            COALESCE(q.consumed_pointe_qty, 0) AS consumed_pointe_qty,
            COALESCE(q.effective_flat_quota, c.annual_flat_quota, 0) AS effective_flat_quota,
            COALESCE(q.consumed_flat_qty, 0) AS consumed_flat_qty,
            COALESCE(p.is_retired, 0) AS is_retired,
            p.retirement_notes,
            p.specific_position
        FROM personnel p
        JOIN personnel_categories c ON c.id = p.category_id
        LEFT JOIN v_dancer_annual_quota_balance q ON q.personnel_id = p.id
        WHERE 1=1 ",
    );

    if active_only {
        sql.push_str("AND p.is_active = 1 ");
    }
    if let Some(role) = role_filter {
        sql.push_str(&format!("AND p.role = '{}' ", role.as_db_str()));
    }
    sql.push_str("ORDER BY p.last_name ASC, p.first_name ASC;");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        let role_str: String = row.get(5)?;
        let gender_str: String = row.get(6)?;
        let eff_pointe: i32 = row.get(12)?;
        let cons_pointe: i32 = row.get(13)?;
        let eff_flat: i32 = row.get(14)?;
        let cons_flat: i32 = row.get(15)?;
        let is_retired_raw: i32 = row.get(16).unwrap_or(0);
        let retirement_notes: Option<String> = row.get(17).unwrap_or(None);
        let specific_position: Option<String> = row.get(18).unwrap_or(None);

        Ok(PersonnelWithQuotaDto {
            id: row.get(0)?,
            legajo: row.get(1)?,
            first_name: row.get(2)?,
            last_name: row.get(3)?,
            nickname: row.get(4)?,
            role: PersonnelRole::from_db_str(&role_str).unwrap_or(PersonnelRole::Dancer),
            gender_group: GenderGroup::from_db_str(&gender_str).unwrap_or(GenderGroup::Female),
            category_id: row.get(7)?,
            category_name: row.get(8)?,
            is_active: row.get::<_, i32>(9)? == 1,
            default_pointe_raw_code: row.get(10)?,
            default_flat_size: row.get(11)?,
            effective_pointe_quota: eff_pointe,
            consumed_pointe_qty: cons_pointe,
            remaining_pointe_qty: eff_pointe - cons_pointe,
            effective_flat_quota: eff_flat,
            consumed_flat_qty: cons_flat,
            remaining_flat_qty: eff_flat - cons_flat,
            is_retired: is_retired_raw == 1,
            retirement_notes,
            specific_position,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

pub fn search_dancer_quick(
    conn: &Connection,
    query: &str,
) -> CoreResult<Vec<PersonnelWithQuotaDto>> {
    let clean = query.trim();
    if clean.is_empty() {
        return list_personnel(conn, Some(PersonnelRole::Dancer), true);
    }
    let pattern = format!("%{clean}%");

    let sql = "SELECT
            p.id,
            p.legajo,
            p.first_name,
            p.last_name,
            p.nickname,
            p.role,
            p.gender_group,
            p.category_id,
            c.name AS category_name,
            p.is_active,
            p.default_pointe_raw_code,
            p.default_flat_size,
            COALESCE(q.effective_pointe_quota, c.annual_pointe_quota, 0) AS effective_pointe_quota,
            COALESCE(q.consumed_pointe_qty, 0) AS consumed_pointe_qty,
            COALESCE(q.effective_flat_quota, c.annual_flat_quota, 0) AS effective_flat_quota,
            COALESCE(q.consumed_flat_qty, 0) AS consumed_flat_qty,
            COALESCE(p.is_retired, 0) AS is_retired,
            p.retirement_notes,
            p.specific_position
        FROM personnel p
        JOIN personnel_categories c ON c.id = p.category_id
        LEFT JOIN v_dancer_annual_quota_balance q ON q.personnel_id = p.id
        WHERE p.role = 'DANCER' AND p.is_active = 1
          AND (p.last_name LIKE ?1 OR p.first_name LIKE ?1 OR p.nickname LIKE ?1 OR p.legajo LIKE ?1)
        ORDER BY p.last_name ASC, p.first_name ASC;";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([&pattern], |row| {
        let role_str: String = row.get(5)?;
        let gender_str: String = row.get(6)?;
        let eff_pointe: i32 = row.get(12)?;
        let cons_pointe: i32 = row.get(13)?;
        let eff_flat: i32 = row.get(14)?;
        let cons_flat: i32 = row.get(15)?;
        let is_retired_raw: i32 = row.get(16).unwrap_or(0);
        let retirement_notes: Option<String> = row.get(17).unwrap_or(None);
        let specific_position: Option<String> = row.get(18).unwrap_or(None);

        Ok(PersonnelWithQuotaDto {
            id: row.get(0)?,
            legajo: row.get(1)?,
            first_name: row.get(2)?,
            last_name: row.get(3)?,
            nickname: row.get(4)?,
            role: PersonnelRole::from_db_str(&role_str).unwrap_or(PersonnelRole::Dancer),
            gender_group: GenderGroup::from_db_str(&gender_str).unwrap_or(GenderGroup::Female),
            category_id: row.get(7)?,
            category_name: row.get(8)?,
            is_active: row.get::<_, i32>(9)? == 1,
            default_pointe_raw_code: row.get(10)?,
            default_flat_size: row.get(11)?,
            effective_pointe_quota: eff_pointe,
            consumed_pointe_qty: cons_pointe,
            remaining_pointe_qty: eff_pointe - cons_pointe,
            effective_flat_quota: eff_flat,
            consumed_flat_qty: cons_flat,
            remaining_flat_qty: eff_flat - cons_flat,
            is_retired: is_retired_raw == 1,
            retirement_notes,
            specific_position,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

pub fn get_personnel_by_id(conn: &Connection, id: &str) -> CoreResult<PersonnelWithQuotaDto> {
    let sql = "SELECT
        p.id,
        p.legajo,
        p.first_name,
        p.last_name,
        p.nickname,
        p.role,
        p.gender_group,
        p.category_id,
        c.name AS category_name,
        p.is_active,
        p.default_pointe_raw_code,
        p.default_flat_size,
        COALESCE(q.effective_pointe_quota, c.annual_pointe_quota, 0) AS effective_pointe_quota,
        COALESCE(q.consumed_pointe_qty, 0) AS consumed_pointe_qty,
        COALESCE(q.effective_flat_quota, c.annual_flat_quota, 0) AS effective_flat_quota,
        COALESCE(q.consumed_flat_qty, 0) AS consumed_flat_qty,
        COALESCE(p.is_retired, 0) AS is_retired,
        p.retirement_notes,
        p.specific_position
    FROM personnel p
    JOIN personnel_categories c ON c.id = p.category_id
    LEFT JOIN v_dancer_annual_quota_balance q ON q.personnel_id = p.id
    WHERE p.id = ?1;";

    let mut stmt = conn.prepare(sql)?;
    let p = stmt.query_row([id], |row| {
        let role_str: String = row.get(5)?;
        let gender_str: String = row.get(6)?;
        let eff_pointe: i32 = row.get(12)?;
        let cons_pointe: i32 = row.get(13)?;
        let eff_flat: i32 = row.get(14)?;
        let cons_flat: i32 = row.get(15)?;
        let is_retired_raw: i32 = row.get(16).unwrap_or(0);
        let retirement_notes: Option<String> = row.get(17).unwrap_or(None);
        let specific_position: Option<String> = row.get(18).unwrap_or(None);

        Ok(PersonnelWithQuotaDto {
            id: row.get(0)?,
            legajo: row.get(1)?,
            first_name: row.get(2)?,
            last_name: row.get(3)?,
            nickname: row.get(4)?,
            role: PersonnelRole::from_db_str(&role_str).unwrap_or(PersonnelRole::Dancer),
            gender_group: GenderGroup::from_db_str(&gender_str).unwrap_or(GenderGroup::Female),
            category_id: row.get(7)?,
            category_name: row.get(8)?,
            is_active: row.get::<_, i32>(9)? == 1,
            default_pointe_raw_code: row.get(10)?,
            default_flat_size: row.get(11)?,
            effective_pointe_quota: eff_pointe,
            consumed_pointe_qty: cons_pointe,
            remaining_pointe_qty: eff_pointe - cons_pointe,
            effective_flat_quota: eff_flat,
            consumed_flat_qty: cons_flat,
            remaining_flat_qty: eff_flat - cons_flat,
            is_retired: is_retired_raw == 1,
            retirement_notes,
            specific_position,
        })
    })?;
    Ok(p)
}

pub fn upsert_personnel(
    conn: &Connection,
    payload: UpsertPersonnelPayload,
) -> CoreResult<PersonnelWithQuotaDto> {
    let first = payload.first_name.trim();
    let last = payload.last_name.trim();
    let role = payload.role.unwrap_or(PersonnelRole::Dancer);
    let gender = payload.gender_group.unwrap_or(GenderGroup::Female);
    let is_active = payload.is_active.unwrap_or(true);
    let typed_cat = match payload.category_name.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() => Some(category_id_for_name(conn, name)?),
        _ => None,
    };
    let cat_id = typed_cat.or(payload.category_id).unwrap_or_else(|| match role {
        PersonnelRole::Master | PersonnelRole::Pianist | PersonnelRole::Inspector => {
            "CAT_STAFF_DOCENTE".to_string()
        }
        PersonnelRole::Dancer => "CAT_CUERPO_BAILE".to_string(),
        _ => "CAT_CUERPO_BAILE".to_string(),
    });

    let cat_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM personnel_categories WHERE id = ?1",
            [&cat_id],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    let effective_cat_id = if cat_exists {
        cat_id
    } else {
        match role {
            PersonnelRole::Master | PersonnelRole::Pianist | PersonnelRole::Inspector => {
                "CAT_STAFF_DOCENTE".to_string()
            }
            _ => "CAT_CUERPO_BAILE".to_string(),
        }
    };

    let mut existing_id: Option<String> = None;
    if let Some(ref explicit_id) = payload.id {
        if !explicit_id.trim().is_empty() {
            let found: Option<String> = conn
                .query_row(
                    "SELECT id FROM personnel WHERE id = ?1",
                    [explicit_id.trim()],
                    |r| r.get(0),
                )
                .ok();
            if found.is_some() {
                existing_id = found;
            }
        }
    }

    if existing_id.is_none() {
        if let Some(ref leg) = payload.legajo {
            if !leg.trim().is_empty() {
                let found: Option<String> = conn
                    .query_row(
                        "SELECT id FROM personnel WHERE legajo = ?1",
                        [leg.trim()],
                        |r| r.get(0),
                    )
                    .ok();
                if found.is_some() {
                    existing_id = found;
                }
            }
        }
    }

    if existing_id.is_none() {
        let found: Option<String> = conn
            .query_row(
                "SELECT id FROM personnel WHERE LOWER(last_name) = LOWER(?1) AND LOWER(first_name) = LOWER(?2)",
                [last, first],
                |r| r.get(0),
            )
            .ok();
        if found.is_some() {
            existing_id = found;
        }
    }

    let final_id = match existing_id {
        Some(eid) => {
            conn.execute(
                "UPDATE personnel SET
                    legajo = COALESCE(?1, legajo),
                    first_name = ?2,
                    last_name = ?3,
                    nickname = COALESCE(?4, nickname),
                    role = ?5,
                    gender_group = ?6,
                    category_id = ?7,
                    default_pointe_raw_code = COALESCE(?8, default_pointe_raw_code),
                    default_flat_size = COALESCE(?9, default_flat_size),
                    is_active = ?10,
                    is_retired = COALESCE(?11, is_retired),
                    retirement_notes = COALESCE(?12, retirement_notes),
                    specific_position = COALESCE(?13, specific_position),
                    notes = COALESCE(?14, notes),
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE id = ?15",
                rusqlite::params![
                    payload.legajo.as_deref().map(str::trim),
                    first,
                    last,
                    payload.nickname.as_deref().map(str::trim),
                    role.as_db_str(),
                    gender.as_db_str(),
                    effective_cat_id,
                    payload.default_pointe_raw_code.as_deref().map(str::trim),
                    payload.default_flat_size.as_deref().map(str::trim),
                    if is_active { 1 } else { 0 },
                    payload.is_retired.map(|r| if r { 1 } else { 0 }),
                    payload.retirement_notes.as_deref().map(str::trim),
                    payload.specific_position.as_deref().map(str::trim),
                    payload.notes.as_deref().map(str::trim),
                    eid,
                ],
            )?;
            eid
        }
        None => {
            let gen_id = payload.id.clone().unwrap_or_else(|| {
                let slug: String = last
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .take(10)
                    .collect::<String>()
                    .to_uppercase();
                let random_suffix = &uuid::Uuid::new_v4().to_string()[..8];
                format!("PER_{}_{}", slug, random_suffix)
            });

            conn.execute(
                "INSERT INTO personnel (
                    id, legajo, first_name, last_name, nickname,
                    role, gender_group, category_id,
                    default_pointe_raw_code, default_flat_size,
                    is_active, is_retired, retirement_notes, specific_position, notes
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                rusqlite::params![
                    gen_id,
                    payload.legajo.as_deref().map(str::trim),
                    first,
                    last,
                    payload.nickname.as_deref().map(str::trim),
                    role.as_db_str(),
                    gender.as_db_str(),
                    effective_cat_id,
                    payload.default_pointe_raw_code.as_deref().map(str::trim),
                    payload.default_flat_size.as_deref().map(str::trim),
                    if is_active { 1 } else { 0 },
                    if payload.is_retired.unwrap_or(false) { 1 } else { 0 },
                    payload.retirement_notes.as_deref().map(str::trim),
                    payload.specific_position.as_deref().map(str::trim),
                    payload.notes.as_deref().map(str::trim),
                ],
            )?;
            gen_id
        }
    };

    get_personnel_by_id(conn, &final_id)
}

pub fn import_personnel_batch(
    conn: &mut Connection,
    items: Vec<UpsertPersonnelPayload>,
) -> CoreResult<ImportPersonnelResult> {
    let total = items.len();
    let mut inserted_count = 0;
    let mut updated_count = 0;
    let mut errors = Vec::new();

    let tx = conn.transaction()?;
    for (idx, item) in items.into_iter().enumerate() {
        if item.first_name.trim().is_empty() || item.last_name.trim().is_empty() {
            errors.push(format!("Fila #{}: Nombre y Apellido son obligatorios.", idx + 1));
            continue;
        }

        let first = item.first_name.trim();
        let last = item.last_name.trim();

        let mut exists = false;
        if let Some(ref explicit_id) = item.id {
            if !explicit_id.trim().is_empty() {
                let count: i64 = tx
                    .query_row(
                        "SELECT COUNT(*) FROM personnel WHERE id = ?1",
                        [explicit_id.trim()],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if count > 0 {
                    exists = true;
                }
            }
        }
        if !exists {
            if let Some(ref leg) = item.legajo {
                if !leg.trim().is_empty() {
                    let count: i64 = tx
                        .query_row(
                            "SELECT COUNT(*) FROM personnel WHERE legajo = ?1",
                            [leg.trim()],
                            |r| r.get(0),
                        )
                        .unwrap_or(0);
                    if count > 0 {
                        exists = true;
                    }
                }
            }
        }
        if !exists {
            let count: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM personnel WHERE LOWER(last_name) = LOWER(?1) AND LOWER(first_name) = LOWER(?2)",
                    [last, first],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if count > 0 {
                exists = true;
            }
        }

        match upsert_personnel(&tx, item) {
            Ok(_) => {
                if exists {
                    updated_count += 1;
                } else {
                    inserted_count += 1;
                }
            }
            Err(e) => {
                errors.push(format!("Fila #{}: {}", idx + 1, e));
            }
        }
    }
    tx.commit()?;

    Ok(ImportPersonnelResult {
        inserted_count,
        updated_count,
        total_processed: total,
        errors,
    })
}

pub fn delete_personnel(conn: &mut Connection, personnel_id: &str) -> CoreResult<()> {
    // Todo el borrado corre en UNA transacción: cualquier error la revierte completa.
    let tx = conn.transaction()?;

    // El histórico de stock, valeras, clases y ensayos referencia a la persona con
    // ON DELETE RESTRICT / NO ACTION: quien tenga historia no se elimina, queda de baja.
    let history_refs: i64 = tx.query_row(
        "SELECT
            (SELECT COUNT(*) FROM voucher_deliveries WHERE personnel_id = ?1 OR delivered_by_inspector_id = ?1) +
            (SELECT COUNT(*) FROM stock_reservations WHERE personnel_id = ?1 OR created_by = ?1) +
            (SELECT COUNT(*) FROM stock_ledger WHERE performed_by = ?1) +
            (SELECT COUNT(*) FROM morning_classes WHERE master_id = ?1 OR pianist_id = ?1) +
            (SELECT COUNT(*) FROM rehearsal_blocks WHERE master_id = ?1 OR pianist_id = ?1) +
            (SELECT COUNT(*) FROM daily_schedules WHERE created_by = ?1)",
        [personnel_id],
        |r| r.get(0),
    )?;

    if history_refs > 0 {
        tx.execute(
            "UPDATE personnel SET is_active = 0, notes = COALESCE(notes || ' | ', '') || 'Baja de padrón' WHERE id = ?1;",
            [personnel_id],
        )?;
    } else {
        // Purga explícita de los registros operativos de la persona
        tx.execute("DELETE FROM attendance_logs WHERE personnel_id = ?1;", [personnel_id])?;
        tx.execute("DELETE FROM physiotherapy_sessions WHERE personnel_id = ?1;", [personnel_id])?;
        tx.execute("DELETE FROM medical_leave_records WHERE personnel_id = ?1;", [personnel_id])?;
        // En elencos se elimina la fila propia; donde era pareja, la FK define SET NULL
        tx.execute("UPDATE rehearsal_cast_members SET partner_personnel_id = NULL WHERE partner_personnel_id = ?1;", [personnel_id])?;
        tx.execute("DELETE FROM rehearsal_cast_members WHERE personnel_id = ?1;", [personnel_id])?;
        tx.execute("UPDATE work_cast_plan_members SET partner_personnel_id = NULL WHERE partner_personnel_id = ?1;", [personnel_id])?;
        tx.execute("DELETE FROM work_cast_plan_members WHERE personnel_id = ?1;", [personnel_id])?;
        tx.execute("DELETE FROM personnel WHERE id = ?1;", [personnel_id])?;
    }

    tx.commit()?;
    Ok(())
}

/// `id` de la categoría de revista llamada `name` (sin distinguir mayúsculas); si no hay ninguna, la crea sin cupos.
pub fn category_id_for_name(conn: &Connection, name: &str) -> CoreResult<String> {
    let name = name.trim();
    let found: Option<String> = conn
        .query_row(
            "SELECT id FROM personnel_categories WHERE lower(trim(name)) = lower(?1) OR lower(code) = lower(?1) LIMIT 1",
            [name],
            |r| r.get(0),
        )
        .ok();
    if let Some(id) = found {
        return Ok(id);
    }
    let slug: String = name
        .chars()
        .map(|c| match c {
            'á' | 'Á' => 'A',
            'é' | 'É' => 'E',
            'í' | 'Í' => 'I',
            'ó' | 'Ó' => 'O',
            'ú' | 'Ú' | 'ü' | 'Ü' => 'U',
            'ñ' | 'Ñ' => 'N',
            c if c.is_ascii_alphanumeric() => c.to_ascii_uppercase(),
            _ => '_',
        })
        .collect();
    let base = format!("CAT_{}", slug.trim_matches('_'));
    let mut id = base.clone();
    let mut n = 2;
    while conn.query_row("SELECT COUNT(*) FROM personnel_categories WHERE id = ?1 OR code = ?1", [&id], |r| r.get::<_, i64>(0))? > 0 {
        id = format!("{base}_{n}");
        n += 1;
    }
    conn.execute("INSERT INTO personnel_categories (id, code, name) VALUES (?1, ?1, ?2)", rusqlite::params![id, name])?;
    Ok(id)
}
