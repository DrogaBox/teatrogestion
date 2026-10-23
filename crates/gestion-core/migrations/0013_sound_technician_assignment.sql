-- Responsable de sonido por clase, ensayo y función.
-- Se guarda como referencia al personal con rol SOUND_TECH; nulo cuando no hay sonido asignado.
ALTER TABLE morning_classes ADD COLUMN sound_tech_id TEXT REFERENCES personnel(id) ON DELETE SET NULL;
ALTER TABLE rehearsal_blocks ADD COLUMN sound_tech_id TEXT REFERENCES personnel(id) ON DELETE SET NULL;
