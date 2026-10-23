use gestion_license::{activation::Setup, LicenseInfo, MachineId};

#[tauri::command]
pub fn get_license_info() -> Result<LicenseInfo, String> {
    Ok(LicenseInfo::detect())
}

#[tauri::command]
pub fn apply_emergency_activation(key: String) -> Result<LicenseInfo, String> {
    let setup = Setup::installed();
    setup.apply(&key).map_err(|e| e.to_string())?;
    Ok(LicenseInfo::detect())
}

#[tauri::command]
pub fn get_machine_id() -> Result<String, String> {
    MachineId::local().map(|m| m.to_string())
}
