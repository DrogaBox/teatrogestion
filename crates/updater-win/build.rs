//! Incrusta el icono en el .exe y los archivos de la versión nueva de TeatroGestión (el actualizador va en un solo archivo).
//! Los archivos salen de `TG_PAYLOAD_DIR` o de `target/x86_64-pc-windows-gnu/release`; si no están (compilación para
//! pruebas en Linux) el actualizador se compila igual, con el paquete vacío.
//! Usa `windres` de mingw; si no está, el programa se compila igual, solo que sin icono.

use std::path::PathBuf;
use std::process::Command;

fn embed_payload(manifest_dir: &PathBuf) {
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    println!("cargo:rerun-if-env-changed=TG_PAYLOAD_DIR");
    let dir = std::env::var_os("TG_PAYLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("../../target/x86_64-pc-windows-gnu/release"));
    let files = [("EXE", dir.join("gestion-app.exe")), ("DLL", dir.join("WebView2Loader.dll")), ("ICO", manifest_dir.join("../../teatrogestion.ico"))];
    let mut code = String::new();
    for (name, path) in files {
        println!("cargo:rerun-if-changed={}", path.display());
        match path.canonicalize() {
            Ok(abs) => code.push_str(&format!("pub static {name}: &[u8] = include_bytes!({:?});\n", abs.to_string_lossy())),
            Err(_) => code.push_str(&format!("pub static {name}: &[u8] = &[];\n")),
        }
    }
    // Fecha de armado (UTC), para mostrarla al actualizar.
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    code.push_str(&format!("pub const BUILT_UNIX: u64 = {secs};\n"));
    std::fs::write(out.join("payload.rs"), code).expect("no se pudo escribir payload.rs");
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    embed_payload(&manifest_dir);
    let ico = manifest_dir.join("../../teatrogestion.ico");
    println!("cargo:rerun-if-changed={}", ico.display());
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let ico = ico.canonicalize().unwrap_or(ico);
    let rc = out.join("icon.rc");
    let obj = out.join("icon.o");
    let ico_text = ico.to_string_lossy().replace('\\', "/");
    if std::fs::write(&rc, format!("1 ICON \"{ico_text}\"\n")).is_err() {
        println!("cargo:warning=no se pudo escribir icon.rc: el .exe queda sin icono");
        return;
    }
    let windres = std::env::var("WINDRES").unwrap_or_else(|_| "x86_64-w64-mingw32-windres".into());
    match Command::new(&windres).arg("-i").arg(&rc).arg("-o").arg(&obj).status() {
        Ok(s) if s.success() => println!("cargo:rustc-link-arg-bins={}", obj.display()),
        _ => println!("cargo:warning=`{windres}` no está disponible: el .exe queda sin icono"),
    }
}
