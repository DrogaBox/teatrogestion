-- Migración 0016: Baja de artículos del inventario
-- La baja es lógica: los vales y el libro de movimientos siguen apuntando al artículo, y la
-- sincronización transmite la baja al otro equipo en lugar de volver a crear el artículo.
ALTER TABLE stock_items ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1));

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
    CASE WHEN (s.physical_qty - s.reserved_qty) <= s.min_alert_qty THEN 1 ELSE 0 END AS is_low_stock,
    s.min_alert_qty
FROM stock_items s
WHERE s.is_active = 1;
