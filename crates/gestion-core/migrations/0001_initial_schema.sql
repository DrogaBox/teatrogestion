-- =============================================================================
-- TEATROGESTIÓN - ESQUEMA RELACIONAL MAESTRO (SQLite 3.45+ STRICT)
-- Archivo: 0001_initial_schema.sql
-- Institución: Ballet Oficial de Córdoba — Teatro del Libertador San Martín
-- =============================================================================

CREATE TABLE IF NOT EXISTS _schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    checksum_sha256 TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

-- -----------------------------------------------------------------------------
-- 1. PERSONAL, CATEGORÍAS DE REVISTA Y CUPOS ANUALES
-- -----------------------------------------------------------------------------

CREATE TABLE personnel_categories (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE, -- 'PRIMERA_FIGURA', 'SOLISTA', 'CUERPO_BAILE', 'CONTRATADO', 'STAFF_DOCENTE'
    name TEXT NOT NULL,
    annual_pointe_quota INTEGER NOT NULL DEFAULT 0 CHECK (annual_pointe_quota >= 0),
    annual_flat_quota INTEGER NOT NULL DEFAULT 0 CHECK (annual_flat_quota >= 0),
    annual_tights_quota INTEGER NOT NULL DEFAULT 0 CHECK (annual_tights_quota >= 0),
    annual_supporter_quota INTEGER NOT NULL DEFAULT 0 CHECK (annual_supporter_quota >= 0),
    annual_briefs_quota INTEGER NOT NULL DEFAULT 0 CHECK (annual_briefs_quota >= 0),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE TABLE personnel (
    id TEXT PRIMARY KEY,
    legajo TEXT UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    nickname TEXT, -- Ej: 'Popi', 'Brenda'
    role TEXT NOT NULL CHECK (role IN ('DANCER', 'MASTER', 'PIANIST', 'INSPECTOR', 'SOUND_TECH')),
    gender_group TEXT NOT NULL CHECK (gender_group IN ('FEMALE', 'MALE', 'NON_BINARY', 'NOT_APPLICABLE')),
    category_id TEXT NOT NULL REFERENCES personnel_categories(id) ON UPDATE CASCADE,
    phone_whatsapp TEXT,
    default_pointe_raw_code TEXT, -- Código habitual Gaynor/Punta ej. 'SC - 7.5W4PDH'
    default_flat_size TEXT,       -- Talle habitual ej. '7.5D' o '10M'
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE INDEX idx_personnel_role_active ON personnel(role, is_active);
CREATE INDEX idx_personnel_search ON personnel(last_name, first_name, nickname);
CREATE INDEX idx_personnel_nickname ON personnel(nickname);

CREATE TABLE annual_quota_overrides (
    id TEXT PRIMARY KEY,
    personnel_id TEXT NOT NULL REFERENCES personnel(id) ON DELETE CASCADE,
    season_year INTEGER NOT NULL CHECK (season_year >= 2020 AND season_year <= 2100),
    quota_bucket TEXT NOT NULL CHECK (quota_bucket IN ('POINTE', 'FLAT', 'TIGHTS', 'SUPPORTER', 'BRIEFS')),
    extra_authorized_qty INTEGER NOT NULL DEFAULT 0,
    authorized_by TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(personnel_id, season_year, quota_bucket)
) STRICT;

-- -----------------------------------------------------------------------------
-- 2. SALAS, REPERTORIO DE OBRAS Y PASAJES
-- -----------------------------------------------------------------------------

CREATE TABLE rooms (
    id TEXT PRIMARY KEY, -- 'SALA_1', 'SALA_3', 'ESCENARIO', 'SALA_AUX'
    code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL, -- 'Sala 1', 'Sala 3', 'Escenario'
    is_stage INTEGER NOT NULL DEFAULT 0 CHECK (is_stage IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE works (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL UNIQUE,
    composer TEXT,
    choreographer TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

CREATE TABLE work_passages (
    id TEXT PRIMARY KEY,
    work_id TEXT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    name TEXT NOT NULL, -- Ej: 'Princesas y Bufón', 'Danza Española', 'Pas de deux Odette y Sigfrido', 'Diana y Acteón', 'Cisne Negro'
    act_scene TEXT,
    default_duration_min INTEGER NOT NULL DEFAULT 60,
    default_sound_cue TEXT, -- Columna SONIDO por defecto para cabina TeatroCue
    UNIQUE(work_id, name)
) STRICT;

-- -----------------------------------------------------------------------------
-- 3. CRONOGRAMA DIARIO, RÉGIMEN LABORAL, CLASES Y PARTES DE ENSAYO
-- -----------------------------------------------------------------------------

CREATE TABLE daily_schedules (
    id TEXT PRIMARY KEY,
    schedule_date TEXT NOT NULL UNIQUE, -- Formato ISO 'YYYY-MM-DD'
    day_of_week INTEGER NOT NULL CHECK (day_of_week BETWEEN 1 AND 7), -- 1=Lunes .. 7=Domingo
    regime_type TEXT NOT NULL DEFAULT 'ORDINARY' CHECK (
        regime_type IN ('ORDINARY', 'PRE_FUNCTION', 'EXTRAORDINARY')
    ),
    mandatory_checkin_time TEXT NOT NULL DEFAULT '08:45',
    nominal_start_time TEXT NOT NULL DEFAULT '09:00',
    nominal_end_time TEXT NOT NULL DEFAULT '13:45',
    minimum_intershift_rest_hours INTEGER NOT NULL DEFAULT 12,
    footer_legal_clause TEXT NOT NULL DEFAULT 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.',
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'PUBLISHED', 'CLOSED')),
    general_notes TEXT,
    created_by TEXT REFERENCES personnel(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (
        (regime_type = 'ORDINARY' AND nominal_end_time <= '13:45') OR
        (regime_type = 'PRE_FUNCTION' AND nominal_end_time <= '14:45') OR
        (regime_type = 'EXTRAORDINARY')
    )
) STRICT;

CREATE INDEX idx_daily_schedules_date ON daily_schedules(schedule_date);

-- Fichaje / Marcada diaria obligatoria (08:45 h Lunes a Viernes)
CREATE TABLE attendance_logs (
    id TEXT PRIMARY KEY,
    schedule_id TEXT NOT NULL REFERENCES daily_schedules(id) ON DELETE CASCADE,
    personnel_id TEXT NOT NULL REFERENCES personnel(id) ON DELETE RESTRICT,
    expected_checkin_time TEXT NOT NULL DEFAULT '08:45',
    actual_checkin_time TEXT,
    attendance_status TEXT NOT NULL DEFAULT 'PENDING' CHECK (
        attendance_status IN ('PENDING', 'ON_TIME', 'LATE', 'ABSENT_JUSTIFIED', 'MEDICAL_LEAVE', 'TOUR_LICENSE', 'ABSENT_UNJUSTIFIED')
    ),
    minutes_late INTEGER NOT NULL DEFAULT 0,
    remarks TEXT,
    recorded_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(schedule_id, personnel_id)
) STRICT;

-- Clase Técnica Matutina: 09:00 a 10:15 h (Invariable, simultánea en Sala 1 y Sala 3, dividida en Mujeres y Varones)
CREATE TABLE morning_classes (
    id TEXT PRIMARY KEY,
    schedule_id TEXT NOT NULL REFERENCES daily_schedules(id) ON DELETE CASCADE,
    room_id TEXT NOT NULL REFERENCES rooms(id),
    gender_division TEXT NOT NULL CHECK (gender_division IN ('FEMALE', 'MALE', 'UNIFIED_SPECIAL')),
    start_time TEXT NOT NULL DEFAULT '09:00' CHECK (start_time = '09:00'),
    end_time TEXT NOT NULL DEFAULT '10:15' CHECK (end_time = '10:15'),
    master_id TEXT NOT NULL REFERENCES personnel(id),  -- Rotación: Ananiev, Benites, Pino
    pianist_id TEXT NOT NULL REFERENCES personnel(id), -- Rotación: Romero, Villegas, Brusutti
    notes TEXT,
    UNIQUE(schedule_id, room_id),
    UNIQUE(schedule_id, master_id),
    UNIQUE(schedule_id, pianist_id)
) STRICT;

-- Bloques de Ensayos y Funciones
CREATE TABLE rehearsal_blocks (
    id TEXT PRIMARY KEY,
    schedule_id TEXT NOT NULL REFERENCES daily_schedules(id) ON DELETE CASCADE,
    room_id TEXT NOT NULL REFERENCES rooms(id),
    start_time TEXT NOT NULL, -- 'HH:MM' ej. '10:30', '11:30', '12:30'
    end_time TEXT NOT NULL,   -- 'HH:MM' ej. '11:30', '12:30', '13:45'
    activity_type TEXT NOT NULL DEFAULT 'STUDIO_REHEARSAL' CHECK (
        activity_type IN (
            'STUDIO_REHEARSAL',
            'STAGE_TECH_REHEARSAL',
            'STAGE_LIGHTING_REHEARSAL',
            'STAGE_ORCHESTRA_REHEARSAL',
            'DRESS_REHEARSAL',
            'PERFORMANCE'
        )
    ),
    work_id TEXT REFERENCES works(id) ON DELETE SET NULL,
    passage_id TEXT REFERENCES work_passages(id) ON DELETE SET NULL,
    custom_passage_label TEXT,
    confirmation_state TEXT NOT NULL DEFAULT 'CONFIRMED' CHECK (
        confirmation_state IN ('CONFIRMED', 'REHEARSAL_TO_CONFIRM', 'CAST_TO_CONFIRM')
    ),
    sound_requirement TEXT NOT NULL DEFAULT 'PIANO', -- Columna 'SONIDO' para Cabina TeatroCue
    teatrocue_cue_ref TEXT,
    master_id TEXT REFERENCES personnel(id),
    pianist_id TEXT REFERENCES personnel(id),
    allow_shared_room INTEGER NOT NULL DEFAULT 0 CHECK (allow_shared_room IN (0, 1)),
    notes TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    CHECK (start_time < end_time)
) STRICT;

CREATE INDEX idx_rehearsal_blocks_schedule ON rehearsal_blocks(schedule_id, start_time, end_time);
CREATE INDEX idx_rehearsal_blocks_room ON rehearsal_blocks(schedule_id, room_id);

-- Sub-grupos de reparto dentro de un bloque (Ej. 1er Elenco, Parejas dobles, Refuerzos)
CREATE TABLE rehearsal_cast_groups (
    id TEXT PRIMARY KEY,
    rehearsal_block_id TEXT NOT NULL REFERENCES rehearsal_blocks(id) ON DELETE CASCADE,
    group_order INTEGER NOT NULL DEFAULT 1,
    group_label TEXT,
    display_expression TEXT, -- Ej: 'Álvarez/Marcato - Morchio C/Gil'
    is_to_confirm INTEGER NOT NULL DEFAULT 0 CHECK (is_to_confirm IN (0, 1))
) STRICT;

-- Miembros nominales asignados a un grupo de reparto
CREATE TABLE rehearsal_cast_members (
    id TEXT PRIMARY KEY,
    cast_group_id TEXT NOT NULL REFERENCES rehearsal_cast_groups(id) ON DELETE CASCADE,
    personnel_id TEXT NOT NULL REFERENCES personnel(id) ON DELETE RESTRICT,
    character_role TEXT,
    partner_personnel_id TEXT REFERENCES personnel(id) ON DELETE SET NULL,
    assignment_condition TEXT NOT NULL DEFAULT 'TITULAR' CHECK (
        assignment_condition IN ('TITULAR', 'ALTERNATE_SLASH', 'REINFORCEMENT', 'COVER', 'TO_CONFIRM')
    ),
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE(cast_group_id, personnel_id, character_role)
) STRICT;

CREATE INDEX idx_cast_members_personnel ON rehearsal_cast_members(personnel_id);
CREATE INDEX idx_cast_members_group ON rehearsal_cast_members(cast_group_id);

-- -----------------------------------------------------------------------------
-- 4. MÓDULO DE STOCK DE CALZADO E INDUMENTARIA (TeatroStock)
-- -----------------------------------------------------------------------------

CREATE TABLE stock_items (
    id TEXT PRIMARY KEY,
    sku_code TEXT NOT NULL UNIQUE,
    barcode TEXT UNIQUE,
    family TEXT NOT NULL CHECK (
        family IN ('MENS_FLAT', 'WOMENS_FLAT', 'POINTE', 'APPAREL')
    ),
    subcategory TEXT NOT NULL CHECK (
        subcategory IN (
            'MEDIA_PUNTA_VARON',
            'MEDIA_PUNTA_MUJER_UNISEX',
            'PUNTA_GAYNOR_MINDEN',
            'PUNTA_BLOCH',
            'PUNTA_SANSHA_DUVAL',
            'SUSPENSOR_MASCULINO',
            'MEDIA_MALLA',
            'CANCAN',
            'BOMBACHA_ESCENA'
        )
    ),
    quota_bucket TEXT NOT NULL CHECK (
        quota_bucket IN ('POINTE', 'FLAT', 'TIGHTS', 'SUPPORTER', 'BRIEFS', 'EXEMPT')
    ),
    brand TEXT NOT NULL CHECK (
        brand IN ('Sansha', 'Capezio', 'HD', 'Bloch', 'Gaynor Minden', 'Sansha Duval', 'Fouette', 'Silvana')
    ),
    model_name TEXT NOT NULL,
    raw_factory_code TEXT, -- Ej: 'SC - 7.5W4PDH', 'SK - 8.5M4HDL', 'CL - 8M3XDH'
    color TEXT NOT NULL CHECK (
        color IN ('Blanco', 'Negro', 'Piel', 'Rosa', 'Rojas', 'Salmón')
    ),
    material TEXT NOT NULL CHECK (
        material IN ('CUERO', 'TELA_ELASTICA', 'CANVAS', 'SATEN', 'SATEN_VEGAN_LYRA', 'ALGODON_LYCRA', 'MICROFIBRA', 'RED_FISHNET')
    ),
    prop_dye_state TEXT NOT NULL DEFAULT 'ORIGINAL' CHECK (
        prop_dye_state IN ('ORIGINAL', 'TENIDA')
    ),
    dyed_for_work_id TEXT REFERENCES works(id) ON DELETE SET NULL,
    size_system TEXT NOT NULL CHECK (
        size_system IN ('US', 'EU', 'ALPHA')
    ),
    size_value TEXT NOT NULL,
    width_value TEXT,

    -- Atributos específicos de Puntas Gaynor Minden
    gm_line TEXT CHECK (gm_line IS NULL OR gm_line IN ('EUROPEAN_TRIANGLE', 'LYRA_CIRCLE', 'AMERICAN_STAR')),
    gm_fit TEXT CHECK (gm_fit IS NULL OR gm_fit IN ('SCULPTED', 'CLASSIC', 'SLEEK')),
    gm_box INTEGER CHECK (gm_box IS NULL OR gm_box IN (2, 3, 4, 5)),
    gm_shank TEXT CHECK (gm_shank IS NULL OR gm_shank IN ('EXTRAFLEX', 'SUPPLE', 'FEATHER', 'PIANISSIMO', 'HARD')),
    gm_vamp TEXT CHECK (gm_vamp IS NULL OR gm_vamp IN ('LOW', 'DEEP', 'DEEP_PLUS_6MM', 'DEEP_PLUS_HALF_INCH')),
    gm_heel TEXT CHECK (gm_heel IS NULL OR gm_heel IN ('LOW', 'HIGH')),

    -- Atributos específicos de Puntas Sansha Duval 4.0 Americanas
    duval_hardness TEXT CHECK (duval_hardness IS NULL OR duval_hardness IN ('FLEX', 'REGULAR', 'STRONG')),

    -- Cantidades e Invariantes de Depósito
    physical_qty INTEGER NOT NULL DEFAULT 0 CHECK (physical_qty >= 0),
    reserved_qty INTEGER NOT NULL DEFAULT 0 CHECK (reserved_qty >= 0 AND reserved_qty <= physical_qty),
    min_alert_qty INTEGER NOT NULL DEFAULT 2 CHECK (min_alert_qty >= 0),
    storage_bin_location TEXT,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (
        subcategory != 'PUNTA_GAYNOR_MINDEN' OR (
            gm_line IS NOT NULL AND
            gm_fit IS NOT NULL AND
            gm_box IS NOT NULL AND
            gm_shank IS NOT NULL AND
            gm_vamp IS NOT NULL AND
            gm_heel IS NOT NULL
        )
    ),
    CHECK (
        subcategory != 'PUNTA_SANSHA_DUVAL' OR duval_hardness IS NOT NULL
    )
) STRICT;

CREATE INDEX idx_stock_items_subcat_brand ON stock_items(subcategory, brand, size_value, width_value);
CREATE INDEX idx_stock_items_gm_lookup ON stock_items(gm_line, gm_fit, size_value, width_value, gm_box, gm_shank);
CREATE INDEX idx_stock_items_raw_code ON stock_items(raw_factory_code);

-- Reservas Nominativas Previas (ej. reservas para 'Popi' o 'Brenda')
CREATE TABLE stock_reservations (
    id TEXT PRIMARY KEY,
    stock_item_id TEXT NOT NULL REFERENCES stock_items(id) ON DELETE RESTRICT,
    personnel_id TEXT NOT NULL REFERENCES personnel(id) ON DELETE RESTRICT,
    reserved_for_label TEXT NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (
        status IN ('ACTIVE', 'DELIVERED', 'RELEASED')
    ),
    work_id TEXT REFERENCES works(id) ON DELETE SET NULL,
    notes TEXT,
    created_by TEXT REFERENCES personnel(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    resolved_at TEXT
) STRICT;

CREATE INDEX idx_reservations_personnel_active ON stock_reservations(personnel_id, status);
CREATE INDEX idx_reservations_item_active ON stock_reservations(stock_item_id, status);

-- Cabecera de Valera / Entrega Nominal
CREATE TABLE voucher_deliveries (
    id TEXT PRIMARY KEY,
    voucher_number TEXT NOT NULL UNIQUE,
    personnel_id TEXT NOT NULL REFERENCES personnel(id) ON DELETE RESTRICT,
    season_year INTEGER NOT NULL CHECK (season_year >= 2020 AND season_year <= 2100),
    delivered_by_inspector_id TEXT NOT NULL REFERENCES personnel(id) ON DELETE RESTRICT,
    delivery_timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    is_quota_exception INTEGER NOT NULL DEFAULT 0 CHECK (is_quota_exception IN (0, 1)),
    exception_reason TEXT,
    device_id TEXT NOT NULL,
    signature_svg_strokes TEXT,
    notes TEXT,
    CHECK (is_quota_exception = 0 OR (exception_reason IS NOT NULL AND length(trim(exception_reason)) >= 5))
) STRICT;

CREATE INDEX idx_voucher_deliveries_personnel_year ON voucher_deliveries(personnel_id, season_year);

-- Detalle de Ítems Entregados en la Valera
CREATE TABLE voucher_delivery_items (
    id TEXT PRIMARY KEY,
    voucher_id TEXT NOT NULL REFERENCES voucher_deliveries(id) ON DELETE RESTRICT,
    stock_item_id TEXT NOT NULL REFERENCES stock_items(id) ON DELETE RESTRICT,
    reservation_id TEXT REFERENCES stock_reservations(id) ON DELETE RESTRICT,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    quota_bucket TEXT NOT NULL CHECK (
        quota_bucket IN ('POINTE', 'FLAT', 'TIGHTS', 'SUPPORTER', 'BRIEFS', 'EXEMPT')
    )
) STRICT;

CREATE INDEX idx_voucher_items_voucher ON voucher_delivery_items(voucher_id);

-- Libro Mayor Inmutable de Movimientos de Stock (Kardex)
CREATE TABLE stock_ledger (
    id TEXT PRIMARY KEY,
    stock_item_id TEXT NOT NULL REFERENCES stock_items(id) ON DELETE RESTRICT,
    movement_type TEXT NOT NULL CHECK (
        movement_type IN (
            'INITIAL_INVENTORY',
            'PURCHASE_INBOUND',
            'PROP_DYE_TRANSFORM',
            'RESERVATION_LOCK',
            'RESERVATION_RELEASE',
            'VOUCHER_DELIVERY_OUT',
            'AUDIT_ADJUSTMENT'
        )
    ),
    delta_physical INTEGER NOT NULL DEFAULT 0,
    delta_reserved INTEGER NOT NULL DEFAULT 0,
    balance_physical_after INTEGER NOT NULL,
    balance_reserved_after INTEGER NOT NULL,
    reference_id TEXT,
    performed_by TEXT REFERENCES personnel(id),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE INDEX idx_stock_ledger_item_date ON stock_ledger(stock_item_id, created_at);

-- -----------------------------------------------------------------------------
-- 5. SINCRONIZACIÓN P2P LAN (HLC OP-LOG)
-- -----------------------------------------------------------------------------

CREATE TABLE sync_oplog (
    hlc_timestamp TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    entity_table TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    operation TEXT NOT NULL CHECK (operation IN ('UPSERT', 'DELETE', 'TRANSACTION_BUNDLE')),
    payload_json TEXT NOT NULL,
    blake3_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE TABLE sync_peers (
    node_id TEXT PRIMARY KEY,
    device_name TEXT NOT NULL,
    last_ip_addr TEXT,
    last_synced_hlc TEXT,
    last_seen_at TEXT
) STRICT;

-- -----------------------------------------------------------------------------
-- 6. TRIGGERS DE INTEGRIDAD TRANSACCIONAL Y AUDITORÍA AUTOMÁTICA
-- -----------------------------------------------------------------------------

CREATE TRIGGER trg_stock_reservation_before_insert
BEFORE INSERT ON stock_reservations
WHEN NEW.status = 'ACTIVE'
BEGIN
    SELECT CASE
        WHEN (SELECT (physical_qty - reserved_qty) FROM stock_items WHERE id = NEW.stock_item_id) < NEW.quantity
        THEN RAISE(ABORT, 'STOCK_INSUFICIENTE_PARA_RESERVA: No hay suficientes unidades libres en depósito.')
    END;
END;

CREATE TRIGGER trg_stock_reservation_after_insert
AFTER INSERT ON stock_reservations
WHEN NEW.status = 'ACTIVE'
BEGIN
    UPDATE stock_items
    SET reserved_qty = reserved_qty + NEW.quantity,
        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = NEW.stock_item_id;
END;

CREATE TRIGGER trg_stock_reservation_release
AFTER UPDATE OF status ON stock_reservations
WHEN OLD.status = 'ACTIVE' AND NEW.status = 'RELEASED'
BEGIN
    UPDATE stock_items
    SET reserved_qty = reserved_qty - OLD.quantity,
        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = OLD.stock_item_id;
END;

CREATE TRIGGER trg_prevent_voucher_delete
BEFORE DELETE ON voucher_deliveries
BEGIN
    SELECT RAISE(ABORT, 'OPERACION_PROHIBIDA: Las entregas de valera son inmutables y no pueden eliminarse.');
END;

CREATE TRIGGER trg_prevent_voucher_item_delete
BEFORE DELETE ON voucher_delivery_items
BEGIN
    SELECT RAISE(ABORT, 'OPERACION_PROHIBIDA: Los ítems de valera son inmutables y no pueden eliminarse.');
END;

-- -----------------------------------------------------------------------------
-- 7. VISTAS OPERATIVAS (SALDOS DE CUPO ANUAL Y STOCK DISPONIBLE)
-- -----------------------------------------------------------------------------

CREATE VIEW v_stock_availability AS
SELECT
    s.id,
    s.sku_code,
    s.family,
    s.subcategory,
    s.quota_bucket,
    s.brand,
    s.model_name,
    s.raw_factory_code,
    s.color,
    s.material,
    s.prop_dye_state,
    s.size_system,
    s.size_value,
    s.width_value,
    s.gm_line,
    s.gm_fit,
    s.gm_box,
    s.gm_shank,
    s.gm_vamp,
    s.gm_heel,
    s.duval_hardness,
    s.physical_qty,
    s.reserved_qty,
    (s.physical_qty - s.reserved_qty) AS free_available_qty,
    s.storage_bin_location,
    CASE WHEN (s.physical_qty - s.reserved_qty) <= s.min_alert_qty THEN 1 ELSE 0 END AS is_low_stock
FROM stock_items s;

CREATE VIEW v_dancer_annual_quota_balance AS
SELECT
    p.id AS personnel_id,
    p.legajo,
    p.last_name,
    p.first_name,
    p.nickname,
    c.code AS category_code,
    c.name AS category_name,
    CAST(strftime('%Y', 'now') AS INTEGER) AS current_season_year,
    (c.annual_pointe_quota + COALESCE((SELECT extra_authorized_qty FROM annual_quota_overrides o WHERE o.personnel_id = p.id AND o.season_year = CAST(strftime('%Y', 'now') AS INTEGER) AND o.quota_bucket = 'POINTE'), 0)) AS effective_pointe_quota,
    (c.annual_flat_quota + COALESCE((SELECT extra_authorized_qty FROM annual_quota_overrides o WHERE o.personnel_id = p.id AND o.season_year = CAST(strftime('%Y', 'now') AS INTEGER) AND o.quota_bucket = 'FLAT'), 0)) AS effective_flat_quota,
    (c.annual_tights_quota + COALESCE((SELECT extra_authorized_qty FROM annual_quota_overrides o WHERE o.personnel_id = p.id AND o.season_year = CAST(strftime('%Y', 'now') AS INTEGER) AND o.quota_bucket = 'TIGHTS'), 0)) AS effective_tights_quota,
    (c.annual_supporter_quota + COALESCE((SELECT extra_authorized_qty FROM annual_quota_overrides o WHERE o.personnel_id = p.id AND o.season_year = CAST(strftime('%Y', 'now') AS INTEGER) AND o.quota_bucket = 'SUPPORTER'), 0)) AS effective_supporter_quota,
    (c.annual_briefs_quota + COALESCE((SELECT extra_authorized_qty FROM annual_quota_overrides o WHERE o.personnel_id = p.id AND o.season_year = CAST(strftime('%Y', 'now') AS INTEGER) AND o.quota_bucket = 'BRIEFS'), 0)) AS effective_briefs_quota,
    COALESCE((
        SELECT SUM(vdi.quantity)
        FROM voucher_delivery_items vdi
        JOIN voucher_deliveries vd ON vd.id = vdi.voucher_id
        WHERE vd.personnel_id = p.id
          AND vd.season_year = CAST(strftime('%Y', 'now') AS INTEGER)
          AND vdi.quota_bucket = 'POINTE'
    ), 0) AS consumed_pointe_qty,
    COALESCE((
        SELECT SUM(vdi.quantity)
        FROM voucher_delivery_items vdi
        JOIN voucher_deliveries vd ON vd.id = vdi.voucher_id
        WHERE vd.personnel_id = p.id
          AND vd.season_year = CAST(strftime('%Y', 'now') AS INTEGER)
          AND vdi.quota_bucket = 'FLAT'
    ), 0) AS consumed_flat_qty,
    COALESCE((
        SELECT SUM(vdi.quantity)
        FROM voucher_delivery_items vdi
        JOIN voucher_deliveries vd ON vd.id = vdi.voucher_id
        WHERE vd.personnel_id = p.id
          AND vd.season_year = CAST(strftime('%Y', 'now') AS INTEGER)
          AND vdi.quota_bucket = 'TIGHTS'
    ), 0) AS consumed_tights_qty,
    COALESCE((
        SELECT SUM(vdi.quantity)
        FROM voucher_delivery_items vdi
        JOIN voucher_deliveries vd ON vd.id = vdi.voucher_id
        WHERE vd.personnel_id = p.id
          AND vd.season_year = CAST(strftime('%Y', 'now') AS INTEGER)
          AND vdi.quota_bucket = 'SUPPORTER'
    ), 0) AS consumed_supporter_qty,
    COALESCE((
        SELECT SUM(vdi.quantity)
        FROM voucher_delivery_items vdi
        JOIN voucher_deliveries vd ON vd.id = vdi.voucher_id
        WHERE vd.personnel_id = p.id
          AND vd.season_year = CAST(strftime('%Y', 'now') AS INTEGER)
          AND vdi.quota_bucket = 'BRIEFS'
    ), 0) AS consumed_briefs_qty
FROM personnel p
JOIN personnel_categories c ON c.id = p.category_id
WHERE p.role = 'DANCER' AND p.is_active = 1;
