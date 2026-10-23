//! Reactivación de emergencia por desafío-respuesta (offline).

use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};

use crate::clock::{self, MarkStore};
use crate::{from_hex, License, MachineId};

/// Alfabeto Base32 de Crockford: sin I, L, O ni U.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const VERSION: u8 = 1;
const DOMAIN: &[u8] = b"TeatroCue/activation/v1\n";
/// Plazo para canjear una clave desde que se firma (7 días).
pub const REDEEM_WINDOW_SECS: u64 = 7 * 86_400;
/// Máximo de días que se puede conceder en una reactivación de emergencia.
pub const MAX_EMERGENCY_DAYS: u32 = 365;
const PAYLOAD_LEN: usize = 1 + 8 + 4 + 4 + 4;
const BLOB_LEN: usize = PAYLOAD_LEN + 64;
const ACTIVATION_PREFIX: &str = "activation:";

// ---------------------------------------------------------------------------------------------
// Base32 de Crockford
// ---------------------------------------------------------------------------------------------

pub fn b32_encode(bytes: &[u8]) -> String {
    let (mut out, mut buf, mut bits) = (String::new(), 0u32, 0u32);
    for &b in bytes {
        buf = (buf << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(ALPHABET[((buf >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((buf << (5 - bits)) & 31) as usize] as char);
    }
    out
}

fn symbol(c: char) -> Option<u8> {
    let c = match c.to_ascii_uppercase() {
        'O' => '0',
        'I' | 'L' => '1',
        o => o,
    };
    ALPHABET.iter().position(|&a| a as char == c).map(|p| p as u8)
}

pub fn b32_decode(text: &str) -> Result<Vec<u8>, String> {
    let (mut out, mut buf, mut bits) = (Vec::new(), 0u32, 0u32);
    for (i, c) in text.chars().filter(|c| !c.is_whitespace() && *c != '-').enumerate() {
        let v = symbol(c).ok_or_else(|| format!("carácter inválido `{c}` en la posición {}", i + 1))?;
        buf = (buf << 5) | v as u32;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    if bits > 0 && (buf & ((1 << bits) - 1)) != 0 {
        return Err("relleno final inválido".into());
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Desafío
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChallengeCode {
    pub machine: MachineId,
    pub epoch: u32,
}

fn epoch_digits(epoch: u32) -> String {
    let mut digits = Vec::new();
    let mut e = epoch;
    loop {
        digits.push(ALPHABET[(e & 31) as usize] as char);
        e >>= 5;
        if e == 0 {
            break;
        }
    }
    while digits.len() < 2 {
        digits.push('0');
    }
    digits.iter().rev().collect()
}

fn check_char(data: &str) -> char {
    let sum: u32 = data.chars().filter_map(symbol).enumerate().map(|(i, v)| (v as u32 + 1) * (2 * i as u32 + 1)).sum();
    ALPHABET[(sum % 32) as usize] as char
}

impl std::fmt::Display for ChallengeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let digits = epoch_digits(self.epoch);
        let data = format!("{}{digits}", self.machine.as_str().replace('-', ""));
        write!(f, "REQ-{}-{digits}{}", self.machine, check_char(&data))
    }
}

impl std::str::FromStr for ChallengeCode {
    type Err = ActivationError;

    fn from_str(s: &str) -> Result<Self, ActivationError> {
        let bad = |m: &str| ActivationError::Malformed(format!("código de solicitud inválido: {m}"));
        let clean: String = s.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect::<String>().to_ascii_uppercase();
        let parts: Vec<&str> = clean.split('-').filter(|p| !p.is_empty()).collect();
        if parts.len() != 6 || parts[0] != "REQ" {
            return Err(bad("debe tener la forma REQ-XXXX-XXXX-XXXX-XXXX-EEC"));
        }
        let tail = parts[5];
        if tail.len() < 3 || tail.len() > 9 {
            return Err(bad("falta la época o el carácter de control"));
        }
        let norm = |g: &str| g.chars().map(|c| match c { 'O' => '0', 'I' | 'L' => '1', o => o }).collect::<String>();
        let machine: MachineId = parts[1..5].iter().map(|g| norm(g)).collect::<Vec<_>>().join("-").parse().map_err(|e: String| bad(&e))?;
        let (digits, check) = tail.split_at(tail.len() - 1);
        let mut epoch: u64 = 0;
        for c in digits.chars() {
            epoch = (epoch << 5) | symbol(c).ok_or_else(|| bad("época con caracteres inválidos"))? as u64;
        }
        let epoch = u32::try_from(epoch).map_err(|_| bad("época fuera de rango"))?;
        let canonical = ChallengeCode { machine, epoch };
        let want = canonical.to_string();
        let got_check = check.chars().next().and_then(|c| symbol(c).map(|v| ALPHABET[v as usize] as char));
        if want.chars().last() != got_check {
            return Err(bad("el carácter de control no coincide (¿se copió mal alguna letra?)"));
        }
        Ok(canonical)
    }
}

// ---------------------------------------------------------------------------------------------
// Respuesta
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivationKey {
    pub machine: MachineId,
    pub epoch: u32,
    pub valid_until: u64,
    pub redeem_by: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ActivationError {
    Malformed(String),
    BadSignature,
    WrongMachine { expected: String, found: String },
    Replay { key_epoch: u32, local_epoch: u32 },
    KeyTooOld { redeem_by: u64 },
    NoExtension { current: u64, granted: u64 },
    Perpetual,
    NoPublicKey,
    NoLicense(String),
    Clock(String),
    Io(String),
}

impl std::fmt::Display for ActivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(m) => write!(f, "{m}"),
            Self::BadSignature => write!(f, "la clave de activación no es auténtica (firma inválida): revise que esté completa y sin cambios"),
            Self::WrongMachine { expected, found } => write!(f, "esta clave es para otra PC (Machine ID {expected}; esta PC es {found})"),
            Self::Replay { key_epoch, local_epoch } => write!(f, "esta clave ya fue usada o es de una solicitud anterior (época {key_epoch}; esta PC está en la {local_epoch}). Pida un código nuevo"),
            Self::KeyTooOld { redeem_by } => write!(f, "la clave caducó el {}: pida un código nuevo", crate::fmt_date(*redeem_by)),
            Self::NoExtension { current, granted } => write!(f, "la clave no extiende la licencia (vence el {} y la clave llega hasta el {})", crate::fmt_date(*current), crate::fmt_date(*granted)),
            Self::Perpetual => write!(f, "la licencia de esta PC no vence: no necesita reactivación"),
            Self::NoPublicKey => write!(f, "esta compilación no incluye clave pública de licencia"),
            Self::NoLicense(m) => write!(f, "no hay una licencia base válida sobre la que extender: {m}"),
            Self::Clock(m) => write!(f, "el reloj de esta PC no es creíble: {m}"),
            Self::Io(m) => write!(f, "no se pudo escribir la licencia: {m}"),
        }
    }
}

impl std::error::Error for ActivationError {}

fn payload(k: &ActivationKey) -> Vec<u8> {
    let mut p = Vec::with_capacity(PAYLOAD_LEN);
    p.push(VERSION);
    p.extend(from_hex(&k.machine.as_str().replace('-', "")).expect("un MachineId siempre es hexadecimal"));
    p.extend((k.epoch).to_be_bytes());
    p.extend(u32::try_from(k.valid_until).unwrap_or(u32::MAX).to_be_bytes());
    p.extend(u32::try_from(k.redeem_by).unwrap_or(u32::MAX).to_be_bytes());
    p
}

fn message(payload: &[u8]) -> Vec<u8> {
    [DOMAIN, payload].concat()
}

pub fn sign_challenge(challenge: &ChallengeCode, days: u32, now: u64, key: &SigningKey) -> (ActivationKey, String) {
    let k = ActivationKey { machine: challenge.machine.clone(), epoch: challenge.epoch, valid_until: now + u64::from(days.clamp(1, MAX_EMERGENCY_DAYS)) * 86_400, redeem_by: now + REDEEM_WINDOW_SECS };
    let p = payload(&k);
    let sig = key.sign(&message(&p));
    let blob = [p, sig.to_bytes().to_vec()].concat();
    let body = b32_encode(&blob);
    let groups: Vec<&str> = body.as_bytes().chunks(8).map(|c| std::str::from_utf8(c).expect("ascii")).collect();
    (k, format!("RESP-{}", groups.join("-")))
}

pub fn extract_response(text: &str) -> Option<String> {
    let up = text.to_ascii_uppercase();
    let at = up.find("RESP-").or_else(|| up.find("RESP"))?;
    Some(up[at..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || c.is_whitespace()).collect::<String>().trim().to_string())
}

pub fn parse_activation(text: &str) -> Result<(ActivationKey, Signature), ActivationError> {
    let t = extract_response(text).ok_or_else(|| ActivationError::Malformed("la clave debe empezar con RESP-".into()))?;
    let body = t.trim_start_matches("RESP");
    let blob = b32_decode(body).map_err(|e| ActivationError::Malformed(format!("clave de activación inválida: {e}")))?;
    if blob.len() != BLOB_LEN {
        return Err(ActivationError::Malformed(format!("clave de activación incompleta o con caracteres de más ({} de {BLOB_LEN} bytes): cópiela entera", blob.len())));
    }
    let (p, sig) = blob.split_at(PAYLOAD_LEN);
    if p[0] != VERSION {
        return Err(ActivationError::Malformed("versión de clave desconocida".into()));
    }
    let hex: String = p[1..9].iter().map(|b| format!("{b:02X}")).collect();
    let machine: MachineId = format!("{}-{}-{}-{}", &hex[0..4], &hex[4..8], &hex[8..12], &hex[12..16]).parse().map_err(ActivationError::Malformed)?;
    let u32_at = |i: usize| u32::from_be_bytes(p[i..i + 4].try_into().expect("4 bytes"));
    let key = ActivationKey { machine, epoch: u32_at(9), valid_until: u64::from(u32_at(13)), redeem_by: u64::from(u32_at(17)) };
    let sig: [u8; 64] = sig.try_into().expect("64 bytes");
    Ok((key, Signature::from_bytes(&sig)))
}

pub fn verify_activation(text: &str, public: &VerifyingKey) -> Result<ActivationKey, ActivationError> {
    let (key, sig) = parse_activation(text)?;
    public.verify_strict(&message(&payload(&key)), &sig).map_err(|_| ActivationError::BadSignature)?;
    Ok(key)
}

pub fn best_attached(license_text: &str, public: &VerifyingKey, local: &MachineId) -> Option<ActivationKey> {
    license_text
        .lines()
        .filter_map(|l| l.trim().strip_prefix(ACTIVATION_PREFIX))
        .filter_map(|t| verify_activation(t, public).ok())
        .filter(|k| &k.machine == local)
        .max_by_key(|k| k.valid_until)
}

pub struct Env<'a> {
    pub license_path: &'a Path,
    pub now: u64,
    pub stores: &'a [Box<dyn MarkStore>],
    pub anchors: &'a [u64],
    pub public: &'a VerifyingKey,
    pub local: &'a MachineId,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Applied {
    pub valid_until: u64,
    pub epoch: u32,
}

pub fn local_epoch(license: &License, stores: &[Box<dyn MarkStore>], attached: Option<&ActivationKey>) -> u32 {
    clock::epoch(license, stores).max(attached.map_or(0, |a| a.epoch.saturating_add(1)))
}

pub fn challenge_for(license: &License, stores: &[Box<dyn MarkStore>], attached: Option<&ActivationKey>) -> ChallengeCode {
    ChallengeCode { machine: license.machine.clone(), epoch: local_epoch(license, stores, attached) }
}

pub fn apply_activation_response(env: &Env, response: &str) -> Result<Applied, ActivationError> {
    let key = verify_activation(response, env.public)?;
    if &key.machine != env.local {
        return Err(ActivationError::WrongMachine { expected: key.machine.to_string(), found: env.local.to_string() });
    }
    let text = std::fs::read_to_string(env.license_path).map_err(|e| ActivationError::NoLicense(format!("{}: {e}", env.license_path.display())))?;
    let license = crate::verify_with(&text, env.public, env.local).map_err(|e| ActivationError::NoLicense(e.to_string()))?;
    let now = clock::check(env.now, &license, env.stores, env.anchors).map_err(|f| ActivationError::Clock(crate::LicenseError::Clock(f).to_string()))?;
    let attached = best_attached(&text, env.public, env.local);
    let local_epoch = local_epoch(&license, env.stores, attached.as_ref());
    if key.epoch < local_epoch {
        return Err(ActivationError::Replay { key_epoch: key.epoch, local_epoch });
    }
    if now > key.redeem_by {
        return Err(ActivationError::KeyTooOld { redeem_by: key.redeem_by });
    }
    if license.is_perpetual() {
        return Err(ActivationError::Perpetual);
    }
    let current = effective_expires(&license, best_attached(&text, env.public, env.local).as_ref());
    if key.valid_until <= current {
        return Err(ActivationError::NoExtension { current, granted: key.valid_until });
    }
    let base: String = text.lines().filter(|l| !l.trim_start().starts_with(ACTIVATION_PREFIX)).map(|l| format!("{l}\n")).collect();
    let new_text = format!("{base}{ACTIVATION_PREFIX} {}\n", response_canonical(response));
    crate::write_atomic(env.license_path, &new_text).map_err(ActivationError::Io)?;
    let epoch = key.epoch.saturating_add(1);
    let high = env.stores.iter().filter_map(|s| s.read()).filter_map(|t| clock::parse_mark(&t, &license).ok()).max().unwrap_or(0).max(now);
    if clock::reset_with_epoch(high, epoch, &license, env.stores) == 0 {
        return Err(ActivationError::Io("no se pudo escribir el registro de uso en ningún sitio".into()));
    }
    Ok(Applied { valid_until: key.valid_until, epoch })
}

fn response_canonical(response: &str) -> String {
    extract_response(response).unwrap_or_default().split_whitespace().collect::<String>()
}

pub fn effective_expires(license: &License, attached: Option<&ActivationKey>) -> u64 {
    if license.is_perpetual() {
        return 0;
    }
    license.expires_unix.max(attached.map_or(0, |a| a.valid_until))
}

#[derive(Clone)]
pub struct Setup {
    pub license_path: std::path::PathBuf,
    pub stores: std::sync::Arc<Vec<Box<dyn MarkStore>>>,
    pub anchors: Vec<u64>,
    pub public: Option<VerifyingKey>,
    pub machine: Result<MachineId, String>,
    pub now: Option<u64>,
}

impl std::fmt::Debug for Setup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Setup").field("license_path", &self.license_path).field("machine", &self.machine).finish_non_exhaustive()
    }
}

impl Setup {
    pub fn installed() -> Self {
        let path = crate::license_path();
        let dir = path.parent().map_or_else(|| std::path::PathBuf::from("."), Path::to_path_buf);
        Self {
            stores: std::sync::Arc::new(clock::default_stores(&dir)),
            anchors: clock::system_anchors(),
            public: crate::embedded_public_key(),
            machine: MachineId::local(),
            license_path: path,
            now: None,
        }
    }

    fn now(&self) -> u64 {
        self.now.unwrap_or_else(crate::now_unix)
    }

    pub fn status(&self) -> crate::Status {
        crate::check_with_machine(&self.license_path, self.now(), &self.stores, &self.anchors, self.public.as_ref(), self.machine.clone())
    }

    fn base(&self) -> Result<(String, License, MachineId, VerifyingKey), ActivationError> {
        let public = self.public.ok_or(ActivationError::NoPublicKey)?;
        let local = self.machine.clone().map_err(ActivationError::NoLicense)?;
        let text = std::fs::read_to_string(&self.license_path).map_err(|e| ActivationError::NoLicense(format!("{}: {e}", self.license_path.display())))?;
        let license = crate::verify_with(&text, &public, &local).map_err(|e| ActivationError::NoLicense(e.to_string()))?;
        Ok((text, license, local, public))
    }

    pub fn challenge(&self) -> Result<ChallengeCode, ActivationError> {
        let (text, license, local, public) = self.base()?;
        Ok(challenge_for(&license, &self.stores, best_attached(&text, &public, &local).as_ref()))
    }

    pub fn apply(&self, response: &str) -> Result<Applied, ActivationError> {
        let (_, _, local, public) = self.base()?;
        apply_activation_response(&Env { license_path: &self.license_path, now: self.now(), stores: &self.stores, anchors: &self.anchors, public: &public, local: &local }, response)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    #[test]
    fn base32_roundtrips_and_tolerates_confusable_characters() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let enc = b32_encode(&data);
        let dec = b32_decode(&enc).unwrap();
        assert_eq!(data, dec);

        // O -> 0, I/L -> 1
        assert_eq!(b32_decode("O1ILO1IL").unwrap(), b32_decode("01110111").unwrap());
    }

    #[test]
    fn challenge_and_activation_cycle() {
        let sk = SigningKey::generate(&mut OsRng);
        let pk = sk.verifying_key();
        let machine = MachineId::from_guid("pc-del-teatro");
        let challenge = ChallengeCode { machine: machine.clone(), epoch: 0 };
        let code_str = challenge.to_string();
        assert!(code_str.starts_with("REQ-"));

        let parsed: ChallengeCode = code_str.parse().unwrap();
        assert_eq!(parsed, challenge);

        let now = 1_800_000_000;
        let (_key, resp) = sign_challenge(&challenge, 30, now, &sk);
        assert!(resp.starts_with("RESP-"));

        let verified = verify_activation(&resp, &pk).unwrap();
        assert_eq!(verified.machine, machine);
        assert_eq!(verified.epoch, 0);
        assert_eq!(verified.valid_until, now + 30 * 86_400);
    }
}
