use crate::error::{CoreError, CoreResult};
use rusqlite::{ params, Connection, OptionalExtension };

pub struct EmbeddedMigration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const EMBEDDED_MIGRATIONS: &[EmbeddedMigration] = &[
    EmbeddedMigration {
        version: 1,
        name: "0001_initial_schema",
        sql: include_str!("../../migrations/0001_initial_schema.sql"),
    },
    EmbeddedMigration {
        version: 2,
        name: "0002_seed_catalogs_and_staff",
        sql: include_str!("../../migrations/0002_seed_catalogs_and_staff.sql"),
    },
    EmbeddedMigration {
        version: 3,
        name: "0003_real_inspector_notebook_inventory",
        sql: include_str!("../../migrations/0003_real_inspector_notebook_inventory.sql"),
    },
    EmbeddedMigration {
        version: 4,
        name: "0004_attendance_medical_and_weekly_sheet",
        sql: include_str!("../../migrations/0004_attendance_medical_and_weekly_sheet.sql"),
    },
    EmbeddedMigration {
        version: 5,
        name: "0005_retirement_and_medical_leaves_range",
        sql: include_str!("../../migrations/0005_retirement_and_medical_leaves_range.sql"),
    },
    EmbeddedMigration {
        version: 6,
        name: "0006_expanded_ballet_inventory_and_dynamic_stock",
        sql: include_str!("../../migrations/0006_expanded_ballet_inventory_and_dynamic_stock.sql"),
    },
    EmbeddedMigration {
        version: 7,
        name: "0007_add_sala_2_and_flexible_tours",
        sql: include_str!("../../migrations/0007_add_sala_2_and_flexible_tours.sql"),
    },
    EmbeddedMigration {
        version: 8,
        name: "0008_work_casting_plans",
        sql: include_str!("../../migrations/0008_work_casting_plans.sql"),
    },
    EmbeddedMigration {
        version: 9,
        name: "0009_solisticos_and_real_bop_attendance_matrix",
        sql: include_str!("../../migrations/0009_solisticos_and_real_bop_attendance_matrix.sql"),
    },
    EmbeddedMigration {
        version: 10,
        name: "0010_use_dni_in_personnel",
        sql: include_str!("../../migrations/0010_use_dni_in_personnel.sql"),
    },
    EmbeddedMigration {
        version: 11,
        name: "0011_rest_and_compensatory_rules",
        sql: include_str!("../../migrations/0011_rest_and_compensatory_rules.sql"),
    },
    EmbeddedMigration {
        version: 12,
        name: "0012_inspection_vault_and_compensatory_rules",
        sql: include_str!("../../migrations/0012_inspection_vault_and_compensatory_rules.sql"),
    },
    EmbeddedMigration {
        version: 13,
        name: "0013_sound_technician_assignment",
        sql: include_str!("../../migrations/0013_sound_technician_assignment.sql"),
    },
];

/// Implementación determinista de SHA-256 (FIPS 180-4) libre de dependencias externas
pub fn compute_sha256_hex(input: &str) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bytes = input.as_bytes();
    let bit_len = (bytes.len() as u64) * 8;

    let mut padded = Vec::with_capacity(((bytes.len() + 9 + 63) / 64) * 64);
    padded.extend_from_slice(bytes);
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMigrationRecord {
    pub version: u32,
    pub name: String,
    pub checksum_sha256: String,
    pub applied_at: String,
}

/// Ejecuta todas las migraciones pendientes verificando el checksum SHA-256 de las ya aplicadas.
pub fn run_migrations(conn: &mut Connection) -> CoreResult<Vec<AppliedMigrationRecord>> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            checksum_sha256 TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        ) STRICT;",
    )?;

    let fk_was_on: bool = conn
        .query_row("PRAGMA foreign_keys;", [], |r| r.get(0))
        .unwrap_or(true);
    if fk_was_on {
        conn.execute_batch("PRAGMA foreign_keys = OFF;")?;
    }

    for migration in EMBEDDED_MIGRATIONS {
        let expected_checksum = compute_sha256_hex(migration.sql);

        let existing: Option<(String, String)> = conn
            .query_row(
                "SELECT name, checksum_sha256 FROM _schema_migrations WHERE version = ?1",
                params![migration.version],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        if let Some((_recorded_name, recorded_checksum)) = existing {
            if recorded_checksum != expected_checksum {
                if fk_was_on {
                    let _ = conn.execute_batch("PRAGMA foreign_keys = ON;");
                }
                return Err(CoreError::MigrationChecksumMismatch {
                    version: migration.version,
                    name: migration.name.to_string(),
                    expected: expected_checksum,
                    found: recorded_checksum,
                });
            }
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(migration.sql).map_err(|e| {
            CoreError::MigrationExecutionFailed {
                version: migration.version,
                name: migration.name.to_string(),
                reason: e.to_string(),
            }
        })?;

        tx.execute(
            "INSERT INTO _schema_migrations (version, name, checksum_sha256) VALUES (?1, ?2, ?3)",
            params![migration.version, migration.name, expected_checksum],
        )?;
        tx.commit()?;
    }

    if fk_was_on {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    }

    list_applied_migrations(conn)
}

pub fn list_applied_migrations(conn: &Connection) -> CoreResult<Vec<AppliedMigrationRecord>> {
    let mut stmt = conn.prepare(
        "SELECT version, name, checksum_sha256, applied_at
         FROM _schema_migrations
         ORDER BY version ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(AppliedMigrationRecord {
            version: row.get(0)?,
            name: row.get(1)?,
            checksum_sha256: row.get(2)?,
            applied_at: row.get(3)?,
        })
    })?;

    let mut records = Vec::new();
    for r in rows {
        records.push(r?);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseManager;

    #[test]
    fn test_sha256_nist_vectors() {
        assert_eq!(
            compute_sha256_hex(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            compute_sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_migrations_idempotency_and_seed_integrity() {
        let mut db = DatabaseManager::open_in_memory().expect("Debe inicializar DB en memoria");

        // Segunda ejecución debe ser idempotente
        let applied = run_migrations(db.connection_mut()).expect("Re-ejecución idempotente");
        assert_eq!(applied.len(), EMBEDDED_MIGRATIONS.len());

        // Verificar que las reservas iniciales ('Popi' y 'Brenda') activaron el trigger sobre stock_items
        let (phys, res, free): (i32, i32, i32) = db
            .connection()
            .query_row(
                "SELECT physical_qty, reserved_qty, free_available_qty
                 FROM v_stock_availability
                 WHERE id = 'STK_GM_EUR_SC_75W4PDH'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .expect("Debe existir el ítem Gaynor SC - 7.5W4PDH");

        assert_eq!(phys, 10);
        assert_eq!(res, 2); // 2 pares reservados para Popi
        assert_eq!(free, 8);
    }

    #[test]
    fn test_migration_checksum_tamper_detection() {
        let mut db = DatabaseManager::open_in_memory().expect("Debe abrir DB en memoria");
        db.connection_mut()
            .execute(
                "UPDATE _schema_migrations SET checksum_sha256 = '0000_corrupted' WHERE version = 1",
                [],
            )
            .unwrap();

        let err = run_migrations(db.connection_mut()).unwrap_err();
        match err {
            CoreError::MigrationChecksumMismatch { version, .. } => assert_eq!(version, 1),
            other => panic!("Se esperaba MigrationChecksumMismatch, se obtuvo: {:?}", other),
        }
    }
}
