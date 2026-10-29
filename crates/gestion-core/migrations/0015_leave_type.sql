-- =============================================================================
-- TEATROGESTIÓN - TIPO DE LICENCIA EN LA CARPETA MÉDICA
-- Archivo: 0015_leave_type.sql
-- =============================================================================

-- Tipo de licencia en el registro permanente de licencias:
-- ENFERMEDAD (comportamiento histórico), ESTUDIO, SIN_GOCE_DE_HABERES, OTRAS.
ALTER TABLE medical_leave_records ADD COLUMN leave_type TEXT NOT NULL DEFAULT 'ENFERMEDAD';
