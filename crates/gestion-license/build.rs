//! Incrusta la clave pública del desarrollador en el binario.
//!
//! La clave se toma de TEATROGESTION_PUBLIC_KEY_FILE o TEATROCUE_PUBLIC_KEY_FILE, de ~/.config/teatrocue-keys/public.key
//! o, si no existen, de public_key.hex junto a este build.rs.

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=TEATROGESTION_PUBLIC_KEY_FILE");
    println!("cargo:rerun-if-env-changed=TEATROCUE_PUBLIC_KEY_FILE");
    println!("cargo:rerun-if-env-changed=TEATROGESTION_BUILD_UNIX");

    let build_unix = std::env::var("TEATROGESTION_BUILD_UNIX")
        .or_else(|_| std::env::var("TEATROCUE_BUILD_UNIX"))
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
        });
    println!("cargo:rustc-env=TEATROGESTION_BUILD_UNIX_VALUE={build_unix}");

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("public_key.hex");

    let key_candidate = std::env::var("TEATROGESTION_PUBLIC_KEY_FILE")
        .or_else(|_| std::env::var("TEATROCUE_PUBLIC_KEY_FILE"))
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
            let default_path = PathBuf::from(home).join(".config/teatrocue-keys/public.key");
            if default_path.exists() {
                Some(default_path)
            } else {
                None
            }
        })
        // Clave pública versionada en el repo (no es secreta): las compilaciones de CI no
        // tienen la del equipo del desarrollador y saldrían sin clave de verificación.
        .or_else(|| {
            let dir = std::env::var_os("CARGO_MANIFEST_DIR")?;
            Some(PathBuf::from(dir).join("public_key.hex"))
        });

    let hex = match key_candidate {
        Some(path) if path.exists() => {
            println!("cargo:rerun-if-changed={}", path.display());
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let hex: String = text.split_whitespace().collect();
            assert!(
                hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()),
                "la clave pública debe ser 64 caracteres hexadecimales"
            );
            hex.to_ascii_lowercase()
        }
        _ => String::new(),
    };

    std::fs::write(out, hex).expect("escribir la clave pública");
}
