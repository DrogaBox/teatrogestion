//! Lógica de `Actualizar_TeatroGestion.exe`: reemplaza los programas de TeatroGestión por los de la versión nueva
//! **sin tocar nada de lo que es de la usuaria**: ni la base de datos (personal, asistencia, inventario, Bóveda), ni la
//! licencia, ni las marcas de reloj. Antes de cambiar nada respalda los datos y comprueba que la base sea compatible con la
//! versión nueva; si algo falla en el medio, deja todo como estaba.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use gestion_core::db::migrations::{checksum_candidates, EMBEDDED_MIGRATIONS};
use rusqlite::{Connection, OpenFlags};

pub mod payload {
    include!(concat!(env!("OUT_DIR"), "/payload.rs"));
}

pub const EXE_NAME: &str = "TeatroGestion.exe";
pub const DLL_NAME: &str = "WebView2Loader.dll";
pub const ICO_NAME: &str = "teatrogestion.ico";
pub const DB_NAME: &str = "teatro_gestion.sqlite";
pub const KEEP_BACKUPS: usize = 5;

/// Los archivos de la versión nueva.
#[derive(Clone, Copy)]
pub struct Payload<'a> {
    pub exe: &'a [u8],
    pub dll: &'a [u8],
    pub ico: &'a [u8],
}

impl Payload<'static> {
    pub fn embedded() -> Self {
        Payload { exe: payload::EXE, dll: payload::DLL, ico: payload::ICO }
    }
}

impl Payload<'_> {
    /// (nombre en la carpeta de instalación, contenido). El icono y la DLL son opcionales; el programa no.
    fn files(&self) -> Vec<(&'static str, &[u8])> {
        let mut v = vec![(EXE_NAME, self.exe)];
        if !self.dll.is_empty() {
            v.push((DLL_NAME, self.dll));
        }
        if !self.ico.is_empty() {
            v.push((ICO_NAME, self.ico));
        }
        v
    }
}

// ---------------------------------------------------------------------------------------------
// Dónde está instalado y dónde están los datos
// ---------------------------------------------------------------------------------------------

/// Carpetas donde puede estar instalado, en orden de preferencia (`extra`: la que se pidió por argumento).
pub fn install_candidates(extra: Option<&Path>) -> Vec<PathBuf> {
    let mut c = Vec::new();
    if let Some(e) = extra {
        c.push(e.to_path_buf());
    }
    c.push(PathBuf::from(r"C:\TeatroGestion"));
    if let Some(l) = std::env::var_os("LOCALAPPDATA") {
        c.push(PathBuf::from(l).join("TeatroGestion"));
    }
    if let Some(u) = std::env::var_os("USERPROFILE") {
        c.push(PathBuf::from(u).join("TeatroGestion"));
    }
    c
}

pub fn find_install(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|d| d.join(EXE_NAME).is_file()).cloned()
}

/// Carpeta de datos de la app (`%APPDATA%\com.teatrogestion.app`).
pub fn data_dir() -> PathBuf {
    if let Some(a) = std::env::var_os("APPDATA") {
        return PathBuf::from(a).join("com.teatrogestion.app");
    }
    if let Some(l) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(l).join("com.teatrogestion.app");
    }
    PathBuf::from(r"C:\TeatroGestion\data")
}

// ---------------------------------------------------------------------------------------------
// ¿Está abierta la app?
// ---------------------------------------------------------------------------------------------

/// Windows no deja abrir para escribir un programa que está en ejecución: así se sabe si TeatroGestión sigue abierto.
pub fn is_locked(exe: &Path) -> bool {
    match fs::OpenOptions::new().write(true).open(exe) {
        Ok(_) => false,
        Err(e) => e.kind() != std::io::ErrorKind::NotFound,
    }
}

/// ¿Hay un proceso con ese nombre en ejecución? Se pregunta a `tasklist` (viene con Windows). Si no se puede preguntar,
/// `false`: lo decide entonces [`is_locked`].
pub fn process_running(image_name: &str) -> bool {
    let mut cmd = std::process::Command::new("tasklist");
    cmd.args(["/FI", &format!("IMAGENAME eq {image_name}"), "/FO", "CSV", "/NH"]).stdin(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd.output().is_ok_and(|o| String::from_utf8_lossy(&o.stdout).to_lowercase().contains(&image_name.to_lowercase()))
}

/// TeatroGestión está abierto: el archivo está tomado o el proceso aparece en la lista.
pub fn app_is_open(exe: &Path) -> bool {
    is_locked(exe) || exe.file_name().is_some_and(|n| process_running(&n.to_string_lossy()))
}

// ---------------------------------------------------------------------------------------------
// Respaldo y compatibilidad de la base de datos
// ---------------------------------------------------------------------------------------------

/// `AAAAMMDD-HHMMSS` (UTC) de un instante, para nombrar respaldos.
pub fn stamp(unix: u64) -> String {
    let days = (unix / 86_400) as i64;
    let rem = unix % 86_400;
    // Fecha civil desde días desde 1970 (algoritmo de Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, (rem / 60) % 60, rem % 60)
}

/// Copia la base y sus archivos de registro (`-wal`, `-shm`) a `dest`.
pub fn copy_db_files(data: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("no se pudo crear la carpeta {}: {e}", dest.display()))?;
    for suffix in ["", "-wal", "-shm"] {
        let src = data.join(format!("{DB_NAME}{suffix}"));
        if src.is_file() {
            fs::copy(&src, dest.join(format!("{DB_NAME}{suffix}"))).map_err(|e| format!("no se pudo copiar {}: {e}", src.display()))?;
        }
    }
    Ok(())
}

/// Copia la base (y sus archivos de registro `-wal` y `-shm`) a `<datos>/respaldos/actualizacion-<sello>/`. Devuelve esa carpeta,
/// o `None` si todavía no hay base (primera vez). Conserva solo los últimos [`KEEP_BACKUPS`].
pub fn backup_data(data: &Path, sello: &str) -> Result<Option<PathBuf>, String> {
    let db = data.join(DB_NAME);
    if !db.is_file() {
        return Ok(None);
    }
    let root = data.join("respaldos");
    let dest = root.join(format!("actualizacion-{sello}"));
    copy_db_files(data, &dest)?;
    // Se tiran los respaldos más viejos (los nombres llevan la fecha, así que el orden alfabético es el cronológico).
    if let Ok(rd) = fs::read_dir(&root) {
        let mut all: Vec<PathBuf> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_dir() && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("actualizacion-"))).collect();
        all.sort();
        while all.len() > KEEP_BACKUPS {
            let _ = fs::remove_dir_all(all.remove(0));
        }
    }
    Ok(Some(dest))
}

#[derive(Debug, PartialEq, Eq)]
pub enum DbCheck {
    /// No hay base: la app la crea al abrir.
    NoDb,
    /// La base está al día o solo le faltan migraciones nuevas (la app las aplica al abrir).
    Compatible { applied: usize, pending: usize },
    /// Una migración que la base ya aplicó fue modificada: la versión nueva se negaría a abrirla.
    Mismatch { version: u32, name: String },
    /// La base tiene migraciones que esta versión no conoce: el programa instalado es MÁS NUEVO que este actualizador.
    NewerDb { version: u32 },
}

/// Compara las migraciones ya aplicadas en la base con las de la versión nueva (solo lectura; se la pasa una COPIA).
pub fn check_db(db_copy: &Path) -> Result<DbCheck, String> {
    if !db_copy.is_file() {
        return Ok(DbCheck::NoDb);
    }
    let conn = Connection::open_with_flags(db_copy, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| format!("no se pudo abrir la base: {e}"))?;
    let has_table: bool = conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_schema_migrations'", [], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())? > 0;
    if !has_table {
        return Ok(DbCheck::Compatible { applied: 0, pending: EMBEDDED_MIGRATIONS.len() });
    }
    let mut stmt = conn.prepare("SELECT version, checksum_sha256 FROM _schema_migrations ORDER BY version").map_err(|e| e.to_string())?;
    let rows: Vec<(u32, String)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?.filter_map(Result::ok).collect();
    let mut applied = 0;
    for (version, checksum) in &rows {
        match EMBEDDED_MIGRATIONS.iter().find(|m| m.version == *version) {
            None => return Ok(DbCheck::NewerDb { version: *version }),
            // Mismo criterio que la app al abrir: la huella guardada puede ser la de LF o la de CRLF (build de CI).
            Some(m) if !checksum_candidates(m.sql).contains(checksum) => return Ok(DbCheck::Mismatch { version: *version, name: m.name.to_string() }),
            Some(_) => applied += 1,
        }
    }
    Ok(DbCheck::Compatible { applied, pending: EMBEDDED_MIGRATIONS.len().saturating_sub(applied) })
}

// ---------------------------------------------------------------------------------------------
// Reemplazo de los programas, con vuelta atrás
// ---------------------------------------------------------------------------------------------

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    name.push(suffix);
    path.with_file_name(name)
}

fn write_synced(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut f = fs::File::create(path).map_err(|e| format!("no se pudo crear {}: {e}", path.display()))?;
    f.write_all(bytes).map_err(|e| format!("no se pudo escribir {}: {e}", path.display()))?;
    f.sync_all().map_err(|e| format!("no se pudo guardar {}: {e}", path.display()))
}

/// Instala los archivos nuevos en `dir`. Primero escribe todos como `.nuevo` y comprueba que quedaron idénticos; recién
/// entonces pasa cada archivo actual a `.anterior` y pone el nuevo. Si algo falla (p. ej. el programa está abierto), devuelve
/// todo a como estaba. Los `.anterior` quedan como copia de la versión previa.
pub fn apply(dir: &Path, payload: &Payload) -> Result<Vec<&'static str>, String> {
    let files = payload.files();
    if payload.exe.is_empty() {
        return Err("este actualizador no trae el programa nuevo (paquete incompleto)".into());
    }
    fs::create_dir_all(dir).map_err(|e| format!("no se pudo acceder a {}: {e}", dir.display()))?;

    // 1) Todo escrito y verificado antes de tocar lo que funciona.
    let mut staged: Vec<PathBuf> = Vec::new();
    for (name, bytes) in &files {
        let tmp = sibling(&dir.join(name), ".nuevo");
        let r = write_synced(&tmp, bytes).and_then(|()| {
            let back = fs::read(&tmp).map_err(|e| e.to_string())?;
            if back == *bytes { Ok(()) } else { Err(format!("{} quedó distinto de lo esperado (¿disco lleno o con errores?)", tmp.display())) }
        });
        staged.push(tmp);
        if let Err(e) = r {
            for t in &staged {
                let _ = fs::remove_file(t);
            }
            return Err(e);
        }
    }

    // 2) Cambio: actual -> .anterior, .nuevo -> actual. Si falla en el medio se deshace lo hecho.
    let mut done: Vec<(PathBuf, PathBuf, bool)> = Vec::new(); // (destino, anterior, había_archivo)
    for ((name, _), tmp) in files.iter().zip(&staged) {
        let target = dir.join(name);
        let old = sibling(&target, ".anterior");
        let existed = target.is_file();
        let step = (|| -> Result<(), String> {
            if existed {
                let _ = fs::remove_file(&old);
                fs::rename(&target, &old).map_err(|e| format!("no se pudo reemplazar {name}: {e}"))?;
            }
            fs::rename(tmp, &target).map_err(|e| {
                if existed {
                    let _ = fs::rename(&old, &target);
                }
                format!("no se pudo poner {name} nuevo: {e}")
            })
        })();
        match step {
            Ok(()) => done.push((target, old, existed)),
            Err(e) => {
                for (t, o, existed) in done.iter().rev() {
                    let _ = fs::remove_file(t);
                    if *existed {
                        let _ = fs::rename(o, t);
                    }
                }
                for t in &staged {
                    let _ = fs::remove_file(t);
                }
                return Err(e);
            }
        }
    }
    Ok(files.iter().map(|(n, _)| *n).collect())
}

/// ¿Los archivos instalados son exactamente los del paquete?
pub fn verify_installed(dir: &Path, payload: &Payload) -> bool {
    payload.files().iter().all(|(name, bytes)| fs::read(dir.join(name)).is_ok_and(|b| b == *bytes))
}

// ---------------------------------------------------------------------------------------------
// Licencia
// ---------------------------------------------------------------------------------------------

/// Una línea sobre la licencia instalada (no bloquea la actualización: la licencia no se toca).
pub fn license_summary(dir: &Path) -> String {
    use gestion_license::{fmt_date, now_unix, verify_embedded, MachineId, LICENSE_FILE_NAME};
    let Ok(text) = fs::read_to_string(dir.join(LICENSE_FILE_NAME)) else {
        return "No se encontró license.key en la carpeta del programa (la app pedirá la reactivación de emergencia).".into();
    };
    let machine = match MachineId::local() {
        Ok(m) => m,
        Err(e) => return format!("No se pudo comprobar la licencia ({e})."),
    };
    match verify_embedded(&text, &machine) {
        Ok(lic) => match lic.days_left(now_unix()) {
            None => "Licencia: perpetua (no vence).".into(),
            Some(d) if d < 0 => format!("Licencia VENCIDA el {}: hay que renovarla con el desarrollador.", fmt_date(lic.expires_unix)),
            Some(d) => format!("Licencia vigente hasta el {} (quedan {d} días).", fmt_date(lic.expires_unix)),
        },
        Err(e) => format!("La licencia instalada no es válida en esta PC ({e}). La actualización no la cambia."),
    }
}

#[cfg(test)]
mod tests;
