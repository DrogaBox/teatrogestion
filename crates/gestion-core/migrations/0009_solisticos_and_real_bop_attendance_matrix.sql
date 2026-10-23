-- =============================================================================
-- MIGRACIÓN 0009: Certificación Oficial de Solísticos (Plus por Rol)
-- y Matriz Mensual de Ausentismo Histórica de Inspección (Ballet Oficial de Córdoba)
-- =============================================================================

-- 1. REGULARIZACIÓN Y ALTA DEL PLANTEL REAL DEL BALLET OFICIAL
-- (Conforme a las planillas y notas oficiales de Carina Lion)

INSERT OR IGNORE INTO personnel (
    id, legajo, first_name, last_name, nickname, role, gender_group, category_id,
    default_pointe_raw_code, default_flat_size, is_active, notes
) VALUES
('PER_DANCER_BRENDA',        'BOC-102', 'Brenda Mercedes',    'Vera',       'Brenda',     'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'SK - 8.5M4HDL', '8C',   1, 'Primera Figura / Solista Paquita'),
('PER_DANCER_PEREZ_FLOR',     'BOC-125', 'María Florencia',    'Pérez',      'Florencia',  'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'SC - 7.5W4PDH', '7.5D', 1, 'Solista 1era Variación Paquita'),
('PER_DANCER_MORCHIO_JULI',   'BOC-124', 'Julieta',            'Morchio',    'Julieta',    'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'SC - 7.5W4PDH', '7.5D', 1, 'Solista 2da Variación Paquita & Tangos de Piazzolla'),
('PER_DANCER_PEREZ_LAU',      'BOC-126', 'María Laura',        'Pérez',      'Laura',      'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'SC - 7.5W4PDH', '7.5D', 1, 'Solista 3era Variación Paquita'),
('PER_DANCER_MAURO_MATIAS',   'BOC-130', 'Matías Gabriel',     'Mauro',      'Matías',     'DANCER', 'MALE',   'CAT_CUERPO_BAILE', NULL,            '12M',  1, 'Primer Bailarín Lucien d''Hervilly'),
('PER_DANCER_MORCHIO_CAMI',   'BOC-123', 'Camila',             'Morchio',    'Camila',     'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'CL - 8M3XDH',   '8C',   1, 'Solista Paquita'),
('PER_DANCER_MARENCHINO',     'BOC-122', 'María Belén',        'Marenchino', 'Belén',      'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'SC - 7.5W4PDH', '7D',   1, 'Solista 1era Variación Paquita & Tangos de Piazzolla'),
('PER_DANCER_ALVAREZ_MERC',   'BOC-103', 'María Mercedes',     'Álvarez',    'Mercedes',   'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'CL - 8M3XDH',   '7.5D', 1, 'Solista 2da Variación Paquita'),
('PER_DANCER_RAGESSI_MAR',    'BOC-127', 'Mariela Alejandra',  'Ragessi',    'Mariela',    'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'SC - 7.5W4PDH', '7.5D', 1, 'Solista 3era Variación Paquita'),
('PER_DANCER_CORNEJO_FACU',   'BOC-131', 'Facundo Miguel',     'Cornejo',    'Facundo',    'DANCER', 'MALE',   'CAT_CUERPO_BAILE', NULL,            '10.5M',1, 'Primer Bailarín Lucien d''Hervilly'),
('PER_DANCER_VALDEZ',         'BOC-128', 'Andrea',             'Valdez',     'Valdez',     'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'SC - 7.5W4PDH', '7D',   1, 'Cuerpo de Baile Estable'),
('PER_DANCER_GONZALEZ_SOF',   'BOC-129', 'Sofía',              'González',   'González S', 'DANCER', 'FEMALE', 'CAT_CUERPO_BAILE', 'SC - 7.5W4PDH', '7.5D', 1, 'Cuerpo de Baile Estable');

-- Actualizar si ya existían para garantizar nombres oficiales
UPDATE personnel SET first_name = 'Brenda Mercedes', last_name = 'Vera' WHERE id = 'PER_DANCER_BRENDA';
UPDATE personnel SET first_name = 'María Mercedes', last_name = 'Álvarez' WHERE id = 'PER_DANCER_ALVAREZ';

-- -----------------------------------------------------------------------------
-- 2. TABLAS DE CERTIFICACIÓN DE SOLÍSTICOS (PLUS POR PRESTACIÓN DE ROL)
-- -----------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS soloist_certifications (
    id TEXT PRIMARY KEY,
    program_title TEXT NOT NULL,
    hall_name TEXT NOT NULL DEFAULT 'Sala Mayor del Teatro del Libertador Gral. San Martín',
    document_date TEXT NOT NULL,
    recipient_name TEXT NOT NULL DEFAULT 'Hadrián Ávila Arzuza',
    recipient_title TEXT NOT NULL DEFAULT 'Director del Teatro del Libertador',
    intro_body TEXT NOT NULL DEFAULT '',
    closing_text TEXT NOT NULL DEFAULT 'Sin otro particular saludo a Ud. Atte.',
    status TEXT NOT NULL DEFAULT 'DRAFT', -- 'DRAFT', 'SUBMITTED', 'APPROVED'
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE TABLE IF NOT EXISTS soloist_certification_items (
    id TEXT PRIMARY KEY,
    certification_id TEXT NOT NULL REFERENCES soloist_certifications(id) ON DELETE CASCADE,
    personnel_id TEXT REFERENCES personnel(id) ON DELETE SET NULL,
    dancer_name TEXT NOT NULL,
    work_role TEXT NOT NULL,
    performance_dates TEXT NOT NULL,
    performance_count INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE INDEX IF NOT EXISTS idx_soloist_items_cert ON soloist_certification_items(certification_id);
CREATE INDEX IF NOT EXISTS idx_soloist_items_personnel ON soloist_certification_items(personnel_id);

-- -----------------------------------------------------------------------------
-- 3. DATOS SEMILLA: CERTIFICACIÓN REAL DE SOLÍSTICOS "ENTRE DOS MUNDOS" (OCT 2026)
-- -----------------------------------------------------------------------------

INSERT OR IGNORE INTO soloist_certifications (
    id, program_title, hall_name, document_date, recipient_name, recipient_title, intro_body, status, notes
) VALUES (
    'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS',
    'Entre dos Mundos',
    'Sala Mayor del Teatro del Libertador Gral. San Martín',
    '2026-10-05',
    'Hadrián Ávila Arzuza',
    'Director del Teatro del Libertador',
    'Me dirijo a Ud. a los fines de informarle los solísticos correspondientes al Ballet Oficial de la Pcia. con respecto a las funciones "Entre dos Mundos" programadas en la Sala Mayor del Teatro del Libertador Gral. San Martín los días 01, 02, 03 y 04 de octubre de 2026 se cumplió según el siguiente detalle:',
    'APPROVED',
    'Certificación oficial de 24 prestaciones solísticas de Entre dos Mundos (Suite Paquita y Tangos de Piazzolla).'
);

INSERT OR IGNORE INTO soloist_certification_items (
    id, certification_id, personnel_id, dancer_name, work_role, performance_dates, performance_count, sort_order
) VALUES
('ITEM_SOLO_01', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_BRENDA',         'VERA, Brenda Mercedes',       'Paquita "Suite Paquita"',                 '01 y 03/10', 2, 1),
('ITEM_SOLO_02', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_PEREZ_FLOR',      'PÉREZ, María Florencia',      '1era Variación "Suite Paquita"',          '01 y 03/10', 2, 2),
('ITEM_SOLO_03', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_MORCHIO_JULI',    'MORCHIO, Julieta',            '2da Variación "Suite Paquita"',           '01 y 03/10', 2, 3),
('ITEM_SOLO_04', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_PEREZ_LAU',       'PÉREZ, María Laura',          '3era Variación "Suite Paquita"',          '01 y 03/10', 2, 4),
('ITEM_SOLO_05', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_MAURO_MATIAS',    'MAURO, Matías Gabriel',       'Lucien d''Hervilly "Suite Paquita"',      '01 y 03/10', 2, 5),
('ITEM_SOLO_06', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_MORCHIO_CAMI',    'MORCHIO, Camila',             'Paquita "Suite Paquita"',                 '02 y 04/10', 2, 6),
('ITEM_SOLO_07', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_MARENCHINO',      'MARENCHINO, María Belén',     '1era Variación "Suite Paquita"',          '02 y 04/10', 2, 7),
('ITEM_SOLO_08', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_ALVAREZ_MERC',    'ALVAREZ, María Mercedes',     '2da Variación "Suite Paquita"',           '02 y 04/10', 2, 8),
('ITEM_SOLO_09', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_RAGESSI_MAR',     'RAGESSI, Mariela Alejandra',  '3era Variación "Suite Paquita"',          '02 y 04/10', 2, 9),
('ITEM_SOLO_10', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_CORNEJO_FACU',    'CORNEJO, Facundo Miguel',     'Lucien d''Hervilly "Suite Paquita"',      '02 y 04/10', 2, 10),
('ITEM_SOLO_11', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_MORCHIO_JULI',    'MORCHIO, Julieta',            'Tangos de Piazzolla',                     '01 y 03/10', 2, 11),
('ITEM_SOLO_12', 'CERT_SOLOIST_2026_ENTRE_DOS_MUNDOS', 'PER_DANCER_MARENCHINO',      'MARENCHINO, María Belén',     'Tangos de Piazzolla',                     '02 y 04/10', 2, 12);

-- -----------------------------------------------------------------------------
-- 4. SOPORTE DE ANOTACIONES TRANSVERSALES DE DÍA EN DAILY_SCHEDULES
-- -----------------------------------------------------------------------------

ALTER TABLE daily_schedules ADD COLUMN day_annotation TEXT;

-- Semilla para el 11 de Septiembre 2026 ("ENSAYO SUSPENDIDO")
INSERT OR IGNORE INTO daily_schedules (
    id, schedule_date, day_of_week, regime_type, mandatory_checkin_time, nominal_start_time, nominal_end_time,
    day_annotation, footer_legal_clause
) VALUES (
    'SCHED_2026_09_11', '2026-09-11', 5, 'ORDINARY', '08:45', '09:00', '13:45',
    'ENSAYO SUSPENDIDO', 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.'
);

UPDATE daily_schedules SET day_annotation = 'ENSAYO SUSPENDIDO' WHERE schedule_date = '2026-09-11';
