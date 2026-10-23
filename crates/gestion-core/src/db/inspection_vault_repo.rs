use crate::db::migrations::compute_sha256_hex;
use crate::dto::{
    CreateInspectionDocumentPayload, InspectionVaultDocumentDto, InspectionVaultFilterPayload,
    VaultMonthlyTotalsDto, VaultSecurityStatusDto,
};
use crate::error::{CoreError, CoreResult};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

const VAULT_SALT: &str = "boc_carina_lion_vault_salt_2026_";
const KEY_VAULT_HASH: &str = "INSPECTOR_VAULT_PASSWORD_HASH";
const KEY_VAULT_TIMEOUT: &str = "INSPECTOR_VAULT_LOCK_TIMEOUT_MINS";

pub fn hash_vault_password(password: &str) -> String {
    let salted = format!("{}{}", VAULT_SALT, password.trim());
    compute_sha256_hex(&salted)
}

pub fn get_vault_security_status(conn: &Connection) -> CoreResult<VaultSecurityStatusDto> {
    let hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1;",
            [KEY_VAULT_HASH],
            |r| r.get(0),
        )
        .optional()?;

    let timeout_str: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1;",
            [KEY_VAULT_TIMEOUT],
            |r| r.get(0),
        )
        .optional()?;

    let has_password = match hash {
        Some(h) => !h.trim().is_empty(),
        None => false,
    };

    let lock_timeout_minutes = timeout_str
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(5);

    Ok(VaultSecurityStatusDto {
        has_password,
        lock_timeout_minutes,
    })
}

pub fn verify_vault_password(conn: &Connection, password: &str) -> CoreResult<bool> {
    let hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1;",
            [KEY_VAULT_HASH],
            |r| r.get(0),
        )
        .optional()?;

    match hash {
        Some(stored_hash) if !stored_hash.trim().is_empty() => {
            let given_hash = hash_vault_password(password);
            Ok(stored_hash.trim() == given_hash)
        }
        _ => Ok(true), // Si no hay contraseña configurada, se considera desbloqueado
    }
}

pub fn set_vault_password(
    conn: &Connection,
    old_password: Option<&str>,
    new_password: &str,
    lock_timeout_minutes: Option<i32>,
) -> CoreResult<()> {
    if new_password.trim().is_empty() {
        return Err(CoreError::ValidationFailed(
            "La nueva contraseña no puede estar vacía.".into(),
        ));
    }

    let current_hash: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1;",
            [KEY_VAULT_HASH],
            |r| r.get(0),
        )
        .optional()?;

    if let Some(ref stored_hash) = current_hash {
        if !stored_hash.trim().is_empty() {
            let old_pwd = old_password.unwrap_or_default();
            if hash_vault_password(old_pwd) != stored_hash.trim() {
                return Err(CoreError::ValidationFailed(
                    "La contraseña actual de la inspectora es incorrecta.".into(),
                ));
            }
        }
    }

    let new_hash = hash_vault_password(new_password);
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at;",
        params![KEY_VAULT_HASH, new_hash],
    )?;

    if let Some(mins) = lock_timeout_minutes {
        let mins_str = mins.to_string();
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at;",
            params![KEY_VAULT_TIMEOUT, mins_str],
        )?;
    }

    Ok(())
}

pub fn admin_reset_vault_password(conn: &Connection, new_password: &str) -> CoreResult<()> {
    if new_password.trim().is_empty() {
        return Err(CoreError::ValidationFailed(
            "La nueva contraseña no puede estar vacía.".into(),
        ));
    }

    let new_hash = hash_vault_password(new_password);
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at;",
        params![KEY_VAULT_HASH, new_hash],
    )?;

    Ok(())
}

pub fn create_vault_document(
    conn: &Connection,
    payload: CreateInspectionDocumentPayload,
) -> CoreResult<InspectionVaultDocumentDto> {
    if payload.title.trim().is_empty() {
        return Err(CoreError::ValidationFailed(
            "El título del documento es obligatorio.".into(),
        ));
    }
    if payload.file_data.trim().is_empty() {
        return Err(CoreError::ValidationFailed(
            "El archivo o imagen adjunta no puede estar vacío.".into(),
        ));
    }

    let doc_id = payload.id.unwrap_or_else(|| {
        format!(
            "VAULT_{}_{}",
            payload.document_date.replace('-', "_"),
            Uuid::new_v4().simple()
        )
    });

    let file_size = payload
        .file_size_bytes
        .unwrap_or_else(|| (payload.file_data.len() as f64 * 0.75) as i64);

    conn.execute(
        "INSERT INTO inspection_vault_documents (
            id, title, category, document_date, personnel_id,
            file_name, file_type, file_size_bytes, file_data,
            amount_ars, supplier_vendor, notes, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9,
            ?10, ?11, ?12, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        )
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
            updated_at = excluded.updated_at;",
        params![
            doc_id,
            payload.title.trim(),
            payload.category.trim(),
            payload.document_date.trim(),
            payload.personnel_id,
            payload.file_name.trim(),
            payload.file_type.trim(),
            file_size,
            payload.file_data,
            payload.amount_ars,
            payload.supplier_vendor,
            payload.notes,
        ],
    )?;

    // Recuperar el documento recién insertado con el nombre del personal si corresponde
    let doc = conn.query_row(
        "SELECT 
            d.id, d.title, d.category, d.document_date, d.personnel_id,
            p.last_name || ', ' || p.first_name as personnel_name,
            d.file_name, d.file_type, d.file_size_bytes, d.file_data,
            d.amount_ars, d.supplier_vendor, d.notes, d.created_at, d.updated_at
         FROM inspection_vault_documents d
         LEFT JOIN personnel p ON p.id = d.personnel_id
         WHERE d.id = ?1;",
        [&doc_id],
        |r| {
            Ok(InspectionVaultDocumentDto {
                id: r.get(0)?,
                title: r.get(1)?,
                category: r.get(2)?,
                document_date: r.get(3)?,
                personnel_id: r.get(4)?,
                personnel_name: r.get(5)?,
                file_name: r.get(6)?,
                file_type: r.get(7)?,
                file_size_bytes: r.get(8)?,
                file_data: r.get(9)?,
                amount_ars: r.get(10)?,
                supplier_vendor: r.get(11)?,
                notes: r.get(12)?,
                created_at: r.get(13)?,
                updated_at: r.get(14)?,
            })
        },
    )?;

    Ok(doc)
}

pub fn list_vault_documents(
    conn: &Connection,
    filter: InspectionVaultFilterPayload,
) -> CoreResult<Vec<InspectionVaultDocumentDto>> {
    let mut sql = String::from(
        "SELECT 
            d.id, d.title, d.category, d.document_date, d.personnel_id,
            p.last_name || ', ' || p.first_name as personnel_name,
            d.file_name, d.file_type, d.file_size_bytes, d.file_data,
            d.amount_ars, d.supplier_vendor, d.notes, d.created_at, d.updated_at
         FROM inspection_vault_documents d
         LEFT JOIN personnel p ON p.id = d.personnel_id
         WHERE 1=1 ",
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(cat) = filter.category {
        if !cat.trim().is_empty() && cat != "ALL" {
            sql.push_str("AND d.category = ? ");
            params_vec.push(Box::new(cat));
        }
    }

    if let Some(p_id) = filter.personnel_id {
        if !p_id.trim().is_empty() {
            sql.push_str("AND d.personnel_id = ? ");
            params_vec.push(Box::new(p_id));
        }
    }

    if let Some(start) = filter.start_date {
        if !start.trim().is_empty() {
            sql.push_str("AND d.document_date >= ? ");
            params_vec.push(Box::new(start));
        }
    }

    if let Some(end) = filter.end_date {
        if !end.trim().is_empty() {
            sql.push_str("AND d.document_date <= ? ");
            params_vec.push(Box::new(end));
        }
    }

    if let Some(q) = filter.query {
        let trimmed = q.trim();
        if !trimmed.is_empty() {
            sql.push_str(
                "AND (d.title LIKE ? OR d.supplier_vendor LIKE ? OR d.notes LIKE ? OR d.file_name LIKE ?) ",
            );
            let pattern = format!("%{}%", trimmed);
            params_vec.push(Box::new(pattern.clone()));
            params_vec.push(Box::new(pattern.clone()));
            params_vec.push(Box::new(pattern.clone()));
            params_vec.push(Box::new(pattern));
        }
    }

    sql.push_str("ORDER BY d.document_date DESC, d.created_at DESC;");

    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_slice.as_slice(), |r| {
        Ok(InspectionVaultDocumentDto {
            id: r.get(0)?,
            title: r.get(1)?,
            category: r.get(2)?,
            document_date: r.get(3)?,
            personnel_id: r.get(4)?,
            personnel_name: r.get(5)?,
            file_name: r.get(6)?,
            file_type: r.get(7)?,
            file_size_bytes: r.get(8)?,
            file_data: r.get(9)?,
            amount_ars: r.get(10)?,
            supplier_vendor: r.get(11)?,
            notes: r.get(12)?,
            created_at: r.get(13)?,
            updated_at: r.get(14)?,
        })
    })?;

    let mut docs = Vec::new();
    for row in rows {
        docs.push(row?);
    }
    Ok(docs)
}

pub fn delete_vault_document(conn: &Connection, id: &str) -> CoreResult<()> {
    let rows = conn.execute("DELETE FROM inspection_vault_documents WHERE id = ?1;", [id])?;
    if rows == 0 {
        return Err(CoreError::EntityNotFound {
            entity: "inspection_vault_documents",
            id: id.to_string(),
        });
    }
    Ok(())
}

pub fn get_vault_monthly_totals(
    conn: &Connection,
    month: u32,
    year: i32,
) -> CoreResult<VaultMonthlyTotalsDto> {
    let month_prefix = format!("{:04}-{:02}-%", year, month);

    let total_amount: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount_ars), 0.0) 
             FROM inspection_vault_documents 
             WHERE document_date LIKE ?1 AND category = 'RECIBO_COMPRA';",
            [&month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let receipts_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inspection_vault_documents 
             WHERE document_date LIKE ?1 AND category = 'RECIBO_COMPRA';",
            [&month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let certificates_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inspection_vault_documents 
             WHERE document_date LIKE ?1 AND category = 'CERTIFICADO_MEDICO';",
            [&month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let documents_total_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inspection_vault_documents WHERE document_date LIKE ?1;",
            [&month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(VaultMonthlyTotalsDto {
        month,
        year,
        total_amount_ars: total_amount,
        receipts_count: receipts_count as usize,
        certificates_count: certificates_count as usize,
        documents_total_count: documents_total_count as usize,
    })
}
