-- =============================================================================
-- TEATROGESTIÓN - CONTROL DE ASISTENCIA, CARPETAS MÉDICAS Y SEMANA OFICIAL
-- Archivo: 0004_attendance_medical_and_weekly_sheet.sql
-- =============================================================================

-- -----------------------------------------------------------------------------
-- 1. EXTENSIÓN DE TABLA ATTENDANCE_LOGS PARA CARPETAS MÉDICAS Y CERTIFICADOS
-- -----------------------------------------------------------------------------
ALTER TABLE attendance_logs ADD COLUMN certificate_file_name TEXT;
ALTER TABLE attendance_logs ADD COLUMN certificate_data TEXT; -- Base64 o URI de comprobante adjunto
ALTER TABLE attendance_logs ADD COLUMN medical_diagnosis TEXT;
ALTER TABLE attendance_logs ADD COLUMN doctor_details TEXT; -- Médico tratante y matrícula
ALTER TABLE attendance_logs ADD COLUMN leave_end_date TEXT; -- Fecha estimada de alta médica
ALTER TABLE attendance_logs ADD COLUMN is_in_class INTEGER NOT NULL DEFAULT 1; -- 1: Asiste a clase matutina, 0: No está en clase
ALTER TABLE attendance_logs ADD COLUMN operational_status TEXT DEFAULT 'PENDING'; -- 'PRESENT', 'ABSENT', 'NOT_IN_CLASS', 'IN_PHYSIOTHERAPY', 'LATE', 'MEDICAL_LEAVE', 'PENDING'
ALTER TABLE attendance_logs ADD COLUMN physio_notes TEXT; -- Notas de la Dra. de Fisioterapia para el día

-- Seguimiento clínico continuo de fisioterapia en la ficha del bailarín
ALTER TABLE personnel ADD COLUMN current_physio_diagnosis TEXT; -- Diagnóstico kinésico activo
ALTER TABLE personnel ADD COLUMN physio_tracking_notes TEXT; -- Historial y notas de seguimiento de la Dra. de Fisioterapia

-- -----------------------------------------------------------------------------
-- 1.B OFICINA DE FISIOTERAPIA: ATENCIONES CLÍNICAS Y REPORTES DE LA DRA.
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS physiotherapy_sessions (
    id TEXT PRIMARY KEY,
    personnel_id TEXT NOT NULL REFERENCES personnel(id) ON DELETE CASCADE,
    session_date TEXT NOT NULL,
    session_time TEXT NOT NULL DEFAULT (strftime('%H:%M', 'now')),
    doctor_name TEXT NOT NULL DEFAULT 'Dra. de Fisioterapia',
    diagnosis TEXT,
    treatment_applied TEXT NOT NULL,
    evolution_notes TEXT,
    activity_restriction TEXT NOT NULL DEFAULT 'NORMAL' CHECK (
        activity_restriction IN ('NORMAL', 'LIGHT_BARRE_ONLY', 'NO_JUMPS', 'NO_POINTE', 'TOTAL_REST')
    ),
    attachment_name TEXT,
    attachment_data TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_physio_sessions_date ON physiotherapy_sessions(session_date);
CREATE INDEX IF NOT EXISTS idx_physio_sessions_personnel ON physiotherapy_sessions(personnel_id);

-- -----------------------------------------------------------------------------
-- 2. INCORPORACIÓN DEL PLANTEL DE LA DIRECCIÓN Y ELENCO OFICIAL DE ENSAYOS
-- -----------------------------------------------------------------------------
-- Dirección / Maestros
INSERT OR IGNORE INTO personnel (
    id, legajo, first_name, last_name, nickname, role, gender_group, category_id, is_active, notes
) VALUES
('PER_MASTER_BACA', 'DOC-004', 'Patricia', 'Baca Urquiza', 'Directora Baca', 'MASTER', 'FEMALE', 'CAT_STAFF_DOCENTE', 1, 'Directora / Maestra de Ensayo');

-- Bailarines y Refuerzos citados en la planilla oficial de la semana
INSERT OR IGNORE INTO personnel (id, legajo, first_name, last_name, nickname, role, gender_group, category_id, default_pointe_raw_code, default_flat_size, is_active, notes) VALUES
('PER_DANCER_GACITUA',    'BOC-107', 'Rodrigo',   'Gacitúa',   'Gacitúa',    'DANCER', 'MALE',   'CAT_CONTRATADO',     NULL, '10M',  1, 'Refuerzo - Princesas y Bufón'),
('PER_DANCER_BEACON',     'BOC-108', 'Federico',  'Beacon',    'Beacon',     'DANCER', 'MALE',   'CAT_CONTRATADO',     NULL, '11M',  1, 'Refuerzo - Princesas y Bufón'),
('PER_DANCER_HAZEL',      'BOC-109', 'Ezequiel',  'Hazel',     'Hazel',      'DANCER', 'MALE',   'CAT_CONTRATADO',     NULL, '10W',  1, 'Refuerzo - Princesas y Bufón'),
('PER_DANCER_MARTINI',    'BOC-110', 'Valentín',  'Martini',   'Martini',    'DANCER', 'MALE',   'CAT_CONTRATADO',     NULL, '12M',  1, 'Refuerzo - Princesas y Bufón'),
('PER_DANCER_PEREZ_L',    'BOC-111', 'Lorena',    'Pérez',     'Pérez L',    'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE',   'SC - 7.5W4PDH', '7.5D', 1, 'Danza Napolitana'),
('PER_DANCER_NATALI',     'BOC-112', 'Facundo',   'Natali',    'Natali',     'DANCER', 'MALE',   'CAT_CUERPO_BAILE',   NULL, '11M',  1, 'Danza Napolitana'),
('PER_DANCER_PEREZ_F',    'BOC-113', 'Franco',    'Pérez',     'Pérez F',    'DANCER', 'MALE',   'CAT_CUERPO_BAILE',   NULL, '10.5W',1, 'Danza Napolitana'),
('PER_DANCER_PAREDES',    'BOC-114', 'Camila',    'Paredes',   'Paredes',    'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE',   'CL - 8M3XDH', '8C', 1, 'Danza Napolitana'),
('PER_DANCER_MORCHIO_J',  'BOC-115', 'Javier',    'Morchio',   'Morchio J',  'DANCER', 'MALE',   'CAT_PRIMERA_FIGURA', NULL, '10.5M',1, 'Pas de deux Odette y Sigfrido'),
('PER_DANCER_BRAVO',      'BOC-116', 'Silvia',    'Bravo',     'Bravo',      'DANCER', 'FEMALE', 'CAT_PRIMERA_FIGURA', 'SK - 8.5M4HDL', '7.5D', 1, 'Pas de deux Odette y Sigfrido'),
('PER_DANCER_ALTAMIRANO', 'BOC-117', 'Ignacio',   'Altamirano','Altamirano', 'DANCER', 'MALE',   'CAT_SOLISTA',        NULL, '11W',  1, 'Odette y Sigfrido / Diana y Acteón'),
('PER_DANCER_GONZALEZ',   'BOC-118', 'Martina',   'González',  'González',   'DANCER', 'FEMALE', 'CAT_SOLISTA',        'SC - 7.5W4PDH', '7.5D', 1, 'Diana y Acteón'),
('PER_DANCER_CORNEJO',    'BOC-119', 'Matías',    'Cornejo',   'Cornejo',    'DANCER', 'MALE',   'CAT_CUERPO_BAILE',   NULL, '10M',  1, 'Diana y Acteón'),
('PER_DANCER_RAGESSI',    'BOC-120', 'Lucas',     'Ragessi',   'Ragessi',    'DANCER', 'MALE',   'CAT_CUERPO_BAILE',   NULL, '11M',  1, 'Diana y Acteón'),
('PER_DANCER_MAURO',      'BOC-121', 'Agustín',   'Mauro',     'Mauro',      'DANCER', 'MALE',   'CAT_CUERPO_BAILE',   NULL, '12M',  1, 'Diana y Acteón');

-- -----------------------------------------------------------------------------
-- 3. OBRAS Y PASAJES COMPLEMENTARIOS DE LA SEMANA
-- -----------------------------------------------------------------------------
INSERT OR IGNORE INTO work_passages (id, work_id, name, act_scene, default_duration_min, default_sound_cue) VALUES
('PAS_DANZA_NAPOLITANA', 'WORK_LAGO_CISNES', 'Danza Napolitana', 'Acto III', 75, 'PISTA TeatroCue #12'),
('PAS_DANZA_HUNGARA',    'WORK_LAGO_CISNES', 'Danza Húngara',    'Acto III', 60, 'PISTA TeatroCue #09'),
('PAS_DANZA_MAZURCA',    'WORK_LAGO_CISNES', 'Danza Mazurca',    'Acto III', 60, 'PISTA TeatroCue #10');

-- -----------------------------------------------------------------------------
-- 4. SEMANA OFICIAL DE PARTE DIARIO (19 AL 23 DE OCTUBRE 2026)
-- -----------------------------------------------------------------------------
INSERT OR IGNORE INTO daily_schedules (id, schedule_date, day_of_week, regime_type, mandatory_checkin_time, nominal_start_time, nominal_end_time, minimum_intershift_rest_hours, footer_legal_clause) VALUES
('SCHED_2026_10_19', '2026-10-19', 1, 'ORDINARY', '08:45', '09:00', '13:45', 12, 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.'),
('SCHED_2026_10_20', '2026-10-20', 2, 'ORDINARY', '08:45', '09:00', '13:45', 12, 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.'),
('SCHED_2026_10_21', '2026-10-21', 3, 'ORDINARY', '08:45', '09:00', '13:45', 12, 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.'),
('SCHED_2026_10_22', '2026-10-22', 4, 'ORDINARY', '08:45', '09:00', '13:45', 12, 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.'),
('SCHED_2026_10_23', '2026-10-23', 5, 'ORDINARY', '08:45', '09:00', '13:45', 12, 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.');

-- Clases Matutinas según "PARTE DE CLASES - SEMANA 19 al 23 octubre 2026"
-- Lunes 19/10: Sala 1 Mujeres (Ananiev / Romero), Sala 3 Varones (Benites / Villegas)
INSERT OR IGNORE INTO morning_classes (id, schedule_id, room_id, gender_division, start_time, end_time, master_id, pianist_id) VALUES
('CLS_19_S1', 'SCHED_2026_10_19', 'SALA_1', 'FEMALE', '09:00', '10:15', 'PER_MASTER_ANANIEV', 'PER_PIANIST_ROMERO'),
('CLS_19_S3', 'SCHED_2026_10_19', 'SALA_3', 'MALE',   '09:00', '10:15', 'PER_MASTER_BENITES', 'PER_PIANIST_VILLEGAS'),

-- Martes 20/10: Sala 1 Varones (Ananiev / Brusutti), Sala 3 Mujeres (Pino / Romero)
('CLS_20_S1', 'SCHED_2026_10_20', 'SALA_1', 'MALE',   '09:00', '10:15', 'PER_MASTER_ANANIEV', 'PER_PIANIST_BRUSUTTI'),
('CLS_20_S3', 'SCHED_2026_10_20', 'SALA_3', 'FEMALE', '09:00', '10:15', 'PER_MASTER_PINO',    'PER_PIANIST_ROMERO'),

-- Miércoles 21/10: Sala 1 Mujeres (Ananiev / Villegas), Sala 3 Varones (Benites / Brusutti)
('CLS_21_S1', 'SCHED_2026_10_21', 'SALA_1', 'FEMALE', '09:00', '10:15', 'PER_MASTER_ANANIEV', 'PER_PIANIST_VILLEGAS'),
('CLS_21_S3', 'SCHED_2026_10_21', 'SALA_3', 'MALE',   '09:00', '10:15', 'PER_MASTER_BENITES', 'PER_PIANIST_BRUSUTTI'),

-- Jueves 22/10: Sala 1 Varones (Pino / Romero), Sala 3 Mujeres (Benites / Villegas)
('CLS_22_S1', 'SCHED_2026_10_22', 'SALA_1', 'MALE',   '09:00', '10:15', 'PER_MASTER_PINO',    'PER_PIANIST_ROMERO'),
('CLS_22_S3', 'SCHED_2026_10_22', 'SALA_3', 'FEMALE', '09:00', '10:15', 'PER_MASTER_BENITES', 'PER_PIANIST_VILLEGAS'),

-- Viernes 23/10: Sala 1 Mujeres (Pino / Brusutti), Sala 3 Varones (Ananiev / Romero)
('CLS_23_S1', 'SCHED_2026_10_23', 'SALA_1', 'FEMALE', '09:00', '10:15', 'PER_MASTER_PINO',    'PER_PIANIST_BRUSUTTI'),
('CLS_23_S3', 'SCHED_2026_10_23', 'SALA_3', 'MALE',   '09:00', '10:15', 'PER_MASTER_ANANIEV', 'PER_PIANIST_ROMERO');

-- Ensayos según "PARTE DE ENSAYO – Semana 19 al 23 octubre"
-- Lunes 19/10
INSERT OR IGNORE INTO rehearsal_blocks (id, schedule_id, room_id, start_time, end_time, activity_type, work_id, passage_id, custom_passage_label, confirmation_state, sound_requirement, master_id, allow_shared_room) VALUES
('BLK_19_01', 'SCHED_2026_10_19', 'SALA_1', '10:30', '11:30', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_PRINCESAS_BUFON',     'Princesas y Bufón',  'CONFIRMED', '',                      NULL, 0),
('BLK_19_02', 'SCHED_2026_10_19', 'SALA_1', '11:30', '12:30', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_DANZA_ESPANOLA',      'Danza Española',     'CONFIRMED', 'Sonido TeatroCue #11',  'PER_MASTER_ANANIEV', 0),
('BLK_19_03', 'SCHED_2026_10_19', 'SALA_1', '12:30', '13:45', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_DANZA_NAPOLITANA',     'Danza Napolitana',   'CONFIRMED', '',                      'PER_MASTER_BENITES', 0),

-- Martes 20/10
('BLK_20_01', 'SCHED_2026_10_20', 'SALA_1', '10:30', '12:00', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_PDD_ODETTE_SIGF',     'Pas de deux Odette y Sigfrido', 'CONFIRMED', 'Sonido TeatroCue #08', 'PER_MASTER_BACA', 0),
('BLK_20_02', 'SCHED_2026_10_20', 'SALA_1', '12:00', '13:00', 'STUDIO_REHEARSAL', 'WORK_GALA_CLASICA','PAS_DIANA_ACTEON',        'Diana y Acteón',     'CONFIRMED', 'Sonido TeatroCue #21',  'PER_MASTER_ANANIEV', 0),

-- Miércoles 21/10
('BLK_21_01', 'SCHED_2026_10_21', 'SALA_1', '10:30', '11:30', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_DANZA_HUNGARA',       'Danza Húngara',      'CAST_TO_CONFIRM', 'Sonido TeatroCue #09', 'PER_MASTER_PINO', 0),
('BLK_21_02', 'SCHED_2026_10_21', 'SALA_1', '11:30', '12:00', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_PRINCESAS_BUFON',     'Princesas y Bufón',  'CONFIRMED', 'Sonido TeatroCue #04',  'PER_MASTER_BENITES', 0),
('BLK_21_03', 'SCHED_2026_10_21', 'SALA_1', '12:00', '13:30', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_CISNE_NEGRO',         'Pas de deux - Cisne Negro', 'CAST_TO_CONFIRM', 'Sonido TeatroCue #14', 'PER_MASTER_BACA', 0),

-- Jueves 22/10
('BLK_22_01', 'SCHED_2026_10_22', 'SALA_1', '10:30', '11:30', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_DANZA_MAZURCA',       'Danza Mazurca',      'CAST_TO_CONFIRM', 'Sonido TeatroCue #10', 'PER_MASTER_ANANIEV', 0),
('BLK_22_02', 'SCHED_2026_10_22', 'SALA_1', '11:30', '12:00', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_DANZA_NAPOLITANA',     'Danza Napolitana',   'CONFIRMED', '',                      'PER_MASTER_BENITES', 0),
('BLK_22_03', 'SCHED_2026_10_22', 'SALA_1', '12:00', '13:45', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', 'PAS_DANZA_HUNGARA',       'Danza Húngara y Danza Española', 'CONFIRMED', 'Sonido TeatroCue', 'PER_MASTER_PINO', 0),

-- Viernes 23/10
('BLK_23_01', 'SCHED_2026_10_23', 'SALA_1', '10:30', '13:45', 'STUDIO_REHEARSAL', 'WORK_LAGO_CISNES', NULL,                      'ENSAYO A CONFIRMAR', 'REHEARSAL_TO_CONFIRM', 'Sonido', 'PER_MASTER_BACA', 0);

-- Grupos de Elenco para los bloques
INSERT OR IGNORE INTO rehearsal_cast_groups (id, rehearsal_block_id, group_order, group_label, display_expression, is_to_confirm) VALUES
('CG_19_01', 'BLK_19_01', 1, 'Refuerzos', 'Refuerzos – Gacitúa – Beacon – Hazel – Martini', 0),
('CG_19_02', 'BLK_19_02', 1, 'Doble Elenco', 'Álvarez/Marcato – Morchio C/Gil', 0),
('CG_19_03', 'BLK_19_03', 1, 'Parejas', 'Pérez L/Natali – Pérez F/Paredes', 0),
('CG_20_01', 'BLK_20_01', 1, 'Solistas PDD', 'Morchio J/Bravo - Vera/Altamirano', 0),
('CG_20_02', 'BLK_20_02', 1, 'Diana y Acteón', 'González/Altamirano – Morchio C/Cornejo – Ragessi/Mauro', 0);
