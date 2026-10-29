-- =============================================================================
-- TEATROGESTIÓN - CHEQUEO DE INTEGRIDAD DE LA BASE DE DATOS (SOLO LECTURA)
-- Uso:  sqlite3 -readonly "copia_de_la_base.sqlite" ".read scripts/db_integrity_check.sql"
-- No modifica ningún dato: ejecuta PRAGMAs de diagnóstico y SELECTs.
-- =============================================================================

SELECT '=== 1. INTEGRIDAD ESTRUCTURAL (PRAGMA integrity_check) ===' AS seccion;
PRAGMA integrity_check;

SELECT '=== 2. VIOLACIONES DE CLAVES FORANEAS (PRAGMA foreign_key_check; sin filas = OK) ===' AS seccion;
PRAGMA foreign_key_check;

SELECT '=== 3. FILAS HUERFANAS (todas deben dar 0) ===' AS seccion;

-- Hijas de personnel (padrón)
SELECT 'attendance_logs.personnel_id -> personnel' AS referencia, COUNT(*) AS filas_huerfanas
FROM attendance_logs t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'physiotherapy_sessions.personnel_id -> personnel', COUNT(*)
FROM physiotherapy_sessions t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'medical_leave_records.personnel_id -> personnel', COUNT(*)
FROM medical_leave_records t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'annual_quota_overrides.personnel_id -> personnel', COUNT(*)
FROM annual_quota_overrides t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'rehearsal_cast_members.personnel_id -> personnel', COUNT(*)
FROM rehearsal_cast_members t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'rehearsal_cast_members.partner_personnel_id -> personnel', COUNT(*)
FROM rehearsal_cast_members t LEFT JOIN personnel p ON p.id = t.partner_personnel_id
WHERE t.partner_personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'work_cast_plan_members.personnel_id -> personnel', COUNT(*)
FROM work_cast_plan_members t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'work_cast_plan_members.partner_personnel_id -> personnel', COUNT(*)
FROM work_cast_plan_members t LEFT JOIN personnel p ON p.id = t.partner_personnel_id
WHERE t.partner_personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'stock_reservations.personnel_id -> personnel', COUNT(*)
FROM stock_reservations t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'stock_reservations.created_by -> personnel', COUNT(*)
FROM stock_reservations t LEFT JOIN personnel p ON p.id = t.created_by
WHERE t.created_by IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'voucher_deliveries.personnel_id -> personnel', COUNT(*)
FROM voucher_deliveries t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'voucher_deliveries.delivered_by_inspector_id -> personnel', COUNT(*)
FROM voucher_deliveries t LEFT JOIN personnel p ON p.id = t.delivered_by_inspector_id
WHERE t.delivered_by_inspector_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'stock_ledger.performed_by -> personnel', COUNT(*)
FROM stock_ledger t LEFT JOIN personnel p ON p.id = t.performed_by
WHERE t.performed_by IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'morning_classes.master_id -> personnel', COUNT(*)
FROM morning_classes t LEFT JOIN personnel p ON p.id = t.master_id
WHERE t.master_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'morning_classes.pianist_id -> personnel', COUNT(*)
FROM morning_classes t LEFT JOIN personnel p ON p.id = t.pianist_id
WHERE t.pianist_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'morning_classes.sound_tech_id -> personnel', COUNT(*)
FROM morning_classes t LEFT JOIN personnel p ON p.id = t.sound_tech_id
WHERE t.sound_tech_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'rehearsal_blocks.master_id -> personnel', COUNT(*)
FROM rehearsal_blocks t LEFT JOIN personnel p ON p.id = t.master_id
WHERE t.master_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'rehearsal_blocks.pianist_id -> personnel', COUNT(*)
FROM rehearsal_blocks t LEFT JOIN personnel p ON p.id = t.pianist_id
WHERE t.pianist_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'rehearsal_blocks.sound_tech_id -> personnel', COUNT(*)
FROM rehearsal_blocks t LEFT JOIN personnel p ON p.id = t.sound_tech_id
WHERE t.sound_tech_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'daily_schedules.created_by -> personnel', COUNT(*)
FROM daily_schedules t LEFT JOIN personnel p ON p.id = t.created_by
WHERE t.created_by IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'soloist_certifications.personnel_id -> personnel', COUNT(*)
FROM soloist_certifications t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
UNION ALL
SELECT 'inspection_vault_documents.personnel_id -> personnel', COUNT(*)
FROM inspection_vault_documents t LEFT JOIN personnel p ON p.id = t.personnel_id
WHERE t.personnel_id IS NOT NULL AND p.id IS NULL
-- Detalle de otras cabeceras
UNION ALL
SELECT 'voucher_delivery_items.voucher_id -> voucher_deliveries', COUNT(*)
FROM voucher_delivery_items t LEFT JOIN voucher_deliveries v ON v.id = t.voucher_id
WHERE t.voucher_id IS NOT NULL AND v.id IS NULL
UNION ALL
SELECT 'soloist_certification_items.certification_id -> soloist_certifications', COUNT(*)
FROM soloist_certification_items t LEFT JOIN soloist_certifications s ON s.id = t.certification_id
WHERE t.certification_id IS NOT NULL AND s.id IS NULL
UNION ALL
SELECT 'rehearsal_cast_members.cast_group_id -> rehearsal_cast_groups', COUNT(*)
FROM rehearsal_cast_members t LEFT JOIN rehearsal_cast_groups g ON g.id = t.cast_group_id
WHERE t.cast_group_id IS NOT NULL AND g.id IS NULL
UNION ALL
SELECT 'work_cast_plan_members.plan_id -> work_cast_plans', COUNT(*)
FROM work_cast_plan_members t LEFT JOIN work_cast_plans w ON w.id = t.plan_id
WHERE t.plan_id IS NOT NULL AND w.id IS NULL;

SELECT '=== 4. MIGRACIONES APLICADAS (_schema_migrations) ===' AS seccion;
SELECT version, name, applied_at FROM _schema_migrations ORDER BY version;

SELECT '=== 5. ESTADO DEL PADRON (personnel) ===' AS seccion;
SELECT COUNT(*) AS total,
       SUM(CASE WHEN is_active = 1 THEN 1 ELSE 0 END) AS activos,
       SUM(CASE WHEN is_active = 0 THEN 1 ELSE 0 END) AS inactivos
FROM personnel;
