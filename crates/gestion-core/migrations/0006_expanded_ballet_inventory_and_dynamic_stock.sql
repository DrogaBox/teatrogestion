-- =============================================================================
-- TEATROGESTIÓN - INVENTARIO DE BALLET EXPANDIDO Y STOCK DINÁMICO
-- Archivo: 0006_expanded_ballet_inventory_and_dynamic_stock.sql
-- Institución: Ballet Oficial de Córdoba — Teatro del Libertador San Martín
-- 1. Eliminar temporalmente vistas y triggers que referencian stock_items
DROP VIEW IF EXISTS v_stock_availability;
DROP TRIGGER IF EXISTS trg_stock_reservation_before_insert;
DROP TRIGGER IF EXISTS trg_stock_reservation_after_insert;
DROP TRIGGER IF EXISTS trg_stock_reservation_release;

-- 2. Recreación de tabla stock_items flexibilizando categorías, marcas y materiales
-- para permitir la creación dinámica de nuevos ítems sin restricciones rígidas CHECK
CREATE TABLE stock_items_new (
    id TEXT PRIMARY KEY,
    sku_code TEXT NOT NULL UNIQUE,
    barcode TEXT UNIQUE,
    family TEXT NOT NULL,
    subcategory TEXT NOT NULL,
    quota_bucket TEXT NOT NULL CHECK (
        quota_bucket IN ('POINTE', 'FLAT', 'TIGHTS', 'SUPPORTER', 'BRIEFS', 'EXEMPT')
    ),
    brand TEXT NOT NULL,
    model_name TEXT NOT NULL,
    raw_factory_code TEXT,
    color TEXT NOT NULL,
    material TEXT NOT NULL,
    prop_dye_state TEXT NOT NULL DEFAULT 'ORIGINAL' CHECK (
        prop_dye_state IN ('ORIGINAL', 'TENIDA')
    ),
    dyed_for_work_id TEXT REFERENCES works(id) ON DELETE SET NULL,
    size_system TEXT NOT NULL CHECK (
        size_system IN ('US', 'EU', 'ALPHA', 'OTRO')
    ),
    size_value TEXT NOT NULL,
    width_value TEXT,

    -- Atributos específicos Gaynor Minden y Duval (opcionales)
    gm_line TEXT,
    gm_fit TEXT,
    gm_box INTEGER,
    gm_shank TEXT,
    gm_vamp TEXT,
    gm_heel TEXT,
    duval_hardness TEXT,

    -- Cantidades e Invariantes de Depósito
    physical_qty INTEGER NOT NULL DEFAULT 0 CHECK (physical_qty >= 0),
    reserved_qty INTEGER NOT NULL DEFAULT 0 CHECK (reserved_qty >= 0 AND reserved_qty <= physical_qty),
    min_alert_qty INTEGER NOT NULL DEFAULT 2 CHECK (min_alert_qty >= 0),
    storage_bin_location TEXT,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

INSERT INTO stock_items_new (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location, updated_at
)
SELECT
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location, updated_at
FROM stock_items;

DROP TABLE stock_items;

ALTER TABLE stock_items_new RENAME TO stock_items;

CREATE INDEX idx_stock_items_subcat_brand ON stock_items(subcategory, brand, size_value, width_value);
CREATE INDEX idx_stock_items_gm_lookup ON stock_items(gm_line, gm_fit, size_value, width_value, gm_box, gm_shank);
CREATE INDEX idx_stock_items_raw_code ON stock_items(raw_factory_code);

-- 2. Recreación de vista operativa v_stock_availability
DROP VIEW IF EXISTS v_stock_availability;
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

-- 3. Recreación de triggers de integridad para stock_reservations sobre la nueva tabla
DROP TRIGGER IF EXISTS trg_stock_reservation_before_insert;
CREATE TRIGGER trg_stock_reservation_before_insert
BEFORE INSERT ON stock_reservations
WHEN NEW.status = 'ACTIVE'
BEGIN
    SELECT CASE
        WHEN (SELECT (physical_qty - reserved_qty) FROM stock_items WHERE id = NEW.stock_item_id) < NEW.quantity
        THEN RAISE(ABORT, 'STOCK_INSUFICIENTE_PARA_RESERVA: No hay suficientes unidades libres en depósito.')
    END;
END;

DROP TRIGGER IF EXISTS trg_stock_reservation_after_insert;
CREATE TRIGGER trg_stock_reservation_after_insert
AFTER INSERT ON stock_reservations
WHEN NEW.status = 'ACTIVE'
BEGIN
    UPDATE stock_items
    SET reserved_qty = reserved_qty + NEW.quantity,
        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = NEW.stock_item_id;
END;

DROP TRIGGER IF EXISTS trg_stock_reservation_release;
CREATE TRIGGER trg_stock_reservation_release
AFTER UPDATE OF status ON stock_reservations
WHEN OLD.status = 'ACTIVE' AND NEW.status = 'RELEASED'
BEGIN
    UPDATE stock_items
    SET reserved_qty = reserved_qty - OLD.quantity,
        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = OLD.stock_item_id;
END;

-- -----------------------------------------------------------------------------
-- 4. POBLACIÓN DEL INVENTARIO OFICIAL AMPLIADO DEL BALLET OFICIAL DE CÓRDOBA
-- -----------------------------------------------------------------------------

-- A. CALZAS MASCULINAS (BLANCO, NEGRO, GRIS)
INSERT OR REPLACE INTO stock_items (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location
) VALUES
('STK_CAL_CAP_BL_S', 'CAL-CAP-BL-S', NULL, 'APPAREL', 'CALZA_MASCULINA', 'TIGHTS', 'Capezio', 'M9000 Calza Hombre con Pie', NULL, 'Blanco', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'S', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 6, 0, 2, 'Gaveta Calzas Varón'),
('STK_CAL_CAP_BL_M', 'CAL-CAP-BL-M', NULL, 'APPAREL', 'CALZA_MASCULINA', 'TIGHTS', 'Capezio', 'M9000 Calza Hombre con Pie', NULL, 'Blanco', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 12, 0, 3, 'Gaveta Calzas Varón'),
('STK_CAL_CAP_BL_L', 'CAL-CAP-BL-L', NULL, 'APPAREL', 'CALZA_MASCULINA', 'TIGHTS', 'Capezio', 'M9000 Calza Hombre con Pie', NULL, 'Blanco', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'L', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 8, 0, 2, 'Gaveta Calzas Varón'),

('STK_CAL_CAP_NG_S', 'CAL-CAP-NG-S', NULL, 'APPAREL', 'CALZA_MASCULINA', 'TIGHTS', 'Capezio', 'M9000 Calza Hombre con Pie', NULL, 'Negro', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'S', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 8, 0, 2, 'Gaveta Calzas Varón'),
('STK_CAL_CAP_NG_M', 'CAL-CAP-NG-M', NULL, 'APPAREL', 'CALZA_MASCULINA', 'TIGHTS', 'Capezio', 'M9000 Calza Hombre con Pie', NULL, 'Negro', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 15, 0, 3, 'Gaveta Calzas Varón'),
('STK_CAL_CAP_NG_L', 'CAL-CAP-NG-L', NULL, 'APPAREL', 'CALZA_MASCULINA', 'TIGHTS', 'Capezio', 'M9000 Calza Hombre con Pie', NULL, 'Negro', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'L', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 10, 0, 2, 'Gaveta Calzas Varón'),

('STK_CAL_WM_GR_M',  'CAL-WM-GR-M',  NULL, 'APPAREL', 'CALZA_MASCULINA', 'TIGHTS', 'Wear Moi', 'Orion Calza Varón sin Pie', NULL, 'Gris', 'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5, 0, 2, 'Gaveta Calzas Varón'),
('STK_CAL_WM_GR_L',  'CAL-WM-GR-L',  NULL, 'APPAREL', 'CALZA_MASCULINA', 'TIGHTS', 'Wear Moi', 'Orion Calza Varón sin Pie', NULL, 'Gris', 'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'L', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Gaveta Calzas Varón');

-- B. SUSPENSORES MASCULINOS ADICIONALES (BLANCO, PIEL, NEGRO)
INSERT OR REPLACE INTO stock_items (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location
) VALUES
('STK_IND_SUS_CAP_BL_S', 'IND-SUS-CAP-BL-S', NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Capezio', 'Quilted Dance Belt', NULL, 'Blanco', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'S', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5, 0, 2, 'Gaveta Suspensores'),
('STK_IND_SUS_CAP_BL_M', 'IND-SUS-CAP-BL-M', NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Capezio', 'Quilted Dance Belt', NULL, 'Blanco', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 7, 0, 2, 'Gaveta Suspensores'),
('STK_IND_SUS_CAP_BL_L', 'IND-SUS-CAP-BL-L', NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Capezio', 'Quilted Dance Belt', NULL, 'Blanco', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'L', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Gaveta Suspensores'),
('STK_IND_SUS_BLO_PL_M', 'IND-SUS-BLO-PL-M', NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Bloch',   'Dance Belt Nude',    NULL, 'Piel',   'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 9, 0, 2, 'Gaveta Suspensores'),
('STK_IND_SUS_BLO_PL_L', 'IND-SUS-BLO-PL-L', NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Bloch',   'Dance Belt Nude',    NULL, 'Piel',   'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'L', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 6, 0, 2, 'Gaveta Suspensores');

-- C. CANCÁN ENTERO Y CONVERTIBLE (ROSA BALLET, PIEL, NEGRO, BLANCO)
INSERT OR REPLACE INTO stock_items (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location
) VALUES
('STK_CAN_CAP_RS_SM',  'CAN-CAP-RS-SM',  NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'Capezio', 'Ultra Soft Convertible', NULL, 'Rosa',   'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'S/M',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 25, 0, 5, 'Gaveta Cancán'),
('STK_CAN_CAP_RS_LXL', 'CAN-CAP-RS-LXL', NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'Capezio', 'Ultra Soft Convertible', NULL, 'Rosa',   'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'L/XL', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 20, 0, 5, 'Gaveta Cancán'),
('STK_CAN_CAP_BL_SM',  'CAN-CAP-BL-SM',  NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'Capezio', 'Ultra Soft Convertible', NULL, 'Blanco', 'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'S/M',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 14, 0, 3, 'Gaveta Cancán'),
('STK_CAN_CAP_BL_LXL', 'CAN-CAP-BL-LXL', NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'Capezio', 'Ultra Soft Convertible', NULL, 'Blanco', 'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'L/XL', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 12, 0, 3, 'Gaveta Cancán'),
('STK_CAN_SIL_NG_M',   'CAN-SIL-NG-M',   NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'Silvana', 'Ballet Opaco',          NULL, 'Negro',  'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'M',    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 18, 0, 4, 'Gaveta Cancán');

-- D. MEDIAS DE DANZA / SOQUETES (BLANCO, NEGRO, PIEL)
INSERT OR REPLACE INTO stock_items (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location
) VALUES
('STK_MED_BLO_BL_UNI', 'MED-BLO-BL-UNI', NULL, 'APPAREL', 'MEDIAS_DANZA', 'TIGHTS', 'Bloch',   'Soquete Danza Microfibra', NULL, 'Blanco', 'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'Único', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 35, 0, 6, 'Gaveta Medias'),
('STK_MED_BLO_NG_UNI', 'MED-BLO-NG-UNI', NULL, 'APPAREL', 'MEDIAS_DANZA', 'TIGHTS', 'Bloch',   'Soquete Danza Microfibra', NULL, 'Negro',  'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'Único', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 40, 0, 6, 'Gaveta Medias'),
('STK_MED_CAP_PL_UNI', 'MED-CAP-PL-UNI', NULL, 'APPAREL', 'MEDIAS_DANZA', 'TIGHTS', 'Capezio', 'Media Corta Clásica',    NULL, 'Piel',   'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'Único', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 28, 0, 5, 'Gaveta Medias');

-- E. MEDIA PUNTAS DE CUERO Y TELA (PIEL, BLANCO, NEGRO)
INSERT OR REPLACE INTO stock_items (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location
) VALUES
('STK_MPC_SAN_PL_8W',  'MPC-SAN-PL-8W',  NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_CUERO', 'FLAT', 'Sansha', 'Pro 1C Cuero',  NULL, 'Piel',   'CUERO',         'ORIGINAL', NULL, 'US', '8',  'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 6, 0, 2, 'Estante MP Cuero'),
('STK_MPC_SAN_PL_9W',  'MPC-SAN-PL-9W',  NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_CUERO', 'FLAT', 'Sansha', 'Pro 1C Cuero',  NULL, 'Piel',   'CUERO',         'ORIGINAL', NULL, 'US', '9',  'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 8, 0, 2, 'Estante MP Cuero'),
('STK_MPT_SAN_BL_8W',  'MPT-SAN-BL-8W',  NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_TELA',  'FLAT', 'Sansha', 'Pro 1C Tela',   NULL, 'Blanco', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '8',  'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 10, 0, 2, 'Estante MP Tela'),
('STK_MPT_SAN_NG_8W',  'MPT-SAN-NG-8W',  NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_TELA',  'FLAT', 'Sansha', 'Pro 1C Tela',   NULL, 'Negro',  'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '8',  'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 12, 0, 2, 'Estante MP Tela'),
('STK_MPT_SD_PL_6C',   'MPT-SD-PL-6C',   NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_TELA',  'FLAT', 'So Danca','SD16 Stretch',  NULL, 'Piel',   'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '6',  'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 15, 0, 3, 'Estante MP Tela'),
('STK_MPT_SD_PL_7C',   'MPT-SD-PL-7C',   NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_TELA',  'FLAT', 'So Danca','SD16 Stretch',  NULL, 'Piel',   'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '7',  'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 18, 0, 3, 'Estante MP Tela');

-- F. ZAPATOS DE JAZZ (CUERO / TELA CON SUELA PARTIDA)
INSERT OR REPLACE INTO stock_items (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location
) VALUES
('STK_JAZ_BLO_NG_7',   'JAZ-BLO-NG-7',   NULL, 'WOMENS_FLAT', 'ZAPATOS_JAZZ', 'FLAT', 'Bloch',   'Neo-Flex Suela Partida', NULL, 'Negro', 'CUERO',         'ORIGINAL', NULL, 'US', '7',  'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5, 0, 1, 'Estante Zapatos Jazz'),
('STK_JAZ_BLO_NG_8',   'JAZ-BLO-NG-8',   NULL, 'WOMENS_FLAT', 'ZAPATOS_JAZZ', 'FLAT', 'Bloch',   'Neo-Flex Suela Partida', NULL, 'Negro', 'CUERO',         'ORIGINAL', NULL, 'US', '8',  'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 8, 0, 2, 'Estante Zapatos Jazz'),
('STK_JAZ_BLO_NG_9',   'JAZ-BLO-NG-9',   NULL, 'MENS_FLAT',   'ZAPATOS_JAZZ', 'FLAT', 'Bloch',   'Neo-Flex Suela Partida', NULL, 'Negro', 'CUERO',         'ORIGINAL', NULL, 'US', '9',  'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Estante Zapatos Jazz'),
('STK_JAZ_BLO_NG_10',  'JAZ-BLO-NG-10',  NULL, 'MENS_FLAT',   'ZAPATOS_JAZZ', 'FLAT', 'Bloch',   'Neo-Flex Suela Partida', NULL, 'Negro', 'CUERO',         'ORIGINAL', NULL, 'US', '10', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 1, 'Estante Zapatos Jazz'),
('STK_JAZ_CAP_PL_75',  'JAZ-CAP-PL-7.5', NULL, 'WOMENS_FLAT', 'ZAPATOS_JAZZ', 'FLAT', 'Capezio', 'E-Series Slip-on Caramel',NULL, 'Piel',  'CUERO',         'ORIGINAL', NULL, 'US', '7.5','M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 6, 0, 2, 'Estante Zapatos Jazz'),
('STK_JAZ_CAP_PL_85',  'JAZ-CAP-PL-8.5', NULL, 'WOMENS_FLAT', 'ZAPATOS_JAZZ', 'FLAT', 'Capezio', 'E-Series Slip-on Caramel',NULL, 'Piel',  'CUERO',         'ORIGINAL', NULL, 'US', '8.5','M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5, 0, 1, 'Estante Zapatos Jazz');

-- G. ZAPATOS DE CARÁCTER (TACO CUBANO Y BAJO EN CUERO NEGRO Y MARRÓN)
INSERT OR REPLACE INTO stock_items (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location
) VALUES
('STK_CAR_CAP_NG_65',  'CAR-CAP-NG-6.5', NULL, 'WOMENS_FLAT', 'ZAPATOS_CARACTER', 'FLAT', 'Capezio', 'Carácter Taco Cubano 2"', NULL, 'Negro',  'CUERO', 'ORIGINAL', NULL, 'US', '6.5', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Estante Zapatos Carácter'),
('STK_CAR_CAP_NG_70',  'CAR-CAP-NG-7',   NULL, 'WOMENS_FLAT', 'ZAPATOS_CARACTER', 'FLAT', 'Capezio', 'Carácter Taco Cubano 2"', NULL, 'Negro',  'CUERO', 'ORIGINAL', NULL, 'US', '7',   'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 6, 0, 2, 'Estante Zapatos Carácter'),
('STK_CAR_CAP_NG_75',  'CAR-CAP-NG-7.5', NULL, 'WOMENS_FLAT', 'ZAPATOS_CARACTER', 'FLAT', 'Capezio', 'Carácter Taco Cubano 2"', NULL, 'Negro',  'CUERO', 'ORIGINAL', NULL, 'US', '7.5', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 7, 0, 2, 'Estante Zapatos Carácter'),
('STK_CAR_CAP_NG_80',  'CAR-CAP-NG-8',   NULL, 'WOMENS_FLAT', 'ZAPATOS_CARACTER', 'FLAT', 'Capezio', 'Carácter Taco Cubano 2"', NULL, 'Negro',  'CUERO', 'ORIGINAL', NULL, 'US', '8',   'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5, 0, 1, 'Estante Zapatos Carácter'),
('STK_CAR_BLO_MR_70',  'CAR-BLO-MR-7',   NULL, 'WOMENS_FLAT', 'ZAPATOS_CARACTER', 'FLAT', 'Bloch',   'Carácter Taco Bajo 1.5"', NULL, 'Marrón', 'CUERO', 'ORIGINAL', NULL, 'US', '7',   'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 1, 'Estante Zapatos Carácter'),
('STK_CAR_SAN_NG_10M', 'CAR-SAN-NG-10M', NULL, 'MENS_FLAT',   'ZAPATOS_CARACTER', 'FLAT', 'Sansha', 'Carácter Varón Suela Cuero', NULL, 'Negro', 'CUERO', 'ORIGINAL', NULL, 'US', '10',  'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Estante Zapatos Carácter'),
('STK_CAR_SAN_NG_11M', 'CAR-SAN-NG-11M', NULL, 'MENS_FLAT',   'ZAPATOS_CARACTER', 'FLAT', 'Sansha', 'Carácter Varón Suela Cuero', NULL, 'Negro', 'CUERO', 'ORIGINAL', NULL, 'US', '11',  'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 1, 'Estante Zapatos Carácter');
