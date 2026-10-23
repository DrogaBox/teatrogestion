-- =============================================================================
-- TEATROGESTIÓN - ESTADO JUBILATORIO (NO CONCURREN) Y RANGO DE LICENCIAS MÉDICAS
-- Archivo: 0005_retirement_and_medical_leaves_range.sql
-- =============================================================================

-- 1. Campos de Estado Jubilatorio y Cargo Específico del Sector
ALTER TABLE personnel ADD COLUMN is_retired INTEGER NOT NULL DEFAULT 0; -- 1: Estado Jubilatorio (Eximido de concurrencia)
ALTER TABLE personnel ADD COLUMN retirement_notes TEXT; -- Notas del trámite previsional (Caja Jubilaciones Cba / ANSES)
ALTER TABLE personnel ADD COLUMN specific_position TEXT; -- Cargo específico ('Supervisora / Inspectora', 'Director Artístico', 'Pianista', 'Asistente Técnico', etc.)

-- 2. Fecha de inicio de licencia médica en registros diarios
ALTER TABLE attendance_logs ADD COLUMN leave_start_date TEXT; -- YYYY-MM-DD

-- 3. Tabla permanente de Licencias Médicas con Rango [Desde, Hasta] y Foto de Certificado
CREATE TABLE IF NOT EXISTS medical_leave_records (
    id TEXT PRIMARY KEY,
    personnel_id TEXT NOT NULL REFERENCES personnel(id) ON DELETE CASCADE,
    start_date TEXT NOT NULL, -- YYYY-MM-DD (Desde)
    end_date TEXT NOT NULL,   -- YYYY-MM-DD (Hasta)
    diagnosis TEXT,
    doctor_name TEXT,
    certificate_file_name TEXT,
    certificate_data TEXT,    -- Foto o PDF del certificado en Base64
    remarks TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_med_leaves_dancer_dates ON medical_leave_records(personnel_id, start_date, end_date);
