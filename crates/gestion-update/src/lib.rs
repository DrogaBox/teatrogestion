//! Búsqueda y descarga de versiones nuevas desde los *releases* de un repositorio de GitHub (privado o público).
//!
//! La comprobación solo lee: pregunta cuál es el último release, compara su versión con la instalada y devuelve las notas
//! de ese release (el registro de cambios) y el archivo que corresponde a este sistema. Lo que se descarga se verifica
//! contra el SHA-256 que informa GitHub para cada archivo antes de entregarlo.

use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const API: &str = "https://api.github.com";
const MAX_DOWNLOAD_BYTES: u64 = 400 * 1024 * 1024;

/// Qué archivo del release corresponde a este sistema.
#[derive(Clone, Copy, Debug)]
pub struct AssetRule {
    /// El nombre del archivo debe terminar así (en minúsculas), p. ej. `.apk`.
    pub suffix: &'static str,
    /// Si no está vacío, el nombre además debe empezar así (en minúsculas).
    pub prefix: &'static str,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateInfo {
    /// Versión nueva sin la `v` inicial, p. ej. `1.2`.
    pub version: String,
    pub name: String,
    /// Notas del release: el registro de cambios.
    pub notes: String,
    pub published_at: String,
    pub asset_id: u64,
    pub asset_name: String,
    pub asset_size: u64,
    /// `sha256:<hex>` informado por GitHub para el archivo, si lo trae.
    pub asset_digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseJson {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    draft: bool,
    prerelease: bool,
    assets: Vec<AssetJson>,
}

#[derive(Debug, Deserialize)]
struct AssetJson {
    id: u64,
    name: String,
    size: u64,
    digest: Option<String>,
}

// ---------------------------------------------------------------------------------------------
// Versiones
// ---------------------------------------------------------------------------------------------

/// `v1.2.3`, `1.2` o `1.2.3-beta` → `[1, 2, 3]`. Las partes que faltan valen 0.
pub fn parse_version(text: &str) -> Option<Vec<u64>> {
    let core = text.trim().trim_start_matches(['v', 'V']);
    let core = core.split(['-', '+']).next()?;
    if core.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for p in core.split('.') {
        parts.push(p.parse::<u64>().ok()?);
    }
    Some(parts)
}

/// ¿`remote` es una versión estrictamente más nueva que `local`? Si alguna no se puede leer, `false`.
pub fn is_newer(remote: &str, local: &str) -> bool {
    let (Some(mut r), Some(mut l)) = (parse_version(remote), parse_version(local)) else {
        return false;
    };
    let n = r.len().max(l.len());
    r.resize(n, 0);
    l.resize(n, 0);
    r > l
}

fn pick_asset<'a>(assets: &'a [AssetJson], rule: &AssetRule) -> Option<&'a AssetJson> {
    assets.iter().find(|a| {
        let n = a.name.to_lowercase();
        n.ends_with(rule.suffix) && (rule.prefix.is_empty() || n.starts_with(rule.prefix))
    })
}

// ---------------------------------------------------------------------------------------------
// Red
// ---------------------------------------------------------------------------------------------

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .redirects(0) // las redirecciones se siguen a mano para no mandar el token a otro servidor
        .build()
}

fn explain(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(401, _) => "El token no es válido o venció.".into(),
        ureq::Error::Status(403, _) => "GitHub rechazó el acceso (token sin permiso o límite de consultas).".into(),
        ureq::Error::Status(404, _) => "No se encontró el repositorio o el token no tiene acceso a él.".into(),
        ureq::Error::Status(c, _) => format!("GitHub respondió con el código {c}."),
        ureq::Error::Transport(t) => format!("No se pudo conectar con GitHub ({t}). Revise la conexión a internet."),
    }
}

fn api_get(agent: &ureq::Agent, url: &str, token: &str, accept: &str) -> Result<ureq::Response, String> {
    agent
        .get(url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", accept)
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("User-Agent", "teatro-updater")
        .call()
        .or_else(|e| match e {
            // Las respuestas 3xx llegan como éxito porque no se siguen solas.
            ureq::Error::Status(c, r) if (300..400).contains(&c) => Ok(r),
            other => Err(explain(other)),
        })
}

/// Comprueba que el token da acceso de lectura al repositorio (`owner/nombre`).
pub fn verify_access(repo: &str, token: &str) -> Result<(), String> {
    api_get(&agent(), &format!("{API}/repos/{repo}"), token, "application/vnd.github+json").map(|_| ())
}

/// Busca el último release publicado. Devuelve `None` si ya se tiene la última versión o si el release no trae el archivo
/// que corresponde a este sistema.
pub fn check(repo: &str, token: &str, current: &str, rule: &AssetRule) -> Result<Option<UpdateInfo>, String> {
    let resp = api_get(&agent(), &format!("{API}/repos/{repo}/releases/latest"), token, "application/vnd.github+json")?;
    let release: ReleaseJson = resp.into_json().map_err(|e| format!("La respuesta de GitHub no se pudo leer: {e}"))?;
    if release.draft || release.prerelease || !is_newer(&release.tag_name, current) {
        return Ok(None);
    }
    let Some(asset) = pick_asset(&release.assets, rule) else {
        return Ok(None);
    };
    Ok(Some(UpdateInfo {
        version: release.tag_name.trim_start_matches(['v', 'V']).to_string(),
        name: release.name.unwrap_or_else(|| release.tag_name.clone()),
        notes: release.body.unwrap_or_default(),
        published_at: release.published_at.unwrap_or_default(),
        asset_id: asset.id,
        asset_name: asset.name.clone(),
        asset_size: asset.size,
        asset_digest: asset.digest.clone(),
    }))
}

/// Descarga el archivo del release a `dest` y comprueba tamaño y SHA-256. Si algo no coincide, borra lo descargado.
pub fn download(repo: &str, token: &str, info: &UpdateInfo, dest: &Path) -> Result<(), String> {
    let agent = agent();
    let first = api_get(&agent, &format!("{API}/repos/{repo}/releases/assets/{}", info.asset_id), token, "application/octet-stream")?;
    // GitHub responde con una redirección a un servidor de archivos: ahí ya no se manda el token.
    let resp = if (300..400).contains(&first.status()) {
        let location = first.header("Location").ok_or("GitHub no indicó dónde está el archivo.")?.to_string();
        if !location.starts_with("https://") {
            return Err("GitHub indicó una dirección de descarga no segura.".into());
        }
        agent.get(&location).set("User-Agent", "teatro-updater").call().map_err(explain)?
    } else {
        first
    };

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("No se pudo crear {}: {e}", parent.display()))?;
    }
    let result = (|| -> Result<(), String> {
        let mut reader = resp.into_reader().take(MAX_DOWNLOAD_BYTES + 1);
        let mut file = fs::File::create(dest).map_err(|e| format!("No se pudo crear {}: {e}", dest.display()))?;
        let mut hasher = Sha256::new();
        let mut total = 0u64;
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let n = reader.read(&mut buf).map_err(|e| format!("Se cortó la descarga: {e}"))?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > MAX_DOWNLOAD_BYTES {
                return Err("El archivo es más grande de lo esperado.".into());
            }
            hasher.update(&buf[..n]);
            file.write_all(&buf[..n]).map_err(|e| format!("No se pudo escribir el archivo: {e}"))?;
        }
        file.sync_all().map_err(|e| e.to_string())?;
        if total != info.asset_size {
            return Err(format!("La descarga quedó incompleta ({total} de {} bytes).", info.asset_size));
        }
        if let Some(expected) = info.asset_digest.as_deref().and_then(|d| d.strip_prefix("sha256:")) {
            let got = hex(&hasher.finalize());
            if !got.eq_ignore_ascii_case(expected) {
                return Err("La descarga no coincide con la huella (SHA-256) publicada: no se instala.".into());
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(dest);
    }
    result
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versiones_se_leen_y_comparan() {
        assert_eq!(parse_version("v1.2.3"), Some(vec![1, 2, 3]));
        assert_eq!(parse_version("1.1"), Some(vec![1, 1]));
        assert_eq!(parse_version("1.2.0-beta.1"), Some(vec![1, 2, 0]));
        assert_eq!(parse_version("abc"), None);
        assert_eq!(parse_version(""), None);
        assert!(is_newer("v1.2", "1.1.0"));
        assert!(is_newer("1.10", "1.9"));
        assert!(is_newer("2.0.0", "1.99.99"));
        assert!(!is_newer("1.1", "1.1.0"));
        assert!(!is_newer("1.0", "1.1"));
        assert!(!is_newer("sin-version", "1.1"));
    }

    #[test]
    fn elige_el_archivo_que_corresponde() {
        let assets = vec![
            AssetJson { id: 1, name: "TeatroGestion-1.2.apk".into(), size: 10, digest: None },
            AssetJson { id: 2, name: "Actualizar_TeatroGestion.exe".into(), size: 20, digest: None },
            AssetJson { id: 3, name: "notas.txt".into(), size: 1, digest: None },
        ];
        let win = AssetRule { suffix: ".exe", prefix: "actualizar_" };
        let apk = AssetRule { suffix: ".apk", prefix: "" };
        assert_eq!(pick_asset(&assets, &win).map(|a| a.id), Some(2));
        assert_eq!(pick_asset(&assets, &apk).map(|a| a.id), Some(1));
        assert!(pick_asset(&assets, &AssetRule { suffix: ".dmg", prefix: "" }).is_none());
    }

    #[test]
    fn huella_en_hexadecimal() {
        assert_eq!(hex(&[0x00, 0xab, 0xff]), "00abff");
    }
}
