//! Machine ID: identificador estable y legible de una PC, derivado del `MachineGuid` de Windows.

use sha2::{Digest, Sha256};

/// `XXXX-XXXX-XXXX-XXXX` (16 caracteres hexadecimales en mayúscula).
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MachineId(String);

const DOMAIN: &str = "TeatroCue/machine-id/v1\n";

impl MachineId {
    /// Normaliza un GUID de máquina: SHA-256 de `dominio + guid` (sin espacios, en minúsculas) y los 8 primeros
    /// bytes en hexadecimal agrupado de a 4.
    pub fn from_guid(guid: &str) -> Self {
        let mut h = Sha256::new();
        h.update(DOMAIN.as_bytes());
        h.update(guid.trim().to_ascii_lowercase().as_bytes());
        let d = h.finalize();
        let hex: String = d[..8].iter().map(|b| format!("{b:02X}")).collect();
        Self(format!("{}-{}-{}-{}", &hex[0..4], &hex[4..8], &hex[8..12], &hex[12..16]))
    }

    /// Machine ID de esta PC. En Windows sale del registro; en otros sistemas (desarrollo) de `/etc/machine-id`.
    pub fn local() -> Result<Self, String> {
        read_machine_guid().map(|g| Self::from_guid(&g))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MachineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for MachineId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim().to_ascii_uppercase();
        let groups: Vec<&str> = s.split('-').collect();
        let ok = groups.len() == 4 && groups.iter().all(|g| g.len() == 4 && g.chars().all(|c| c.is_ascii_hexdigit()));
        if ok {
            Ok(Self(s))
        } else {
            Err(format!("Machine ID inválido: `{s}` (se esperaba XXXX-XXXX-XXXX-XXXX)"))
        }
    }
}

#[cfg(windows)]
fn read_machine_guid() -> Result<String, String> {
    use windows::core::w;
    use windows::Win32::System::Registry::{
        RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RRF_SUBKEY_WOW6464KEY,
    };

    let mut buf = [0u16; 128];
    let mut bytes = std::mem::size_of_val(&buf) as u32;
    // SAFETY: `buf` y `bytes` son locales y válidos durante la llamada; `RegGetValueW` escribe como mucho `bytes`.
    let r = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            w!(r"SOFTWARE\Microsoft\Cryptography"),
            w!("MachineGuid"),
            RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut bytes),
        )
    };
    if r.0 != 0 {
        return Err(format!("no se pudo leer MachineGuid del registro (error {})", r.0));
    }
    let len = (bytes as usize / 2).saturating_sub(1).min(buf.len());
    let guid = String::from_utf16_lossy(&buf[..len]);
    if guid.trim().is_empty() {
        Err("MachineGuid vacío".into())
    } else {
        Ok(guid)
    }
}

#[cfg(not(windows))]
fn read_machine_guid() -> Result<String, String> {
    // Para desarrollo y pruebas en Linux o Android
    if let Ok(s) = std::fs::read_to_string("/etc/machine-id") {
        let s = s.trim();
        if !s.is_empty() {
            return Ok(s.to_string());
        }
    }
    if let Ok(s) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
        let s = s.trim();
        if !s.is_empty() {
            return Ok(s.to_string());
        }
    }
    // Fallback reproducible para entornos de desarrollo donde no hay machine-id
    Ok("cachyos-teatro-dev-machine-guid".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_has_the_readable_format_and_is_stable() {
        let a = MachineId::from_guid("0A1B2C3D-0000-1111-2222-333344445555");
        assert_eq!(a.as_str().len(), 19);
        assert!(a.as_str().split('-').all(|g| g.len() == 4 && g.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase())), "{a}");
        assert_eq!(a, MachineId::from_guid("  0a1b2c3d-0000-1111-2222-333344445555\n"));
        assert_ne!(a, MachineId::from_guid("0A1B2C3D-0000-1111-2222-333344445556"));
        assert_eq!(MachineId::from_guid("test-guid").as_str(), MachineId::from_guid("test-guid").as_str());
    }

    #[test]
    fn ids_parse_back_and_reject_garbage() {
        let a = MachineId::from_guid("x");
        assert_eq!(a.to_string().parse::<MachineId>().unwrap(), a);
        assert_eq!(a.to_string().to_lowercase().parse::<MachineId>().unwrap(), a);
        for bad in ["", "1234", "ZZZZ-ZZZZ-ZZZZ-ZZZZ", "1234-5678-9ABC", "1234-5678-9ABC-DEF01"] {
            assert!(bad.parse::<MachineId>().is_err(), "{bad}");
        }
    }
}
