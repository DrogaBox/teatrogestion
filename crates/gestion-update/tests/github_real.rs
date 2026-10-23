//! Prueba contra GitHub de verdad. No corre sola: `TG_TEST_TOKEN=... TG_TEST_REPO=owner/repo cargo test -p gestion-update -- --ignored`.

use gestion_update::{check, download, verify_access, AssetRule};

fn entorno() -> Option<(String, String)> {
    Some((std::env::var("TG_TEST_REPO").ok()?, std::env::var("TG_TEST_TOKEN").ok()?))
}

#[test]
#[ignore]
fn busca_y_descarga_el_ultimo_release() {
    let Some((repo, token)) = entorno() else { panic!("faltan TG_TEST_REPO y TG_TEST_TOKEN") };
    verify_access(&repo, &token).expect("acceso al repositorio");
    assert!(verify_access(&repo, "token-falso").is_err(), "un token falso debe rechazarse");

    let apk = AssetRule { suffix: ".apk", prefix: "" };
    assert!(check(&repo, &token, "99.0", &apk).unwrap().is_none(), "no hay nada más nuevo que 99.0");
    let info = check(&repo, &token, "1.0", &apk).expect("consulta").expect("debe haber una versión nueva");
    assert!(!info.version.is_empty());
    assert!(!info.notes.is_empty());
    assert!(info.asset_digest.is_some(), "GitHub debe informar la huella del archivo");

    let dest = std::env::temp_dir().join("tg-prueba-descarga.apk");
    download(&repo, &token, &info, &dest).expect("descarga verificada");
    assert_eq!(std::fs::metadata(&dest).unwrap().len(), info.asset_size);
    let _ = std::fs::remove_file(&dest);

    // Con una huella equivocada la descarga se rechaza y no deja archivo.
    let mut mala = info.clone();
    mala.asset_digest = Some("sha256:00".into());
    let dest2 = std::env::temp_dir().join("tg-prueba-mala.apk");
    assert!(download(&repo, &token, &mala, &dest2).is_err());
    assert!(!dest2.exists());
}
