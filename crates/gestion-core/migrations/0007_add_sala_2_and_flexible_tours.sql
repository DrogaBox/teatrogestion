-- =============================================================================
-- MIGRACIÓN 0007: Incorporación de Sala 2, Salas de Gira y Flexibilidad de Horarios
-- Ballet Oficial de Córdoba - Teatro del Libertador San Martín
-- =============================================================================

PRAGMA foreign_keys = OFF;

-- 1. Incorporar Sala 2 y Salas de Gira Oficiales
INSERT OR IGNORE INTO rooms (id, code, display_name, is_stage, sort_order) VALUES
    ('SALA_2',         'SALA_2',         'Sala 2',                     0, 2),
    ('GIRA_TRASLADO',  'GIRA_TRASLADO',  '🚌 Traslado / Salida de Gira', 0, 10),
    ('GIRA_ESCENARIO', 'GIRA_ESCENARIO', '🎭 Escenario de Gira',       1, 11),
    ('GIRA_SALA',      'GIRA_SALA',      '🩰 Sala de Ensayo de Gira',   0, 12);

UPDATE rooms SET sort_order = 1 WHERE id = 'SALA_1';
UPDATE rooms SET sort_order = 2 WHERE id = 'SALA_2';
UPDATE rooms SET sort_order = 3 WHERE id = 'SALA_3';
UPDATE rooms SET sort_order = 4 WHERE id = 'ESCENARIO';
UPDATE rooms SET sort_order = 5 WHERE id = 'SALA_AUX';

-- 2. Flexibilizar Clases Matutinas (sin chequeo rígido de 09:00 o 10:15)
CREATE TABLE morning_classes_new (
    id TEXT PRIMARY KEY,
    schedule_id TEXT NOT NULL REFERENCES daily_schedules(id) ON DELETE CASCADE,
    room_id TEXT NOT NULL REFERENCES rooms(id),
    gender_division TEXT NOT NULL CHECK (gender_division IN ('FEMALE', 'MALE', 'UNIFIED_SPECIAL')),
    start_time TEXT NOT NULL DEFAULT '09:00',
    end_time TEXT NOT NULL DEFAULT '10:15',
    master_id TEXT NOT NULL REFERENCES personnel(id),
    pianist_id TEXT NOT NULL REFERENCES personnel(id),
    notes TEXT,
    UNIQUE(schedule_id, room_id),
    UNIQUE(schedule_id, master_id),
    UNIQUE(schedule_id, pianist_id)
) STRICT;

INSERT INTO morning_classes_new SELECT * FROM morning_classes;
DROP TABLE morning_classes;
ALTER TABLE morning_classes_new RENAME TO morning_classes;

-- 3. Flexibilizar Bloques de Ensayo y Logística de Gira
CREATE TABLE rehearsal_blocks_new (
    id TEXT PRIMARY KEY,
    schedule_id TEXT NOT NULL REFERENCES daily_schedules(id) ON DELETE CASCADE,
    room_id TEXT NOT NULL REFERENCES rooms(id),
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    activity_type TEXT NOT NULL DEFAULT 'STUDIO_REHEARSAL' CHECK (
        activity_type IN (
            'STUDIO_REHEARSAL',
            'STAGE_TECH_REHEARSAL',
            'STAGE_LIGHTING_REHEARSAL',
            'STAGE_ORCHESTRA_REHEARSAL',
            'DRESS_REHEARSAL',
            'PERFORMANCE',
            'TOUR_TRAVEL',
            'TOUR_STAGE_REHEARSAL',
            'CLASS_MORNING',
            'COMPANY_BREAK'
        )
    ),
    work_id TEXT REFERENCES works(id) ON DELETE SET NULL,
    passage_id TEXT REFERENCES work_passages(id) ON DELETE SET NULL,
    custom_passage_label TEXT,
    confirmation_state TEXT NOT NULL DEFAULT 'CONFIRMED' CHECK (
        confirmation_state IN ('CONFIRMED', 'REHEARSAL_TO_CONFIRM', 'CAST_TO_CONFIRM')
    ),
    sound_requirement TEXT NOT NULL DEFAULT 'PIANO',
    teatrocue_cue_ref TEXT,
    master_id TEXT REFERENCES personnel(id),
    pianist_id TEXT REFERENCES personnel(id),
    allow_shared_room INTEGER NOT NULL DEFAULT 0 CHECK (allow_shared_room IN (0, 1)),
    notes TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    CHECK (start_time < end_time)
) STRICT;

INSERT INTO rehearsal_blocks_new SELECT * FROM rehearsal_blocks;
DROP TABLE rehearsal_blocks;
ALTER TABLE rehearsal_blocks_new RENAME TO rehearsal_blocks;
CREATE INDEX idx_rehearsal_blocks_schedule ON rehearsal_blocks(schedule_id, start_time, end_time);
CREATE INDEX idx_rehearsal_blocks_room ON rehearsal_blocks(schedule_id, room_id);

PRAGMA foreign_keys = ON;
