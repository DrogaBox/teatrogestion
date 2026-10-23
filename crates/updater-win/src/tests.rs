use super::*;

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("updater-win-{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn new_payload() -> Payload<'static> {
    Payload { exe: b"EXE-NUEVO-v2", dll: b"DLL-NUEVA", ico: b"ICO" }
}

/// Base con las migraciones dadas (con su suma correcta) como si las hubiera aplicado la app.
fn db_with(path: &Path, versions: &[u32], tamper: Option<u32>) {
    let c = Connection::open(path).unwrap();
    c.execute_batch("CREATE TABLE _schema_migrations (version INTEGER PRIMARY KEY, name TEXT NOT NULL, checksum_sha256 TEXT NOT NULL, applied_at TEXT NOT NULL DEFAULT '');").unwrap();
    for m in EMBEDDED_MIGRATIONS.iter().filter(|m| versions.contains(&m.version)) {
        let sum = if tamper == Some(m.version) { "0".repeat(64) } else { compute_sha256_hex(m.sql) };
        c.execute("INSERT INTO _schema_migrations (version, name, checksum_sha256) VALUES (?1, ?2, ?3)", rusqlite::params![m.version, m.name, sum]).unwrap();
    }
}

#[test]
fn the_backup_name_is_a_utc_timestamp() {
    assert_eq!(stamp(0), "19700101-000000");
    assert_eq!(stamp(1_761_177_600), "20251023-000000");
    assert_eq!(stamp(1_761_186_135), "20251023-022215");
}

#[test]
fn the_install_is_found_in_the_first_folder_that_has_the_program() {
    let a = tmp("find-a");
    let b = tmp("find-b");
    fs::write(b.join(EXE_NAME), b"x").unwrap();
    assert_eq!(find_install(&[a.clone(), b.clone()]), Some(b.clone()));
    assert_eq!(find_install(&[a.clone()]), None, "una carpeta vacía no es una instalación");
    fs::write(a.join(EXE_NAME), b"x").unwrap();
    assert_eq!(find_install(&[a.clone(), b.clone()]), Some(a), "manda el orden de preferencia");
}

#[test]
fn the_backup_copies_the_database_with_its_log_files_and_keeps_only_the_last_five() {
    let data = tmp("backup");
    assert_eq!(backup_data(&data, "20260101-000000").unwrap(), None, "sin base no hay nada que respaldar");
    fs::write(data.join(DB_NAME), b"base").unwrap();
    fs::write(data.join(format!("{DB_NAME}-wal")), b"wal").unwrap();
    for i in 1..=7 {
        let dest = backup_data(&data, &format!("2026010{i}-120000")).unwrap().unwrap();
        assert_eq!(fs::read(dest.join(DB_NAME)).unwrap(), b"base");
        assert_eq!(fs::read(dest.join(format!("{DB_NAME}-wal"))).unwrap(), b"wal");
        assert!(!dest.join(format!("{DB_NAME}-shm")).exists());
    }
    let mut left: Vec<String> = fs::read_dir(data.join("respaldos")).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    left.sort();
    assert_eq!(left.len(), KEEP_BACKUPS);
    assert_eq!(left.first().unwrap(), "actualizacion-20260103-120000", "se tiraron los más viejos");
    assert_eq!(fs::read(data.join(DB_NAME)).unwrap(), b"base", "el original no se toca");
}

#[test]
fn the_database_check_accepts_older_and_current_databases_and_refuses_the_ones_the_new_version_could_not_open() {
    let d = tmp("dbcheck");
    let all: Vec<u32> = EMBEDDED_MIGRATIONS.iter().map(|m| m.version).collect();
    assert_eq!(check_db(&d.join("no-existe.sqlite")).unwrap(), DbCheck::NoDb);

    db_with(&d.join("al-dia.sqlite"), &all, None);
    assert_eq!(check_db(&d.join("al-dia.sqlite")).unwrap(), DbCheck::Compatible { applied: all.len(), pending: 0 });

    db_with(&d.join("vieja.sqlite"), &all[..all.len() - 3], None);
    assert_eq!(check_db(&d.join("vieja.sqlite")).unwrap(), DbCheck::Compatible { applied: all.len() - 3, pending: 3 }, "las migraciones nuevas las aplica la app al abrir");

    db_with(&d.join("editada.sqlite"), &all, Some(9));
    match check_db(&d.join("editada.sqlite")).unwrap() {
        DbCheck::Mismatch { version, name } => assert_eq!((version, name.as_str()), (9, "0009_solisticos_and_real_bop_attendance_matrix")),
        other => panic!("{other:?}"),
    }

    let futura = d.join("futura.sqlite");
    db_with(&futura, &all, None);
    Connection::open(&futura).unwrap().execute("INSERT INTO _schema_migrations (version, name, checksum_sha256) VALUES (99, 'x', 'y')", []).unwrap();
    assert_eq!(check_db(&futura).unwrap(), DbCheck::NewerDb { version: 99 }, "no se baja de versión");

    Connection::open(d.join("sin-tabla.sqlite")).unwrap().execute_batch("CREATE TABLE otra (a)").unwrap();
    assert_eq!(check_db(&d.join("sin-tabla.sqlite")).unwrap(), DbCheck::Compatible { applied: 0, pending: all.len() });
}

#[test]
fn the_update_replaces_the_programs_keeps_the_previous_ones_and_never_touches_the_license_or_other_files() {
    let dir = tmp("apply");
    fs::write(dir.join(EXE_NAME), b"EXE-VIEJO").unwrap();
    fs::write(dir.join(DLL_NAME), b"DLL-VIEJA").unwrap();
    fs::write(dir.join("license.key"), b"MI-LICENCIA").unwrap();
    fs::write(dir.join("marcas.dat"), b"RELOJ").unwrap();
    let names = apply(&dir, &new_payload()).unwrap();
    assert_eq!(names, vec![EXE_NAME, DLL_NAME, ICO_NAME]);
    assert!(verify_installed(&dir, &new_payload()));
    assert_eq!(fs::read(dir.join("TeatroGestion.exe.anterior")).unwrap(), b"EXE-VIEJO", "queda la versión previa por si hace falta volver");
    assert_eq!(fs::read(dir.join("license.key")).unwrap(), b"MI-LICENCIA");
    assert_eq!(fs::read(dir.join("marcas.dat")).unwrap(), b"RELOJ");
    assert!(fs::read_dir(&dir).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".nuevo")), "no quedan temporales");
    // Una segunda actualización reemplaza el `.anterior` sin quejarse.
    let again = Payload { exe: b"EXE-v3", dll: b"DLL", ico: b"" };
    apply(&dir, &again).unwrap();
    assert_eq!(fs::read(dir.join("TeatroGestion.exe.anterior")).unwrap(), b"EXE-NUEVO-v2");
}

#[test]
fn if_something_fails_halfway_everything_goes_back_to_how_it_was() {
    let dir = tmp("rollback");
    fs::write(dir.join(EXE_NAME), b"EXE-VIEJO").unwrap();
    // La DLL «es» una carpeta: poner el archivo nuevo en su lugar falla DESPUÉS de haber cambiado el programa.
    fs::create_dir_all(dir.join(DLL_NAME)).unwrap();
    fs::write(dir.join(DLL_NAME).join("dentro"), b"x").unwrap();
    let err = apply(&dir, &new_payload()).unwrap_err();
    assert!(err.contains("WebView2Loader.dll"), "{err}");
    assert_eq!(fs::read(dir.join(EXE_NAME)).unwrap(), b"EXE-VIEJO", "el programa viejo volvió a su lugar");
    assert!(dir.join(DLL_NAME).join("dentro").exists());
    let leftovers: Vec<String> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".nuevo") || n.ends_with(".anterior")).collect();
    assert!(leftovers.is_empty(), "no quedan restos: {leftovers:?}");
}

#[test]
fn an_empty_package_is_refused_before_touching_anything() {
    let dir = tmp("empty");
    fs::write(dir.join(EXE_NAME), b"EXE-VIEJO").unwrap();
    assert!(apply(&dir, &Payload { exe: b"", dll: b"", ico: b"" }).is_err());
    assert_eq!(fs::read(dir.join(EXE_NAME)).unwrap(), b"EXE-VIEJO");
}

#[test]
fn a_missing_program_is_not_reported_as_open_and_the_license_line_never_fails() {
    let dir = tmp("lock");
    assert!(!is_locked(&dir.join(EXE_NAME)));
    fs::write(dir.join(EXE_NAME), b"x").unwrap();
    assert!(!is_locked(&dir.join(EXE_NAME)));
    assert!(license_summary(&dir).contains("license.key"), "sin licencia lo dice, no falla");
}
