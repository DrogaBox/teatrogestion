pub mod attendance_repo;
pub mod inspection_vault_repo;
pub mod migrations;
pub mod personnel_repo;
pub mod schedule_repo;
pub mod soloist_repo;
pub mod stock_repo;
pub mod sync_repo;
pub mod works_repo;

pub use attendance_repo::*;
pub use inspection_vault_repo::*;
pub use soloist_repo::*;
pub use sync_repo::*;
pub use works_repo::*;

use crate::error::CoreResult;
use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;

pub struct DatabaseManager {
    conn: Connection,
}

impl DatabaseManager {
    /// Abre o crea la base de datos SQLite en disco, aplica los PRAGMAs de alta confiabilidad (WAL)
    /// y ejecuta las migraciones embebidas con verificación SHA-256.
    pub fn open<P: AsRef<Path>>(db_path: P) -> CoreResult<Self> {
        let mut conn = Connection::open(db_path)?;
        Self::configure_pragmas(&mut conn)?;
        migrations::run_migrations(&mut conn)?;
        Ok(Self { conn })
    }

    /// Abre una base de datos SQLite en memoria (`:memory:`), aplica PRAGMAs y ejecuta migraciones.
    pub fn open_in_memory() -> CoreResult<Self> {
        let mut conn = Connection::open_in_memory()?;
        Self::configure_pragmas(&mut conn)?;
        migrations::run_migrations(&mut conn)?;
        Ok(Self { conn })
    }

    /// Aplica los PRAGMAs obligatorios definidos en la arquitectura de TeatroGestión:
    /// - `journal_mode = WAL`
    /// - `synchronous = NORMAL`
    /// - `foreign_keys = ON`
    /// - `busy_timeout = 5000`
    pub fn configure_pragmas(conn: &mut Connection) -> CoreResult<()> {
        conn.busy_timeout(Duration::from_millis(5000))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA temp_store = MEMORY;",
        )?;
        Ok(())
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    pub fn into_inner(self) -> Connection {
        self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pragmas_and_foreign_keys_enabled() {
        let db = DatabaseManager::open_in_memory().expect("Debe abrir DB en memoria");
        let fk_enabled: i32 = db
            .connection()
            .query_row("PRAGMA foreign_keys;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);

        // Verificar que las salas principales existen tras el seed
        let room_count: i32 = db
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM rooms WHERE code IN ('SALA_1', 'SALA_2', 'SALA_3', 'ESCENARIO')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(room_count, 4);
    }
}
