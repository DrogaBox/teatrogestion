-- Migración 0011: Reglas de Descanso entre Clases y Ensayos, y Francos Compensatorios por Función (Supervisora Carina Lion)
ALTER TABLE daily_schedules ADD COLUMN class_to_rehearsal_rest_minutes INTEGER NOT NULL DEFAULT 15;
ALTER TABLE daily_schedules ADD COLUMN is_compensatory_rest_day INTEGER NOT NULL DEFAULT 0;
ALTER TABLE daily_schedules ADD COLUMN compensatory_rest_policy TEXT NOT NULL DEFAULT 'NEXT_DAY';
