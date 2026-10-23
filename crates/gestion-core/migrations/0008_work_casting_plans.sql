-- =============================================================================
-- MIGRACIÓN 0008: Planificación de Repartos para Obras (Casting & Role Matrix)
-- Ballet Oficial de Córdoba - Teatro del Libertador San Martín
-- =============================================================================

CREATE TABLE IF NOT EXISTS work_cast_plans (
    id TEXT PRIMARY KEY,
    work_id TEXT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    passage_id TEXT REFERENCES work_passages(id) ON DELETE CASCADE,
    tier_name TEXT NOT NULL, -- '1° Elenco', '2° Elenco', '3° Elenco', 'Covers / Suplentes', 'Cuerpo de Baile', 'Estudio'
    cast_label TEXT NOT NULL, -- 'Brenda Vera & Morchio C.', 'Julieta Álvarez & Nicolás Gil'
    default_sound_cue TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS work_cast_plan_members (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES work_cast_plans(id) ON DELETE CASCADE,
    personnel_id TEXT NOT NULL REFERENCES personnel(id) ON DELETE CASCADE,
    character_role TEXT, -- 'Odette / Cisne Blanco', 'Príncipe Sigfrido', 'Diana', 'Acteón', 'Kitri', 'Basilio', 'Princesa', 'Bufón'
    partner_personnel_id TEXT REFERENCES personnel(id) ON DELETE SET NULL,
    assignment_condition TEXT NOT NULL DEFAULT 'TITULAR', -- 'TITULAR', 'COVER', 'ALTERNATE_SLASH', 'REINFORCEMENT'
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE(plan_id, personnel_id, character_role)
);

CREATE INDEX IF NOT EXISTS idx_work_cast_plans_work ON work_cast_plans(work_id);
CREATE INDEX IF NOT EXISTS idx_work_cast_plans_passage ON work_cast_plans(passage_id);
CREATE INDEX IF NOT EXISTS idx_work_cast_members_plan ON work_cast_plan_members(plan_id);
CREATE INDEX IF NOT EXISTS idx_work_cast_members_dancer ON work_cast_plan_members(personnel_id);

-- Semilla de Repartos de Estudio y Elencos Oficiales

-- 1. EL LAGO DE LOS CISNES
-- 1.A Pas de deux Odette y Sigfrido (Acto II)
INSERT OR IGNORE INTO work_cast_plans (id, work_id, passage_id, tier_name, cast_label, default_sound_cue, notes) VALUES
    ('PLAN_LAGO_PDD_1', 'WORK_LAGO_CISNES', 'PAS_PDD_ODETTE_SIGF', '1° Elenco', 'Brenda Vera & Cristian Morchio C.', 'PISTA TeatroCue #08 / Piano', 'Primer elenco titular'),
    ('PLAN_LAGO_PDD_2', 'WORK_LAGO_CISNES', 'PAS_PDD_ODETTE_SIGF', '2° Elenco', 'Julieta Álvarez & Nicolás Gil', 'PISTA TeatroCue #08 / Piano', 'Segundo elenco alternante'),
    ('PLAN_LAGO_PDD_COV', 'WORK_LAGO_CISNES', 'PAS_PDD_ODETTE_SIGF', 'Covers / Suplentes', 'Lucía Marcato (Cover Odette)', 'PISTA TeatroCue #08 / Piano', 'Estudio y reemplazo');

INSERT OR IGNORE INTO work_cast_plan_members (id, plan_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order) VALUES
    ('MEM_LAGO_PDD_1_BRENDA',  'PLAN_LAGO_PDD_1',   'PER_DANCER_BRENDA',    'Odette (Cisne Blanco)',  'PER_DANCER_MORCHIO_C', 'TITULAR', 1),
    ('MEM_LAGO_PDD_1_MORCHIO', 'PLAN_LAGO_PDD_1',   'PER_DANCER_MORCHIO_C', 'Príncipe Sigfrido',      'PER_DANCER_BRENDA',    'TITULAR', 2),
    ('MEM_LAGO_PDD_2_ALVAREZ', 'PLAN_LAGO_PDD_2',   'PER_DANCER_ALVAREZ',   'Odette (Cisne Blanco)',  'PER_DANCER_GIL',       'TITULAR', 1),
    ('MEM_LAGO_PDD_2_GIL',     'PLAN_LAGO_PDD_2',   'PER_DANCER_GIL',       'Príncipe Sigfrido',      'PER_DANCER_ALVAREZ',   'TITULAR', 2),
    ('MEM_LAGO_PDD_COV_MARC',  'PLAN_LAGO_PDD_COV', 'PER_DANCER_MARCATO',   'Odette (Cisne Blanco)',  NULL,                   'COVER',   1);

-- 1.B Princesas y Bufón (Acto I / Acto III)
INSERT OR IGNORE INTO work_cast_plans (id, work_id, passage_id, tier_name, cast_label, default_sound_cue, notes) VALUES
    ('PLAN_LAGO_PRINCESAS_1', 'WORK_LAGO_CISNES', 'PAS_PRINCESAS_BUFON', '1° Elenco', 'Gómez (Popi), Marcato & Gil', 'PISTA TeatroCue #04 / Piano', 'Princesas y Bufón titular');

INSERT OR IGNORE INTO work_cast_plan_members (id, plan_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order) VALUES
    ('MEM_LAGO_PRIN_POPI', 'PLAN_LAGO_PRINCESAS_1', 'PER_DANCER_POPI',    'Princesa Candidata', NULL, 'TITULAR', 1),
    ('MEM_LAGO_PRIN_MARC', 'PLAN_LAGO_PRINCESAS_1', 'PER_DANCER_MARCATO', 'Princesa Candidata', NULL, 'TITULAR', 2),
    ('MEM_LAGO_PRIN_GIL',  'PLAN_LAGO_PRINCESAS_1', 'PER_DANCER_GIL',     'El Bufón',           NULL, 'TITULAR', 3);

-- 1.C Cisne Negro (Acto III)
INSERT OR IGNORE INTO work_cast_plans (id, work_id, passage_id, tier_name, cast_label, default_sound_cue, notes) VALUES
    ('PLAN_LAGO_NEGRO_1', 'WORK_LAGO_CISNES', 'PAS_CISNE_NEGRO', '1° Elenco', 'Julieta Álvarez & Cristian Morchio C.', 'PISTA TeatroCue #14 / Piano', 'Grand Pas Odile y Sigfrido'),
    ('PLAN_LAGO_NEGRO_2', 'WORK_LAGO_CISNES', 'PAS_CISNE_NEGRO', '2° Elenco', 'Brenda Vera & Nicolás Gil', 'PISTA TeatroCue #14 / Piano', 'Grand Pas Odile y Sigfrido alternante');

INSERT OR IGNORE INTO work_cast_plan_members (id, plan_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order) VALUES
    ('MEM_LAGO_NEG_1_ALVAREZ', 'PLAN_LAGO_NEGRO_1', 'PER_DANCER_ALVAREZ',   'Odile (Cisne Negro)', 'PER_DANCER_MORCHIO_C', 'TITULAR', 1),
    ('MEM_LAGO_NEG_1_MORCHIO', 'PLAN_LAGO_NEGRO_1', 'PER_DANCER_MORCHIO_C', 'Príncipe Sigfrido',   'PER_DANCER_ALVAREZ',   'TITULAR', 2),
    ('MEM_LAGO_NEG_2_BRENDA',  'PLAN_LAGO_NEGRO_2', 'PER_DANCER_BRENDA',    'Odile (Cisne Negro)', 'PER_DANCER_GIL',       'TITULAR', 1),
    ('MEM_LAGO_NEG_2_GIL',     'PLAN_LAGO_NEGRO_2', 'PER_DANCER_GIL',       'Príncipe Sigfrido',   'PER_DANCER_BRENDA',    'TITULAR', 2);

-- 2. DON QUIJOTE
-- Grand Pas de Deux Kitri y Basilio (Acto III)
INSERT OR IGNORE INTO work_cast_plans (id, work_id, passage_id, tier_name, cast_label, default_sound_cue, notes) VALUES
    ('PLAN_QUIJOTE_PDD_1', 'WORK_DON_QUIJOTE', 'PAS_KITRI_BASILIO', '1° Elenco', 'Sofía Gómez (Popi) & Nicolás Gil', 'PISTA TeatroCue #30', 'Kitri y Basilio 1er elenco'),
    ('PLAN_QUIJOTE_PDD_2', 'WORK_DON_QUIJOTE', 'PAS_KITRI_BASILIO', '2° Elenco', 'Brenda Vera & Cristian Morchio C.', 'PISTA TeatroCue #30', 'Kitri y Basilio 2do elenco');

INSERT OR IGNORE INTO work_cast_plan_members (id, plan_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order) VALUES
    ('MEM_QUIJOTE_1_POPI',    'PLAN_QUIJOTE_PDD_1', 'PER_DANCER_POPI',      'Kitri',   'PER_DANCER_GIL',       'TITULAR', 1),
    ('MEM_QUIJOTE_1_GIL',     'PLAN_QUIJOTE_PDD_1', 'PER_DANCER_GIL',       'Basilio', 'PER_DANCER_POPI',      'TITULAR', 2),
    ('MEM_QUIJOTE_2_BRENDA',  'PLAN_QUIJOTE_PDD_2', 'PER_DANCER_BRENDA',    'Kitri',   'PER_DANCER_MORCHIO_C', 'TITULAR', 1),
    ('MEM_QUIJOTE_2_MORCHIO', 'PLAN_QUIJOTE_PDD_2', 'PER_DANCER_MORCHIO_C', 'Basilio', 'PER_DANCER_BRENDA',    'TITULAR', 2);

-- 3. GALA DE BALLET CLÁSICO
-- Diana y Acteón
INSERT OR IGNORE INTO work_cast_plans (id, work_id, passage_id, tier_name, cast_label, default_sound_cue, notes) VALUES
    ('PLAN_DIANA_1', 'WORK_GALA_CLASICA', 'PAS_DIANA_ACTEON', '1° Elenco', 'Julieta Álvarez & Nicolás Gil', 'PISTA TeatroCue #21', 'Grand Pas de Deux');

INSERT OR IGNORE INTO work_cast_plan_members (id, plan_id, personnel_id, character_role, partner_personnel_id, assignment_condition, sort_order) VALUES
    ('MEM_DIANA_1_ALVAREZ', 'PLAN_DIANA_1', 'PER_DANCER_ALVAREZ', 'Diana',  'PER_DANCER_GIL',     'TITULAR', 1),
    ('MEM_DIANA_1_GIL',     'PLAN_DIANA_1', 'PER_DANCER_GIL',     'Acteón', 'PER_DANCER_ALVAREZ', 'TITULAR', 2);
