//! Licencia offline de TeatroGestión (Ballet Oficial de Córdoba).

pub mod activation;
pub mod clock;
pub mod keyfile;
pub mod machine;

use std::path::{Path, PathBuf};

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};

pub use machine::MachineId;

pub const LICENSE_FILE_NAME: &str = "license.key";
pub const MAGIC: &str = "TEATROGESTION-LICENSE-1";
pub const MAGIC_TEATROCUE_2: &str = "TEATROCUE-LICENSE-2";
pub const MAGIC_TEATROCUE_1: &str = "TEATROCUE-LICENSE-1";

/// Clave pública incrustada al compilar (vacía si no se configuró TEATROGESTION_PUBLIC_KEY_FILE / TEATROCUE_PUBLIC_KEY_FILE).
const EMBEDDED_PUBLIC_KEY_HEX: &str = include_str!(concat!(env!("OUT_DIR"), "/public_key.hex"));

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct License {
    pub licensee: String,
    pub machine: MachineId,
    pub issued_unix: u64,
    pub expires_unix: u64,
}

impl License {
    pub fn is_perpetual(&self) -> bool {
        self.expires_unix == 0
    }

    pub fn days_left(&self, now: u64) -> Option<i64> {
        (!self.is_perpetual()).then(|| (self.expires_unix as i64 - now as i64).div_euclid(86_400))
    }

    pub fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        Sha256::digest(signed_message(self, MAGIC)).iter().take(8).map(|b| format!("{b:02x}")).collect()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LicenseError {
    NotFound(PathBuf),
    Io(String),
    Malformed(String),
    BadSignature,
    WrongMachine { expected: String, found: String },
    NoPublicKey,
    NoMachineId(String),
    Expired { expires: u64 },
    Clock(clock::ClockFault),
}

impl std::fmt::Display for LicenseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(p) => write!(f, "no se encontró la licencia ({})", p.display()),
            Self::Io(e) => write!(f, "no se pudo leer la licencia: {e}"),
            Self::Malformed(e) => write!(f, "licencia ilegible: {e}"),
            Self::BadSignature => write!(f, "la firma de la licencia no es válida"),
            Self::WrongMachine { expected, found } => write!(f, "la licencia es para otra PC (Machine ID {expected}; esta PC es {found})"),
            Self::NoPublicKey => write!(f, "esta compilación no incluye clave pública de licencia"),
            Self::NoMachineId(e) => write!(f, "no se pudo obtener el Machine ID: {e}"),
            Self::Expired { expires } => write!(f, "la licencia venció el {}", fmt_date(*expires)),
            Self::Clock(clock::ClockFault::Rollback { seen, now }) => write!(f, "el reloj de esta PC está atrasado: ya se había usado el sistema el {} y ahora la fecha es {}. Corrija la fecha y hora de Windows", fmt_date(*seen), fmt_date(*now)),
            Self::Clock(clock::ClockFault::Tampered(why)) => write!(f, "el registro de uso del sistema fue alterado ({why})"),
            Self::Clock(clock::ClockFault::MarksMissing) => write!(f, "falta el registro de uso del sistema en esta PC (reinstale con el pendrive)"),
        }
    }
}

impl std::error::Error for LicenseError {}

fn signed_message(l: &License, magic: &str) -> Vec<u8> {
    format!("{magic}\nlicensee={}\nmachine={}\nissued={}\nexpires={}\n", l.licensee, l.machine, l.issued_unix, l.expires_unix).into_bytes()
}

fn signed_message_v1(l: &License) -> Vec<u8> {
    format!("{MAGIC_TEATROCUE_1}\nlicensee={}\nmachine={}\nissued={}\n", l.licensee, l.machine, l.issued_unix).into_bytes()
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if s.len() % 2 != 0 || !s.is_ascii() {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

pub fn issue(licensee: &str, machine: &MachineId, issued_unix: u64, expires_unix: u64, key: &SigningKey) -> String {
    let l = License { licensee: licensee.trim().to_string(), machine: machine.clone(), issued_unix, expires_unix };
    let sig = key.sign(&signed_message(&l, MAGIC));
    format!("{MAGIC}\nlicensee: {}\nmachine: {}\nissued: {}\nexpires: {}\nsignature: {}\n", l.licensee, l.machine, l.issued_unix, l.expires_unix, to_hex(&sig.to_bytes()))
}

fn parse(text: &str) -> Result<(License, Signature, String), LicenseError> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let magic_used = match lines.next() {
        Some(MAGIC) => MAGIC.to_string(),
        Some(MAGIC_TEATROCUE_2) => MAGIC_TEATROCUE_2.to_string(),
        Some(MAGIC_TEATROCUE_1) => MAGIC_TEATROCUE_1.to_string(),
        _ => return Err(LicenseError::Malformed(format!("falta la cabecera {MAGIC} o {MAGIC_TEATROCUE_2}"))),
    };
    let legacy = magic_used == MAGIC_TEATROCUE_1;
    let (mut licensee, mut machine, mut issued, mut expires, mut sig) = (None, None, None, None, None);
    for l in lines {
        if l.starts_with("activation:") {
            continue;
        }
        let Some((k, v)) = l.split_once(':') else { return Err(LicenseError::Malformed(format!("línea inválida: `{l}`"))) };
        let v = v.trim();
        match k.trim() {
            "licensee" => licensee = Some(v.to_string()),
            "machine" => machine = Some(v.parse::<MachineId>().map_err(LicenseError::Malformed)?),
            "issued" => issued = Some(v.parse::<u64>().map_err(|_| LicenseError::Malformed("fecha de emisión inválida".into()))?),
            "expires" => expires = Some(v.parse::<u64>().map_err(|_| LicenseError::Malformed("fecha de vencimiento inválida".into()))?),
            "signature" => sig = Some(v.to_string()),
            _ => {}
        }
    }
    let missing = |what: &str| LicenseError::Malformed(format!("falta `{what}`"));
    let sig_bytes = from_hex(&sig.ok_or_else(|| missing("signature"))?).ok_or_else(|| LicenseError::Malformed("firma no hexadecimal".into()))?;
    let sig_arr: [u8; 64] = sig_bytes.try_into().map_err(|_| LicenseError::Malformed("la firma debe medir 64 bytes".into()))?;
    let expires_unix = if legacy { 0 } else { expires.ok_or_else(|| missing("expires"))? };
    Ok((
        License { licensee: licensee.ok_or_else(|| missing("licensee"))?, machine: machine.ok_or_else(|| missing("machine"))?, issued_unix: issued.ok_or_else(|| missing("issued"))?, expires_unix },
        Signature::from_bytes(&sig_arr),
        magic_used,
    ))
}

pub fn verify_with(text: &str, public: &VerifyingKey, local: &MachineId) -> Result<License, LicenseError> {
    let (license, sig, magic_used) = parse(text)?;
    let message = if magic_used == MAGIC_TEATROCUE_1 {
        signed_message_v1(&license)
    } else {
        signed_message(&license, &magic_used)
    };
    public.verify_strict(&message, &sig).map_err(|_| LicenseError::BadSignature)?;
    if &license.machine != local {
        return Err(LicenseError::WrongMachine { expected: license.machine.to_string(), found: local.to_string() });
    }
    Ok(license)
}

pub fn embedded_public_key() -> Option<VerifyingKey> {
    let bytes: [u8; 32] = from_hex(EMBEDDED_PUBLIC_KEY_HEX)?.try_into().ok()?;
    VerifyingKey::from_bytes(&bytes).ok()
}

pub fn verify_embedded(text: &str, local: &MachineId) -> Result<License, LicenseError> {
    let key = embedded_public_key().ok_or(LicenseError::NoPublicKey)?;
    verify_with(text, &key, local)
}

pub fn license_path() -> PathBuf {
    let current_dir_key = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(LICENSE_FILE_NAME)));
    if let Some(ref path) = current_dir_key {
        if path.exists() {
            return path.clone();
        }
    }
    // Verificar carpeta de instalación estándar en Windows
    #[cfg(windows)]
    {
        let standard = PathBuf::from(r"C:\TeatroGestion").join(LICENSE_FILE_NAME);
        if standard.exists() {
            return standard;
        }
    }
    current_dir_key.unwrap_or_else(|| PathBuf::from(LICENSE_FILE_NAME))
}

#[derive(Debug)]
pub struct Status {
    pub machine: Option<MachineId>,
    pub result: Result<License, LicenseError>,
    pub path: PathBuf,
    pub now: u64,
    pub effective_expires_unix: u64,
}

impl Status {
    pub fn is_valid(&self) -> bool {
        self.result.is_ok()
    }
}

pub fn check_file(path: &Path) -> Status {
    let dir = path.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    check_with(path, now_unix(), &clock::default_stores(&dir), &clock::system_anchors(), embedded_public_key().as_ref())
}

pub fn check_with(path: &Path, now: u64, stores: &[Box<dyn clock::MarkStore>], anchors: &[u64], public: Option<&VerifyingKey>) -> Status {
    check_with_machine(path, now, stores, anchors, public, MachineId::local())
}

pub fn check_with_machine(path: &Path, now: u64, stores: &[Box<dyn clock::MarkStore>], anchors: &[u64], public: Option<&VerifyingKey>, machine: Result<MachineId, String>) -> Status {
    let mut effective_expires_unix = 0;
    let result = (|| {
        let local = machine.clone().map_err(LicenseError::NoMachineId)?;
        let text = std::fs::read_to_string(path).map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { LicenseError::NotFound(path.to_path_buf()) } else { LicenseError::Io(e.to_string()) })?;
        let public = public.ok_or(LicenseError::NoPublicKey)?;
        let license = verify_with(&text, public, &local)?;
        effective_expires_unix = activation::effective_expires(&license, activation::best_attached(&text, public, &local).as_ref());
        let effective = clock::check(now, &license, stores, anchors).map_err(LicenseError::Clock)?;
        clock::record(effective, &license, stores);
        if effective_expires_unix != 0 && effective >= effective_expires_unix {
            return Err(LicenseError::Expired { expires: effective_expires_unix });
        }
        Ok(license)
    })();
    Status { machine: machine.ok(), result, path: path.to_path_buf(), now, effective_expires_unix }
}

pub fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    let tmp = path.with_extension("key.tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", path.display())
    })
}

pub fn check_installed() -> Status {
    check_file(&license_path())
}

pub fn lock_message(status: &Status) -> Option<String> {
    let Err(e) = &status.result else { return None };
    let id = status.machine.as_ref().map_or("(no disponible)".to_string(), ToString::to_string);
    Some(format!("TeatroGestión no está autorizado en esta PC.\nMotivo: {e}\nMachine ID de esta PC: {id}\nEnvíe este Machine ID al desarrollador para recibir su license.key y cópielo junto a la aplicación."))
}

pub fn fmt_date(unix: u64) -> String {
    let z = (unix / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{:02}-{:02}", y + (m <= 2) as i64, m, d)
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

pub const EXPIRY_WARNING_DAYS: i64 = 14;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LicenseInfo {
    pub is_valid: bool,
    pub is_locked: bool,
    pub state: String,
    pub reason: Option<String>,
    pub licensee: Option<String>,
    pub machine_id: Option<String>,
    pub issued_date: Option<String>,
    pub expires_date: Option<String>,
    pub days_left: Option<i64>,
    pub warning: Option<String>,
    pub reactivable: bool,
    pub challenge_code: Option<String>,
}

impl LicenseInfo {
    pub fn not_enforced() -> Self {
        let machine_id = MachineId::local().ok().map(|m| m.to_string());
        Self {
            is_valid: true,
            is_locked: false,
            state: "NotEnforced".to_string(),
            reason: None,
            licensee: Some("Ballet Oficial de Córdoba (Modo Desarrollo)".to_string()),
            machine_id,
            issued_date: None,
            expires_date: None,
            days_left: None,
            warning: None,
            reactivable: false,
            challenge_code: None,
        }
    }

    pub fn from_status(status: &Status) -> Self {
        let machine_id = status.machine.as_ref().map(ToString::to_string);
        match &status.result {
            Ok(l) => {
                let expires = status.effective_expires_unix;
                let days_left = (expires != 0).then(|| (expires as i64 - status.now as i64).div_euclid(86_400));
                let warning = days_left.and_then(|d| {
                    if d <= EXPIRY_WARNING_DAYS {
                        let when = fmt_date(expires);
                        match d {
                            0 => Some(format!("La licencia de TeatroGestión vence HOY ({when}). Renuévela con el pendrive antes del próximo ensayo.")),
                            1 => Some(format!("La licencia de TeatroGestión vence MAÑANA ({when}). Renuévela con el pendrive.")),
                            n if n < 0 => Some(format!("La licencia de TeatroGestión venció hace {} días.", -n)),
                            n => Some(format!("La licencia de TeatroGestión vence en {n} días ({when}). Renuévela con el pendrive.")),
                        }
                    } else {
                        None
                    }
                });
                Self {
                    is_valid: true,
                    is_locked: false,
                    state: "Valid".to_string(),
                    reason: None,
                    licensee: Some(l.licensee.clone()),
                    machine_id,
                    issued_date: Some(fmt_date(l.issued_unix)),
                    expires_date: if expires == 0 { None } else { Some(fmt_date(expires)) },
                    days_left,
                    warning,
                    reactivable: false,
                    challenge_code: None,
                }
            }
            Err(e) => {
                let reactivable = matches!(e, LicenseError::Expired { .. });
                let setup = activation::Setup::installed();
                let challenge_code = setup.challenge().ok().map(|c| c.to_string());
                Self {
                    is_valid: false,
                    is_locked: true,
                    state: "Invalid".to_string(),
                    reason: Some(e.to_string()),
                    licensee: None,
                    machine_id,
                    issued_date: None,
                    expires_date: match e {
                        LicenseError::Expired { expires } => Some(fmt_date(*expires)),
                        _ => None,
                    },
                    days_left: match e {
                        LicenseError::Expired { expires } => Some(((*expires as i64) - (status.now as i64)).div_euclid(86_400)),
                        _ => None,
                    },
                    warning: None,
                    reactivable,
                    challenge_code,
                }
            }
        }
    }

    pub fn detect() -> Self {
        // En Windows por defecto o con enforce-license
        if cfg!(feature = "enforce-license") || cfg!(windows) {
            Self::from_status(&check_installed())
        } else {
            Self::not_enforced()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    fn pair() -> (SigningKey, VerifyingKey) {
        let k = SigningKey::generate(&mut OsRng);
        let v = k.verifying_key();
        (k, v)
    }

    fn id(s: &str) -> MachineId {
        MachineId::from_guid(s)
    }

    #[test]
    fn a_license_verifies_for_its_machine_only() {
        let (sk, pk) = pair();
        let (here, there) = (id("guid-de-la-pc-del-teatro"), id("otra-pc"));
        let text = issue("Ballet Oficial de Córdoba", &here, 1_800_000_000, 1_900_000_000, &sk);
        assert!(verify_with(&text, &pk, &here).is_ok());
        assert_eq!(
            verify_with(&text, &pk, &there).err(),
            Some(LicenseError::WrongMachine {
                expected: here.to_string(),
                found: there.to_string(),
            })
        );
    }
}
