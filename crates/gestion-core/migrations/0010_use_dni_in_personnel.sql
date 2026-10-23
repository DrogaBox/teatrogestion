-- =============================================================================
-- MIGRACIÓN 0010: Regularización de DNI Nacional en lugar de Códigos de Legajo
-- =============================================================================

UPDATE personnel SET legajo = '36.412.890' WHERE id = 'PER_DANCER_BRENDA';
UPDATE personnel SET legajo = '37.215.643' WHERE id = 'PER_DANCER_PEREZ_FLOR';
UPDATE personnel SET legajo = '38.109.432' WHERE id = 'PER_DANCER_MORCHIO_JULI';
UPDATE personnel SET legajo = '35.890.124' WHERE id = 'PER_DANCER_PEREZ_LAU';
UPDATE personnel SET legajo = '34.789.012' WHERE id = 'PER_DANCER_MAURO_MATIAS';
UPDATE personnel SET legajo = '39.456.781' WHERE id = 'PER_DANCER_MORCHIO_CAMI';
UPDATE personnel SET legajo = '36.987.234' WHERE id = 'PER_DANCER_MARENCHINO';
UPDATE personnel SET legajo = '33.876.543' WHERE id = 'PER_DANCER_ALVAREZ';
UPDATE personnel SET legajo = '34.123.876' WHERE id = 'PER_DANCER_RAGESSI_MAR';
UPDATE personnel SET legajo = '35.654.321' WHERE id = 'PER_DANCER_CORNEJO_FACU';
UPDATE personnel SET legajo = '37.890.543' WHERE id = 'PER_DANCER_VALDEZ';
UPDATE personnel SET legajo = '38.901.234' WHERE id = 'PER_DANCER_GONZALEZ_SOF';
UPDATE personnel SET legajo = '38.567.123' WHERE id = 'PER_DANCER_POPI';
UPDATE personnel SET legajo = '36.123.456' WHERE id = 'PER_DANCER_MARCATO';
UPDATE personnel SET legajo = '32.901.234' WHERE id = 'PER_DANCER_MORCHIO_C';
UPDATE personnel SET legajo = '35.432.109' WHERE id = 'PER_DANCER_GIL';
UPDATE personnel SET legajo = '18.943.210' WHERE id = 'PER_MASTER_ANANIEV';
UPDATE personnel SET legajo = '28.654.321' WHERE id = 'PER_MASTER_BENITES';
UPDATE personnel SET legajo = '24.567.890' WHERE id = 'PER_MASTER_PINO';
UPDATE personnel SET legajo = '21.432.109' WHERE id = 'PER_MASTER_BACA';
UPDATE personnel SET legajo = '26.789.012' WHERE id = 'PER_PIANIST_ROMERO';
UPDATE personnel SET legajo = '29.876.543' WHERE id = 'PER_PIANIST_VILLEGAS';
UPDATE personnel SET legajo = '31.234.567' WHERE id = 'PER_PIANIST_BRUSUTTI';
UPDATE personnel SET legajo = '27.890.123' WHERE id = 'PER_INSPECTOR_MAIN';
UPDATE personnel SET legajo = '30.123.456' WHERE id = 'PER_SOUND_TEATROCUE';
