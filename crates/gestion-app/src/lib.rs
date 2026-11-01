pub mod commands;
pub mod state;

use commands::*;
use state::AppState;
use gestion_core::DatabaseManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app_data_dir");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app_data_dir");
            let db_path = app_data_dir.join("teatro_gestion.sqlite");

            println!("[TeatroGestión] Conexión SQLite WAL inicializada en: {}", db_path.display());
            let db = DatabaseManager::open(&db_path).expect("Failed to initialize SQLite database");

            let app_state = AppState::new(db);
            start_lan_sync_server(app_state.db.clone(), DEFAULT_SYNC_PORT);

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_personnel,
            search_dancer_quick,
            upsert_personnel,
            delete_personnel,
            import_personnel_batch,
            list_daily_schedules,
            get_or_create_daily_schedule,
            set_schedule_regime,
            update_schedule_rules,
            upsert_morning_class,
            delete_morning_class,
            upsert_rehearsal_block,
            delete_rehearsal_block,
            parse_gaynor_code_preview,
            list_stock_catalog,
            upsert_stock_item,
            execute_voucher_delivery,
            list_attendance_for_date,
            record_attendance,
            get_agent_monthly_attendance_sheet,
            list_monthly_agents_summary,
            create_physiotherapy_session,
            list_physiotherapy_sessions,
            get_lan_sync_info,
            export_local_sync_bundle,
            import_remote_sync_bundle,
            sync_with_lan_peer,
            get_lan_sync_key,
            set_lan_sync_key,
            list_works_with_casts,
            upsert_work,
            delete_work,
            upsert_work_passage,
            delete_work_passage,
            upsert_work_casting_plan,
            delete_work_casting_plan,
            get_update_status,
            set_update_token,
            clear_update_token,
            check_for_update,
            install_update,
            get_license_info,
            apply_emergency_activation,
            get_machine_id,
            get_monthly_bop_matrix,
            update_bop_matrix_cell,
            set_day_schedule_annotation,
            list_soloist_certifications,
            get_soloist_certification,
            save_soloist_certification,
            delete_soloist_certification,
            generate_soloist_certification_from_work_cast,
            list_vault_documents,
            create_vault_document,
            delete_vault_document,
            get_vault_monthly_totals,
            get_vault_security_status,
            verify_vault_password,
            set_vault_password,
            // Sin `admin_reset_vault_password`: cambiaba la contraseña de la Bóveda sin pedir ninguna credencial.
            // El reseteo administrativo queda en el instalador, que exige la llave maestra del pendrive.
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
