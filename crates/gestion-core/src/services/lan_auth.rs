//! Autenticación de la sincronización por red local (Wi-Fi).
//!
//! Antes el servidor de sincronización aceptaba a cualquier dispositivo de la red: se podía pedir toda la base (personal,
//! licencias médicas, inventario) o inyectarle datos. Ahora los dos equipos comparten una **clave de sincronización** (se genera
//! sola la primera vez y se ve en la pestaña de sincronización; el otro equipo la ingresa una vez). Cada pedido y cada respuesta
//! llevan un código de autenticación (BLAKE3 con clave) que cubre la hora, qué se pidió y el contenido: sin la clave no se puede
//! pedir ni responder, y un pedido viejo (más de 2 minutos) se rechaza.
//!
//! No cifra el tráfico: en un Wi-Fi confiable del teatro alcanza para cortar el acceso de terceros, pero quien pueda *espiar* la red
//! seguiría viendo lo que viaja.

use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::{CoreError, CoreResult};

const KEY_SETTING: &str = "LAN_SYNC_KEY";
const CONTEXT: &str = "TeatroGestion LAN sync v1";
/// Cuánto se acepta que difieran los relojes de los dos equipos (y cuánto vale un pedido capturado).
pub const MAX_SKEW_SECS: u64 = 120;
pub const MIN_KEY_LEN: usize = 8;
pub const MAX_KEY_LEN: usize = 128;

/// La clave de este equipo; si todavía no hay, se genera una al azar y se guarda.
pub fn get_or_create_key(conn: &Connection) -> CoreResult<String> {
    let existing: Option<String> = conn.query_row("SELECT value FROM app_settings WHERE key = ?1;", [KEY_SETTING], |r| r.get(0)).optional()?;
    if let Some(k) = existing.filter(|k| k.trim().len() >= MIN_KEY_LEN) {
        return Ok(k.trim().to_string());
    }
    let key = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    save_key(conn, &key)?;
    Ok(key)
}

/// Cambia la clave (para adoptar la del otro equipo).
pub fn set_key(conn: &Connection, key: &str) -> CoreResult<()> {
    let key = key.trim();
    if key.len() < MIN_KEY_LEN || key.len() > MAX_KEY_LEN || key.chars().any(char::is_whitespace) {
        return Err(CoreError::ValidationFailed(format!("La clave de sincronización debe tener entre {MIN_KEY_LEN} y {MAX_KEY_LEN} caracteres, sin espacios.")));
    }
    save_key(conn, key)
}

fn save_key(conn: &Connection, key: &str) -> CoreResult<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at;",
        [KEY_SETTING, key],
    )?;
    Ok(())
}

fn derive(secret: &str) -> [u8; 32] {
    blake3::derive_key(CONTEXT, secret.trim().as_bytes())
}

fn mac(secret: &str, ts: u64, what: &str, body: &[u8]) -> blake3::Hash {
    let mut h = blake3::Hasher::new_keyed(&derive(secret));
    h.update(&ts.to_le_bytes());
    h.update(&(what.len() as u64).to_le_bytes());
    h.update(what.as_bytes());
    h.update(&(body.len() as u64).to_le_bytes());
    h.update(body);
    h.finalize()
}

/// Valor del encabezado de autenticación: `TG1 <hora>.<código>`. `what` dice qué se firma («POST /sync», «RESPONSE»).
pub fn sign(secret: &str, now: u64, what: &str, body: &[u8]) -> String {
    format!("TG1 {now}.{}", mac(secret, now, what, body).to_hex())
}

/// ¿El encabezado es válido para este contenido, con la clave y a esta hora? La comparación es de tiempo constante.
pub fn verify(secret: &str, header: &str, now: u64, what: &str, body: &[u8]) -> bool {
    let Some(rest) = header.trim().strip_prefix("TG1 ") else { return false };
    let Some((ts, code)) = rest.split_once('.') else { return false };
    let Ok(ts) = ts.parse::<u64>() else { return false };
    if ts.abs_diff(now) > MAX_SKEW_SECS {
        return false;
    }
    let Ok(code) = blake3::Hash::from_hex(code.trim()) else { return false };
    // `blake3::Hash` compara en tiempo constante.
    mac(secret, ts, what, body) == code
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseManager;

    #[test]
    fn a_signed_request_is_accepted_only_with_the_same_key_content_time_and_purpose() {
        let h = sign("clave-secreta-1", 1_000, "POST /sync", b"{\"a\":1}");
        assert!(verify("clave-secreta-1", &h, 1_000, "POST /sync", b"{\"a\":1}"));
        assert!(verify("clave-secreta-1", &h, 1_000 + MAX_SKEW_SECS, "POST /sync", b"{\"a\":1}"), "tolera la diferencia de relojes");
        assert!(!verify("otra-clave-123", &h, 1_000, "POST /sync", b"{\"a\":1}"), "otra clave");
        assert!(!verify("clave-secreta-1", &h, 1_000, "POST /sync", b"{\"a\":2}"), "contenido alterado");
        assert!(!verify("clave-secreta-1", &h, 1_000, "GET /ping", b"{\"a\":1}"), "otro pedido");
        assert!(!verify("clave-secreta-1", &h, 1_000 + MAX_SKEW_SECS + 1, "POST /sync", b"{\"a\":1}"), "pedido viejo (repetido)");
        assert!(!verify("clave-secreta-1", &h, 1_000 - MAX_SKEW_SECS - 1, "POST /sync", b"{\"a\":1}"), "pedido del futuro");
        // Un pedido viejo no se arregla cambiando la hora del encabezado: el código la incluye.
        let forged = h.replacen("1000.", "1100.", 1);
        assert!(!verify("clave-secreta-1", &forged, 1_100, "POST /sync", b"{\"a\":1}"));
        for bad in ["", "TG1", "TG1 x.y", "TG1 1000.", "TG2 1000.aa", "Bearer abc", "TG1 1000.zz"] {
            assert!(!verify("clave-secreta-1", bad, 1_000, "POST /sync", b"{\"a\":1}"), "{bad:?}");
        }
    }

    #[test]
    fn the_key_is_created_once_kept_and_can_be_replaced_with_a_valid_one() {
        let db = DatabaseManager::open_in_memory().unwrap();
        let conn = db.connection();
        let k1 = get_or_create_key(conn).unwrap();
        assert!(k1.len() >= 32);
        assert_eq!(get_or_create_key(conn).unwrap(), k1, "no cambia sola");
        set_key(conn, "  clave-del-otro-equipo  ").unwrap();
        assert_eq!(get_or_create_key(conn).unwrap(), "clave-del-otro-equipo");
        for bad in ["corta", "tiene espacios en medio", &"x".repeat(MAX_KEY_LEN + 1), ""] {
            assert!(set_key(conn, bad).is_err(), "{bad:?}");
        }
        assert_eq!(get_or_create_key(conn).unwrap(), "clave-del-otro-equipo", "una clave rechazada no cambia nada");
    }
}
