//! Actualización desde los releases del repositorio privado de GitHub.
//!
//! El token de lectura se escribe una sola vez en el equipo y queda en la carpeta de datos de la aplicación; no viaja
//! dentro del programa. En Windows se descarga y se lanza el actualizador (que respalda los datos y reemplaza solo los
//! programas); en Android se descarga el APK y se lo entrega al instalador del sistema.

use std::path::PathBuf;

use gestion_update::{AssetRule, UpdateInfo};
use serde::Serialize;
use tauri::{AppHandle, Manager};

const REPO: &str = "DrogaBox/teatrogestion";
const TOKEN_FILE: &str = "update_token";

fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Qué archivo del release sirve en este sistema (`None`: la actualización automática no está disponible).
fn asset_rule() -> Option<(&'static str, AssetRule)> {
    if cfg!(windows) {
        Some(("windows", AssetRule { suffix: ".exe", prefix: "actualizar_" }))
    } else if cfg!(target_os = "android") {
        Some(("android", AssetRule { suffix: ".apk", prefix: "" }))
    } else {
        None
    }
}

fn token_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map(|d| d.join(TOKEN_FILE)).map_err(|e| e.to_string())
}

fn read_token(app: &AppHandle) -> Option<String> {
    let t = std::fs::read_to_string(token_path(app).ok()?).ok()?;
    let t = t.trim().to_string();
    (!t.is_empty()).then_some(t)
}

#[derive(Serialize)]
pub struct UpdateStatus {
    pub version: String,
    pub platform: Option<String>,
    pub has_token: bool,
}

#[tauri::command]
pub fn get_update_status(app: AppHandle) -> UpdateStatus {
    UpdateStatus {
        version: current_version().to_string(),
        platform: asset_rule().map(|(p, _)| p.to_string()),
        has_token: read_token(&app).is_some(),
    }
}

/// Comprueba el token contra el repositorio y, si da acceso, lo guarda.
#[tauri::command]
pub async fn set_update_token(app: AppHandle, token: String) -> Result<(), String> {
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err("Escriba el token.".into());
    }
    let t = token.clone();
    tauri::async_runtime::spawn_blocking(move || gestion_update::verify_access(REPO, &t)).await.map_err(|e| e.to_string())??;
    let path = token_path(&app)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, token).map_err(|e| format!("No se pudo guardar el token: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

#[tauri::command]
pub fn clear_update_token(app: AppHandle) -> Result<(), String> {
    let path = token_path(&app)?;
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Busca una versión más nueva. `Ok(None)`: no hay nada para instalar (o no hay token configurado).
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let Some((_, rule)) = asset_rule() else { return Ok(None) };
    let Some(token) = read_token(&app) else { return Ok(None) };
    tauri::async_runtime::spawn_blocking(move || gestion_update::check(REPO, &token, current_version(), &rule)).await.map_err(|e| e.to_string())?
}

#[derive(Serialize)]
pub struct InstallOutcome {
    /// `windows`: el actualizador ya se lanzó y la app se cierra. `android`: falta entregar el APK al instalador del sistema.
    pub mode: String,
    pub path: String,
}

/// Descarga la versión nueva (verificada) y la pone en marcha.
#[tauri::command]
pub async fn install_update(app: AppHandle, info: UpdateInfo) -> Result<InstallOutcome, String> {
    let Some((platform, _)) = asset_rule() else { return Err("La actualización automática no está disponible en este sistema.".into()) };
    let token = read_token(&app).ok_or("Falta el token de actualización.")?;
    let base = if platform == "android" {
        app.path().app_cache_dir().map_err(|e| e.to_string())?
    } else {
        std::env::temp_dir()
    };
    let dir = base.join("teatrogestion-actualizacion");
    let name = std::path::Path::new(&info.asset_name).file_name().ok_or("Nombre de archivo inválido.")?.to_os_string();
    let dest = dir.join(name);

    let (d, i) = (dest.clone(), info.clone());
    tauri::async_runtime::spawn_blocking(move || gestion_update::download(REPO, &token, &i, &d)).await.map_err(|e| e.to_string())??;

    if platform == "windows" {
        launch_windows_updater(&dest)?;
        let handle = app.clone();
        tauri::async_runtime::spawn(async move {
            std::thread::sleep(std::time::Duration::from_millis(800));
            handle.exit(0);
        });
        return Ok(InstallOutcome { mode: "windows".into(), path: dest.to_string_lossy().into_owned() });
    }
    Ok(InstallOutcome { mode: "android".into(), path: dest.to_string_lossy().into_owned() })
}

#[cfg(windows)]
fn launch_windows_updater(exe: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    std::process::Command::new(exe)
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("No se pudo iniciar el actualizador: {e}"))
}

#[cfg(not(windows))]
fn launch_windows_updater(_exe: &std::path::Path) -> Result<(), String> {
    Err("Solo disponible en Windows.".into())
}
