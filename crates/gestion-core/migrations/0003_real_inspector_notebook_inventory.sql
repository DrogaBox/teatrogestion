-- =============================================================================
-- TEATROGESTIÓN - INVENTARIO OFICIAL TRANSTRITO DE LA LIBRETA DE LA INSPECTORA
-- Archivo: 0003_real_inspector_notebook_inventory.sql
-- =============================================================================

-- -----------------------------------------------------------------------------
-- 1. MEDIA PUNTAS VARÓN (SANSHA BLANCAS, NEGRAS CUERO/TELA, PIEL Y TEÑIDA)
-- -----------------------------------------------------------------------------
INSERT OR REPLACE INTO stock_items (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location
) VALUES
-- Sansha Blancas (Cuero)
('STK_MPV_SAN_BL_8W',  'MPV-SAN-BL-8W',  NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '8',  'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 2, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_9W',  'MPV-SAN-BL-9W',  NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '9',  'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 2, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_10M', 'MPV-SAN-BL-10M', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '10', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 17, 0, 3, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_10W', 'MPV-SAN-BL-10W', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '10', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 8, 0, 2, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_11M', 'MPV-SAN-BL-11M', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '11', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 18, 0, 3, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_11W', 'MPV-SAN-BL-11W', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '11', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 14, 0, 3, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_12M', 'MPV-SAN-BL-12M', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '12', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 45, 0, 5, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_12W', 'MPV-SAN-BL-12W', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '12', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 7, 0, 2, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_13M', 'MPV-SAN-BL-13M', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '13', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_13W', 'MPV-SAN-BL-13W', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '13', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_15M', 'MPV-SAN-BL-15M', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '15', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_15W', 'MPV-SAN-BL-15W', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '15', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5, 0, 2, 'Estante V-Sansha-BL'),
('STK_MPV_SAN_BL_16W', 'MPV-SAN-BL-16W', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C', NULL, 'Blanco', 'CUERO', 'ORIGINAL', NULL, 'US', '16', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-BL'),

-- Sansha Negras (Cuero / Tela)
('STK_MPV_SAN_NG_8M_CUE',  'MPV-SAN-NG-8M-CUE',  NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C Cuero', NULL, 'Negro', 'CUERO',         'ORIGINAL', NULL, 'US', '8',  'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5, 0, 2, 'Estante V-Sansha-NG'),
('STK_MPV_SAN_NG_10M_CUE', 'MPV-SAN-NG-10M-CUE', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C Cuero', NULL, 'Negro', 'CUERO',         'ORIGINAL', NULL, 'US', '10', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 32, 0, 4, 'Estante V-Sansha-NG'),
('STK_MPV_SAN_NG_11M_TEL', 'MPV-SAN-NG-11M-TEL', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C Tela',  NULL, 'Negro', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '11', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-NG'),
('STK_MPV_SAN_NG_11W_TEL', 'MPV-SAN-NG-11W-TEL', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C Tela',  NULL, 'Negro', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '11', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-NG'),
('STK_MPV_SAN_NG_12M_CUE', 'MPV-SAN-NG-12M-CUE', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C Cuero', NULL, 'Negro', 'CUERO',         'ORIGINAL', NULL, 'US', '12', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante V-Sansha-NG'),
('STK_MPV_SAN_NG_13M_TEL', 'MPV-SAN-NG-13M-TEL', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C Tela',  NULL, 'Negro', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '13', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-NG'),
('STK_MPV_SAN_NG_13W_TEL', 'MPV-SAN-NG-13W-TEL', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C Tela',  NULL, 'Negro', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '13', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 7, 0, 2, 'Estante V-Sansha-NG'),
('STK_MPV_SAN_NG_15M_CUE', 'MPV-SAN-NG-15M-CUE', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C Cuero', NULL, 'Negro', 'CUERO',         'ORIGINAL', NULL, 'US', '15', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-NG'),

-- Sansha Piel (Original y Teñida)
('STK_MPV_SAN_PL_10W',     'MPV-SAN-PL-10W',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '10', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_11M',     'MPV-SAN-PL-11M',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '11', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 1, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_11W',     'MPV-SAN-PL-11W',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '11', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 5, 0, 2, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_12W',     'MPV-SAN-PL-12W',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '12', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 8, 0, 2, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_12W_TEN', 'MPV-SAN-PL-12W-TEN', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C Teñida',NULL, 'Piel', 'TELA_ELASTICA', 'TENIDA',   'WORK_LAGO_CISNES', 'US', '12', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Utilería Teñida'),
('STK_MPV_SAN_PL_12M',     'MPV-SAN-PL-12M',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '12', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_13M',     'MPV-SAN-PL-13M',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '13', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_13W',     'MPV-SAN-PL-13W',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '13', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 1, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_15W',     'MPV-SAN-PL-15W',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '15', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_16W',     'MPV-SAN-PL-16W',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '16', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_17W',     'MPV-SAN-PL-17W',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '17', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-PL'),
('STK_MPV_SAN_PL_18W',     'MPV-SAN-PL-18W',     NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha', 'Pro 1C',       NULL, 'Piel', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '18', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Sansha-PL'),

-- Capezio Blancas (Tela Elástica)
('STK_MPV_CAP_BL_9W',  'MPV-CAP-BL-9W',  NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Capezio', 'Hanami Stretch', NULL, 'Blanco', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '9',  'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante V-Capezio-BL'),
('STK_MPV_CAP_BL_12W', 'MPV-CAP-BL-12W', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Capezio', 'Hanami Stretch', NULL, 'Blanco', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '12', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Capezio-BL'),
('STK_MPV_CAP_BL_13M', 'MPV-CAP-BL-13M', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Capezio', 'Hanami Stretch', NULL, 'Blanco', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '13', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 1, 'Estante V-Capezio-BL'),
('STK_MPV_CAP_BL_14W', 'MPV-CAP-BL-14W', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Capezio', 'Hanami Stretch', NULL, 'Blanco', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '14', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Capezio-BL'),
('STK_MPV_CAP_BL_16W', 'MPV-CAP-BL-16W', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Capezio', 'Hanami Stretch', NULL, 'Blanco', 'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '16', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante V-Capezio-BL'),

-- HD (Blancas, Piel, Negras)
('STK_MPV_HD_BL_T40', 'MPV-HD-BL-T40', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'HD', 'Clásica HD', NULL, 'Blanco', 'CUERO',         'ORIGINAL', NULL, 'EU', 'T40', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante V-HD'),
('STK_MPV_HD_PL_T39', 'MPV-HD-PL-T39', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'HD', 'Elástica HD',NULL, 'Piel',   'TELA_ELASTICA', 'ORIGINAL', NULL, 'EU', 'T39', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Estante V-HD'),
('STK_MPV_HD_PL_T40', 'MPV-HD-PL-T40', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'HD', 'Elástica HD',NULL, 'Piel',   'TELA_ELASTICA', 'ORIGINAL', NULL, 'EU', 'T40', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante V-HD'),
('STK_MPV_HD_PL_T44', 'MPV-HD-PL-T44', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'HD', 'Elástica HD',NULL, 'Piel',   'TELA_ELASTICA', 'ORIGINAL', NULL, 'EU', 'T44', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Estante V-HD'),
('STK_MPV_HD_NG_T39', 'MPV-HD-NG-T39', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'HD', 'Clásica HD', NULL, 'Negro',  'CUERO',         'ORIGINAL', NULL, 'EU', 'T39', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante V-HD'),
('STK_MPV_HD_NG_T40', 'MPV-HD-NG-T40', NULL, 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'HD', 'Clásica HD', NULL, 'Negro',  'CUERO',         'ORIGINAL', NULL, 'EU', 'T40', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante V-HD'),

-- -----------------------------------------------------------------------------
-- 2. BLOCH SYNCHRONY & CANVAS PRO
-- -----------------------------------------------------------------------------
('STK_MPM_BLO_SYN_RS_8C',  'MPM-BLO-SYN-RS-8C',  NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Synchrony',  NULL, 'Rosa',   'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '8',   'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante M-Bloch'),
('STK_MPM_BLO_SYN_PL_11C', 'MPM-BLO-SYN-PL-11C', NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Synchrony',  NULL, 'Piel',   'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '11',  'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante M-Bloch'),

('STK_MPM_BLO_CAN_BL_75D', 'MPM-BLO-CAN-BL-7.5D', NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Canvas Pro', NULL, 'Blanco', 'CANVAS',        'ORIGINAL', NULL, 'US', '7.5', 'D', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante M-Canvas'),
('STK_MPM_BLO_CAN_BL_8C',  'MPM-BLO-CAN-BL-8C',   NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Canvas Pro', NULL, 'Blanco', 'CANVAS',        'ORIGINAL', NULL, 'US', '8',   'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 8, 0, 2, 'Estante M-Canvas'),
('STK_MPM_BLO_CAN_BL_9C',  'MPM-BLO-CAN-BL-9C',   NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Canvas Pro', NULL, 'Blanco', 'CANVAS',        'ORIGINAL', NULL, 'US', '9',   'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante M-Canvas'),
('STK_MPM_BLO_CAN_BL_10C', 'MPM-BLO-CAN-BL-10C',  NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Canvas Pro', NULL, 'Blanco', 'CANVAS',        'ORIGINAL', NULL, 'US', '10',  'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante M-Canvas'),

('STK_MPM_BLO_CAN_NG_65C', 'MPM-BLO-CAN-NG-6.5C', NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Canvas Pro', NULL, 'Negro',  'CANVAS',        'ORIGINAL', NULL, 'US', '6.5', 'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante M-Canvas'),
('STK_MPM_BLO_CAN_NG_75D', 'MPM-BLO-CAN-NG-7.5D', NULL, 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Canvas Pro', NULL, 'Negro',  'CANVAS',        'ORIGINAL', NULL, 'US', '7.5', 'D', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante M-Canvas'),

-- -----------------------------------------------------------------------------
-- 3. SUSPENSORES (BLOCH & CAPEZIO)
-- -----------------------------------------------------------------------------
('STK_IND_SUS_BLO_S',  'IND-SUS-BLO-S',  NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Bloch',   'Dance Belt',  NULL, 'Negro', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'S',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Gaveta Suspensores'),
('STK_IND_SUS_BLO_L',  'IND-SUS-BLO-L',  NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Bloch',   'Dance Belt',  NULL, 'Negro', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'L',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Gaveta Suspensores'),

('STK_IND_SUS_CAP_S',  'IND-SUS-CAP-S',  NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Capezio', 'Quilted Belt',NULL, 'Piel',  'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'S',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 8, 0, 2, 'Gaveta Suspensores'),
('STK_IND_SUS_CAP_M',  'IND-SUS-CAP-M',  NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Capezio', 'Quilted Belt',NULL, 'Piel',  'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'M',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 1, 'Gaveta Suspensores'),
('STK_IND_SUS_CAP_XL', 'IND-SUS-CAP-XL', NULL, 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Capezio', 'Quilted Belt',NULL, 'Piel',  'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'XL', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Gaveta Suspensores'),

-- -----------------------------------------------------------------------------
-- 4. MEDIA MALLAS (FOUETTE & CAPEZIO)
-- -----------------------------------------------------------------------------
('STK_IND_MM_FOU_PL_S', 'IND-MM-FOU-PL-S', NULL, 'APPAREL', 'MEDIA_MALLA', 'TIGHTS', 'Fouette', 'Clásica Microfibra', NULL, 'Piel',   'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'S', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 17, 0, 4, 'Gaveta Mallas'),
('STK_IND_MM_FOU_PL_L', 'IND-MM-FOU-PL-L', NULL, 'APPAREL', 'MEDIA_MALLA', 'TIGHTS', 'Fouette', 'Clásica Microfibra', NULL, 'Piel',   'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'L', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  7, 0, 2, 'Gaveta Mallas'),

('STK_IND_MM_FOU_NG_S', 'IND-MM-FOU-NG-S', NULL, 'APPAREL', 'MEDIA_MALLA', 'TIGHTS', 'Fouette', 'Clásica Microfibra', NULL, 'Negro',  'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'S', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 20, 0, 4, 'Gaveta Mallas'),
('STK_IND_MM_FOU_NG_M', 'IND-MM-FOU-NG-M', NULL, 'APPAREL', 'MEDIA_MALLA', 'TIGHTS', 'Fouette', 'Clásica Microfibra', NULL, 'Negro',  'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  3, 0, 1, 'Gaveta Mallas'),
('STK_IND_MM_FOU_NG_L', 'IND-MM-FOU-NG-L', NULL, 'APPAREL', 'MEDIA_MALLA', 'TIGHTS', 'Fouette', 'Clásica Microfibra', NULL, 'Negro',  'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'L', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  4, 0, 1, 'Gaveta Mallas'),

('STK_IND_MM_CAP_NG_L', 'IND-MM-CAP-NG-L', NULL, 'APPAREL', 'MEDIA_MALLA', 'TIGHTS', 'Capezio', 'Ultra Soft',         NULL, 'Negro',  'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'L', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  8, 0, 2, 'Gaveta Mallas'),
('STK_IND_MM_CAP_BL_M', 'IND-MM-CAP-BL-M', NULL, 'APPAREL', 'MEDIA_MALLA', 'TIGHTS', 'Capezio', 'Ultra Soft',         NULL, 'Blanco', 'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 24, 0, 5, 'Gaveta Mallas'),
('STK_IND_MM_CAP_BL_L', 'IND-MM-CAP-BL-L', NULL, 'APPAREL', 'MEDIA_MALLA', 'TIGHTS', 'Capezio', 'Ultra Soft',         NULL, 'Blanco', 'MICROFIBRA', 'ORIGINAL', NULL, 'ALPHA', 'L', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 31, 0, 6, 'Gaveta Mallas'),

-- -----------------------------------------------------------------------------
-- 5. PUNTAS GAYNOR MINDEN (▲ EUROPEAS, ● LYRA, ★ AMERICANAS)
-- -----------------------------------------------------------------------------
-- ▲ Gaynor Europeas
('STK_GM_EUR_SC_65N3SDH', 'GM-EUR-SC-6.5N3SDH', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 6.5N3SDH', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '6.5', 'N', 'EUROPEAN_TRIANGLE', 'SCULPTED', 3, 'SUPPLE',     'DEEP', 'HIGH', NULL, 2, 0, 1, 'Gaveta GM-▲'),
('STK_GM_EUR_SC_65N3PDH', 'GM-EUR-SC-6.5N3PDH', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 6.5N3PDH', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '6.5', 'N', 'EUROPEAN_TRIANGLE', 'SCULPTED', 3, 'PIANISSIMO', 'DEEP', 'HIGH', NULL, 3, 0, 1, 'Gaveta GM-▲'),
('STK_GM_EUR_SC_75W4PLL', 'GM-EUR-SC-7.5W4PLL', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 7.5W4PLL', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7.5', 'W', 'EUROPEAN_TRIANGLE', 'SCULPTED', 4, 'PIANISSIMO', 'LOW',  'LOW',  NULL, 3, 0, 1, 'Gaveta GM-▲'),
('STK_GM_EUR_SC_75W4SDL', 'GM-EUR-SC-7.5W4SDL', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 7.5W4SDL', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7.5', 'W', 'EUROPEAN_TRIANGLE', 'SCULPTED', 4, 'SUPPLE',     'DEEP', 'LOW',  NULL, 1, 0, 1, 'Gaveta GM-▲'),
('STK_GM_EUR_SC_75W4SLL', 'GM-EUR-SC-7.5W4SLL', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 7.5W4SLL', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7.5', 'W', 'EUROPEAN_TRIANGLE', 'SCULPTED', 4, 'SUPPLE',     'LOW',  'LOW',  NULL, 2, 0, 1, 'Gaveta GM-▲'),
('STK_GM_EUR_SC_8M3HDH',  'GM-EUR-SC-8M3HDH',  NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 8M3HDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8',   'M', 'EUROPEAN_TRIANGLE', 'SCULPTED', 3, 'HARD',       'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-▲'),
('STK_GM_EUR_SC_8M3XDH',  'GM-EUR-SC-8M3XDH',  NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 8M3XDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8',   'M', 'EUROPEAN_TRIANGLE', 'SCULPTED', 3, 'EXTRAFLEX',  'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-▲'),
('STK_GM_EUR_SC_8M3SDH',  'GM-EUR-SC-8M3SDH',  NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 8M3SDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8',   'M', 'EUROPEAN_TRIANGLE', 'SCULPTED', 3, 'SUPPLE',     'DEEP', 'HIGH', NULL, 4, 0, 1, 'Gaveta GM-▲'),
('STK_GM_EUR_CL_9M4HDH',  'GM-EUR-CL-9M4HDH',  NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 9M4HDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '9',   'M', 'EUROPEAN_TRIANGLE', 'CLASSIC',  4, 'HARD',       'DEEP', 'HIGH', NULL, 4, 0, 1, 'Gaveta GM-▲'),
('STK_GM_EUR_CL_95M4SDH', 'GM-EUR-CL-9.5M4SDH',NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 9.5M4SDH', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '9.5', 'M', 'EUROPEAN_TRIANGLE', 'CLASSIC',  4, 'SUPPLE',     'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-▲'),

-- ▲ Capellada Alta Europeas (+6mm)
('STK_GM_EUR_SK_8M4HDH_6MM',  'GM-EUR-SK-8M4HDH-6MM',  NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sleek',    'SK - 8M4HDH +6mm',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8',   'M', 'EUROPEAN_TRIANGLE', 'SLEEK',    4, 'HARD', 'DEEP_PLUS_6MM', 'HIGH', NULL, 4, 0, 1, 'Gaveta GM-▲ (Brenda)'),
('STK_GM_EUR_SK_85M4HDH_6MM', 'GM-EUR-SK-8.5M4HDH-6MM',NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sleek',    'SK - 8.5M4HDH +6mm', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8.5', 'M', 'EUROPEAN_TRIANGLE', 'SLEEK',    4, 'HARD', 'DEEP_PLUS_6MM', 'HIGH', NULL, 6, 0, 2, 'Gaveta GM-▲'),
('STK_GM_EUR_SK_85M4HDL_6MM', 'GM-EUR-SK-8.5M4HDL-6MM',NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sleek',    'SK - 8.5M4HDL +6mm', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8.5', 'M', 'EUROPEAN_TRIANGLE', 'SLEEK',    4, 'HARD', 'DEEP_PLUS_6MM', 'LOW',  NULL, 5, 0, 2, 'Gaveta GM-▲'),
('STK_GM_EUR_SC_85M3HDH_6MM', 'GM-EUR-SC-8.5M3HDH-6MM',NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 8.5M3HDH +6mm', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8.5', 'M', 'EUROPEAN_TRIANGLE', 'SCULPTED', 3, 'HARD', 'DEEP_PLUS_6MM', 'HIGH', NULL, 2, 0, 1, 'Gaveta GM-▲'),

-- ● Gaynor Lyra
('STK_GM_LYR_LCL_75W4HDL', 'GM-LYR-LCL-7.5W4HDL', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'LCL - 7.5W4HDL', 'Rosa', 'SATEN_VEGAN_LYRA', 'ORIGINAL', NULL, 'US', '7.5', 'W', 'LYRA_CIRCLE', 'CLASSIC',  4, 'HARD',   'DEEP', 'LOW',  NULL, 1, 0, 1, 'Gaveta GM-●'),
('STK_GM_LYR_LSC_75W4SDL', 'GM-LYR-LSC-7.5W4SDL', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'LSC - 7.5W4SDL', 'Rosa', 'SATEN_VEGAN_LYRA', 'ORIGINAL', NULL, 'US', '7.5', 'W', 'LYRA_CIRCLE', 'SCULPTED', 4, 'SUPPLE', 'DEEP', 'LOW',  NULL, 1, 0, 1, 'Gaveta GM-●'),
('STK_GM_LYR_LCL_85M3HDH', 'GM-LYR-LCL-8.5M3HDH', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'LCL - 8.5M3HDH', 'Rosa', 'SATEN_VEGAN_LYRA', 'ORIGINAL', NULL, 'US', '8.5', 'M', 'LYRA_CIRCLE', 'CLASSIC',  3, 'HARD',   'DEEP', 'HIGH', NULL, 3, 0, 1, 'Gaveta GM-●'),
('STK_GM_LYR_LCL_85M4HDL', 'GM-LYR-LCL-8.5M4HDL', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'LCL - 8.5M4HDL', 'Rosa', 'SATEN_VEGAN_LYRA', 'ORIGINAL', NULL, 'US', '8.5', 'M', 'LYRA_CIRCLE', 'CLASSIC',  4, 'HARD',   'DEEP', 'LOW',  NULL, 1, 0, 1, 'Gaveta GM-●'),

-- ★ Gaynor Americanas
('STK_GM_USA_CL_65N3PLH', 'GM-USA-CL-6.5N3PLH', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 6.5N3PLH', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '6.5', 'N', 'AMERICAN_STAR', 'CLASSIC',  3, 'PIANISSIMO', 'LOW',  'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_CL_65N3FLH', 'GM-USA-CL-6.5N3FLH', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 6.5N3FLH', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '6.5', 'N', 'AMERICAN_STAR', 'CLASSIC',  3, 'FEATHER',    'LOW',  'HIGH', NULL, 5, 0, 2, 'Gaveta GM-★'),
('STK_GM_USA_SK_7N4SDL',  'GM-USA-SK-7N4SDL',   NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sleek',    'SK - 7N4SDL',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7',   'N', 'AMERICAN_STAR', 'SLEEK',    4, 'SUPPLE',     'DEEP', 'LOW',  NULL, 1, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_CL_7N4HDH',  'GM-USA-CL-7N4HDH',   NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 7N4HDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7',   'N', 'AMERICAN_STAR', 'CLASSIC',  4, 'HARD',       'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_CL_7M4PDH',  'GM-USA-CL-7M4PDH',   NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 7M4PDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7',   'M', 'AMERICAN_STAR', 'CLASSIC',  4, 'PIANISSIMO', 'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_SC_7M4PDH',  'GM-USA-SC-7M4PDH',   NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 7M4PDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7',   'M', 'AMERICAN_STAR', 'SCULPTED', 4, 'PIANISSIMO', 'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_SK_7N4FDL',  'GM-USA-SK-7N4FDL',   NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sleek',    'SK - 7N4FDL',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7',   'N', 'AMERICAN_STAR', 'SLEEK',    4, 'FEATHER',    'DEEP', 'LOW',  NULL, 2, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_CL_75M3XDH', 'GM-USA-CL-7.5M3XDH', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 7.5M3XDH', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7.5', 'M', 'AMERICAN_STAR', 'CLASSIC',  3, 'EXTRAFLEX',  'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_CL_8M4SDH',  'GM-USA-CL-8M4SDH',   NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 8M4SDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8',   'M', 'AMERICAN_STAR', 'CLASSIC',  4, 'SUPPLE',     'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★ (Popi)'),
('STK_GM_USA_CL_8N4HDH',  'GM-USA-CL-8N4HDH',   NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 8N4HDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8',   'N', 'AMERICAN_STAR', 'CLASSIC',  4, 'HARD',       'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_CL_8M3HDH',  'GM-USA-CL-8M3HDH',   NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 8M3HDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8',   'M', 'AMERICAN_STAR', 'CLASSIC',  3, 'HARD',       'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_CL_85M4HDH', 'GM-USA-CL-8.5M4HDH', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 8.5M4HDH', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8.5', 'M', 'AMERICAN_STAR', 'CLASSIC',  4, 'HARD',       'DEEP', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★'),
('STK_GM_USA_CL_9N4HDH',  'GM-USA-CL-9N4HDH',   NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 9N4HDH',   'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '9',   'N', 'AMERICAN_STAR', 'CLASSIC',  4, 'HARD',       'DEEP', 'HIGH', NULL, 2, 0, 1, 'Gaveta GM-★'),

-- ★ Capellada Alta Americana (+1/2")
('STK_GM_USA_CL_8M3HDH_HALF', 'GM-USA-CL-8M3HDH-HALF', NULL, 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic', 'CL - 8M3HDH +1/2"', 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8', 'M', 'AMERICAN_STAR', 'CLASSIC', 3, 'HARD', 'DEEP_PLUS_HALF_INCH', 'HIGH', NULL, 1, 0, 1, 'Gaveta GM-★'),

-- -----------------------------------------------------------------------------
-- 6. PUNTAS BLOCH (BMS III & ETU)
-- -----------------------------------------------------------------------------
('STK_PT_BLO_BMS3_5_XX',   'PT-BLO-BMS3-5-XX',   NULL, 'POINTE', 'PUNTA_BLOCH', 'POINTE', 'Bloch', 'BMS III', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '5',   'XX',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante Puntas Bloch'),
('STK_PT_BLO_BMS3_55_X',   'PT-BLO-BMS3-5.5-X',   NULL, 'POINTE', 'PUNTA_BLOCH', 'POINTE', 'Bloch', 'BMS III', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '5.5', 'X',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, 2, 0, 1, 'Estante Puntas Bloch'),
('STK_PT_BLO_BMS3_7_X',    'PT-BLO-BMS3-7-X',    NULL, 'POINTE', 'PUNTA_BLOCH', 'POINTE', 'Bloch', 'BMS III', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7',   'X',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, 4, 0, 1, 'Estante Puntas Bloch'),
('STK_PT_BLO_BMS3_75_X',   'PT-BLO-BMS3-7.5-X',   NULL, 'POINTE', 'PUNTA_BLOCH', 'POINTE', 'Bloch', 'BMS III', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7.5', 'X',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, 0, 1, 'Estante Puntas Bloch'),
('STK_PT_BLO_ETU_65_XXX',  'PT-BLO-ETU-6.5-XXX',  NULL, 'POINTE', 'PUNTA_BLOCH', 'POINTE', 'Bloch', 'ETU',     NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '6.5', 'XXX', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 3, 0, 1, 'Estante Puntas Bloch'),

-- -----------------------------------------------------------------------------
-- 7. PUNTAS SANSHA DUVAL 4.0 AMERICANAS (FLEX, REGULAR, STRONG)
-- -----------------------------------------------------------------------------
('STK_PT_DUV_40_75_X_FLX', 'PT-DUV-4.0-7.5-X-FLX', NULL, 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7.5', 'X', NULL, NULL, NULL, NULL, NULL, NULL, 'FLEX',    1, 0, 1, 'Estante Duval 4.0'),
('STK_PT_DUV_40_75_W_FLX', 'PT-DUV-4.0-7.5-W-FLX', NULL, 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7.5', 'W', NULL, NULL, NULL, NULL, NULL, NULL, 'FLEX',    1, 0, 1, 'Estante Duval 4.0 (Popi)'),
('STK_PT_DUV_40_75_X_REG', 'PT-DUV-4.0-7.5-X-REG', NULL, 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7.5', 'X', NULL, NULL, NULL, NULL, NULL, NULL, 'REGULAR', 1, 0, 1, 'Estante Duval 4.0'),
('STK_PT_DUV_40_75_W_REG', 'PT-DUV-4.0-7.5-W-REG', NULL, 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7.5', 'W', NULL, NULL, NULL, NULL, NULL, NULL, 'REGULAR', 1, 0, 1, 'Estante Duval 4.0 (Popi)'),
('STK_PT_DUV_40_95_W_REG', 'PT-DUV-4.0-9.5-W-REG', NULL, 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '9.5', 'W', NULL, NULL, NULL, NULL, NULL, NULL, 'REGULAR', 5, 0, 2, 'Estante Duval 4.0'),
('STK_PT_DUV_40_95_X_STR', 'PT-DUV-4.0-9.5-X-STR', NULL, 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '9.5', 'X', NULL, NULL, NULL, NULL, NULL, NULL, 'STRONG',  4, 0, 1, 'Estante Duval 4.0'),
('STK_PT_DUV_40_11_W_STR', 'PT-DUV-4.0-11-W-STR',  NULL, 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '11',  'W', NULL, NULL, NULL, NULL, NULL, NULL, 'STRONG',  1, 0, 1, 'Estante Duval 4.0'),

-- -----------------------------------------------------------------------------
-- 8. CANCÁN (SILVANA, HD, CAPEZIO)
-- -----------------------------------------------------------------------------
('STK_IND_CAN_SIL_A',      'IND-CAN-SIL-A',      NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'Silvana', 'Ballet',      NULL, 'Rosa', 'MICROFIBRA',  'ORIGINAL', NULL, 'ALPHA', 'A',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  2, 0, 1, 'Gaveta Cancán'),
('STK_IND_CAN_HD_S',       'IND-CAN-HD-S',       NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'HD',      'Convertible', NULL, 'Piel', 'MICROFIBRA',  'ORIGINAL', NULL, 'ALPHA', 'S',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  1, 0, 1, 'Gaveta Cancán'),
('STK_IND_CAN_HD_L',       'IND-CAN-HD-L',       NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'HD',      'Convertible', NULL, 'Piel', 'MICROFIBRA',  'ORIGINAL', NULL, 'ALPHA', 'L',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  6, 0, 2, 'Gaveta Cancán'),
('STK_IND_CAN_HD_XL',      'IND-CAN-HD-XL',      NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'HD',      'Convertible', NULL, 'Piel', 'MICROFIBRA',  'ORIGINAL', NULL, 'ALPHA', 'XL',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  3, 0, 1, 'Gaveta Cancán'),

('STK_IND_CAN_CAP_RED_PL_M',  'IND-CAN-CAP-RED-PL-M',  NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'Capezio', 'Red Profesional', NULL, 'Piel', 'RED_FISHNET', 'ORIGINAL', NULL, 'ALPHA', 'M',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 11, 0, 3, 'Gaveta Cancán'),
('STK_IND_CAN_CAP_RED_PL_XL', 'IND-CAN-CAP-RED-PL-XL', NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'Capezio', 'Red Profesional', NULL, 'Piel', 'RED_FISHNET', 'ORIGINAL', NULL, 'ALPHA', 'XL',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 15, 0, 4, 'Gaveta Cancán'),
('STK_IND_CAN_CAP_RED_RS_LXL','IND-CAN-CAP-RED-RS-LXL',NULL, 'APPAREL', 'CANCAN', 'TIGHTS', 'Capezio', 'Red Profesional', NULL, 'Rosa', 'RED_FISHNET', 'ORIGINAL', NULL, 'ALPHA', 'L/XL',NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 17, 0, 4, 'Gaveta Cancán'),

-- -----------------------------------------------------------------------------
-- 9. BOMBACHAS (FOUETTE ROJAS Y PIEL)
-- -----------------------------------------------------------------------------
('STK_IND_BOM_FOU_RJ_M', 'IND-BOM-FOU-RJ-M', NULL, 'APPAREL', 'BOMBACHA_ESCENA', 'BRIEFS', 'Fouette', 'Escena', NULL, 'Rojas', 'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 10, 0, 3, 'Gaveta Bombachas'),
('STK_IND_BOM_FOU_PL_S', 'IND-BOM-FOU-PL-S', NULL, 'APPAREL', 'BOMBACHA_ESCENA', 'BRIEFS', 'Fouette', 'Escena', NULL, 'Piel',  'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'S', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  8, 0, 2, 'Gaveta Bombachas');

-- -----------------------------------------------------------------------------
-- 10. RESERVAS NOMINATIVAS ASIGNADAS EN LA LIBRETA
--     (Brenda: 4 pares Gaynor SK-8M4HDH +6mm / Popi: 1 par Gaynor CL-8M4SDH)
-- -----------------------------------------------------------------------------
INSERT OR REPLACE INTO stock_reservations (
    id, stock_item_id, personnel_id, reserved_for_label, quantity, status, work_id, notes, created_by
) VALUES
('RES_NOTEBOOK_BRENDA_01', 'STK_GM_EUR_SK_8M4HDH_6MM', 'PER_DANCER_BRENDA', 'Brenda', 4, 'ACTIVE', 'WORK_LAGO_CISNES', 'Reserva manuscrita en libreta de inspección: 4 pares capellada alta 6mm', 'PER_INSPECTOR_MAIN'),
('RES_NOTEBOOK_POPI_01',   'STK_GM_USA_CL_8M4SDH',     'PER_DANCER_POPI',   'Popi',   1, 'ACTIVE', 'WORK_LAGO_CISNES', 'Reserva manuscrita en libreta de inspección: 1 par Americana',         'PER_INSPECTOR_MAIN');
