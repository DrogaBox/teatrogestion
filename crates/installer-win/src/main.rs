//! Instalador nativo de TeatroGestión para Windows 11.
//! Ballet Oficial de Córdoba - Teatro del Libertador San Martín.
//!
//! Emite y activa la licencia offline en el acto mediante `master.key.enc`,
//! instala los ejecutables, inicializa las marcas de reloj, asienta la
//! instalación en `licencias.db`, y configura o recupera administrativamente
//! la contraseña de la Bóveda de Inspección (Carina Lion).
//!
//! Cláusula institucional invariable:
//! ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use gestion_license::{
    clock, embedded_public_key, fmt_date, issue, keyfile, now_unix, verify_with, License,
    MachineId, LICENSE_FILE_NAME,
};
use rusqlite::{params, Connection};
use zeroize::Zeroizing;

#[cfg(windows)]
extern "system" {
    fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
    fn SetConsoleCP(wCodePageID: u32) -> i32;
}

fn setup_console() {
    #[cfg(windows)]
    unsafe {
        let _ = SetConsoleOutputCP(65001);
        let _ = SetConsoleCP(65001);
    }
}

const LEGAL_CLAUSE: &str = "★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★";
const APP_NAME: &str = "TeatroGestión";
const EXE_NAME: &str = "TeatroGestion.exe";
const ALT_EXE_NAME: &str = "gestion-app.exe";
const DLL_NAME: &str = "WebView2Loader.dll";
const ICO_NAME: &str = "teatrogestion.ico";
const LICENSEE: &str = "Ballet Oficial de Córdoba";
const MAX_PASSWORD_ATTEMPTS: usize = 3;

// Seguridad Bóveda de Inspección
const VAULT_SALT: &str = "boc_carina_lion_vault_salt_2026_";
const KEY_VAULT_HASH: &str = "INSPECTOR_VAULT_PASSWORD_HASH";
const KEY_VAULT_TIMEOUT: &str = "INSPECTOR_VAULT_LOCK_TIMEOUT_MINS";

/// SHA-256 (FIPS 180-4) determinista libre de dependencias externas
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

pub fn hash_vault_password(password: &str) -> String {
    let salted = format!("{}{}", VAULT_SALT, password.trim());
    compute_sha256_hex(&salted)
}

fn print_banner() {
    println!(
        r#"
==============================================================================
   ████████╗███████╗ █████╗ ████████╗██████╗  ██████╗     ██████╗ ███████╗███████╗████████╗██╗ ██████╗ ███╗   ██╗
   ╚══██╔══╝██╔════╝██╔══██╗╚══██╔══╝██╔══██╗██╔═══██╗   ██╔════╝ ██╔════╝██╔════╝╚══██╔══╝██║██╔═══██╗████╗  ██║
      ██║   █████╗  ███████║   ██║   ██████╔╝██║   ██║   ██║  ███╗█████╗  ███████╗   ██║   ██║██║   ██║██╔██╗ ██║
      ██║   ██╔══╝  ██╔══██║   ██║   ██╔══██╗██║   ██║   ██║   ██║██╔══╝  ╚════██║   ██║   ██║██║   ██║██║╚██╗██║
      ██║   ███████╗██║  ██║   ██║   ██║  ██║╚██████╔╝   ╚██████╔╝███████╗███████║   ██║   ██║╚██████╔╝██║ ╚████║
      ╚═╝   ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝     ╚═════╝ ╚══════╝╚══════╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
                      BALLET OFICIAL DE CORDOBA — WINDOWS 11
==============================================================================
"#
    );
    println!("  Sistema Integral de Personal, Inventario, Repartos, Cues y Stock");
    println!("  Teatro del Libertador Gral. San Martín\n");
    println!("  {}\n", LEGAL_CLAUSE);
    println!("------------------------------------------------------------------------------\n");
}

fn find_source_dir() -> PathBuf {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = current_exe.parent().unwrap_or_else(|| Path::new("."));

    let cand1 = exe_dir.join("TeatroGestion");
    if cand1.exists() && (cand1.join(EXE_NAME).exists() || cand1.join(ALT_EXE_NAME).exists()) {
        return cand1;
    }

    let cand2 = exe_dir.join("payload");
    if cand2.exists() && (cand2.join(EXE_NAME).exists() || cand2.join(ALT_EXE_NAME).exists()) {
        return cand2;
    }

    if exe_dir.join(EXE_NAME).exists() || exe_dir.join(ALT_EXE_NAME).exists() {
        return exe_dir.to_path_buf();
    }

    exe_dir.to_path_buf()
}

fn determine_target_dir() -> PathBuf {
    let standard = PathBuf::from(r"C:\TeatroGestion");
    if let Ok(()) = fs::create_dir_all(&standard) {
        return standard;
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local_app_data).join("TeatroGestion");
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        return PathBuf::from(userprofile).join("TeatroGestion");
    }
    standard
}

fn get_app_data_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("com.teatrogestion.app")
    } else if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(localappdata).join("com.teatrogestion.app")
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile)
            .join("AppData")
            .join("Roaming")
            .join("com.teatrogestion.app")
    } else {
        PathBuf::from(r"C:\TeatroGestion\data")
    }
}

fn find_master_key(source_dir: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("TEATROGESTION_MASTER_KEY").or_else(|_| std::env::var("TEATROCUE_MASTER_KEY")) {
        let pb = PathBuf::from(p);
        if pb.exists() {
            return Some(pb);
        }
    }
    let candidates = [
        source_dir.join("master.key.enc"),
        source_dir.join("..").join("master.key.enc"),
        PathBuf::from("master.key.enc"),
    ];
    for c in &candidates {
        if c.exists() {
            return Some(c.clone());
        }
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let p = PathBuf::from(home).join(".config/teatrocue-keys/master.key.enc");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn ask_password(attempt: usize, prompt_msg: &str) -> Result<Zeroizing<String>, String> {
    if let Ok(p) = std::env::var("TEATROGESTION_MASTER_PASSWORD").or_else(|_| std::env::var("TEATROCUE_MASTER_PASSWORD")) {
        if attempt > 1 {
            return Err("la contraseña maestra del entorno es incorrecta".into());
        }
        return Ok(Zeroizing::new(p));
    }
    print!("{prompt_msg}");
    let _ = io::stdout().flush();
    rpassword::read_password()
        .map(Zeroizing::new)
        .map_err(|e| format!("no se pudo leer la contraseña: {e}"))
}

/// Lee una línea de la entrada. `None` si no hay entrada (consola cerrada o ejecución automática): así ningún menú ni pregunta
/// queda dando vueltas para siempre.
/// Un secreto de una variable de entorno; la variable se borra del proceso apenas se lee. Vacío = ausente.
fn secret_from_env(name: &str) -> Option<Zeroizing<String>> {
    let v = std::env::var(name).ok();
    std::env::remove_var(name);
    v.filter(|v| !v.is_empty()).map(Zeroizing::new)
}

fn read_line_opt() -> Option<String> {
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line),
    }
}

/// Pide una contraseña nueva con confirmación. `None` si no se puede leer (sin consola).
fn ask_new_password(prompt: &str, confirm: &str) -> Option<String> {
    loop {
        print!("{prompt}");
        let _ = io::stdout().flush();
        let p1 = rpassword::read_password().ok()?;
        if p1.trim().is_empty() {
            println!("La contraseña no puede estar vacía. Reintente.");
            continue;
        }
        print!("{confirm}");
        let _ = io::stdout().flush();
        let p2 = rpassword::read_password().ok()?;
        if p1 != p2 {
            println!("Las contraseñas no coinciden. Reintente.");
            continue;
        }
        return Some(p1);
    }
}

fn save_vault_password_to_db(db_path: &Path, password: &str, timeout_mins: Option<i32>) -> Result<(), String> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("no se pudo crear directorio {}: {e}", parent.display()))?;
    }

    let conn = Connection::open(db_path)
        .map_err(|e| format!("no se pudo abrir SQLite en {}: {e}", db_path.display()))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        ) STRICT;"
    ).map_err(|e| format!("error al asegurar tabla app_settings: {e}"))?;

    let hash = hash_vault_password(password);
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at;",
        params![KEY_VAULT_HASH, hash],
    ).map_err(|e| format!("error al guardar hash de bóveda: {e}"))?;

    if let Some(mins) = timeout_mins {
        let mins_str = mins.to_string();
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at;",
            params![KEY_VAULT_TIMEOUT, mins_str],
        ).map_err(|e| format!("error al guardar timeout de bóveda: {e}"))?;
    }

    Ok(())
}

fn find_installed_sqlite_dbs(target_dir: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let app_data = get_app_data_dir().join("teatro_gestion.sqlite");
    if app_data.exists() {
        candidates.push(app_data);
    }

    let target_sqlite = target_dir.join("teatro_gestion.sqlite");
    if target_sqlite.exists() && !candidates.contains(&target_sqlite) {
        candidates.push(target_sqlite);
    }

    let target_data_sqlite = target_dir.join("data").join("teatro_gestion.sqlite");
    if target_data_sqlite.exists() && !candidates.contains(&target_data_sqlite) {
        candidates.push(target_data_sqlite);
    }

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let local_db = PathBuf::from(local_app_data).join("com.teatrogestion.app").join("teatro_gestion.sqlite");
        if local_db.exists() && !candidates.contains(&local_db) {
            candidates.push(local_db);
        }
    }

    candidates
}

fn choose_license_term() -> Result<(u64, i64), String> {
    println!("\n¿Por cuánto tiempo debe durar la licencia de esta PC?");
    println!("  1) 1 mes (30 días)");
    println!("  2) 3 meses (90 días)");
    println!("  3) 6 meses (180 días)");
    println!("  4) 1 año (365 días)");
    println!("  5) Sin vencimiento (Perpetua)");
    println!("  6) Cantidad de días personalizada…");

    let now = now_unix();
    loop {
        print!("Elija una opción [1-6]: ");
        let _ = io::stdout().flush();
        let Some(line) = read_line_opt() else { return Err("no hay entrada por consola".into()) };
        match line.trim() {
            "1" => return Ok((now + 30 * 86_400, 30)),
            "2" => return Ok((now + 90 * 86_400, 90)),
            "3" => return Ok((now + 180 * 86_400, 180)),
            "4" => return Ok((now + 365 * 86_400, 365)),
            "5" => return Ok((0, 0)),
            "6" => loop {
                print!("Cantidad de días (1 a 3650): ");
                let _ = io::stdout().flush();
                let Some(dline) = read_line_opt() else { return Err("no hay entrada por consola".into()) };
                if let Ok(d) = dline.trim().parse::<u64>() {
                    if (1..=3650).contains(&d) {
                        return Ok((now + d * 86_400, d as i64));
                    }
                }
                println!("Por favor ingrese un número entre 1 y 3650.");
            },
            _ => println!("Opción no válida, reintente."),
        }
    }
}

/// La base de licencias del pendrive: la misma que usa TeatroCue (`admin/licencias.db`). Solo se **asienta en una que ya
/// existe**: no se crea ninguna nueva (una segunda base suelta en la raíz del pendrive divergiría de la real).
fn find_registry_db(source_dir: &Path) -> Option<PathBuf> {
    let candidates = [
        source_dir.join("admin").join("licencias.db"),
        source_dir.join("..").join("admin").join("licencias.db"),
        source_dir.join("licencias.db"),
        source_dir.join("..").join("licencias.db"),
    ];
    candidates.into_iter().find(|c| c.is_file())
}

fn record_in_licencias_db(
    db_path: &Path,
    machine: &MachineId,
    puesto: &str,
    now: u64,
    expires: u64,
    days: i64,
    license_text: &str,
) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| format!("no se pudo abrir {}: {e}", db_path.display()))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS equipos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            institucion TEXT NOT NULL DEFAULT '',
            puesto_sala TEXT NOT NULL DEFAULT '',
            machine_id TEXT UNIQUE NOT NULL,
            created_at INTEGER NOT NULL DEFAULT 0,
            notas TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS licencias_emitidas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            equipo_id INTEGER NOT NULL REFERENCES equipos(id),
            tipo_emision TEXT NOT NULL,
            fecha_emision INTEGER NOT NULL,
            fecha_vencimiento INTEGER NOT NULL,
            dias_otorgados INTEGER NOT NULL DEFAULT 0,
            activation_epoch INTEGER NOT NULL DEFAULT 0,
            token_generado TEXT NOT NULL DEFAULT '',
            notas TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_lic_equipo ON licencias_emitidas(equipo_id);",
    )
    .map_err(|e| format!("error inicializando tablas: {e}"))?;

    conn.execute(
        "INSERT INTO equipos (institucion, puesto_sala, machine_id, created_at, notas)
         VALUES (?1, ?2, ?3, ?4, 'Alta desde instalador TeatroGestión')
         ON CONFLICT(machine_id) DO UPDATE SET puesto_sala=?2, notas='Actualizado por instalador TeatroGestión'",
        params![LICENSEE, puesto, machine.as_str(), now as i64],
    )
    .map_err(|e| format!("error registrando equipo: {e}"))?;

    let equipo_id: i64 = conn
        .query_row(
            "SELECT id FROM equipos WHERE machine_id = ?1",
            params![machine.as_str()],
            |row| row.get(0),
        )
        .map_err(|e| format!("error obteniendo ID de equipo: {e}"))?;

    conn.execute(
        "INSERT INTO licencias_emitidas (
            equipo_id, tipo_emision, fecha_emision, fecha_vencimiento, dias_otorgados, activation_epoch, token_generado, notas
        ) VALUES (?1, 'Pendrive Instalador', ?2, ?3, ?4, 0, ?5, 'Instalación TeatroGestión')",
        params![equipo_id, now as i64, expires as i64, days, license_text],
    )
    .map_err(|e| format!("error registrando emisión: {e}"))?;

    Ok(())
}

fn create_shortcut(target_exe: &Path, work_dir: &Path, icon_path: &Path, shortcut_path: &Path) -> Result<(), String> {
    let q = |p: &Path| p.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$ErrorActionPreference='Stop'; \
         $s=(New-Object -ComObject WScript.Shell).CreateShortcut('{shortcut}'); \
         $s.TargetPath='{target}'; \
         $s.WorkingDirectory='{work}'; \
         $s.IconLocation='{icon},0'; \
         $s.Description='TeatroGestión - Ballet Oficial de Córdoba'; \
         $s.Save()",
        shortcut = q(shortcut_path),
        target = q(target_exe),
        work = q(work_dir),
        icon = q(icon_path),
    );

    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
        .map_err(|e| format!("Error invocando PowerShell: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }

    Ok(())
}

fn pause_prompt() {
    print!("\nPresione [ENTER] para salir...");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().lock().read_line(&mut buf);
}

/// Asistente CLI de Recuperación Administrativa de la Bóveda de Inspección
fn admin_reset_vault_cli(source_dir: &Path) {
    println!("\n==============================================================================");
    println!("   RECUPERACIÓN ADMINISTRATIVA — BÓVEDA DE INSPECCIÓN");
    println!("   Ballet Oficial de Córdoba — Teatro del Libertador San Martín");
    println!("==============================================================================");
    println!("  Esta operación permite restablecer la contraseña de la Bóveda Documental");
    println!("  utilizando la llave maestra ('master.key.enc') del pendrive.\n");

    let master_key_path = match find_master_key(source_dir) {
        Some(p) => {
            println!("[*] Llave maestra detectada en: {}", p.display());
            p
        }
        None => {
            eprintln!("[!] ERROR: No se encontró 'master.key.enc' en el pendrive ni en el sistema.");
            eprintln!("    Asegúrese de insertar el pendrive de instalación del administrador.");
            pause_prompt();
            return;
        }
    };

    let key_bytes = match fs::read(&master_key_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[!] ERROR al leer {}: {e}", master_key_path.display());
            pause_prompt();
            return;
        }
    };

    let mut authenticated = false;
    for attempt in 1..=MAX_PASSWORD_ATTEMPTS {
        match ask_password(attempt, "Contraseña maestra del desarrollador/administrador: ") {
            Ok(pwd) => match keyfile::decrypt(&key_bytes, &pwd) {
                Ok(_signing_key) => {
                    println!("  [OK] Autenticación administrativa confirmada.");
                    authenticated = true;
                    break;
                }
                Err(e) => println!("  [!] Intento {attempt}/{MAX_PASSWORD_ATTEMPTS}: {e}"),
            },
            Err(e) => {
                println!("  [!] Error: {e}");
                break;
            }
        }
    }

    if !authenticated {
        eprintln!("\n[!] Falló la autenticación con la llave maestra. Operación cancelada.");
        pause_prompt();
        return;
    }

    let default_target = determine_target_dir();
    let existing_dbs = find_installed_sqlite_dbs(&default_target);

    let chosen_db = if let Some(first) = existing_dbs.first() {
        println!("\n[*] Base de datos de TeatroGestión detectada en: {}", first.display());
        print!("¿Usar esta base de datos? [S/n] (o ingrese otra ruta): ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        let _ = io::stdin().lock().read_line(&mut line);
        let tr = line.trim();
        if tr.is_empty() || tr.eq_ignore_ascii_case("s") || tr.eq_ignore_ascii_case("si") || tr.eq_ignore_ascii_case("sí") {
            first.clone()
        } else {
            PathBuf::from(tr)
        }
    } else {
        let appdata_default = get_app_data_dir().join("teatro_gestion.sqlite");
        println!("\n[*] No se detectó una base de datos existente.");
        print!("Ruta destino para la base de datos [{}]: ", appdata_default.display());
        let _ = io::stdout().flush();
        let mut line = String::new();
        let _ = io::stdin().lock().read_line(&mut line);
        let tr = line.trim();
        if tr.is_empty() {
            appdata_default
        } else {
            PathBuf::from(tr)
        }
    };

    println!("\n==> Estableciendo NUEVA contraseña para la Bóveda de Inspección:");
    let Some(new_pwd) = ask_new_password("Nueva contraseña de la Inspectora: ", "Confirmar nueva contraseña: ") else {
        eprintln!("[!] No se pudo leer la contraseña (no hay consola interactiva). Operación cancelada.");
        pause_prompt();
        return;
    };

    println!("\nTiempo de bloqueo por inactividad:");
    println!("  1) 1 minuto");
    println!("  2) 5 minutos (Recomendado)");
    println!("  3) 15 minutos");
    println!("  4) 30 minutos");
    println!("  5) Nunca bloquear");
    print!("Seleccione [1-5, por defecto 2]: ");
    let _ = io::stdout().flush();
    let mut to_line = String::new();
    let _ = io::stdin().lock().read_line(&mut to_line);
    let timeout_mins = match to_line.trim() {
        "1" => Some(1),
        "3" => Some(15),
        "4" => Some(30),
        "5" => Some(0),
        _ => Some(5),
    };

    match save_vault_password_to_db(&chosen_db, &new_pwd, timeout_mins) {
        Ok(()) => {
            println!("\n==============================================================================");
            println!("  [OK] ¡CONTRASEÑA DE LA BÓVEDA RESTABLECIDA EXITOSAMENTE!");
            println!("  Base de datos actualizada: {}", chosen_db.display());
            println!("==============================================================================");
        }
        Err(e) => {
            eprintln!("\n[!] ERROR al actualizar la base de datos: {e}");
        }
    }

    pause_prompt();
}

/// Reseteo administrativo silencioso / programático (utilizado por la GUI o CLI automatizado)
fn admin_reset_vault_silent(
    db_path: &Path,
    source_dir: &Path,
    master_pass: &str,
    new_vault_pass: &str,
    timeout_mins: Option<i32>,
) -> Result<(), String> {
    let master_key_path = find_master_key(source_dir)
        .ok_or_else(|| "No se encontró 'master.key.enc' en el pendrive ni en las rutas maestras.".to_string())?;

    let key_bytes = fs::read(&master_key_path)
        .map_err(|e| format!("Error al leer {}: {e}", master_key_path.display()))?;

    // Validar contraseña maestra descifrando la llave
    let _signing_key = keyfile::decrypt(&key_bytes, master_pass)
        .map_err(|e| format!("Contraseña maestra incorrecta: {e}"))?;

    if new_vault_pass.trim().is_empty() {
        return Err("La nueva contraseña de la bóveda no puede estar vacía.".into());
    }

    save_vault_password_to_db(db_path, new_vault_pass, timeout_mins)
}

/// Instalación completa en modo interactivo (CLI)
fn run_interactive_install(source_dir: &Path) {
    let machine = match MachineId::local() {
        Ok(m) => {
            println!("[*] Machine ID detectado en esta PC: {}", m);
            m
        }
        Err(e) => {
            eprintln!("[!] ERROR al obtener Machine ID: {e}");
            pause_prompt();
            return;
        }
    };

    println!("[*] Directorio de origen: {}", source_dir.display());

    let src_exe = if source_dir.join(EXE_NAME).exists() {
        source_dir.join(EXE_NAME)
    } else if source_dir.join(ALT_EXE_NAME).exists() {
        source_dir.join(ALT_EXE_NAME)
    } else {
        eprintln!(
            "[!] ERROR: No se encontró el ejecutable principal ({} ni {}) en {}",
            EXE_NAME,
            ALT_EXE_NAME,
            source_dir.display()
        );
        pause_prompt();
        return;
    };

    let src_dll = source_dir.join(DLL_NAME);
    let src_ico = source_dir.join(ICO_NAME);

    // --- Emisión / Verificación de Licencia Offline ---
    let now = now_unix();
    let mut generated_license_text: Option<String> = None;
    let mut license_expires: u64 = 0;

    if let Some(master_key_path) = find_master_key(source_dir) {
        println!("\n==> Llave maestra encontrada en: {}", master_key_path.display());
        let mut key = None;
        for attempt in 1..=MAX_PASSWORD_ATTEMPTS {
            match ask_password(attempt, "Contraseña maestra del desarrollador: ") {
                Ok(pwd) => match keyfile::decrypt(&fs::read(&master_key_path).unwrap_or_default(), &pwd) {
                    Ok(k) => {
                        println!("  [OK] Llave maestra descifrada correctamente en RAM.");
                        key = Some(k);
                        break;
                    }
                    Err(e) => println!("  [!] Intento {attempt}/{MAX_PASSWORD_ATTEMPTS}: {e}"),
                },
                Err(e) => {
                    println!("  [!] Error: {e}");
                    break;
                }
            }
        }

        if let Some(signing_key) = key {
            let (expires, days) = choose_license_term().unwrap_or((0, 0));
            license_expires = expires;

            print!("Puesto / sala de esta PC [Cabina / Oficina]: ");
            let _ = io::stdout().flush();
            let mut puesto_in = String::new();
            let _ = io::stdin().lock().read_line(&mut puesto_in);
            let puesto = if puesto_in.trim().is_empty() {
                "Cabina / Oficina".to_string()
            } else {
                puesto_in.trim().to_string()
            };

            let issued_text = issue(LICENSEE, &machine, now, expires, &signing_key);
            drop(signing_key); // Borrado seguro de RAM

            generated_license_text = Some(issued_text.clone());

            if let Some(reg_db) = find_registry_db(source_dir) {
                match record_in_licencias_db(&reg_db, &machine, &puesto, now, expires, days, &issued_text) {
                    Ok(()) => println!("  [OK] Emisión asentada en {}", reg_db.display()),
                    Err(e) => println!("  [i] Aviso: no se pudo escribir en licencias.db ({e})"),
                }
            }
        } else {
            println!("  [!] No se pudo descifrar la llave maestra.");
        }
    }

    let license_content = if let Some(t) = generated_license_text {
        t
    } else if let Ok(existing) = fs::read_to_string(source_dir.join(LICENSE_FILE_NAME)) {
        println!("  [*] Utilizando license.key existente en origen.");
        existing
    } else {
        eprintln!("\n[!] AVISO: No se generó ni se encontró un license.key válido.");
        eprintln!("    Al iniciar TeatroGestión se requerirá la reactivación de emergencia con el Machine ID.");
        String::new()
    };

    if !license_content.is_empty() {
        if let Some(public_key) = embedded_public_key() {
            match verify_with(&license_content, &public_key, &machine) {
                Ok(lic) => {
                    println!("  [OK] Firma Ed25519 verificada con éxito.");
                    println!("       Titular: {}", lic.licensee);
                    if lic.expires_unix == 0 {
                        println!("       Vigencia: PERPETUA (Sin vencimiento).");
                    } else {
                        println!("       Vencimiento: {} (dentro de {} días).", fmt_date(lic.expires_unix), lic.days_left(now).unwrap_or(0));
                    }
                }
                Err(err) => {
                    println!("  [!] Advertencia: la firma no corresponde a esta PC ({err}).");
                }
            }
        }
    }

    // --- Destino de Instalación ---
    let default_target = determine_target_dir();
    println!("\n[*] Carpeta de instalación sugerida: {}", default_target.display());
    print!("Presione [ENTER] para confirmar o escriba otra ruta: ");
    let _ = io::stdout().flush();

    let mut user_input = String::new();
    let _ = io::stdin().lock().read_line(&mut user_input);
    let target_dir = if user_input.trim().is_empty() {
        default_target
    } else {
        PathBuf::from(user_input.trim())
    };

    println!("\n==> Instalando en: {}", target_dir.display());

    if let Err(e) = fs::create_dir_all(&target_dir) {
        eprintln!("[!] ERROR al crear carpeta de destino {}: {}", target_dir.display(), e);
        pause_prompt();
        return;
    }

    // Copiar Ejecutable
    let dest_exe = target_dir.join(EXE_NAME);
    print!("  -> Copiando ejecutable {}... ", EXE_NAME);
    let _ = io::stdout().flush();
    if let Err(e) = fs::copy(&src_exe, &dest_exe) {
        eprintln!("FALLÓ: {}", e);
    } else {
        println!("OK ({:.1} MB)", fs::metadata(&dest_exe).map(|m| m.len() as f64 / 1_048_576.0).unwrap_or(0.0));
    }

    // Copiar DLL
    let dest_dll = target_dir.join(DLL_NAME);
    if src_dll.exists() {
        print!("  -> Copiando runtime {}... ", DLL_NAME);
        let _ = io::stdout().flush();
        if let Err(e) = fs::copy(&src_dll, &dest_dll) {
            eprintln!("FALLÓ: {}", e);
        } else {
            println!("OK");
        }
    }

    // Copiar Icono
    let dest_ico = target_dir.join(ICO_NAME);
    if src_ico.exists() {
        print!("  -> Copiando icono oficial {}... ", ICO_NAME);
        let _ = io::stdout().flush();
        let _ = fs::copy(&src_ico, &dest_ico);
        println!("OK");
    }

    // Escribir license.key
    if !license_content.is_empty() {
        let dest_key = target_dir.join(LICENSE_FILE_NAME);
        print!("  -> Instalando {}... ", LICENSE_FILE_NAME);
        let _ = io::stdout().flush();
        if let Err(e) = fs::write(&dest_key, &license_content) {
            eprintln!("FALLÓ: {}", e);
        } else {
            println!("OK");
        }

        let dummy_lic = License {
            licensee: LICENSEE.into(),
            machine: machine.clone(),
            issued_unix: now,
            expires_unix: license_expires,
        };
        let stores = clock::default_stores(&target_dir);
        clock::reset(now, &dummy_lic, &stores);
    }

    // --- Configuración Inicial de la Bóveda de Inspección ---
    println!("\n==> Seguridad de la Bóveda de Inspección (Carina Lion):");
    print!("¿Desea configurar la contraseña de la Bóveda ahora? [S/n]: ");
    let _ = io::stdout().flush();
    let mut vault_ans = String::new();
    let _ = io::stdin().lock().read_line(&mut vault_ans);
    let vtr = vault_ans.trim();
    if vtr.is_empty() || vtr.eq_ignore_ascii_case("s") || vtr.eq_ignore_ascii_case("si") || vtr.eq_ignore_ascii_case("sí") {
        'vault: {
        let Some(vault_pwd) = ask_new_password("Contraseña para la Inspectora: ", "Confirmar contraseña: ") else {
            eprintln!("[!] No se pudo leer la contraseña: la Bóveda queda sin configurar.");
            break 'vault;
        };

        println!("\nTiempo de bloqueo por inactividad:");
        println!("  1) 1 minuto");
        println!("  2) 5 minutos (Recomendado)");
        println!("  3) 15 minutos");
        println!("  4) 30 minutos");
        println!("  5) Nunca bloquear");
        print!("Seleccione [1-5, por defecto 2]: ");
        let _ = io::stdout().flush();
        let mut t_line = String::new();
        let _ = io::stdin().lock().read_line(&mut t_line);
        let mins = match t_line.trim() {
            "1" => Some(1),
            "3" => Some(15),
            "4" => Some(30),
            "5" => Some(0),
            _ => Some(5),
        };

        let appdata_db = get_app_data_dir().join("teatro_gestion.sqlite");
        print!("  -> Guardando credenciales de bóveda en base de datos... ");
        let _ = io::stdout().flush();
        match save_vault_password_to_db(&appdata_db, &vault_pwd, mins) {
            Ok(()) => println!("OK"),
            Err(e) => println!("Aviso: {e}"),
        }
        }
    }

    // Crear accesos directos
    println!("\n==> Creando accesos directos...");
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let desktop_dir = PathBuf::from(&userprofile).join("Desktop");
        if desktop_dir.exists() {
            let shortcut_file = desktop_dir.join(format!("{}.lnk", APP_NAME));
            let icon_target = if dest_ico.exists() { &dest_ico } else { &dest_exe };
            match create_shortcut(&dest_exe, &target_dir, icon_target, &shortcut_file) {
                Ok(_) => println!("  [OK] Acceso directo en el Escritorio: {}", shortcut_file.display()),
                Err(err) => eprintln!("  [!] Advertencia al crear acceso en Escritorio: {}", err),
            }
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        let start_menu_dir = PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs");
        if start_menu_dir.exists() {
            let start_shortcut = start_menu_dir.join(format!("{}.lnk", APP_NAME));
            let icon_target = if dest_ico.exists() { &dest_ico } else { &dest_exe };
            match create_shortcut(&dest_exe, &target_dir, icon_target, &start_shortcut) {
                Ok(_) => println!("  [OK] Acceso directo en el Menú Inicio: {}", start_shortcut.display()),
                Err(err) => eprintln!("  [!] Advertencia al crear acceso en Menú Inicio: {}", err),
            }
        }
    }

    println!("\n==============================================================================");
    println!("  ¡INSTALACION Y ACTIVACION COMPLETADAS EXITOSAMENTE!");
    println!("  TeatroGestión ha quedado listo para su uso en Windows 11.");
    println!("  Ubicación: {}", target_dir.display());
    println!("==============================================================================");
    println!("  {}\n", LEGAL_CLAUSE);

    print!("¿Desea iniciar TeatroGestión ahora? [S/n]: ");
    let _ = io::stdout().flush();
    let mut launch_ans = String::new();
    let _ = io::stdin().lock().read_line(&mut launch_ans);
    let ans = launch_ans.trim().to_lowercase();
    if ans.is_empty() || ans == "s" || ans == "si" || ans == "sí" || ans == "y" {
        println!("Iniciando {}...", dest_exe.display());
        let _ = Command::new(&dest_exe).current_dir(&target_dir).spawn();
    }

    pause_prompt();
}

/// Instalación programática / silenciosa para invocación desde GUI o scripts
fn run_silent_install(
    source_dir: &Path,
    target_dir: &Path,
    term_option: u32,
    custom_days: u64,
    puesto: &str,
    master_pass: &str,
    vault_pass: Option<&str>,
    vault_timeout: Option<i32>,
) -> Result<(), String> {
    println!("[*] Iniciando instalación en: {}", target_dir.display());

    let machine = MachineId::local().map_err(|e| format!("Error de Machine ID: {e}"))?;
    println!("[OK] Machine ID local: {}", machine);

    let src_exe = if source_dir.join(EXE_NAME).exists() {
        source_dir.join(EXE_NAME)
    } else if source_dir.join(ALT_EXE_NAME).exists() {
        source_dir.join(ALT_EXE_NAME)
    } else {
        return Err(format!("No se encontró {} en {}", EXE_NAME, source_dir.display()));
    };

    let src_dll = source_dir.join(DLL_NAME);
    let src_ico = source_dir.join(ICO_NAME);

    fs::create_dir_all(target_dir).map_err(|e| format!("No se pudo crear carpeta de destino: {e}"))?;

    // Licencia
    let now = now_unix();
    let (expires, days) = match term_option {
        1 => (now + 30 * 86_400, 30),
        2 => (now + 90 * 86_400, 90),
        3 => (now + 180 * 86_400, 180),
        4 => (now + 365 * 86_400, 365),
        5 => (0, 0),
        6 => (now + custom_days * 86_400, custom_days as i64),
        _ => (now + 365 * 86_400, 365),
    };

    if let Some(master_key_path) = find_master_key(source_dir) {
        let key_bytes = fs::read(&master_key_path).map_err(|e| format!("Error leyendo master.key.enc: {e}"))?;
        let signing_key = keyfile::decrypt(&key_bytes, master_pass).map_err(|e| format!("Contraseña maestra incorrecta: {e}"))?;

        let issued_text = issue(LICENSEE, &machine, now, expires, &signing_key);
        drop(signing_key);

        let dest_key = target_dir.join(LICENSE_FILE_NAME);
        fs::write(&dest_key, &issued_text).map_err(|e| format!("Error guardando license.key: {e}"))?;
        println!("[OK] Licencia offline Ed25519 generada e instalada.");

        let dummy_lic = License {
            licensee: LICENSEE.into(),
            machine: machine.clone(),
            issued_unix: now,
            expires_unix: expires,
        };
        let stores = clock::default_stores(target_dir);
        clock::reset(now, &dummy_lic, &stores);

        if let Some(reg_db) = find_registry_db(source_dir) {
            let _ = record_in_licencias_db(&reg_db, &machine, puesto, now, expires, days, &issued_text);
            println!("[OK] Registro asentado en licencias.db");
        }
    } else {
        println!("[!] Aviso: master.key.enc no encontrado, omitiendo firma automática.");
    }

    // Copiar archivos
    let dest_exe = target_dir.join(EXE_NAME);
    fs::copy(&src_exe, &dest_exe).map_err(|e| format!("Error copiando ejecutable: {e}"))?;
    println!("[OK] Copiado ejecutable {}", EXE_NAME);

    if src_dll.exists() {
        let dest_dll = target_dir.join(DLL_NAME);
        let _ = fs::copy(&src_dll, &dest_dll);
        println!("[OK] Copiado {}", DLL_NAME);
    }

    let dest_ico = target_dir.join(ICO_NAME);
    if src_ico.exists() {
        let _ = fs::copy(&src_ico, &dest_ico);
        println!("[OK] Copiado icono {}", ICO_NAME);
    }

    // Configuración de Bóveda
    if let Some(vp) = vault_pass {
        if !vp.trim().is_empty() {
            let appdata_db = get_app_data_dir().join("teatro_gestion.sqlite");
            save_vault_password_to_db(&appdata_db, vp, vault_timeout)?;
            println!("[OK] Bóveda de Inspección configurada con contraseña y timeout.");
        }
    }

    // Accesos directos
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let desktop_dir = PathBuf::from(&userprofile).join("Desktop");
        if desktop_dir.exists() {
            let shortcut_file = desktop_dir.join(format!("{}.lnk", APP_NAME));
            let icon_target = if dest_ico.exists() { &dest_ico } else { &dest_exe };
            match create_shortcut(&dest_exe, target_dir, icon_target, &shortcut_file) {
                Ok(()) => println!("[OK] Acceso directo en el Escritorio creado."),
                Err(e) => println!("[!] Aviso: no se pudo crear el acceso directo del Escritorio ({e})."),
            }
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        let start_menu_dir = PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs");
        if start_menu_dir.exists() {
            let start_shortcut = start_menu_dir.join(format!("{}.lnk", APP_NAME));
            let icon_target = if dest_ico.exists() { &dest_ico } else { &dest_exe };
            match create_shortcut(&dest_exe, target_dir, icon_target, &start_shortcut) {
                Ok(()) => println!("[OK] Acceso directo en Menú Inicio creado."),
                Err(e) => println!("[!] Aviso: no se pudo crear el acceso directo del Menú Inicio ({e})."),
            }
        }
    }

    println!("[EXITO] ¡Instalación completada correctamente!");
    Ok(())
}

/// Lanza la Interfaz Gráfica WPF moderna (PowerShell STA)
fn launch_gui(source_dir: &Path) {
    let script_candidates = [
        source_dir.join("Instalador_Grafico.ps1"),
        source_dir.join("..").join("Instalador_Grafico.ps1"),
        PathBuf::from("Instalador_Grafico.ps1"),
    ];

    let found_script = script_candidates.iter().find(|p| p.exists());

    if let Some(ps_path) = found_script {
        println!("[*] Iniciando Interfaz Gráfica de TeatroGestión...");
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-STA",
                "-File",
                &ps_path.to_string_lossy(),
            ])
            .status();

        if let Err(e) = status {
            eprintln!("[!] No se pudo abrir la interfaz gráfica: {e}");
            println!("[*] Cambiando automáticamente a la interfaz de consola...");
            run_interactive_install(source_dir);
        }
    } else {
        eprintln!("[!] No se encontró 'Instalador_Grafico.ps1' en el pendrive.");
        println!("[*] Ejecutando instalador en consola...");
        run_interactive_install(source_dir);
    }
}

fn main() {
    setup_console();
    let source_dir = find_source_dir();
    let args: Vec<String> = std::env::args().collect();

    // Procesar flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--gui" | "-g" => {
                launch_gui(&source_dir);
                return;
            }
            "--cli" | "--consola" | "-c" => {
                print_banner();
                run_interactive_install(&source_dir);
                return;
            }
            "--admin-reset-vault" => {
                print_banner();
                admin_reset_vault_cli(&source_dir);
                return;
            }
            "--silent-install" => {
                // Formato: --silent-install <target_dir> <term_idx> <custom_days> <puesto> [vault_timeout]
                // Las contraseñas NO van por la línea de comandos (cualquier programa las vería en la lista de procesos):
                // TEATROGESTION_MASTER_PASSWORD (obligatoria) y TEATROGESTION_VAULT_PASSWORD (opcional).
                if args.len() >= 6 {
                    let target_dir = PathBuf::from(&args[2]);
                    let term_idx: u32 = args[3].parse().unwrap_or(4);
                    let custom_days: u64 = args[4].parse().unwrap_or(365);
                    let puesto = &args[5];
                    let vault_timeout = args.get(6).and_then(|s| s.parse::<i32>().ok());
                    let Some(master_pass) = secret_from_env("TEATROGESTION_MASTER_PASSWORD") else {
                        eprintln!("[ERROR] Falta la contraseña maestra (variable TEATROGESTION_MASTER_PASSWORD).");
                        std::process::exit(1);
                    };
                    let vault_pass = secret_from_env("TEATROGESTION_VAULT_PASSWORD");

                    let result = run_silent_install(
                        &source_dir,
                        &target_dir,
                        term_idx,
                        custom_days,
                        puesto,
                        &master_pass,
                        vault_pass.as_deref().map(String::as_str),
                        vault_timeout,
                    );
                    match result {
                        Ok(()) => std::process::exit(0),
                        Err(e) => {
                            eprintln!("[ERROR] {e}");
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("[!] Argumentos insuficientes para --silent-install");
                    std::process::exit(1);
                }
            }
            "--silent-reset-vault" => {
                // Formato: --silent-reset-vault <sqlite_path> [timeout_mins]
                // Secretos por variables de entorno: TEATROGESTION_MASTER_PASSWORD y TEATROGESTION_NEW_VAULT_PASSWORD.
                if args.len() >= 3 {
                    let db_path = PathBuf::from(&args[2]);
                    let timeout_mins = args.get(3).and_then(|s| s.parse::<i32>().ok());
                    let (Some(master_pass), Some(new_vault_pass)) = (secret_from_env("TEATROGESTION_MASTER_PASSWORD"), secret_from_env("TEATROGESTION_NEW_VAULT_PASSWORD")) else {
                        eprintln!("[ERROR] Faltan TEATROGESTION_MASTER_PASSWORD y/o TEATROGESTION_NEW_VAULT_PASSWORD.");
                        std::process::exit(1);
                    };

                    match admin_reset_vault_silent(&db_path, &source_dir, &master_pass, &new_vault_pass, timeout_mins) {
                        Ok(()) => {
                            println!("[OK] ¡Contraseña de Bóveda restablecida exitosamente!");
                            std::process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("[ERROR] {e}");
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("[!] Argumentos insuficientes para --silent-reset-vault");
                    std::process::exit(1);
                }
            }
            "--machine-id" => {
                match MachineId::local() {
                    Ok(m) => println!("{}", m),
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
                return;
            }
            _ => {}
        }
    }

    // Sin argumentos: Menú interactivo principal
    print_banner();
    println!("Seleccione el modo de operación:");
    println!("  1) Instalar TeatroGestión (Asistente en Terminal)");
    println!("  2) Abrir Instalador Gráfico (Ventana Moderna)");
    println!("  3) Restablecer Contraseña de Bóveda de Inspección (Admin)");
    println!("  4) Salir\n");

    loop {
        print!("Elija una opción [1-4, por defecto 1]: ");
        let _ = io::stdout().flush();
        let Some(op) = read_line_opt() else {
            println!("\nNo hay entrada por consola: saliendo.");
            break;
        };
        match op.trim() {
            "1" | "" => {
                run_interactive_install(&source_dir);
                break;
            }
            "2" => {
                launch_gui(&source_dir);
                break;
            }
            "3" => {
                admin_reset_vault_cli(&source_dir);
                break;
            }
            "4" => {
                println!("Saliendo...");
                break;
            }
            _ => println!("Opción no válida. Ingrese 1, 2, 3 o 4."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("gestion-installer-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn the_built_in_sha256_matches_the_standard_vectors_and_the_vault_hash_uses_the_fixed_salt() {
        assert_eq!(compute_sha256_hex(""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(compute_sha256_hex("abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        // La app (gestion-core) calcula lo mismo: si esto cambia, nadie podría abrir la Bóveda con la clave puesta al instalar.
        assert_eq!(hash_vault_password("  clave-de-prueba \n"), compute_sha256_hex("boc_carina_lion_vault_salt_2026_clave-de-prueba"));
    }

    #[test]
    fn the_vault_password_is_saved_replaced_and_keeps_its_timeout() {
        let dir = tmp("vault");
        let db = dir.join("sub").join("teatro_gestion.sqlite");
        save_vault_password_to_db(&db, "una", Some(5)).unwrap();
        save_vault_password_to_db(&db, "otra", None).unwrap();
        let conn = Connection::open(&db).unwrap();
        let get = |k: &str| conn.query_row("SELECT value FROM app_settings WHERE key = ?1", params![k], |r| r.get::<_, String>(0)).unwrap();
        assert_eq!(get(KEY_VAULT_HASH), hash_vault_password("otra"), "la segunda contraseña reemplaza a la primera");
        assert_eq!(get(KEY_VAULT_TIMEOUT), "5", "sin timeout nuevo se conserva el anterior");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_license_registry_is_the_one_in_admin_and_none_is_ever_created() {
        let root = tmp("registry");
        let payload = root.join("TeatroGestion");
        fs::create_dir_all(&payload).unwrap();
        assert_eq!(find_registry_db(&payload), None, "sin base de licencias no se crea ninguna");
        assert!(!root.join("licencias.db").exists());
        // Una base suelta en la raíz se usa solo si no hay una en admin/.
        fs::write(root.join("licencias.db"), b"x").unwrap();
        assert_eq!(find_registry_db(&payload).unwrap().file_name().unwrap(), "licencias.db");
        fs::create_dir_all(root.join("admin")).unwrap();
        fs::write(root.join("admin").join("licencias.db"), b"x").unwrap();
        assert!(find_registry_db(&payload).unwrap().to_string_lossy().contains("admin"), "con admin/licencias.db manda esa");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_registry_entry_is_added_without_touching_the_existing_rows() {
        let root = tmp("record");
        let db = root.join("licencias.db");
        let machine = MachineId::local().unwrap();
        record_in_licencias_db(&db, &machine, "Cabina", 1_000, 0, 0, "token-1").unwrap();
        record_in_licencias_db(&db, &machine, "Oficina", 2_000, 5_000, 30, "token-2").unwrap();
        let conn = Connection::open(&db).unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM licencias_emitidas", [], |r| r.get(0)).unwrap();
        let equipos: i64 = conn.query_row("SELECT COUNT(*) FROM equipos", [], |r| r.get(0)).unwrap();
        assert_eq!((n, equipos), (2, 1), "dos emisiones, un solo equipo");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn secrets_come_from_the_environment_and_are_removed_from_it() {
        std::env::set_var("TG_TEST_SECRET", "s3creto");
        assert_eq!(secret_from_env("TG_TEST_SECRET").as_deref().map(String::as_str), Some("s3creto"));
        assert!(std::env::var("TG_TEST_SECRET").is_err(), "no queda en el entorno");
        std::env::set_var("TG_TEST_EMPTY", "");
        assert!(secret_from_env("TG_TEST_EMPTY").is_none());
        assert!(secret_from_env("TG_TEST_MISSING").is_none());
    }
}
