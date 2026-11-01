use crate::domain::{
    GaynorManufacturingLine, PropDyeState, QuotaBucket, ReservationStatus,
    StockReservation, StockSubcategory,
};
use crate::dto::{
    ExecuteVoucherPayload, StockFilterPayload, StockItemAvailabilityDto, UpsertStockItemPayload,
    VoucherReceiptDto,
};
use crate::error::{CoreError, CoreResult};
use chrono::{Datelike, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

pub fn list_stock_catalog(
    conn: &Connection,
    filter: StockFilterPayload,
) -> CoreResult<Vec<StockItemAvailabilityDto>> {
    let mut sql = String::from(
        "SELECT
            id,
            sku_code,
            family,
            subcategory,
            brand,
            model_name,
            raw_factory_code,
            color,
            material,
            prop_dye_state,
            size_system,
            size_value,
            width_value,
            gm_line,
            gm_fit,
            gm_box,
            gm_shank,
            gm_vamp,
            gm_heel,
            duval_hardness,
            physical_qty,
            reserved_qty,
            free_available_qty,
            storage_bin_location,
            is_low_stock
        FROM v_stock_availability
        WHERE 1=1 ",
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(q) = filter.query {
        let clean = q.trim().to_string();
        if !clean.is_empty() {
            let pat = format!("%{clean}%");
            sql.push_str("AND (sku_code LIKE ? OR raw_factory_code LIKE ? OR model_name LIKE ? OR brand LIKE ?) ");
            params_vec.push(Box::new(pat.clone()));
            params_vec.push(Box::new(pat.clone()));
            params_vec.push(Box::new(pat.clone()));
            params_vec.push(Box::new(pat));
        }
    }

    if let Some(sub_str) = filter.subcategory_str {
        let clean = sub_str.trim().to_string();
        if !clean.is_empty() {
            sql.push_str("AND subcategory = ? ");
            params_vec.push(Box::new(clean));
        }
    } else if let Some(subcat) = filter.subcategory {
        let subcat_str = match subcat {
            StockSubcategory::MediaPuntaVaron => "MEDIA_PUNTA_VARON",
            StockSubcategory::MediaPuntaMujerUnisex => "MEDIA_PUNTA_MUJER_UNISEX",
            StockSubcategory::PuntaGaynorMinden => "PUNTA_GAYNOR_MINDEN",
            StockSubcategory::PuntaBloch => "PUNTA_BLOCH",
            StockSubcategory::PuntaSanshaDuval => "PUNTA_SANSHA_DUVAL",
            StockSubcategory::SuspensorMasculino => "SUSPENSOR_MASCULINO",
            StockSubcategory::MediaMalla => "MEDIA_MALLA",
            StockSubcategory::Cancan => "CANCAN",
            StockSubcategory::BombachaEscena => "BOMBACHA_ESCENA",
        };
        sql.push_str("AND subcategory = ? ");
        params_vec.push(Box::new(subcat_str));
    }

    if let Some(b_str) = filter.brand_str {
        let clean = b_str.trim().to_string();
        if !clean.is_empty() {
            sql.push_str("AND brand = ? ");
            params_vec.push(Box::new(clean));
        }
    } else if let Some(brand) = filter.brand {
        let brand_str = brand.display_name();
        sql.push_str("AND brand = ? ");
        params_vec.push(Box::new(brand_str));
    }

    if let Some(color) = filter.color {
        sql.push_str("AND color = ? ");
        params_vec.push(Box::new(color));
    }

    if let Some(sz) = filter.size_value {
        sql.push_str("AND size_value = ? ");
        params_vec.push(Box::new(sz));
    }

    if let Some(wv) = filter.width_value {
        sql.push_str("AND width_value = ? ");
        params_vec.push(Box::new(wv));
    }

    if let Some(gm_line) = filter.gm_line {
        let line_str = match gm_line {
            GaynorManufacturingLine::EuropeanTriangle => "EUROPEAN_TRIANGLE",
            GaynorManufacturingLine::LyraCircle => "LYRA_CIRCLE",
            GaynorManufacturingLine::AmericanStar => "AMERICAN_STAR",
        };
        sql.push_str("AND gm_line = ? ");
        params_vec.push(Box::new(line_str));
    }

    if let Some(dye) = filter.prop_dye_state {
        let dye_str = match dye {
            PropDyeState::Original => "ORIGINAL",
            PropDyeState::Tenida => "TENIDA",
        };
        sql.push_str("AND prop_dye_state = ? ");
        params_vec.push(Box::new(dye_str));
    }

    if filter.low_stock_only {
        sql.push_str("AND is_low_stock = 1 ");
    }

    sql.push_str("ORDER BY brand ASC, model_name ASC, size_value ASC;");

    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_slice.as_slice(), |row| {
        Ok(StockItemAvailabilityDto {
            id: row.get(0)?,
            sku_code: row.get(1)?,
            family: row.get(2)?,
            subcategory: row.get(3)?,
            brand: row.get(4)?,
            model_name: row.get(5)?,
            raw_factory_code: row.get(6)?,
            color: row.get(7)?,
            material: row.get(8)?,
            prop_dye_state: row.get(9)?,
            size_system: row.get(10)?,
            size_value: row.get(11)?,
            width_value: row.get(12)?,
            gm_line: row.get(13)?,
            gm_fit: row.get(14)?,
            gm_box: row.get(15)?,
            gm_shank: row.get(16)?,
            gm_vamp: row.get(17)?,
            gm_heel: row.get(18)?,
            duval_hardness: row.get(19)?,
            physical_qty: row.get(20)?,
            reserved_qty: row.get(21)?,
            free_available_qty: row.get(22)?,
            storage_bin_location: row.get(23)?,
            is_low_stock: row.get::<_, i64>(24)? == 1,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

pub fn create_nominal_reservation(
    conn: &mut Connection,
    stock_item_id: &str,
    personnel_id: &str,
    label: &str,
    quantity: u32,
    notes: Option<&str>,
) -> CoreResult<StockReservation> {
    if quantity == 0 {
        return Err(CoreError::IntegrityViolation(
            "La cantidad de la reserva debe ser mayor a cero.".to_string(),
        ));
    }

    let tx = conn.transaction()?;

    let (physical_qty, reserved_qty, sku_code): (u32, u32, String) = tx.query_row(
        "SELECT physical_qty, reserved_qty, sku_code FROM stock_items WHERE id = ?1;",
        [stock_item_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CoreError::EntityNotFound {
            entity: "stock_items",
            id: stock_item_id.to_string(),
        },
        other => CoreError::Database(other),
    })?;

    let free = physical_qty.saturating_sub(reserved_qty);
    if free < quantity {
        return Err(CoreError::InsufficientFreeStock {
            sku_code,
            requested: quantity,
            available_free: free,
        });
    }

    let res_id = format!("RES_{}", Uuid::new_v4().simple());
    let now_iso = Utc::now().to_rfc3339();

    tx.execute(
        "INSERT INTO stock_reservations (id, stock_item_id, personnel_id, reserved_for_label, quantity, status, notes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'ACTIVE', ?6, ?7);",
        params![res_id, stock_item_id, personnel_id, label, quantity, notes, now_iso],
    )?;

    tx.commit()?;

    Ok(StockReservation {
        id: res_id,
        stock_item_id: stock_item_id.to_string(),
        personnel_id: personnel_id.to_string(),
        reserved_for_label: label.to_string(),
        quantity,
        status: ReservationStatus::Active,
        work_id: None,
        notes: notes.map(|s| s.to_string()),
        created_at: now_iso,
        resolved_at: None,
    })
}

pub fn release_nominal_reservation(
    conn: &mut Connection,
    reservation_id: &str,
) -> CoreResult<()> {
    let tx = conn.transaction()?;

    let rows_updated = tx.execute(
        "UPDATE stock_reservations
         SET status = 'RELEASED', resolved_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?1 AND status = 'ACTIVE';",
        [reservation_id],
    )?;

    if rows_updated == 0 {
        return Err(CoreError::IntegrityViolation(
            "Reserva no encontrada o no está en estado activo.".to_string(),
        ));
    }

    tx.commit()?;
    Ok(())
}

pub fn execute_voucher_delivery(
    conn: &mut Connection,
    payload: ExecuteVoucherPayload,
) -> CoreResult<VoucherReceiptDto> {
    if payload.items.is_empty() {
        return Err(CoreError::IntegrityViolation(
            "La valera debe contener al menos un ítem.".to_string(),
        ));
    }

    // Hora local: con UTC, una entrega después de las 21 h del 31 de diciembre caía en la temporada siguiente.
    let season_year = payload.season_year.unwrap_or_else(|| chrono::Local::now().year());

    if payload.is_quota_exception {
        match &payload.exception_reason {
            Some(reason) if reason.trim().len() >= 5 => {}
            _ => return Err(CoreError::QuotaExceptionReasonRequired),
        }
    }

    let tx = conn.transaction()?;

    let recipient_name: String = tx.query_row(
        "SELECT (last_name || ', ' || first_name || COALESCE(' (\"' || nickname || '\")', ''))
         FROM personnel WHERE id = ?1;",
        [&payload.personnel_id],
        |r| r.get(0),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CoreError::EntityNotFound {
            entity: "personnel",
            id: payload.personnel_id.clone(),
        },
        other => CoreError::Database(other),
    })?;

    // Validar cupos anuales acumulados si no es excepción
    if !payload.is_quota_exception {
        let mut requested_pointe: u32 = 0;
        let mut requested_flat: u32 = 0;

        for item in &payload.items {
            match item.quota_bucket {
                QuotaBucket::Pointe => requested_pointe += item.quantity,
                QuotaBucket::Flat => requested_flat += item.quantity,
                _ => {}
            }
        }

        if requested_pointe > 0 || requested_flat > 0 {
            let quota_row: Option<(i32, i32, i32, i32)> = tx.query_row(
                "SELECT effective_pointe_quota, consumed_pointe_qty, effective_flat_quota, consumed_flat_qty
                 FROM v_dancer_annual_quota_balance
                 WHERE personnel_id = ?1;",
                [&payload.personnel_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            ).ok();

            if let Some((eff_p, cons_p, eff_f, cons_f)) = quota_row {
                let rem_p = eff_p - cons_p;
                if requested_pointe > 0 && (rem_p < requested_pointe as i32) {
                    return Err(CoreError::AnnualQuotaExceeded {
                        personnel_id: payload.personnel_id,
                        dancer_name: recipient_name,
                        quota_bucket: "POINTE".to_string(),
                        remaining: rem_p,
                        requested: requested_pointe,
                    });
                }

                let rem_f = eff_f - cons_f;
                if requested_flat > 0 && (rem_f < requested_flat as i32) {
                    return Err(CoreError::AnnualQuotaExceeded {
                        personnel_id: payload.personnel_id,
                        dancer_name: recipient_name,
                        quota_bucket: "FLAT".to_string(),
                        remaining: rem_f,
                        requested: requested_flat,
                    });
                }
            }
        }
    }

    let voucher_id = format!("VOU_{}", Uuid::new_v4().simple());
    let voucher_seq: i64 = tx.query_row(
        "SELECT COALESCE(COUNT(*), 0) + 1 FROM voucher_deliveries WHERE season_year = ?1;",
        [season_year],
        |r| r.get(0),
    )?;
    let voucher_number = format!("VAL-{}-{:06}", season_year, voucher_seq);
    let now_iso = Utc::now().to_rfc3339();

    tx.execute(
        "INSERT INTO voucher_deliveries (
            id, voucher_number, personnel_id, season_year, delivered_by_inspector_id,
            delivery_timestamp, is_quota_exception, exception_reason, device_id, signature_svg_strokes, notes
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);",
        params![
            voucher_id,
            voucher_number,
            payload.personnel_id,
            season_year,
            payload.delivered_by_inspector_id,
            now_iso,
            if payload.is_quota_exception { 1 } else { 0 },
            payload.exception_reason,
            payload.device_id,
            payload.signature_svg_strokes,
            payload.notes,
        ],
    )?;

    for item in &payload.items {
        let item_id = format!("VDI_{}", Uuid::new_v4().simple());
        let bucket_str = match item.quota_bucket {
            QuotaBucket::Pointe => "POINTE",
            QuotaBucket::Flat => "FLAT",
            QuotaBucket::Tights => "TIGHTS",
            QuotaBucket::Supporter => "SUPPORTER",
            QuotaBucket::Briefs => "BRIEFS",
            QuotaBucket::Exempt => "EXEMPT",
        };

        let (mut phys, mut res, sku): (i64, i64, String) = tx.query_row(
            "SELECT physical_qty, reserved_qty, sku_code FROM stock_items WHERE id = ?1;",
            [&item.stock_item_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => CoreError::EntityNotFound {
                entity: "stock_items",
                id: item.stock_item_id.clone(),
            },
            other => CoreError::Database(other),
        })?;

        if let Some(res_id) = &item.reservation_id {
            // Reserva nominativa consumida
            tx.execute(
                "UPDATE stock_reservations
                 SET status = 'DELIVERED', resolved_at = ?1
                 WHERE id = ?2 AND status = 'ACTIVE';",
                params![now_iso, res_id],
            )?;

            phys = phys.saturating_sub(item.quantity as i64);
            res = res.saturating_sub(item.quantity as i64);

            tx.execute(
                "UPDATE stock_items
                 SET physical_qty = ?1, reserved_qty = ?2, updated_at = ?3
                 WHERE id = ?4;",
                params![phys, res, now_iso, item.stock_item_id],
            )?;
        } else {
            // Entrega directa desde stock libre
            let free = phys.saturating_sub(res);
            if free < (item.quantity as i64) {
                return Err(CoreError::InsufficientFreeStock {
                    sku_code: sku,
                    requested: item.quantity,
                    available_free: free as u32,
                });
            }

            phys = phys.saturating_sub(item.quantity as i64);
            tx.execute(
                "UPDATE stock_items
                 SET physical_qty = ?1, updated_at = ?2
                 WHERE id = ?3;",
                params![phys, now_iso, item.stock_item_id],
            )?;
        }

        // Registrar detalle de la valera
        tx.execute(
            "INSERT INTO voucher_delivery_items (id, voucher_id, stock_item_id, reservation_id, quantity, quota_bucket)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            params![item_id, voucher_id, item.stock_item_id, item.reservation_id, item.quantity, bucket_str],
        )?;

        // Registrar en Kardex / Libro mayor
        let ledger_id = format!("LED_{}", Uuid::new_v4().simple());
        tx.execute(
            "INSERT INTO stock_ledger (
                id, stock_item_id, movement_type, delta_physical, delta_reserved,
                balance_physical_after, balance_reserved_after, reference_id, performed_by, notes, created_at
             ) VALUES (?1, ?2, 'VOUCHER_DELIVERY_OUT', ?3, 0, ?4, ?5, ?6, ?7, 'Entrega de valera', ?8);",
            params![
                ledger_id,
                item.stock_item_id,
                -(item.quantity as i64),
                phys,
                res,
                voucher_id,
                payload.delivered_by_inspector_id,
                now_iso,
            ],
        )?;
    }

    tx.commit()?;

    Ok(VoucherReceiptDto {
        voucher_id,
        voucher_number,
        delivery_timestamp: now_iso,
        recipient_name,
        total_items_count: payload.items.len(),
        is_quota_exception: payload.is_quota_exception,
        message: "Entrega de valera registrada con éxito y stock descontado.".to_string(),
    })
}

pub fn upsert_stock_item(
    conn: &mut Connection,
    payload: UpsertStockItemPayload,
) -> CoreResult<StockItemAvailabilityDto> {
    let tx = conn.transaction()?;

    let final_id = payload
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("STK_{}", Uuid::new_v4().simple()));

    let subcat_abbr = payload
        .subcategory
        .chars()
        .take(3)
        .collect::<String>()
        .to_uppercase();
    let brand_abbr = payload
        .brand
        .chars()
        .take(3)
        .collect::<String>()
        .to_uppercase();
    let color_abbr = payload
        .color
        .chars()
        .take(2)
        .collect::<String>()
        .to_uppercase();

    let final_sku = payload
        .sku_code
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            format!("{}-{}-{}-{}", subcat_abbr, brand_abbr, color_abbr, payload.size_value.trim())
        });

    let existing: Option<(i64, i64)> = tx
        .query_row(
            "SELECT physical_qty, reserved_qty FROM stock_items WHERE id = ?1",
            params![&final_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    let quota_str = match payload.quota_bucket {
        QuotaBucket::Pointe => "POINTE",
        QuotaBucket::Flat => "FLAT",
        QuotaBucket::Tights => "TIGHTS",
        QuotaBucket::Supporter => "SUPPORTER",
        QuotaBucket::Briefs => "BRIEFS",
        QuotaBucket::Exempt => "EXEMPT",
    };

    if let Some((old_phys, old_res)) = existing {
        let delta = (payload.physical_qty as i64) - old_phys;
        tx.execute(
            "UPDATE stock_items SET
                sku_code = ?1,
                family = ?2,
                subcategory = ?3,
                quota_bucket = ?4,
                brand = ?5,
                model_name = ?6,
                raw_factory_code = ?7,
                color = ?8,
                material = ?9,
                size_system = ?10,
                size_value = ?11,
                width_value = ?12,
                physical_qty = ?13,
                min_alert_qty = ?14,
                storage_bin_location = ?15,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?16",
            params![
                final_sku,
                payload.family,
                payload.subcategory,
                quota_str,
                payload.brand,
                payload.model_name,
                payload.raw_factory_code,
                payload.color,
                payload.material,
                payload.size_system,
                payload.size_value,
                payload.width_value,
                payload.physical_qty,
                payload.min_alert_qty,
                payload.storage_bin_location,
                final_id,
            ],
        )?;

        if delta != 0 {
            let ledger_id = format!("LED_{}", Uuid::new_v4().simple());
            tx.execute(
                "INSERT INTO stock_ledger (
                    id, stock_item_id, movement_type, delta_physical, delta_reserved,
                    balance_physical_after, balance_reserved_after, notes
                ) VALUES (?1, ?2, 'AUDIT_ADJUSTMENT', ?3, 0, ?4, ?5, 'Ajuste manual de inventario')",
                params![ledger_id, final_id, delta, payload.physical_qty, old_res],
            )?;
        }
    } else {
        tx.execute(
            "INSERT INTO stock_items (
                id, sku_code, barcode, family, subcategory, quota_bucket, brand,
                model_name, raw_factory_code, color, material, prop_dye_state,
                dyed_for_work_id, size_system, size_value, width_value,
                gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
                physical_qty, reserved_qty, min_alert_qty, storage_bin_location, updated_at
            ) VALUES (
                ?1, ?2, NULL, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, 'ORIGINAL',
                NULL, ?11, ?12, ?13,
                NULL, NULL, NULL, NULL, NULL, NULL, NULL,
                ?14, 0, ?15, ?16, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            )",
            params![
                final_id,
                final_sku,
                payload.family,
                payload.subcategory,
                quota_str,
                payload.brand,
                payload.model_name,
                payload.raw_factory_code,
                payload.color,
                payload.material,
                payload.size_system,
                payload.size_value,
                payload.width_value,
                payload.physical_qty,
                payload.min_alert_qty,
                payload.storage_bin_location,
            ],
        )?;

        if payload.physical_qty > 0 {
            let ledger_id = format!("LED_{}", Uuid::new_v4().simple());
            tx.execute(
                "INSERT INTO stock_ledger (
                    id, stock_item_id, movement_type, delta_physical, delta_reserved,
                    balance_physical_after, balance_reserved_after, notes
                ) VALUES (?1, ?2, 'INITIAL_INVENTORY', ?3, 0, ?3, 0, 'Alta inicial de inventario dinámico')",
                params![ledger_id, final_id, payload.physical_qty],
            )?;
        }
    }

    let item = tx.query_row(
        "SELECT
            id, sku_code, family, subcategory, brand, model_name, raw_factory_code,
            color, material, prop_dye_state, size_system, size_value, width_value,
            gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
            physical_qty, reserved_qty, free_available_qty, storage_bin_location, is_low_stock
        FROM v_stock_availability
        WHERE id = ?1",
        params![&final_id],
        |row| {
            Ok(StockItemAvailabilityDto {
                id: row.get(0)?,
                sku_code: row.get(1)?,
                family: row.get(2)?,
                subcategory: row.get(3)?,
                brand: row.get(4)?,
                model_name: row.get(5)?,
                raw_factory_code: row.get(6)?,
                color: row.get(7)?,
                material: row.get(8)?,
                prop_dye_state: row.get(9)?,
                size_system: row.get(10)?,
                size_value: row.get(11)?,
                width_value: row.get(12)?,
                gm_line: row.get(13)?,
                gm_fit: row.get(14)?,
                gm_box: row.get(15)?,
                gm_shank: row.get(16)?,
                gm_vamp: row.get(17)?,
                gm_heel: row.get(18)?,
                duval_hardness: row.get(19)?,
                physical_qty: row.get(20)?,
                reserved_qty: row.get(21)?,
                free_available_qty: row.get(22)?,
                storage_bin_location: row.get(23)?,
                is_low_stock: row.get::<_, i64>(24)? == 1,
            })
        },
    )?;

    tx.commit()?;
    Ok(item)
}
