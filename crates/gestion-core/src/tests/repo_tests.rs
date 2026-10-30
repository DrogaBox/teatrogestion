use crate::db::attendance_repo::{
    create_physiotherapy_session, get_monthly_bop_matrix, list_attendance_for_date,
    list_physiotherapy_sessions, record_attendance, update_bop_matrix_cell,
};
use crate::db::soloist_repo::{list_soloist_certifications, save_soloist_certification};
use crate::db::personnel_repo::{
    delete_personnel, import_personnel_batch, list_personnel, search_dancer_quick,
};
use crate::db::schedule_repo::{
    get_or_create_daily_schedule, set_schedule_regime, upsert_rehearsal_block,
};
use crate::db::stock_repo::{
    create_nominal_reservation, execute_voucher_delivery, list_stock_catalog,
    release_nominal_reservation, upsert_stock_item,
};
use crate::db::DatabaseManager;
use crate::domain::{
    ActivityRestriction, ActivityType, AttendanceStatus, CastAssignmentCondition,
    ConfirmationState, CreatePhysioSessionPayload, GenderGroup, LeaveType, PersonnelRole,
    QuotaBucket, RecordAttendancePayload, RoomCode, WorkRegime,
};
use crate::dto::{
    ExecuteVoucherItemPayload, ExecuteVoucherPayload, StockFilterPayload,
    UpsertCastGroupPayload, UpsertCastMemberPayload, UpsertPersonnelPayload,
    UpsertRehearsalBlockPayload, UpsertStockItemPayload,
};

#[test]
fn test_personnel_queries_and_quick_search() {
    let db = DatabaseManager::open_in_memory().expect("open memory db");
    let conn = db.connection();

    // 1. List dancers
    let dancers = list_personnel(conn, Some(PersonnelRole::Dancer), true).expect("list dancers");
    assert!(!dancers.is_empty());
    assert!(dancers.iter().any(|d| (d.last_name == "González" || d.last_name == "Gómez") && d.nickname.as_deref() == Some("Popi")));

    // 2. Search by nickname "Popi"
    let popi = search_dancer_quick(conn, "Popi").expect("search popi");
    assert_eq!(popi.len(), 1);
    assert_eq!(popi[0].first_name, "Sofía");
    assert_eq!(popi[0].nickname.as_deref(), Some("Popi"));
    assert_eq!(popi[0].category_name, "Solista");
    assert_eq!(popi[0].effective_pointe_quota, 18);

    // 3. Search by nickname "Brenda"
    let brenda = search_dancer_quick(conn, "Brenda").expect("search brenda");
    assert_eq!(brenda.len(), 1);
    assert!(brenda[0].first_name.starts_with("Brenda"));
    assert_eq!(brenda[0].effective_pointe_quota, 24);
}

#[test]
fn test_stock_catalog_reservation_and_voucher_delivery() {
    let mut db = DatabaseManager::open_in_memory().expect("open memory db");
    let conn = db.connection_mut();

    // 1. List catalog
    let catalog = list_stock_catalog(conn, StockFilterPayload::default()).expect("list catalog");
    assert!(!catalog.is_empty());

    // Encontrar el item Gaynor Sculpted 7.5W4PDH
    let gm_item = catalog
        .iter()
        .find(|i| i.raw_factory_code.as_deref() == Some("SC - 7.5W4PDH"))
        .expect("must find Gaynor Sculpted");
    assert_eq!(gm_item.physical_qty, 10);
    // El seed 0002_seed_catalogs_and_staff ya inserta RES_INIT_POPI_01 con 2 unidades reservadas
    assert_eq!(gm_item.reserved_qty, 2);
    assert_eq!(gm_item.free_available_qty, 8);

    // 2. Ejecutar entrega de valera consumiendo la reserva semilla RES_INIT_POPI_01
    let delivery_payload = ExecuteVoucherPayload {
        personnel_id: "PER_DANCER_POPI".to_string(),
        season_year: Some(2026),
        delivered_by_inspector_id: "PER_INSPECTOR_MAIN".to_string(),
        device_id: "GALAXY-A53".to_string(),
        signature_svg_strokes: Some("<svg>signature</svg>".to_string()),
        is_quota_exception: false,
        exception_reason: None,
        notes: Some("Entrega de 2 pares Gaynor con reserva".to_string()),
        items: vec![ExecuteVoucherItemPayload {
            stock_item_id: gm_item.id.clone(),
            reservation_id: Some("RES_INIT_POPI_01".to_string()),
            quantity: 2,
            quota_bucket: QuotaBucket::Pointe,
        }],
    };

    let receipt = execute_voucher_delivery(conn, delivery_payload).expect("execute voucher delivery");
    assert!(receipt.voucher_number.starts_with("VAL-2026-"));
    assert_eq!(receipt.total_items_count, 1);

    // Verificar que el stock físico bajó a 8 y el reservado a 0
    let post_catalog = list_stock_catalog(conn, StockFilterPayload::default()).expect("list catalog");
    let post_gm = post_catalog.iter().find(|i| i.id == gm_item.id).unwrap();
    assert_eq!(post_gm.physical_qty, 8);
    assert_eq!(post_gm.reserved_qty, 0);
    assert_eq!(post_gm.free_available_qty, 8);

    // 3. Crear una nueva reserva nominativa para "Popi"
    let res = create_nominal_reservation(
        conn,
        &gm_item.id,
        "PER_DANCER_POPI",
        "Popi",
        2,
        Some("Nueva reserva para ensayo general"),
    )
    .expect("create reservation");

    assert_eq!(res.quantity, 2);

    let updated_catalog = list_stock_catalog(conn, StockFilterPayload::default()).expect("list catalog");
    let updated_gm = updated_catalog.iter().find(|i| i.id == gm_item.id).unwrap();
    assert_eq!(updated_gm.physical_qty, 8);
    assert_eq!(updated_gm.reserved_qty, 2);
    assert_eq!(updated_gm.free_available_qty, 6);

    // 4. Test liberar reserva
    release_nominal_reservation(conn, &res.id).expect("release reservation");

    let post_rel_catalog = list_stock_catalog(conn, StockFilterPayload::default()).expect("list catalog");
    let post_rel_gm = post_rel_catalog.iter().find(|i| i.id == gm_item.id).unwrap();
    assert_eq!(post_rel_gm.physical_qty, 8);
    assert_eq!(post_rel_gm.reserved_qty, 0);
    assert_eq!(post_rel_gm.free_available_qty, 8);
}

#[test]
fn test_daily_schedule_workflow_and_conflict_engine() {
    let mut db = DatabaseManager::open_in_memory().expect("open memory db");
    let conn = db.connection_mut();

    // 1. Obtener o crear schedule del día (fecha no pre-sembrada)
    let sched_dto = get_or_create_daily_schedule(conn, "2026-11-05").expect("create schedule");
    assert_eq!(sched_dto.schedule.schedule_date, "2026-11-05");
    assert_eq!(sched_dto.schedule.regime, WorkRegime::Ordinary);
    assert_eq!(sched_dto.schedule.morning_classes.len(), 2);

    // 2. Insertar bloque que excede las 13:45h en régimen Ordinary -> debe detectar conflicto ShiftLimitExceeded
    let block_payload = UpsertRehearsalBlockPayload {
        sound_tech_id: None,
        id: None,
        schedule_id: sched_dto.schedule.id.clone(),
        room: RoomCode::Sala1,
        start_time: "10:30".to_string(),
        end_time: "14:15".to_string(), // Excede 13:45!
        activity_type: ActivityType::StudioRehearsal,
        work_id: Some("WORK_LAGO_CISNES".to_string()),
        passage_id: Some("PAS_PRINCESAS_BUFON".to_string()),
        custom_passage_label: None,
        confirmation_state: ConfirmationState::Confirmed,
        sound_requirement: "PISTA TeatroCue #04".to_string(),
        teatrocue_cue_ref: Some("CUE-04".to_string()),
        master_id: Some("PER_MASTER_ANANIEV".to_string()),
        pianist_id: None,
        allow_shared_room: false,
        notes: Some("Ensayo con sonido".to_string()),
        cast_groups: vec![UpsertCastGroupPayload {
            id: None,
            group_order: 1,
            group_label: Some("1er Reparto".to_string()),
            is_to_confirm: false,
            members: vec![UpsertCastMemberPayload {
                personnel_id: "PER_DANCER_POPI".to_string(),
                character_role: Some("Princesa".to_string()),
                partner_personnel_id: None,
                condition: CastAssignmentCondition::Titular,
            }],
        }],
    };

    let audited = upsert_rehearsal_block(conn, block_payload).expect("upsert rehearsal block");
    assert_eq!(audited.schedule.rehearsal_blocks.len(), 1);
    assert!(!audited.conflicts.is_empty(), "Debe detectar ShiftLimitExceeded");

    // 3. Conmutar a PreFunction -> la hora límite se extiende a 14:45 -> conflicto desaparece
    let pre_func_audited = set_schedule_regime(conn, &sched_dto.schedule.id, WorkRegime::PreFunction)
        .expect("set prefunction regime");
    assert_eq!(pre_func_audited.schedule.regime, WorkRegime::PreFunction);
    assert!(pre_func_audited.conflicts.is_empty(), "En PreFunction hasta 14:45 no hay conflicto");
}

#[test]
fn test_attendance_medical_leaves_and_physiotherapy() {
    let mut db = DatabaseManager::open_in_memory().expect("open memory db");
    let conn = db.connection_mut();

    let date = "2026-10-19";

    // 1. Listar asistencia inicial para la fecha pre-sembrada
    let initial_list = list_attendance_for_date(conn, date).expect("list attendance");
    assert!(!initial_list.is_empty(), "Debe listar todo el personal activo");

    // 2. Marcar Popi como "Está" (Presente a las 08:45)
    let popi_log = record_attendance(
        conn,
        RecordAttendancePayload {
            schedule_date: date.to_string(),
            personnel_id: "PER_DANCER_POPI".to_string(),
            attendance_status: AttendanceStatus::Present,
            actual_checkin_time: Some("08:45".to_string()),
            minutes_late: None,
            remarks: None,
            certificate_file_name: None,
            certificate_data: None,
            medical_diagnosis: None,
            doctor_details: None,
            leave_start_date: None,
            leave_end_date: None,
            leave_type: None,
            physio_notes: None,
            update_permanent_physio_tracking: None,
        },
    )
    .expect("record popi attendance");
    assert_eq!(popi_log.attendance_status, AttendanceStatus::Present);
    assert_eq!(popi_log.minutes_late, 0);
    assert!(popi_log.is_in_class);

    // 3. Marcar Brenda como "Llegó tarde" (09:05 h -> 20 min tarde)
    let brenda_log = record_attendance(
        conn,
        RecordAttendancePayload {
            schedule_date: date.to_string(),
            personnel_id: "PER_DANCER_BRENDA".to_string(),
            attendance_status: AttendanceStatus::Late,
            actual_checkin_time: Some("09:05".to_string()),
            minutes_late: Some(20),
            remarks: Some("Demora de transporte".to_string()),
            certificate_file_name: None,
            certificate_data: None,
            medical_diagnosis: None,
            doctor_details: None,
            leave_start_date: None,
            leave_end_date: None,
            leave_type: None,
            physio_notes: None,
            update_permanent_physio_tracking: None,
        },
    )
    .expect("record brenda late");
    assert_eq!(brenda_log.attendance_status, AttendanceStatus::Late);
    assert_eq!(brenda_log.minutes_late, 20);

    // 4. Marcar Álvarez con "Licencia médica" y certificado adjunto
    let alvarez_log = record_attendance(
        conn,
        RecordAttendancePayload {
            schedule_date: date.to_string(),
            personnel_id: "PER_DANCER_ALVAREZ".to_string(),
            attendance_status: AttendanceStatus::MedicalLeave,
            actual_checkin_time: None,
            minutes_late: None,
            remarks: Some("Carpeta médica por 48 horas".to_string()),
            certificate_file_name: Some("certificado_alvarez_esguince.pdf".to_string()),
            certificate_data: Some("data:application/pdf;base64,JVBERi0xLjQK...".to_string()),
            medical_diagnosis: Some("Esguince de tobillo derecho grado 1".to_string()),
            doctor_details: Some("Dr. Ramos - MP 14820".to_string()),
            leave_start_date: Some("2026-10-19".to_string()),
            leave_end_date: Some("2026-10-21".to_string()),
            leave_type: None,
            physio_notes: None,
            update_permanent_physio_tracking: None,
        },
    )
    .expect("record alvarez medical leave");
    assert_eq!(alvarez_log.attendance_status, AttendanceStatus::MedicalLeave);
    assert!(!alvarez_log.is_in_class);
    assert_eq!(alvarez_log.certificate_file_name.as_deref(), Some("certificado_alvarez_esguince.pdf"));

    // 5. Registrar atención en la Oficina de Fisioterapia para Morchio C con restricción NoJumps
    let physio_session = create_physiotherapy_session(
        conn,
        CreatePhysioSessionPayload {
            personnel_id: "PER_DANCER_MORCHIO_C".to_string(),
            session_date: date.to_string(),
            session_time: Some("09:30".to_string()),
            doctor_name: Some("Dra. Kinesióloga Ballet".to_string()),
            diagnosis: Some("Tendinitis patelar izquierda".to_string()),
            treatment_applied: "Punción seca, ultrasonido y elongación excéntrica".to_string(),
            evolution_notes: Some("Presenta dolor a la flexión de 90°. Evitar saltos de impacto.".to_string()),
            activity_restriction: ActivityRestriction::NoJumps,
            attachment_name: None,
            attachment_data: None,
            sync_attendance_status: true,
        },
    )
    .expect("create physio session");

    assert_eq!(physio_session.activity_restriction, ActivityRestriction::NoJumps);

    // Verificar que la asistencia de Morchio C se sincronizó automáticamente como "Está en la fisioterapia"
    let list_after = list_attendance_for_date(conn, date).expect("list after");
    let morchio_att = list_after.iter().find(|i| i.personnel_id == "PER_DANCER_MORCHIO_C").unwrap();
    assert_eq!(morchio_att.attendance_status, AttendanceStatus::InPhysiotherapy);
    assert!(!morchio_att.is_in_class);
    assert!(morchio_att.current_physio_diagnosis.as_deref().unwrap().contains("Tendinitis"));

    // Verificar listado de sesiones de fisioterapia
    let all_sessions = list_physiotherapy_sessions(conn, Some(date), None).expect("list physio sessions");
    assert_eq!(all_sessions.len(), 1);
    assert_eq!(all_sessions[0].personnel_id, "PER_DANCER_MORCHIO_C");
}

#[test]
fn test_personnel_import_export_retirement_and_medical_date_ranges() {
    let mut db = DatabaseManager::open_in_memory().expect("in memory db");

    // 1. Simular importación por lote desde planilla Excel con diversos roles del sector
    let batch = vec![
        UpsertPersonnelPayload {
            id: None,
            legajo: Some("BOC-901".to_string()),
            first_name: "Carina".to_string(),
            last_name: "Lion".to_string(),
            nickname: Some("Inspectora".to_string()),
            role: Some(PersonnelRole::Inspector),
            gender_group: Some(GenderGroup::Female),
            category_id: Some("CAT_STAFF_DOCENTE".to_string()),
            default_pointe_raw_code: None,
            default_flat_size: None,
            is_active: Some(true),
            is_retired: Some(false),
            retirement_notes: None,
            specific_position: Some("Supervisora General e Inspección de Escenario".to_string()),
            notes: Some("Columna indispensable del Ballet Oficial".to_string()),
        },
        UpsertPersonnelPayload {
            id: None,
            legajo: Some("BOC-902".to_string()),
            first_name: "Guillermo".to_string(),
            last_name: "Romero".to_string(),
            nickname: Some("Pianista".to_string()),
            role: Some(PersonnelRole::Pianist),
            gender_group: Some(GenderGroup::Male),
            category_id: Some("CAT_STAFF_DOCENTE".to_string()),
            default_pointe_raw_code: None,
            default_flat_size: None,
            is_active: Some(true),
            is_retired: Some(false),
            retirement_notes: None,
            specific_position: Some("Mtro. Pianista Acompañante".to_string()),
            notes: None,
        },
        UpsertPersonnelPayload {
            id: None,
            legajo: Some("BOC-903".to_string()),
            first_name: "Beatriz".to_string(),
            last_name: "Vázquez".to_string(),
            nickname: Some("Bea".to_string()),
            role: Some(PersonnelRole::Dancer),
            gender_group: Some(GenderGroup::Female),
            category_id: Some("CAT_CUERPO_BAILE".to_string()),
            default_pointe_raw_code: None,
            default_flat_size: None,
            is_active: Some(true),
            is_retired: Some(true), // EN ESTADO JUBILATORIO (NO CONCURRE)
            retirement_notes: Some("Expediente previsional Caja de Jubilaciones de Córdoba - Exención de concurrencia".to_string()),
            specific_position: Some("Bailarina Estable (Estado Jubilatorio)".to_string()),
            notes: Some("En trámite jubilatorio, no concurre a la marcada".to_string()),
        },
    ];

    let import_res = import_personnel_batch(db.connection_mut(), batch).expect("import batch");
    assert_eq!(import_res.inserted_count, 3);
    assert_eq!(import_res.errors.len(), 0);

    // 2. Verificar que Carina Lion y el personal jubilado están en la base de datos
    let all = list_personnel(db.connection(), None, true).expect("list all");
    let carina = all.iter().find(|p| p.last_name == "Lion").expect("Carina Lion");
    assert_eq!(carina.role, PersonnelRole::Inspector);
    assert_eq!(carina.specific_position.as_deref(), Some("Supervisora General e Inspección de Escenario"));

    let beatriz = all.iter().find(|p| p.last_name == "Vázquez").expect("Beatriz Vázquez");
    assert!(beatriz.is_retired);

    // 3. Registrar una licencia médica con rango [desde: 2026-10-19, hasta: 2026-10-23]
    let date_target = "2026-10-20";
    let _ = record_attendance(
        db.connection(),
        RecordAttendancePayload {
            schedule_date: "2026-10-19".to_string(),
            personnel_id: "PER_DANCER_POPI".to_string(),
            attendance_status: AttendanceStatus::MedicalLeave,
            actual_checkin_time: None,
            minutes_late: None,
            remarks: Some("Reposo por desgarro".to_string()),
            certificate_file_name: Some("certificado_popi_desgarro.jpg".to_string()),
            certificate_data: Some("data:image/jpeg;base64,mockphoto...".to_string()),
            medical_diagnosis: Some("Desgarro sóleo derecho".to_string()),
            doctor_details: Some("Dr. Fisiatra - MP 9832".to_string()),
            leave_start_date: Some("2026-10-19".to_string()),
            leave_end_date: Some("2026-10-23".to_string()),
            leave_type: None, // Sin tipo explícito: la base aplica ENFERMEDAD por defecto
            physio_notes: None,
            update_permanent_physio_tracking: None,
        },
    ).expect("record medical leave range");

    // 4. Consultar asistencia para el día 2026-10-20 (está dentro del rango de la licencia)
    let att_list = list_attendance_for_date(db.connection(), date_target).expect("list att date_target");
    let popi_att = att_list.iter().find(|a| a.personnel_id == "PER_DANCER_POPI").expect("popi");
    assert_eq!(popi_att.attendance_status, AttendanceStatus::MedicalLeave);
    assert_eq!(popi_att.certificate_file_name.as_deref(), Some("certificado_popi_desgarro.jpg"));
    assert_eq!(popi_att.leave_start_date.as_deref(), Some("2026-10-19"));
    assert_eq!(popi_att.leave_end_date.as_deref(), Some("2026-10-23"));
    assert_eq!(popi_att.leave_type, Some(LeaveType::Enfermedad));

    // 5. Verificar que el personal jubilatorio aparece como RETIRED_EXEMPT automáticamente
    let beatriz_att = att_list.iter().find(|a| a.dancer_name.contains("Vázquez")).expect("beatriz att");
    assert_eq!(beatriz_att.attendance_status, AttendanceStatus::RetiredExempt);
    assert!(!beatriz_att.is_in_class);
    assert!(beatriz_att.is_retired);
}

#[test]
fn test_delete_personnel_daily_records_and_voucher_guard() {
    let mut db = DatabaseManager::open_in_memory().expect("in memory db");

    // 1. Importar personal temporario con registro de asistencia del día
    let batch = vec![UpsertPersonnelPayload {
        id: None,
        legajo: Some("BOC-950".to_string()),
        first_name: "Temporaria".to_string(),
        last_name: "Eliminable".to_string(),
        nickname: None,
        role: Some(PersonnelRole::Dancer),
        gender_group: Some(GenderGroup::Female),
        category_id: Some("CAT_CUERPO_BAILE".to_string()),
        default_pointe_raw_code: None,
        default_flat_size: None,
        is_active: Some(true),
        is_retired: Some(false),
        retirement_notes: None,
        specific_position: None,
        notes: None,
    }];
    let import_res = import_personnel_batch(db.connection_mut(), batch).expect("import batch");
    assert_eq!(import_res.inserted_count, 1);

    let temp_id = list_personnel(db.connection(), None, true)
        .expect("list all")
        .iter()
        .find(|p| p.last_name == "Eliminable")
        .expect("imported person")
        .id
        .clone();

    record_attendance(
        db.connection(),
        RecordAttendancePayload {
            schedule_date: "2026-10-20".to_string(),
            personnel_id: temp_id.clone(),
            attendance_status: AttendanceStatus::Present,
            actual_checkin_time: Some("08:45".to_string()),
            minutes_late: None,
            remarks: None,
            certificate_file_name: None,
            certificate_data: None,
            medical_diagnosis: None,
            doctor_details: None,
            leave_start_date: None,
            leave_end_date: None,
            leave_type: None,
            physio_notes: None,
            update_permanent_physio_tracking: None,
        },
    )
    .expect("record temp attendance");

    // 2. Dar de baja del padrón: borra persona y sus registros diarios
    delete_personnel(db.connection_mut(), &temp_id).expect("delete personnel");
    let remaining: i64 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM attendance_logs WHERE personnel_id = ?1",
            [&temp_id],
            |r| r.get(0),
        )
        .expect("count attendance logs");
    assert_eq!(remaining, 0);
    assert!(!list_personnel(db.connection(), None, false)
        .expect("list after delete")
        .iter()
        .any(|p| p.id == temp_id));

    // 3. Persona con valera entregada no se elimina: queda inactiva (baja de padrón)
    let catalog = list_stock_catalog(db.connection(), StockFilterPayload::default()).expect("list catalog");
    let gm_item = catalog
        .iter()
        .find(|i| i.raw_factory_code.as_deref() == Some("SC - 7.5W4PDH"))
        .expect("must find Gaynor Sculpted");
    execute_voucher_delivery(
        db.connection_mut(),
        ExecuteVoucherPayload {
            personnel_id: "PER_DANCER_POPI".to_string(),
            season_year: Some(2026),
            delivered_by_inspector_id: "PER_INSPECTOR_MAIN".to_string(),
            device_id: "GALAXY-A53".to_string(),
            signature_svg_strokes: None,
            is_quota_exception: false,
            exception_reason: None,
            notes: Some("Entrega con reserva semilla".to_string()),
            items: vec![ExecuteVoucherItemPayload {
                stock_item_id: gm_item.id.clone(),
                reservation_id: Some("RES_INIT_POPI_01".to_string()),
                quantity: 2,
                quota_bucket: QuotaBucket::Pointe,
            }],
        },
    )
    .expect("execute voucher delivery");

    delete_personnel(db.connection_mut(), "PER_DANCER_POPI").expect("delete voucher holder");
    let popi = list_personnel(db.connection(), None, false)
        .expect("list after guarded delete")
        .into_iter()
        .find(|p| p.id == "PER_DANCER_POPI")
        .expect("popi must survive as inactive");
    assert!(!popi.is_active);

    // 4. Persona solo con reserva de stock (sin valera): también queda de baja
    delete_personnel(db.connection_mut(), "PER_DANCER_BRENDA").expect("delete with reservations");
    let brenda = list_personnel(db.connection(), None, false)
        .expect("list after reservation guard")
        .into_iter()
        .find(|p| p.id == "PER_DANCER_BRENDA")
        .expect("brenda must survive as inactive");
    assert!(!brenda.is_active);
}

#[test]
fn test_expanded_inventory_and_dynamic_stock_creation() {
    let mut db = DatabaseManager::open_in_memory().expect("in memory db");

    // 1. Verificar catálogo ampliado de migración 0006 (calzas, suspensores, cancán, medias, jazz, carácter)
    let calzas = list_stock_catalog(
        db.connection(),
        StockFilterPayload {
            subcategory_str: Some("CALZA_MASCULINA".to_string()),
            ..Default::default()
        },
    ).expect("list calzas");
    assert!(calzas.len() >= 6, "Deben existir calzas de varón en blanco, negro y gris");

    let jazz_shoes = list_stock_catalog(
        db.connection(),
        StockFilterPayload {
            subcategory_str: Some("ZAPATOS_JAZZ".to_string()),
            ..Default::default()
        },
    ).expect("list jazz");
    assert!(jazz_shoes.len() >= 4, "Deben existir zapatos de jazz en stock inicial");

    let caracter = list_stock_catalog(
        db.connection(),
        StockFilterPayload {
            subcategory_str: Some("ZAPATOS_CARACTER".to_string()),
            ..Default::default()
        },
    ).expect("list caracter");
    assert!(caracter.len() >= 4, "Deben existir zapatos de carácter taco cubano/bajo");

    // 2. Dar de alta dinámicamente un nuevo ítem de inventario (ej. Calza Especial So Danca)
    let new_item = upsert_stock_item(
        db.connection_mut(),
        UpsertStockItemPayload {
            id: None,
            sku_code: None,
            family: "APPAREL".to_string(),
            subcategory: "CALZA_MASCULINA".to_string(),
            quota_bucket: QuotaBucket::Tights,
            brand: "So Danca".to_string(),
            model_name: "Pro Men Tight Especial".to_string(),
            raw_factory_code: Some("SD-TIGHT-99".to_string()),
            color: "Negro".to_string(),
            material: "ALGODON_LYCRA".to_string(),
            size_system: "ALPHA".to_string(),
            size_value: "XL".to_string(),
            width_value: None,
            physical_qty: 14,
            min_alert_qty: 3,
            storage_bin_location: Some("Gaveta Especial Varones 02".to_string()),
        },
    ).expect("upsert new dynamic stock item");

    assert_eq!(new_item.brand, "So Danca");
    assert_eq!(new_item.physical_qty, 14);
    assert_eq!(new_item.free_available_qty, 14);
    assert_eq!(new_item.storage_bin_location.as_deref(), Some("Gaveta Especial Varones 02"));
    assert!(!new_item.is_low_stock);

    // 3. Ajustar el stock físico del ítem creado a 2 unidades (debe disparar alerta de bajo stock ya que min_alert_qty = 3)
    let updated = upsert_stock_item(
        db.connection_mut(),
        UpsertStockItemPayload {
            id: Some(new_item.id.clone()),
            sku_code: Some(new_item.sku_code.clone()),
            family: new_item.family,
            subcategory: new_item.subcategory,
            quota_bucket: QuotaBucket::Tights,
            brand: new_item.brand,
            model_name: new_item.model_name,
            raw_factory_code: new_item.raw_factory_code,
            color: new_item.color,
            material: new_item.material,
            size_system: new_item.size_system,
            size_value: new_item.size_value,
            width_value: new_item.width_value,
            physical_qty: 2, // Ajuste a 2
            min_alert_qty: 3,
            storage_bin_location: new_item.storage_bin_location,
        },
    ).expect("update stock item");

    assert_eq!(updated.physical_qty, 2);
    assert_eq!(updated.free_available_qty, 2);
    assert!(updated.is_low_stock, "Con 2 unidades y min_alert_qty=3 debe marcar is_low_stock = true");
}

#[test]
fn test_export_and_import_sync_bundle_p2p_lan() {
    use crate::db::sync_repo::{export_sync_bundle, import_sync_bundle};
    use crate::db::schedule_repo::upsert_morning_class;
    use crate::dto::UpsertMorningClassPayload;

    let mut db_mobile = DatabaseManager::open_in_memory().expect("open mobile db");
    let mut db_pc = DatabaseManager::open_in_memory().expect("open pc db");

    let test_date = "2026-10-21";

    // 1. En el móvil (Samsung Galaxy A53), la inspectora registra asistencia y licencia médica
    record_attendance(
        db_mobile.connection_mut(),
        RecordAttendancePayload {
            schedule_date: test_date.to_string(),
            personnel_id: "PER_DANCER_POPI".to_string(),
            attendance_status: AttendanceStatus::Present,
            actual_checkin_time: Some("08:42".to_string()),
            minutes_late: None,
            remarks: Some("En sala 1 lista para la barra".to_string()),
            certificate_file_name: None,
            certificate_data: None,
            medical_diagnosis: None,
            doctor_details: None,
            leave_start_date: None,
            leave_end_date: None,
            leave_type: None,
            physio_notes: None,
            update_permanent_physio_tracking: None,
        },
    ).expect("record attendance on mobile");

    record_attendance(
        db_mobile.connection_mut(),
        RecordAttendancePayload {
            schedule_date: test_date.to_string(),
            personnel_id: "PER_DANCER_ALVAREZ".to_string(),
            attendance_status: AttendanceStatus::MedicalLeave,
            actual_checkin_time: None,
            minutes_late: None,
            remarks: Some("Reposo por 72 hs".to_string()),
            certificate_file_name: Some("alvarez_certificado.pdf".to_string()),
            certificate_data: Some("data:application/pdf;base64,TEST".to_string()),
            medical_diagnosis: Some("Distensión muscular sóleo".to_string()),
            doctor_details: Some("Dr. Especialista MP 9912".to_string()),
            leave_start_date: Some("2026-10-21".to_string()),
            leave_end_date: Some("2026-10-24".to_string()),
            leave_type: Some(LeaveType::Estudio),
            physio_notes: None,
            update_permanent_physio_tracking: None,
        },
    ).expect("record medical leave on mobile");

    // 2. Exportar bundle desde el móvil (Galaxy A53)
    let bundle_mobile = export_sync_bundle(
        db_mobile.connection(),
        "Samsung Galaxy A53 (Depósito / Sala)",
        "node_a53_01",
    ).expect("export mobile bundle");

    assert!(!bundle_mobile.attendance_logs.is_empty());
    assert_eq!(bundle_mobile.source_device_name, "Samsung Galaxy A53 (Depósito / Sala)");

    // 3. Importar bundle en la PC de oficina (Windows 11 / Linux)
    let report_pc = import_sync_bundle(
        db_pc.connection_mut(),
        &bundle_mobile,
    ).expect("import bundle on pc");

    assert!(report_pc.success);
    assert!(report_pc.records_applied > 0);

    // 4. Verificar que en la PC de oficina ahora existen los registros tomados en el celular
    let pc_attendance = list_attendance_for_date(db_pc.connection(), test_date).expect("list attendance on pc");
    let popi_on_pc = pc_attendance.iter().find(|i| i.personnel_id == "PER_DANCER_POPI").unwrap();
    assert_eq!(popi_on_pc.attendance_status, AttendanceStatus::Present);
    assert_eq!(popi_on_pc.actual_checkin_time.as_deref(), Some("08:42"));

    let alvarez_on_pc = pc_attendance.iter().find(|i| i.personnel_id == "PER_DANCER_ALVAREZ").unwrap();
    assert_eq!(alvarez_on_pc.attendance_status, AttendanceStatus::MedicalLeave);
    assert_eq!(alvarez_on_pc.certificate_file_name.as_deref(), Some("alvarez_certificado.pdf"));
    assert_eq!(alvarez_on_pc.medical_diagnosis.as_deref(), Some("Distensión muscular sóleo"));

    // 5. En la PC, la inspectora añade un ítem nuevo al inventario
    let pc_item = upsert_stock_item(
        db_pc.connection_mut(),
        UpsertStockItemPayload {
            id: None,
            sku_code: None,
            family: "INDUMENTARIA".to_string(),
            subcategory: "SUSPENSOR_MASCULINO".to_string(),
            quota_bucket: QuotaBucket::Supporter,
            brand: "Bloch".to_string(),
            model_name: "Dance Belt Pro Nude".to_string(),
            raw_factory_code: Some("BLOCH-DB-01".to_string()),
            color: "Piel / Nude".to_string(),
            material: "ALGODON_LYCRA".to_string(),
            size_system: "ALPHA".to_string(),
            size_value: "M".to_string(),
            width_value: None,
            physical_qty: 25,
            min_alert_qty: 5,
            storage_bin_location: Some("Gaveta Central 04".to_string()),
        },
    ).expect("create item on pc");

    // 6. Sincronizar de vuelta de la PC al celular móvil
    let bundle_pc = export_sync_bundle(
        db_pc.connection(),
        "Notebook Oficina Windows 11",
        "node_pc_01",
    ).expect("export pc bundle");

    let report_mobile = import_sync_bundle(
        db_mobile.connection_mut(),
        &bundle_pc,
    ).expect("import on mobile");

    assert!(report_mobile.success);

    // 7. Verificar que el celular móvil ahora tiene el ítem creado en la PC
    let mobile_catalog = list_stock_catalog(
        db_mobile.connection(),
        crate::dto::StockFilterPayload {
            query: Some("Dance Belt Pro".to_string()),
            subcategory_str: None,
            brand_str: None,
            subcategory: None,
            brand: None,
            color: None,
            size_value: None,
            width_value: None,
            gm_line: None,
            prop_dye_state: None,
            low_stock_only: false,
        },
    ).expect("list catalog on mobile");

    assert_eq!(mobile_catalog.len(), 1);
    assert_eq!(mobile_catalog[0].id, pc_item.id);
    assert_eq!(mobile_catalog[0].physical_qty, 25);
    assert_eq!(mobile_catalog[0].storage_bin_location.as_deref(), Some("Gaveta Central 04"));
}

#[test]
fn test_sync_merges_schedules_created_on_both_devices_for_the_same_date() {
    use crate::db::sync_repo::{export_sync_bundle, import_sync_bundle};

    let mut db_mobile = DatabaseManager::open_in_memory().expect("open mobile db");
    let mut db_pc = DatabaseManager::open_in_memory().expect("open pc db");
    let date = "2026-10-30";
    let mark = |db: &mut DatabaseManager, person: &str, time: &str| {
        record_attendance(
            db.connection_mut(),
            RecordAttendancePayload {
                schedule_date: date.to_string(),
                personnel_id: person.to_string(),
                attendance_status: AttendanceStatus::Present,
                actual_checkin_time: Some(time.to_string()),
                minutes_late: None,
                remarks: None,
                certificate_file_name: None,
                certificate_data: None,
                medical_diagnosis: None,
                doctor_details: None,
                leave_start_date: None,
                leave_end_date: None,
                leave_type: None,
                physio_notes: None,
                update_permanent_physio_tracking: None,
            },
        )
        .expect("record attendance");
    };
    // El celular abre el día en el planificador (id aleatorio) y la PC toma asistencia (id fijo por fecha).
    crate::db::schedule_repo::get_or_create_daily_schedule(db_mobile.connection_mut(), date).expect("open day on mobile");
    mark(&mut db_mobile, "PER_DANCER_POPI", "08:40");
    mark(&mut db_pc, "PER_DANCER_ALVAREZ", "08:41");
    let id_of = |db: &DatabaseManager| -> String {
        db.connection()
            .query_row("SELECT id FROM daily_schedules WHERE schedule_date = ?1", [date], |r| r.get(0))
            .expect("schedule id")
    };
    assert_ne!(id_of(&db_mobile), id_of(&db_pc));

    let bundle_mobile = export_sync_bundle(db_mobile.connection(), "A53", "node_a53").expect("export mobile");
    import_sync_bundle(db_pc.connection_mut(), &bundle_mobile).expect("import on pc");
    let bundle_pc = export_sync_bundle(db_pc.connection(), "PC", "node_pc").expect("export pc");
    import_sync_bundle(db_mobile.connection_mut(), &bundle_pc).expect("import on mobile");

    for db in [&db_pc, &db_mobile] {
        let list = list_attendance_for_date(db.connection(), date).expect("list attendance");
        for person in ["PER_DANCER_POPI", "PER_DANCER_ALVAREZ"] {
            let row = list.iter().find(|i| i.personnel_id == person).expect("person present");
            assert_eq!(row.attendance_status, AttendanceStatus::Present, "{person}");
        }
        let n: i64 = db
            .connection()
            .query_row("SELECT COUNT(*) FROM daily_schedules WHERE schedule_date = ?1", [date], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }
}

#[test]
fn test_work_casting_plans_and_one_touch_casting_repo() {
    use crate::db::works_repo::*;
    use crate::dto::{
        UpsertWorkPayload, UpsertWorkPassagePayload,
        UpsertWorkCastingPlanPayload, UpsertWorkCastPlanMemberPayload,
    };

    let mut db = DatabaseManager::open_in_memory().expect("open memory db");
    let conn = db.connection_mut();

    // 1. Listar obras sembradas por la migración 0008
    let seeded_works = list_works_with_casts(conn).expect("list works");
    assert!(seeded_works.len() >= 3);
    assert!(seeded_works.iter().any(|w| w.title == "El Lago de los Cisnes"));
    assert!(seeded_works.iter().any(|w| w.title == "Don Quijote"));
    assert!(seeded_works.iter().any(|w| w.title == "Gala de Ballet Clásico"));

    // Verificar que El Lago de los Cisnes tenga pasajes y repartos
    let lago = seeded_works.iter().find(|w| w.title == "El Lago de los Cisnes").unwrap();
    assert!(!lago.passages.is_empty());
    assert!(!lago.casting_plans.is_empty());
    let pdd_plan = lago.casting_plans.iter().find(|p| p.tier_name == "1° Elenco").unwrap();
    assert!(!pdd_plan.members.is_empty());

    // 2. Crear una nueva obra (Giselle)
    let giselle = upsert_work(conn, UpsertWorkPayload {
        id: None,
        title: "Giselle".to_string(),
        composer: Some("Adolphe Adam".to_string()),
        choreographer: Some("Jean Coralli / Jules Perrot".to_string()),
        is_active: true,
    }).expect("upsert work Giselle");

    assert_eq!(giselle.title, "Giselle");

    // 3. Crear pasaje (Pas de Deux Acto II)
    let passage = upsert_work_passage(conn, UpsertWorkPassagePayload {
        id: None,
        work_id: giselle.id.clone(),
        name: "Pas de Deux - Acto II (Giselle y Albrecht)".to_string(),
        act_scene: Some("Acto II".to_string()),
        default_duration_min: Some(25),
        default_sound_cue: Some("PISTA TeatroCue #07".to_string()),
    }).expect("upsert passage");

    assert_eq!(passage.name, "Pas de Deux - Acto II (Giselle y Albrecht)");

    // 4. Crear plan de reparto (1° Elenco: Popi Gómez y David Marcato)
    let plan = upsert_work_casting_plan(conn, UpsertWorkCastingPlanPayload {
        id: None,
        work_id: giselle.id.clone(),
        passage_id: Some(passage.id.clone()),
        tier_name: "1° Elenco".to_string(),
        cast_label: "Gómez / Marcato".to_string(),
        default_sound_cue: Some("PISTA TeatroCue #07".to_string()),
        notes: Some("Ensayo con zapatillas nuevas".to_string()),
        members: vec![
            UpsertWorkCastPlanMemberPayload {
                personnel_id: "PER_DANCER_POPI".to_string(), // Popi Gómez
                character_role: Some("Giselle".to_string()),
                partner_personnel_id: Some("PER_DANCER_GIL".to_string()),
                condition: CastAssignmentCondition::Titular,
            },
            UpsertWorkCastPlanMemberPayload {
                personnel_id: "PER_DANCER_GIL".to_string(), // Nicolás Gil
                character_role: Some("Albrecht".to_string()),
                partner_personnel_id: Some("PER_DANCER_POPI".to_string()),
                condition: CastAssignmentCondition::Titular,
            },
        ],
    }).expect("upsert casting plan");

    assert_eq!(plan.tier_name, "1° Elenco");
    assert_eq!(plan.members.len(), 2);

    // 5. Verificar que list_works_with_casts incluya a Giselle con su pasaje y plan
    let all_works = list_works_with_casts(conn).expect("list works with giselle");
    let giselle_full = all_works.iter().find(|w| w.id == giselle.id).expect("find giselle");
    assert_eq!(giselle_full.passages.len(), 1);
    assert_eq!(giselle_full.casting_plans.len(), 1);
    assert_eq!(giselle_full.casting_plans[0].members.len(), 2);

    // 6. Eliminar el plan y verificar
    delete_work_casting_plan(conn, &plan.id).expect("delete casting plan");
    let after_delete = list_works_with_casts(conn).expect("list works after delete");
    let giselle_after = after_delete.iter().find(|w| w.id == giselle.id).expect("find giselle");
    assert_eq!(giselle_after.casting_plans.len(), 0);
}

#[test]
fn test_soloist_certifications_and_bop_attendance_matrix() {
    let mut db = DatabaseManager::open_in_memory().expect("open memory db");
    let conn = db.connection_mut();

    // 1. Verificar la certificación seed "Entre dos Mundos" (24 prestaciones)
    let certs = list_soloist_certifications(conn).expect("list certs");
    assert!(!certs.is_empty());
    let entre_dos_mundos = certs
        .iter()
        .find(|c| c.program_title == "Entre dos Mundos")
        .expect("find entre dos mundos");

    assert_eq!(entre_dos_mundos.total_performances, 24);
    assert_eq!(entre_dos_mundos.items.len(), 12);
    assert!(entre_dos_mundos.items.iter().any(|i| i.dancer_name == "VERA, Brenda Mercedes" && i.performance_count == 2));
    assert!(entre_dos_mundos.items.iter().any(|i| i.work_role == "Tangos de Piazzolla" && i.performance_count == 2));

    // 2. Probar creación de una nueva certificación de solísticos
    let new_cert = save_soloist_certification(
        conn,
        crate::domain::SaveSoloistCertificationPayload {
            id: None,
            program_title: "Giselle 2026".to_string(),
            hall_name: "Sala Mayor".to_string(),
            document_date: "2026-11-15".to_string(),
            recipient_name: "Hadrián Ávila Arzuza".to_string(),
            recipient_title: "Director del Teatro del Libertador".to_string(),
            intro_body: "Certificación de Giselle".to_string(),
            closing_text: "Sin otro particular saludo a Ud. Atte.".to_string(),
            status: "DRAFT".to_string(),
            notes: Some("Notas de prueba".to_string()),
            items: vec![
                crate::domain::SaveSoloistItemPayload {
                    id: None,
                    personnel_id: Some("PER_DANCER_BRENDA".to_string()),
                    dancer_name: "VERA, Brenda Mercedes".to_string(),
                    work_role: "Giselle (Acto I y II)".to_string(),
                    performance_dates: "14 y 15/11".to_string(),
                    performance_count: 2,
                    sort_order: 1,
                },
            ],
        },
    )
    .expect("save soloist cert");

    assert_eq!(new_cert.total_performances, 2);
    assert_eq!(new_cert.items.len(), 1);

    // 3. Probar la Matriz Mensual de Ausentismo BOP para Septiembre 2026
    let matrix_sep = get_monthly_bop_matrix(conn, 2026, 9).expect("get bop matrix sep 2026");
    assert_eq!(matrix_sep.month_name, "SEPTIEMBRE");
    assert_eq!(matrix_sep.days_in_month, 30);
    assert_eq!(matrix_sep.days_header.len(), 30);

    // El día 11 de septiembre debe tener la anotación de "ENSAYO SUSPENDIDO"
    let day_11 = matrix_sep.days_header.iter().find(|d| d.day_number == 11).expect("day 11");
    assert_eq!(day_11.day_annotation.as_deref(), Some("ENSAYO SUSPENDIDO"));

    // Los días 5 y 6 deben ser fines de semana
    let day_5 = matrix_sep.days_header.iter().find(|d| d.day_number == 5).expect("day 5");
    assert!(day_5.is_weekend);

    // 4. Probar actualización de celda en la matriz (P, EI, SP)
    let p_brenda = matrix_sep.rows.iter().find(|r| r.dancer_name.contains("Vera")).expect("find brenda row");
    update_bop_matrix_cell(conn, "2026-09-01", &p_brenda.personnel_id, "P", Some("Presente a horario")).expect("update cell P");
    update_bop_matrix_cell(conn, "2026-09-02", &p_brenda.personnel_id, "EI", Some("Gripe")).expect("update cell EI");
    update_bop_matrix_cell(conn, "2026-09-03", &p_brenda.personnel_id, "SP", None).expect("update cell SP");

    let matrix_updated = get_monthly_bop_matrix(conn, 2026, 9).expect("get matrix updated");
    let brenda_updated = matrix_updated.rows.iter().find(|r| r.dancer_name.contains("Vera")).expect("find brenda updated");

    assert_eq!(brenda_updated.cells[0].code, "P");
    assert_eq!(brenda_updated.cells[1].code, "EI");
    assert_eq!(brenda_updated.cells[2].code, "SP");
    assert!(brenda_updated.total_p >= 1);
    assert!(brenda_updated.total_ei >= 1);
    assert!(brenda_updated.total_sp >= 1);

    // 5. Probar "S/P" (Sin Parte / Sin Convocar)
    update_bop_matrix_cell(conn, "2026-09-04", &p_brenda.personnel_id, "S/P", Some("No convocada al parte")).expect("update cell S/P");
    let matrix_sp = get_monthly_bop_matrix(conn, 2026, 9).expect("get matrix sp");
    let brenda_sp = matrix_sp.rows.iter().find(|r| r.dancer_name.contains("Vera")).expect("find brenda sp");
    assert_eq!(brenda_sp.cells[3].code, "S/P");
    // S/P no debe aumentar el total de inasistencias total_sp
    assert_eq!(brenda_sp.total_sp, 1);

    // 6. IJ: máximo 2 por mes y 10 por año
    let id = &p_brenda.personnel_id;
    update_bop_matrix_cell(conn, "2026-09-07", id, "IJ", None).expect("IJ 1");
    update_bop_matrix_cell(conn, "2026-09-08", id, "IJ", None).expect("IJ 2");
    assert!(update_bop_matrix_cell(conn, "2026-09-09", id, "IJ", None).is_err(), "3ra IJ del mes");
    // reescribir una IJ existente no cuenta doble
    update_bop_matrix_cell(conn, "2026-09-08", id, "IJ", Some("nota")).expect("IJ re-guardada");
    // cambiar una IJ por otro código libera cupo
    update_bop_matrix_cell(conn, "2026-09-08", id, "P", None).expect("IJ -> P");
    update_bop_matrix_cell(conn, "2026-09-09", id, "IJ", None).expect("IJ tras liberar");
    let m = get_monthly_bop_matrix(conn, 2026, 9).expect("matrix ij");
    let r = m.rows.iter().find(|r| r.personnel_id == *id).unwrap();
    assert_eq!(r.cells[6].code, "IJ");
    // tope anual: 8 más en otros meses (10 en total con las 2 de septiembre)
    for mes in 1..=8 {
        let d = format!("2026-{:02}-10", mes);
        update_bop_matrix_cell(conn, &d, id, "IJ", None).expect("IJ anual");
    }
    assert!(update_bop_matrix_cell(conn, "2026-10-12", id, "IJ", None).is_err(), "11ra IJ del año");
    update_bop_matrix_cell(conn, "2027-01-11", id, "IJ", None).expect("año nuevo");

    // 7. Adecuación de tareas (AT): trabaja en otra área, no es falta
    update_bop_matrix_cell(conn, "2026-09-15", id, "AT", Some("Trabaja en archivo")).expect("AT");
    let m = get_monthly_bop_matrix(conn, 2026, 9).expect("matrix at");
    let r = m.rows.iter().find(|r| r.personnel_id == *id).unwrap();
    assert_eq!(r.cells[14].code, "AT");
    let att = record_attendance(conn, RecordAttendancePayload {
        schedule_date: "2026-09-16".into(),
        personnel_id: id.clone(),
        attendance_status: AttendanceStatus::TaskAdjustment,
        actual_checkin_time: None, minutes_late: None,
        remarks: Some("Adecuación de tareas: depósito".into()),
        certificate_file_name: None, certificate_data: None, medical_diagnosis: None, doctor_details: None,
        leave_start_date: None, leave_end_date: None, leave_type: None, physio_notes: None,
        update_permanent_physio_tracking: None,
    }).expect("record AT");
    assert_eq!(att.attendance_status, AttendanceStatus::TaskAdjustment);

    // 8. Lo cargado en la planilla BOP se lee igual en la asistencia diaria
    update_bop_matrix_cell(conn, "2026-09-17", id, "EI", None).expect("EI");
    update_bop_matrix_cell(conn, "2026-09-18", id, "SP", None).expect("SP");
    let find = |d: &str| list_attendance_for_date(conn, d).unwrap().into_iter().find(|l| l.personnel_id == *id).unwrap().attendance_status;
    assert_eq!(find("2026-09-17"), AttendanceStatus::MedicalLeave);
    assert_eq!(find("2026-09-18"), AttendanceStatus::Absent);
    assert_eq!(find("2026-09-07"), AttendanceStatus::JustifiedAbsence);

    // 9. El tope de IJ también rige desde la asistencia diaria (sept. ya tiene 2: 07 y 09)
    let ij = |d: &str| record_attendance(conn, RecordAttendancePayload {
        schedule_date: d.into(), personnel_id: id.clone(),
        attendance_status: AttendanceStatus::JustifiedAbsence,
        actual_checkin_time: None, minutes_late: None, remarks: None,
        certificate_file_name: None, certificate_data: None, medical_diagnosis: None, doctor_details: None,
        leave_start_date: None, leave_end_date: None, leave_type: None, physio_notes: None, update_permanent_physio_tracking: None,
    });
    assert!(ij("2026-09-21").is_err(), "3ra IJ del mes desde asistencia diaria");
    assert!(ij("2026-09-09").is_ok(), "re-guardar una IJ existente no cuenta doble");
}

#[test]
fn test_inspection_vault_and_security_repository() {
    use crate::db::inspection_vault_repo::*;
    use crate::dto::{CreateInspectionDocumentPayload, InspectionVaultFilterPayload};

    let mut db = DatabaseManager::open_in_memory().expect("open memory db");
    let conn = db.connection_mut();

    // 1. Verificar estado inicial de seguridad (sin clave)
    let sec_init = get_vault_security_status(conn).expect("initial sec status");
    assert!(!sec_init.has_password);
    assert!(verify_vault_password(conn, "cualquiera").expect("verify unconfigured"));

    // 2. Establecer contraseña por primera vez
    set_vault_password(conn, None, "Carina2026!", Some(15)).expect("set password");
    let sec_configured = get_vault_security_status(conn).expect("sec status after set");
    assert!(sec_configured.has_password);
    assert_eq!(sec_configured.lock_timeout_minutes, 15);

    // 3. Verificación de clave correcta y errónea
    assert!(!verify_vault_password(conn, "PasswordErronea").expect("verify wrong"));
    assert!(verify_vault_password(conn, "Carina2026!").expect("verify correct"));

    // 4. Intento de cambio con clave anterior errónea
    let change_fail = set_vault_password(conn, Some("ClaveMala"), "NuevaClave123", Some(30));
    assert!(change_fail.is_err());

    // 5. Cambio exitoso de contraseña
    set_vault_password(conn, Some("Carina2026!"), "CarinaNueva2027!", Some(30)).expect("change password ok");
    assert!(verify_vault_password(conn, "CarinaNueva2027!").expect("verify new pass"));

    // 6. Reseteo administrativo (desde pendrive/instalador)
    admin_reset_vault_password(conn, "AdminRescue999!").expect("admin reset");
    assert!(verify_vault_password(conn, "AdminRescue999!").expect("verify admin reset pass"));

    // 7. Crear documentos en la Bóveda de Inspección
    // A) Recibo de compra de caja chica (Colofonia y botiquín)
    let receipt = create_vault_document(
        conn,
        CreateInspectionDocumentPayload {
            id: None,
            title: "Compra de Pez Rubia (Colofonia) para salas 1 y 3".to_string(),
            category: "RECIBO_COMPRA".to_string(),
            document_date: "2026-09-10".to_string(),
            personnel_id: None,
            file_name: "factura_colofonia_danzacordoba.jpg".to_string(),
            file_type: "image/jpeg".to_string(),
            file_size_bytes: Some(150_000),
            file_data: "data:image/jpeg;base64,/9j/4AAQSkZJRg==".to_string(),
            amount_ars: Some(45500.50),
            supplier_vendor: Some("Danza Córdoba".to_string()),
            notes: Some("Caja chica de Inspección de Ballet".to_string()),
        },
    )
    .expect("create receipt");

    assert_eq!(receipt.category, "RECIBO_COMPRA");
    assert_eq!(receipt.amount_ars, Some(45500.50));

    // B) Certificado médico para Popi Gómez
    let cert = create_vault_document(
        conn,
        CreateInspectionDocumentPayload {
            id: None,
            title: "Certificado Traumatología - Ecografía Tendón de Aquiles".to_string(),
            category: "CERTIFICADO_MEDICO".to_string(),
            document_date: "2026-09-12".to_string(),
            personnel_id: Some("PER_DANCER_POPI".to_string()),
            file_name: "popi_eco_tendon.pdf".to_string(),
            file_type: "application/pdf".to_string(),
            file_size_bytes: Some(350_000),
            file_data: "data:application/pdf;base64,JVBERi0xLjQK".to_string(),
            amount_ars: None,
            supplier_vendor: None,
            notes: Some("Reposo por 5 días indicado por Dr. Allende".to_string()),
        },
    )
    .expect("create cert");

    assert_eq!(cert.category, "CERTIFICADO_MEDICO");
    assert!(cert.personnel_name.is_some());

    // 8. Listar y filtrar documentos
    let all_docs = list_vault_documents(conn, InspectionVaultFilterPayload::default()).expect("list all");
    assert_eq!(all_docs.len(), 2);

    let receipts_only = list_vault_documents(
        conn,
        InspectionVaultFilterPayload {
            category: Some("RECIBO_COMPRA".to_string()),
            ..Default::default()
        },
    )
    .expect("filter receipts");
    assert_eq!(receipts_only.len(), 1);
    assert_eq!(receipts_only[0].supplier_vendor.as_deref(), Some("Danza Córdoba"));

    let popi_docs = list_vault_documents(
        conn,
        InspectionVaultFilterPayload {
            personnel_id: Some("PER_DANCER_POPI".to_string()),
            ..Default::default()
        },
    )
    .expect("filter popi");
    assert_eq!(popi_docs.len(), 1);

    // 9. Totales mensuales de caja chica
    let totals = get_vault_monthly_totals(conn, 9, 2026).expect("get totals");
    assert_eq!(totals.total_amount_ars, 45500.50);
    assert_eq!(totals.receipts_count, 1);
    assert_eq!(totals.certificates_count, 1);
    assert_eq!(totals.documents_total_count, 2);

    // 10. Eliminar documento
    delete_vault_document(conn, &receipt.id).expect("delete receipt");
    let after_del = list_vault_documents(conn, InspectionVaultFilterPayload::default()).expect("list after del");
    assert_eq!(after_del.len(), 1);
}




#[test]
fn test_sound_technician_assignment_and_sync() {
    use crate::db::sync_repo::{export_sync_bundle, import_sync_bundle};
    use crate::db::schedule_repo::upsert_morning_class;
    use crate::dto::UpsertMorningClassPayload;

    let mut db = DatabaseManager::open_in_memory().expect("open memory db");
    let conn = db.connection_mut();

    // Sonidista de ejemplo: el personal con rol SOUND_TECH precargado
    let tech_id: String = conn
        .query_row("SELECT id FROM personnel WHERE role = 'SOUND_TECH' LIMIT 1", [], |r| r.get(0))
        .expect("hay un sonidista precargado");

    let sched = get_or_create_daily_schedule(conn, "2026-10-19").expect("schedule");
    let sid = sched.schedule.id.clone();
    assert!(sched.schedule.rehearsal_blocks.iter().all(|b| b.sound_tech_id.is_none()), "nada tiene sonidista de entrada");

    // Ensayo con sonidista asignado
    let payload = UpsertRehearsalBlockPayload {
        sound_tech_id: Some(tech_id.clone()),
        id: None,
        schedule_id: sid.clone(),
        room: RoomCode::Sala2,
        start_time: "10:30".to_string(),
        end_time: "11:30".to_string(),
        activity_type: ActivityType::StudioRehearsal,
        work_id: None,
        passage_id: None,
        custom_passage_label: Some("Prueba de sonido".to_string()),
        confirmation_state: ConfirmationState::Confirmed,
        sound_requirement: "PISTA".to_string(),
        teatrocue_cue_ref: None,
        master_id: None,
        pianist_id: None,
        allow_shared_room: false,
        notes: None,
        cast_groups: vec![],
    };
    let saved = upsert_rehearsal_block(conn, payload).expect("upsert");
    let block = saved.schedule.rehearsal_blocks.iter().find(|b| b.passage_name == "Prueba de sonido").expect("ensayo nuevo");
    assert_eq!(block.sound_tech_id.as_deref(), Some(tech_id.as_str()));
    assert!(block.sound_tech_name.as_deref().is_some_and(|n| !n.is_empty()));

    // Clase con sonidista
    let class = &sched.schedule.morning_classes[0];
    let saved = upsert_morning_class(
        conn,
        UpsertMorningClassPayload {
            id: Some(class.id.clone()),
            schedule_id: sid.clone(),
            room: class.room,
            gender_division: class.gender_division,
            start_time: class.start_time.clone(),
            end_time: class.end_time.clone(),
            master_id: class.master_id.clone(),
            pianist_id: class.pianist_id.clone(),
            sound_tech_id: Some(tech_id.clone()),
        },
    )
    .expect("upsert class");
    let c = saved.schedule.morning_classes.iter().find(|c| c.id == class.id).unwrap();
    assert_eq!(c.sound_tech_id.as_deref(), Some(tech_id.as_str()));

    // El paquete de sincronización lleva el sonidista y las clases
    let bundle = export_sync_bundle(conn, "PC", "NODO-1").expect("export");
    assert!(bundle.rehearsal_blocks.iter().any(|b| b["sound_tech_id"] == tech_id.as_str()));
    assert!(bundle.morning_classes.iter().any(|b| b["sound_tech_id"] == tech_id.as_str()));

    // Se importa en otra base que ya tiene el mismo parte: el sonidista llega
    let mut other = DatabaseManager::open_in_memory().expect("otra base");
    let oc = other.connection_mut();
    import_sync_bundle(oc, &bundle).expect("import");
    let n: i64 = oc
        .query_row("SELECT COUNT(*) FROM rehearsal_blocks WHERE sound_tech_id = ?1", [&tech_id], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);

    // Un paquete de una versión anterior (sin clases) sigue siendo válido
    let mut old = serde_json::to_value(&bundle).unwrap();
    old.as_object_mut().unwrap().remove("morning_classes");
    let parsed: crate::dto::SyncBundle = serde_json::from_value(old).expect("paquete viejo");
    assert!(parsed.morning_classes.is_empty());
}
