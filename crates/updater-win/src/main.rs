//! `Actualizar_TeatroGestion.exe` — un solo archivo: doble clic y listo.
//!
//! ```text
//! Actualizar_TeatroGestion.exe [--si] [--solo-comprobar] [--carpeta RUTA] [--datos RUTA]
//! ```
//! Sin argumentos pregunta antes de actualizar. `--si` no pregunta nada (ni espera al final); `--solo-comprobar` revisa todo
//! y no cambia nada; `--carpeta`/`--datos` son para pruebas o instalaciones en otro lugar.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use updater_win::{app_is_open, apply, backup_data, check_db, copy_db_files, data_dir, find_install, install_candidates, license_summary, stamp, verify_installed, DbCheck, Payload, DB_NAME, EXE_NAME};

const NOVEDADES: &str = include_str!("../NOVEDADES.txt");

#[cfg(windows)]
fn utf8_console() {
    extern "system" {
        fn SetConsoleOutputCP(cp: u32) -> i32;
    }
    // SAFETY: llamada simple sin punteros.
    unsafe {
        SetConsoleOutputCP(65001);
    }
}
#[cfg(not(windows))]
fn utf8_console() {}

struct Ui {
    auto: bool,
    log: Option<PathBuf>,
}

impl Ui {
    fn say(&self, text: &str) {
        println!("{text}");
        if let Some(p) = &self.log {
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
                let _ = writeln!(f, "[{}] {text}", stamp(now()));
            }
        }
    }

    /// Pregunta sí/no (Enter = sí). Con `--si` responde que sí.
    fn yes(&self, question: &str) -> bool {
        if self.auto {
            return true;
        }
        print!("{question} [S/n]: ");
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => false,
            Ok(_) => !matches!(line.trim().to_lowercase().as_str(), "n" | "no"),
        }
    }

    fn pause(&self) {
        if self.auto {
            return;
        }
        print!("\nPresione Enter para cerrar…");
        let _ = std::io::stdout().flush();
        let _ = std::io::stdin().lock().read_line(&mut String::new());
    }
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Pide al programa que se cierre como lo haría el botón de la ventana (sin forzarlo).
fn ask_app_to_close() {
    let mut cmd = std::process::Command::new("taskkill");
    cmd.args(["/IM", EXE_NAME]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let _ = cmd.status();
}

/// Espera a que TeatroGestión esté cerrado (hasta 2 minutos). `false` si sigue abierto.
fn wait_until_closed(ui: &Ui, exe: &Path) -> bool {
    if !app_is_open(exe) {
        return true;
    }
    ui.say("TeatroGestión está abierto: hay que cerrarlo para actualizar.");
    if ui.yes("¿Lo cierro yo por usted? (guarda lo que tenga abierto como siempre)") {
        ask_app_to_close();
    } else {
        ui.say("Ciérrelo usted (botón X de la ventana) y espero.");
    }
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(120) {
        if !app_is_open(exe) {
            std::thread::sleep(Duration::from_millis(800)); // que termine de soltar los archivos
            return true;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    false
}

fn run(ui: &mut Ui, args: &[String]) -> Result<(), String> {
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(PathBuf::from);
    let only_check = args.iter().any(|a| a == "--solo-comprobar");

    let payload = Payload::embedded();
    if payload.exe.is_empty() {
        return Err("este archivo está incompleto (no trae la versión nueva). Pídale otra copia a quien se lo envió.".into());
    }

    let candidates = install_candidates(flag("--carpeta").as_deref());
    let Some(dir) = find_install(&candidates) else {
        return Err(format!(
            "No encontré TeatroGestión instalado en esta PC (busqué en {}).\nPara instalarlo por primera vez use el instalador del pendrive.",
            candidates.iter().map(|c| c.display().to_string()).collect::<Vec<_>>().join(", ")
        ));
    };
    ui.log = Some(dir.join("actualizacion.log"));
    let data = flag("--datos").unwrap_or_else(data_dir);

    ui.say(&format!("TeatroGestión instalado en: {}", dir.display()));
    ui.say(&format!("Datos (base de datos): {}", data.join(DB_NAME).display()));
    ui.say(&license_summary(&dir));
    println!("\n{NOVEDADES}");

    if only_check {
        // Solo revisar: respalda en una carpeta temporal y comprueba la base, sin tocar la instalación.
        let tmp = std::env::temp_dir().join(format!("tg-comprobar-{}", std::process::id()));
        let res = backup_data_and_check(ui, &data, &tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        res?;
        ui.say("Comprobación terminada: no se cambió nada.");
        return Ok(());
    }

    if !ui.yes("¿Actualizar TeatroGestión ahora?") {
        ui.say("No se hizo ningún cambio.");
        return Ok(());
    }

    if !wait_until_closed(ui, &dir.join(EXE_NAME)) {
        return Err("TeatroGestión sigue abierto. Ciérrelo y vuelva a ejecutar este archivo.".into());
    }

    // 1) Respaldo de los datos y comprobación de compatibilidad, ANTES de cambiar nada.
    let sello = stamp(now());
    let backup = backup_data(&data, &sello)?;
    match &backup {
        Some(b) => ui.say(&format!("Respaldo de sus datos guardado en: {}", b.display())),
        None => ui.say("Todavía no hay base de datos: la app la crea al abrir."),
    }
    if let Some(b) = &backup {
        match check_db(&b.join(DB_NAME))? {
            DbCheck::Compatible { applied, pending } => ui.say(&format!("Base de datos compatible ({applied} actualizaciones internas ya aplicadas, {pending} nuevas para aplicar al abrir).")),
            DbCheck::NoDb => {}
            DbCheck::Mismatch { version, name } => {
                return Err(format!("Su base de datos fue creada con una variante de la actualización interna {version} ({name}) que la versión nueva no reconoce.\nNO se cambió nada. Avise al desarrollador y envíele el archivo actualizacion.log.\n(Su respaldo está en {}.)", b.display()));
            }
            DbCheck::NewerDb { version } => {
                return Err(format!("Su base de datos es de una versión MÁS NUEVA (actualización interna {version}) que la de este archivo.\nNO se cambió nada: pida el actualizador más reciente."));
            }
        }
    }

    // 2) Reemplazo con vuelta atrás.
    let files = apply(&dir, &payload).map_err(|e| format!("{e}\nNo se cambió nada: TeatroGestión sigue como estaba."))?;
    if !verify_installed(&dir, &payload) {
        return Err("Los archivos instalados no coinciden con los esperados. Vuelva a ejecutar el actualizador.".into());
    }
    ui.say(&format!("Archivos actualizados: {}.", files.join(", ")));
    ui.say("La versión anterior quedó guardada como «TeatroGestion.exe.anterior».");
    Ok(())
}

/// Para `--solo-comprobar`: copia la base a una carpeta temporal (no al respaldo real) y la comprueba.
fn backup_data_and_check(ui: &Ui, data: &Path, tmp: &Path) -> Result<(), String> {
    if !data.join(DB_NAME).is_file() {
        ui.say("Todavía no hay base de datos.");
        return Ok(());
    }
    copy_db_files(data, tmp)?;
    match check_db(&tmp.join(DB_NAME))? {
        DbCheck::Compatible { applied, pending } => ui.say(&format!("Base de datos compatible ({applied} aplicadas, {pending} nuevas).")),
        DbCheck::NoDb => ui.say("Todavía no hay base de datos."),
        DbCheck::Mismatch { version, name } => return Err(format!("Base incompatible: la actualización interna {version} ({name}) no coincide.")),
        DbCheck::NewerDb { version } => return Err(format!("La base es más nueva que este actualizador (interna {version}).")),
    }
    Ok(())
}

fn main() -> ExitCode {
    utf8_console();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut ui = Ui { auto: args.iter().any(|a| a == "--si"), log: None };
    println!("==============================================================================");
    println!("   ACTUALIZADOR DE TEATROGESTIÓN — Ballet Oficial de Córdoba");
    println!("==============================================================================\n");
    match run(&mut ui, &args) {
        Ok(()) => {
            if !args.iter().any(|a| a == "--solo-comprobar") {
                println!("\n==============================================================================");
                println!("   ¡LISTO! TeatroGestión está actualizado.");
                println!("   Ábralo desde el acceso directo del Escritorio como siempre.");
                println!("   Sus datos y su licencia siguen igual.");
                println!("==============================================================================");
            }
            ui.pause();
            ExitCode::SUCCESS
        }
        Err(e) => {
            ui.say(&format!("\nNO SE PUDO ACTUALIZAR: {e}"));
            ui.pause();
            ExitCode::from(1)
        }
    }
}
