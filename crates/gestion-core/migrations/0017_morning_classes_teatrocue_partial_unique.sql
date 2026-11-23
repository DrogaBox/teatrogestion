-- =============================================================================
-- MIGRACIÓN 0017: Permitir Pista TeatroCue en múltiples clases matutinas el mismo día
-- Conserva la unicidad por sala, por maestro y por pianista humano real.
-- =============================================================================

CREATE TABLE morning_classes_v17 (
    id TEXT PRIMARY KEY,
    schedule_id TEXT NOT NULL REFERENCES daily_schedules(id) ON DELETE CASCADE,
    room_id TEXT NOT NULL REFERENCES rooms(id),
    gender_division TEXT NOT NULL CHECK (gender_division IN ('FEMALE', 'MALE', 'UNIFIED_SPECIAL')),
    start_time TEXT NOT NULL DEFAULT '09:00',
    end_time TEXT NOT NULL DEFAULT '10:15',
    master_id TEXT NOT NULL REFERENCES personnel(id),
    pianist_id TEXT NOT NULL REFERENCES personnel(id),
    notes TEXT,
    sound_tech_id TEXT REFERENCES personnel(id) ON DELETE SET NULL,
    UNIQUE(schedule_id, room_id),
    UNIQUE(schedule_id, master_id)
) STRICT;

INSERT INTO morning_classes_v17 (
    id, schedule_id, room_id, gender_division, start_time, end_time, master_id, pianist_id, notes, sound_tech_id
)
SELECT
    id, schedule_id, room_id, gender_division, start_time, end_time, master_id, pianist_id, notes, sound_tech_id
FROM morning_classes;

DROP TABLE morning_classes;
ALTER TABLE morning_classes_v17 RENAME TO morning_classes;

CREATE UNIQUE INDEX idx_morning_classes_schedule_human_pianist
    ON morning_classes(schedule_id, pianist_id)
    WHERE pianist_id != 'PER_SOUND_TEATROCUE';
