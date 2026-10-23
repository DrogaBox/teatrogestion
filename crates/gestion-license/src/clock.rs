//! Defensa contra el retroceso del reloj (offline).

use std::path::{Path, PathBuf};

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::{License, MachineId};

/// Margen que se tolera por debajo de la marca máxima: cambios de zona horaria, ajustes de la BIOS, deriva del reloj.
pub const TOLERANCE_SECS: u64 = 36 * 3600;
const MAGIC: &str = "TEATROG-CLOCK-1";
const MAGIC_LEGACY: &str = "TEATROCUE-CLOCK-1";

/// Fecha de compilación del programa (segundos Unix).
pub const BUILD_UNIX: u64 = parse_u64(env!("TEATROGESTION_BUILD_UNIX_VALUE"));

const fn parse_u64(s: &str) -> u64 {
    let b = s.as_bytes();
    let (mut i, mut n) = (0, 0u64);
    while i < b.len() {
        n = n * 10 + (b[i] - b'0') as u64;
        i += 1;
    }
    n
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClockFault {
    /// El reloj está por debajo de lo que ya se había visto (o de la fecha de compilación).
    Rollback { seen: u64, now: u64 },
    /// Una marca existe pero su sello no corresponde (editada, o de otra PC/licencia).
    Tampered(String),
    /// No queda ninguna marca y la licencia no es recién emitida: alguien las borró.
    MarksMissing,
}

/// Un sitio donde se guarda la marca.
pub trait MarkStore: Send + Sync {
    fn label(&self) -> String;
    fn read(&self) -> Option<String>;
    fn write(&self, text: &str) -> std::io::Result<()>;
}

/// Marca en un archivo (escritura atómica: temporal + renombrado).
pub struct FileStore(pub PathBuf);

impl MarkStore for FileStore {
    fn label(&self) -> String {
        self.0.display().to_string()
    }

    fn read(&self) -> Option<String> {
        std::fs::read_to_string(&self.0).ok()
    }

    fn write(&self, text: &str) -> std::io::Result<()> {
        if let Some(dir) = self.0.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = self.0.with_extension(format!("tmp{}", std::process::id()));
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, &self.0).inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
    }
}

/// Clave HMAC de las marcas: atada a la PC y a esta licencia concreta.
fn mark_key(machine: &MachineId, license: &License) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"TeatroCue/clock-mark/v1|");
    h.update(machine.as_str().as_bytes());
    h.update(b"|");
    h.update(license.fingerprint().as_bytes());
    h.finalize().into()
}

/// Contenido de una marca: la fecha máxima vista y cuántas reactivaciones de emergencia se han canjeado (anti-replay).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mark {
    pub mark: u64,
    pub epoch: u32,
}

fn mac_over(machine: &MachineId, license: &License, data: &str) -> String {
    let mut m = <Hmac<Sha256> as Mac>::new_from_slice(&mark_key(machine, license)).expect("HMAC acepta cualquier longitud");
    m.update(data.as_bytes());
    m.finalize().into_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn mark_text_with(license: &License, m: Mark) -> String {
    format!("{MAGIC}\nmark={}\nepoch={}\nmac={}\n", m.mark, m.epoch, mac_over(&license.machine, license, &format!("mark={};epoch={}", m.mark, m.epoch)))
}

/// `Ok(marca)` si el texto es una marca auténtica para esta licencia; `Err` si existe pero no lo es.
pub fn parse_mark_full(text: &str, license: &License) -> Result<Mark, String> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let header = lines.next();
    if header != Some(MAGIC) && header != Some(MAGIC_LEGACY) {
        return Err("cabecera inválida".into());
    }
    let (mut mark, mut epoch, mut tag) = (None, None, None);
    for l in lines {
        if let Some(v) = l.strip_prefix("mark=") {
            mark = v.parse::<u64>().ok();
        } else if let Some(v) = l.strip_prefix("epoch=") {
            epoch = v.parse::<u32>().ok();
        } else if let Some(v) = l.strip_prefix("mac=") {
            tag = Some(v.to_string());
        }
    }
    let (Some(mark), Some(tag)) = (mark, tag) else { return Err("faltan campos".into()) };
    let (data, epoch) = match epoch {
        Some(e) => (format!("mark={mark};epoch={e}"), e),
        None => (format!("mark={mark}"), 0),
    };
    let want = mac_over(&license.machine, license, &data);
    let ok = want.len() == tag.len() && want.bytes().zip(tag.bytes()).fold(0u8, |a, (x, y)| a | (x ^ y)) == 0;
    if ok { Ok(Mark { mark, epoch }) } else { Err("el sello no corresponde a esta PC y licencia".into()) }
}

pub fn parse_mark(text: &str, license: &License) -> Result<u64, String> {
    parse_mark_full(text, license).map(|m| m.mark)
}

/// Contador de reactivaciones de emergencia ya canjeadas en esta PC.
pub fn epoch(license: &License, stores: &[Box<dyn MarkStore>]) -> u32 {
    stores.iter().filter_map(|s| s.read()).filter_map(|t| parse_mark_full(&t, license).ok()).map(|m| m.epoch).max().unwrap_or(0)
}

/// Decide si el reloj `now` es creíble. Devuelve la fecha efectiva.
pub fn check(now: u64, license: &License, stores: &[Box<dyn MarkStore>], anchors: &[u64]) -> Result<u64, ClockFault> {
    let mut high = 0u64;
    let mut present = 0;
    for s in stores {
        let Some(text) = s.read() else { continue };
        present += 1;
        match parse_mark(&text, license) {
            Ok(m) => high = high.max(m),
            Err(why) => return Err(ClockFault::Tampered(format!("{}: {why}", s.label()))),
        }
    }
    if present == 0 {
        let fresh = now.abs_diff(license.issued_unix) <= TOLERANCE_SECS;
        if !fresh {
            return Err(ClockFault::MarksMissing);
        }
    }
    let anchor = anchors.iter().copied().max().unwrap_or(0).max(BUILD_UNIX);
    let seen = high.max(anchor);
    if now + TOLERANCE_SECS < seen {
        return Err(ClockFault::Rollback { seen, now });
    }
    Ok(now.max(high))
}

pub fn record(mark: u64, license: &License, stores: &[Box<dyn MarkStore>]) -> usize {
    let epoch = epoch(license, stores);
    let text = mark_text_with(license, Mark { mark, epoch });
    stores
        .iter()
        .filter(|s| {
            let higher = s.read().and_then(|t| parse_mark_full(&t, license).ok()).is_some_and(|m| m.mark > mark || m.epoch > epoch);
            !higher && s.write(&text).is_ok()
        })
        .count()
}

pub fn reset_with_epoch(mark: u64, epoch: u32, license: &License, stores: &[Box<dyn MarkStore>]) -> usize {
    let text = mark_text_with(license, Mark { mark, epoch });
    stores.iter().filter(|s| s.write(&text).is_ok()).count()
}

pub fn reset(mark: u64, license: &License, stores: &[Box<dyn MarkStore>]) -> usize {
    reset_with_epoch(mark, 0, license, stores)
}

pub fn default_stores(install_dir: &Path) -> Vec<Box<dyn MarkStore>> {
    let mut v: Vec<Box<dyn MarkStore>> = vec![Box::new(FileStore(install_dir.join("license.state")))];
    if let Some(pd) = std::env::var_os("ProgramData").or_else(|| std::env::var_os("TEATROGESTION_STATE_DIR")) {
        v.push(Box::new(FileStore(PathBuf::from(pd).join("TeatroGestion").join("clock.dat"))));
    }
    #[cfg(windows)]
    v.push(Box::new(windows_registry::RegistryStore));
    v
}

pub fn system_anchors() -> Vec<u64> {
    #[cfg(windows)]
    {
        let root = std::env::var_os("SystemRoot").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
        [
            r"System32\config\SYSTEM",
            r"System32\config\SOFTWARE",
            r"System32\config\SECURITY",
            r"System32\winevt\Logs\System.evtx",
            r"System32\winevt\Logs\Application.evtx",
            r"System32\winevt\Logs\Security.evtx",
        ]
        .iter()
        .filter_map(|f| std::fs::metadata(root.join(f)).ok()?.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs()))
        .collect()
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

#[cfg(windows)]
mod windows_registry {
    use super::MarkStore;
    use windows::core::{w, PCWSTR};
    use windows::Win32::System::Registry::{RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ};

    pub struct RegistryStore;

    impl MarkStore for RegistryStore {
        fn label(&self) -> String {
            r"HKCU\Software\TeatroGestion\ClockMark".into()
        }

        fn read(&self) -> Option<String> {
            let mut buf = [0u16; 512];
            let mut bytes = std::mem::size_of_val(&buf) as u32;
            let r = unsafe { RegGetValueW(HKEY_CURRENT_USER, w!(r"Software\TeatroGestion"), w!("ClockMark"), RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr().cast()), Some(&mut bytes)) };
            if r.0 != 0 {
                // Fallback a TeatroCue si existe
                let r2 = unsafe { RegGetValueW(HKEY_CURRENT_USER, w!(r"Software\TeatroCue"), w!("ClockMark"), RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr().cast()), Some(&mut bytes)) };
                if r2.0 != 0 {
                    return None;
                }
            }
            let len = (bytes as usize / 2).saturating_sub(1).min(buf.len());
            Some(String::from_utf16_lossy(&buf[..len]))
        }

        fn write(&self, text: &str) -> std::io::Result<()> {
            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let r = unsafe { RegSetKeyValueW(HKEY_CURRENT_USER, w!(r"Software\TeatroGestion"), w!("ClockMark"), REG_SZ.0, Some(wide.as_ptr().cast()), (wide.len() * 2) as u32) };
            if r.0 == 0 { Ok(()) } else { Err(std::io::Error::from_raw_os_error(r.0 as i32)) }
        }
    }

    #[allow(dead_code)]
    fn _unused(_: PCWSTR) {}
}
#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    pub struct Mem(pub Arc<Mutex<Option<String>>>);

    impl MarkStore for Mem {
        fn label(&self) -> String {
            "memoria".into()
        }
        fn read(&self) -> Option<String> {
            self.0.lock().unwrap().clone()
        }
        fn write(&self, text: &str) -> std::io::Result<()> {
            *self.0.lock().unwrap() = Some(text.to_string());
            Ok(())
        }
    }

    #[test]
    fn rollback_detection_rejects_past_time() {
        let machine = MachineId::from_guid("pc-del-teatro");
        let t0 = BUILD_UNIX + 200_000;
        let license = License {
            licensee: "Ballet Oficial de Córdoba".into(),
            machine,
            issued_unix: t0,
            expires_unix: t0 + 1_000_000,
        };
        let stores: Vec<Box<dyn MarkStore>> = vec![Box::new(Mem::default())];
        reset(t0 + 500_000, &license, &stores);

        // Checked at t0 + 500_000 -> OK
        assert!(check(t0 + 500_000, &license, &stores, &[]).is_ok());

        // Checked at t0 (< t0 + 500_000 - 36h) -> Rollback
        let err = check(t0, &license, &stores, &[]).unwrap_err();
        assert!(matches!(err, ClockFault::Rollback { seen, now } if seen == t0 + 500_000 && now == t0));
    }
}
