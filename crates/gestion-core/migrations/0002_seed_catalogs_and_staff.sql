-- =============================================================================
-- TEATROGESTIÓN - DATOS SEMILLA OFICIALES DEL BALLET OFICIAL DE CÓRDOBA
-- Archivo: 0002_seed_catalogs_and_staff.sql
-- =============================================================================

-- -----------------------------------------------------------------------------
-- 1. SALAS OFICIALES DEL TEATRO DEL LIBERTADOR SAN MARTÍN
-- -----------------------------------------------------------------------------
INSERT INTO rooms (id, code, display_name, is_stage, sort_order) VALUES
    ('SALA_1',    'SALA_1',    'Sala 1',        0, 1),
    ('SALA_3',    'SALA_3',    'Sala 3',        0, 2),
    ('ESCENARIO', 'ESCENARIO', 'Escenario',     1, 3),
    ('SALA_AUX',  'SALA_AUX',  'Sala Auxiliar', 0, 4);

-- -----------------------------------------------------------------------------
-- 2. CATEGORÍAS DE REVISTA Y CUPOS ANUALES REGLAMENTARIOS
-- -----------------------------------------------------------------------------
INSERT INTO personnel_categories (
    id, code, name,
    annual_pointe_quota, annual_flat_quota, annual_tights_quota, annual_supporter_quota, annual_briefs_quota
) VALUES
    ('CAT_PRIMERA_FIGURA', 'PRIMERA_FIGURA', 'Primera Figura / Solista Principal', 24, 12, 12, 6, 6),
    ('CAT_SOLISTA',        'SOLISTA',        'Solista',                            18, 10, 10, 5, 5),
    ('CAT_CUERPO_BAILE',   'CUERPO_BAILE',   'Cuerpo de Baile Estable',            14,  8,  8, 4, 4),
    ('CAT_CONTRATADO',     'CONTRATADO',     'Contratado / Refuerzo',               8,  6,  6, 3, 3),
    ('CAT_STAFF_DOCENTE',  'STAFF_DOCENTE',  'Maestros, Pianistas e Inspección',    0,  0,  0, 0, 0);

-- -----------------------------------------------------------------------------
-- 3. PLANTEL DE MAESTROS, PIANISTAS, INSPECCIÓN Y BAILARINES
-- -----------------------------------------------------------------------------
-- Maestros en rotación de Clases Matutinas (09:00 a 10:15 h) y Ensayos
INSERT INTO personnel (
    id, legajo, first_name, last_name, nickname, role, gender_group, category_id,
    default_pointe_raw_code, default_flat_size, is_active, notes
) VALUES
    ('PER_MASTER_ANANIEV',  'DOC-001', 'Alexander', 'Ananiev',  'Maestro Ananiev', 'MASTER',    'MALE',           'CAT_STAFF_DOCENTE', NULL, NULL, 1, 'Maestro de Clase Técnica y Repositor'),
    ('PER_MASTER_BENITES',  'DOC-002', 'Jairo',     'Benites',  'Maestro Benites', 'MASTER',    'MALE',           'CAT_STAFF_DOCENTE', NULL, NULL, 1, 'Maestro de Clase Técnica y Ensayista'),
    ('PER_MASTER_PINO',     'DOC-003', 'Patricia',  'Pino',     'Maestra Pino',    'MASTER',    'FEMALE',         'CAT_STAFF_DOCENTE', NULL, NULL, 1, 'Maestra de Clase Técnica y Repositora'),

-- Pianistas acompañantes en rotación (Romero, Villegas, Brusutti)
    ('PER_PIANIST_ROMERO',   'MUS-001', 'Carlos',   'Romero',   'Romero',          'PIANIST',   'MALE',           'CAT_STAFF_DOCENTE', NULL, NULL, 1, 'Pianista Acompañante Titular'),
    ('PER_PIANIST_VILLEGAS', 'MUS-002', 'Laura',    'Villegas', 'Villegas',        'PIANIST',   'FEMALE',         'CAT_STAFF_DOCENTE', NULL, NULL, 1, 'Pianista Acompañante Titular'),
    ('PER_PIANIST_BRUSUTTI', 'MUS-003', 'Marcos',   'Brusutti', 'Brusutti',        'PIANIST',   'MALE',           'CAT_STAFF_DOCENTE', NULL, NULL, 1, 'Pianista Acompañante Titular'),

-- Inspección y Operación Técnica TeatroCue
    ('PER_INSPECTOR_MAIN',   'INS-001', 'Inspección', 'Ballet Oficial', 'Inspección', 'INSPECTOR', 'NOT_APPLICABLE', 'CAT_STAFF_DOCENTE', NULL, NULL, 1, 'Inspección General del Ballet Oficial de Córdoba'),
    ('PER_SOUND_TEATROCUE',  'TEC-001', 'Cabina',     'TeatroCue',      'Sonido',     'SOUND_TECH','NOT_APPLICABLE', 'CAT_STAFF_DOCENTE', NULL, NULL, 1, 'Operación de Sonido y Pistas TeatroCue'),

-- Bailarines de Referencia Operativa (Reservas Popi/Brenda y Elencos Álvarez/Marcato - Morchio C/Gil)
    ('PER_DANCER_POPI',      'BOC-101', 'Sofía',     'Gómez',      'Popi',       'DANCER', 'FEMALE', 'CAT_SOLISTA',        'SC - 7.5W4PDH', '7.5D', 1, 'Reserva habitual Gaynor Sculpted 7.5W4PDH'),
    ('PER_DANCER_BRENDA',    'BOC-102', 'Brenda',    'Vera',       'Brenda',     'DANCER', 'FEMALE', 'CAT_PRIMERA_FIGURA', 'SK - 8.5M4HDL', '8C',   1, 'Reserva habitual Gaynor Sleek 8.5M4HDL'),
    ('PER_DANCER_ALVAREZ',   'BOC-103', 'Julieta',   'Álvarez',    'Álvarez',    'DANCER', 'FEMALE', 'CAT_PRIMERA_FIGURA', 'CL - 8M3XDH',   '7.5D', 1, 'Primera Figura - Odette / Odile / Diana'),
    ('PER_DANCER_MARCATO',   'BOC-104', 'Lucía',     'Marcato',    'Marcato',    'DANCER', 'FEMALE', 'CAT_SOLISTA',        'SC - 7.5W4PDH', '6.5C', 1, 'Solista - Odette / Princesas / Diana'),
    ('PER_DANCER_MORCHIO_C', 'BOC-105', 'Cristian',  'Morchio C.', 'Morchio C',  'DANCER', 'MALE',   'CAT_PRIMERA_FIGURA', NULL,            '10M',  1, 'Primera Figura Masculina - Sigfrido / Acteón'),
    ('PER_DANCER_GIL',       'BOC-106', 'Nicolás',   'Gil',        'Gil',        'DANCER', 'MALE',   'CAT_SOLISTA',        NULL,            '9.5M', 1, 'Solista Masculino - Sigfrido / Acteón / Bufón');

-- -----------------------------------------------------------------------------
-- 4. REPERTORIO DE OBRAS Y PASAJES CON REFERENCIAS DE SONIDO (TeatroCue)
-- -----------------------------------------------------------------------------
INSERT INTO works (id, title, composer, choreographer, is_active) VALUES
    ('WORK_LAGO_CISNES', 'El Lago de los Cisnes', 'Piotr Ilich Tchaikovsky', 'Marius Petipa / Lev Ivanov', 1),
    ('WORK_GALA_CLASICA','Gala de Ballet Clásico', 'Cesare Pugni / Adolphe Adam', 'Marius Petipa / Agrippina Vaganova', 1),
    ('WORK_DON_QUIJOTE', 'Don Quijote', 'Ludwig Minkus', 'Marius Petipa / Alexander Gorsky', 1);

INSERT INTO work_passages (id, work_id, name, act_scene, default_duration_min, default_sound_cue) VALUES
    ('PAS_PRINCESAS_BUFON', 'WORK_LAGO_CISNES',  'Princesas y Bufón',              'Acto I / Acto III', 60, 'PISTA TeatroCue #04 / Piano'),
    ('PAS_DANZA_ESPANOLA',  'WORK_LAGO_CISNES',  'Danza Española',                 'Acto III',          60, 'PISTA TeatroCue #11'),
    ('PAS_PDD_ODETTE_SIGF', 'WORK_LAGO_CISNES',  'Pas de deux Odette y Sigfrido',  'Acto II',           60, 'PISTA TeatroCue #08 / Piano'),
    ('PAS_CISNE_NEGRO',     'WORK_LAGO_CISNES',  'Cisne Negro',                    'Acto III',          75, 'PISTA TeatroCue #14 / Piano'),
    ('PAS_DIANA_ACTEON',    'WORK_GALA_CLASICA', 'Diana y Acteón',                 'Grand Pas de Deux', 60, 'PISTA TeatroCue #21'),
    ('PAS_KITRI_BASILIO',   'WORK_DON_QUIJOTE',  'Grand Pas de Deux Kitri y Basilio', 'Acto III',       60, 'PISTA TeatroCue #30');

-- -----------------------------------------------------------------------------
-- 5. CATÁLOGO INICIAL DE TEATROSTOCK (CALZADO E INDUMENTARIA)
-- -----------------------------------------------------------------------------

-- 5.A Media Puntas de Varón (Sansha, Capezio tela elástica, HD; Blanco, Negro, Piel; Cuero vs Tela Elástica; Original y Teñida; 8M-18W y T39-T41)
INSERT INTO stock_items (
    id, sku_code, barcode, family, subcategory, quota_bucket, brand, model_name, raw_factory_code,
    color, material, prop_dye_state, dyed_for_work_id, size_system, size_value, width_value,
    gm_line, gm_fit, gm_box, gm_shank, gm_vamp, gm_heel, duval_hardness,
    physical_qty, reserved_qty, min_alert_qty, storage_bin_location
) VALUES
    ('STK_MPV_SANSHA_CUERO_BL_10M',    'MPV-SAN-CUE-BL-10M-OR',  '7790001001', 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha',  'Pro 1C Cuero',    NULL, 'Blanco', 'CUERO',         'ORIGINAL', NULL,               'US', '10',  'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 12, 0, 3, 'Estante V-1'),
    ('STK_MPV_SANSHA_CUERO_NG_105W',   'MPV-SAN-CUE-NG-10.5W-OR','7790001002', 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha',  'Pro 1C Cuero',    NULL, 'Negro',  'CUERO',         'ORIGINAL', NULL,               'US', '10.5','W', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 10, 0, 3, 'Estante V-1'),
    ('STK_MPV_SANSHA_TELA_PL_T41_TEN', 'MPV-SAN-TEL-PL-T41-TEN', '7790001003', 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Sansha',  'Pro 1C Elástica', NULL, 'Piel',   'TELA_ELASTICA', 'TENIDA',   'WORK_LAGO_CISNES', 'EU', 'T41', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL,  6, 0, 2, 'Estante V-1 (Utilería Teñida)'),
    ('STK_MPV_CAPEZIO_TELA_PL_95M',    'MPV-CAP-TEL-PL-9.5M-OR', '7790001004', 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Capezio', 'Hanami Stretch',  NULL, 'Piel',   'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '9.5', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 14, 0, 4, 'Estante V-2'),
    ('STK_MPV_CAPEZIO_TELA_BL_8M',     'MPV-CAP-TEL-BL-8M-OR',   '7790001005', 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Capezio', 'Hanami Stretch',  NULL, 'Blanco', 'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '8',   'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL,  8, 0, 2, 'Estante V-2'),
    ('STK_MPV_CAPEZIO_TELA_NG_18W',    'MPV-CAP-TEL-NG-18W-OR',  '7790001006', 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'Capezio', 'Hanami Stretch',  NULL, 'Negro',  'TELA_ELASTICA', 'ORIGINAL', NULL,               'US', '18',  'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL,  4, 0, 1, 'Estante V-2'),
    ('STK_MPV_HD_CUERO_BL_T39',        'MPV-HD-CUE-BL-T39-OR',   '7790001007', 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'HD',      'Clásica Varón',   NULL, 'Blanco', 'CUERO',         'ORIGINAL', NULL,               'EU', 'T39', 'M', NULL, NULL, NULL, NULL, NULL, NULL, NULL,  9, 0, 2, 'Estante V-3'),
    ('STK_MPV_HD_TELA_PL_T40_TEN',     'MPV-HD-TEL-PL-T40-TEN',  '7790001008', 'MENS_FLAT', 'MEDIA_PUNTA_VARON', 'FLAT', 'HD',      'Elástica Varón',  NULL, 'Piel',   'TELA_ELASTICA', 'TENIDA',   'WORK_DON_QUIJOTE', 'EU', 'T40', 'W', NULL, NULL, NULL, NULL, NULL, NULL, NULL,  5, 0, 2, 'Estante V-3 (Utilería Teñida)'),

-- 5.B Media Puntas Mujer / Unisex (Bloch Synchrony, Canvas Pro; Rosa, Piel, Blanco, Negro; 6.5C, 7.5D, 8C, 10C)
    ('STK_MPM_BLOCH_SYNC_RS_65C',      'MPM-BLO-SYN-RS-6.5C-OR', '7790002001', 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Synchrony',  NULL, 'Rosa',   'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '6.5', 'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 15, 0, 4, 'Estante M-1'),
    ('STK_MPM_BLOCH_SYNC_PL_75D',      'MPM-BLO-SYN-PL-7.5D-OR', '7790002002', 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Synchrony',  NULL, 'Piel',   'TELA_ELASTICA', 'ORIGINAL', NULL, 'US', '7.5', 'D', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 18, 0, 4, 'Estante M-1'),
    ('STK_MPM_BLOCH_CANV_BL_8C',       'MPM-BLO-CAN-BL-8C-OR',   '7790002003', 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Canvas Pro', NULL, 'Blanco', 'CANVAS',        'ORIGINAL', NULL, 'US', '8',   'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 12, 0, 3, 'Estante M-2'),
    ('STK_MPM_BLOCH_CANV_NG_10C',      'MPM-BLO-CAN-NG-10C-OR',  '7790002004', 'WOMENS_FLAT', 'MEDIA_PUNTA_MUJER_UNISEX', 'FLAT', 'Bloch', 'Canvas Pro', NULL, 'Negro',  'CANVAS',        'ORIGINAL', NULL, 'US', '10',  'C', NULL, NULL, NULL, NULL, NULL, NULL, NULL,  8, 0, 2, 'Estante M-2'),

-- 5.C Puntas Gaynor Minden (Europeas ▲, Lyra ●, Americanas ★) con códigos reales y capelladas +6mm / +1/2"
    ('STK_GM_EUR_SC_75W4PDH',          'GM-EUR-SC-7.5W4PDH',      '7790003001', 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 7.5W4PDH',       'Rosa', 'SATEN',            'ORIGINAL', NULL, 'US', '7.5', 'W', 'EUROPEAN_TRIANGLE', 'SCULPTED', 4, 'PIANISSIMO', 'DEEP',               'HIGH', NULL, 10, 0, 2, 'Gaveta GM-▲1'),
    ('STK_GM_EUR_SC_75W4PDH_6MM',      'GM-EUR-SC-7.5W4PDH-6MM',  '7790003002', 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 7.5W4PDH +6mm',  'Rosa', 'SATEN',            'ORIGINAL', NULL, 'US', '7.5', 'W', 'EUROPEAN_TRIANGLE', 'SCULPTED', 4, 'PIANISSIMO', 'DEEP_PLUS_6MM',      'HIGH', NULL,  6, 0, 2, 'Gaveta GM-▲2'),
    ('STK_GM_LYR_SK_85M4HDL',          'GM-LYR-SK-8.5M4HDL',      '7790003003', 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sleek',    'SK - 8.5M4HDL',       'Rosa', 'SATEN_VEGAN_LYRA', 'ORIGINAL', NULL, 'US', '8.5', 'M', 'LYRA_CIRCLE',       'SLEEK',    4, 'HARD',       'DEEP',               'LOW',  NULL,  8, 0, 2, 'Gaveta GM-●1'),
    ('STK_GM_EUR_CL_8M3XDH',           'GM-EUR-CL-8M3XDH',        '7790003004', 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 8M3XDH',         'Rosa', 'SATEN',            'ORIGINAL', NULL, 'US', '8',   'M', 'EUROPEAN_TRIANGLE', 'CLASSIC',  3, 'EXTRAFLEX',  'DEEP',               'HIGH', NULL,  9, 0, 2, 'Gaveta GM-▲3'),
    ('STK_GM_USA_SC_8W4PDH_HALF',      'GM-USA-SC-8W4PDH-HALF',   '7790003005', 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 8W4PDH +1/2"',   'Rosa', 'SATEN',            'ORIGINAL', NULL, 'US', '8',   'W', 'AMERICAN_STAR',     'SCULPTED', 4, 'PIANISSIMO', 'DEEP_PLUS_HALF_INCH','HIGH', NULL,  4, 0, 1, 'Gaveta GM-★1'),
    ('STK_GM_LYR_SC_7W3FDH',           'GM-LYR-SC-7W3FDH',        '7790003006', 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Sculpted', 'SC - 7W3FDH',         'Rosa', 'SATEN_VEGAN_LYRA', 'ORIGINAL', NULL, 'US', '7',   'W', 'LYRA_CIRCLE',       'SCULPTED', 3, 'FEATHER',    'DEEP',               'HIGH', NULL,  7, 0, 2, 'Gaveta GM-●2'),
    ('STK_GM_USA_CL_75M3SDH',          'GM-USA-CL-7.5M3SDH',      '7790003007', 'POINTE', 'PUNTA_GAYNOR_MINDEN', 'POINTE', 'Gaynor Minden', 'Classic',  'CL - 7.5M3SDH',       'Rosa', 'SATEN',            'ORIGINAL', NULL, 'US', '7.5', 'M', 'AMERICAN_STAR',     'CLASSIC',  3, 'SUPPLE',     'DEEP',               'HIGH', NULL,  5, 0, 1, 'Gaveta GM-★2'),

-- 5.D Puntas Bloch (Modelos BMS III y ETU con anchos X, XX, XXX)
    ('STK_PT_BLOCH_BMS3_55_XX',        'PT-BLO-BMS3-5.5-XX',      '7790004001', 'POINTE', 'PUNTA_BLOCH', 'POINTE', 'Bloch', 'BMS III', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '5.5', 'XX',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, 10, 0, 2, 'Estante P-Bloch 1'),
    ('STK_PT_BLOCH_BMS3_60_XXX',       'PT-BLO-BMS3-6.0-XXX',     '7790004002', 'POINTE', 'PUNTA_BLOCH', 'POINTE', 'Bloch', 'BMS III', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '6',   'XXX', NULL, NULL, NULL, NULL, NULL, NULL, NULL,  8, 0, 2, 'Estante P-Bloch 1'),
    ('STK_PT_BLOCH_ETU_65_X',          'PT-BLO-ETU-6.5-X',        '7790004003', 'POINTE', 'PUNTA_BLOCH', 'POINTE', 'Bloch', 'ETU',     NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '6.5', 'X',   NULL, NULL, NULL, NULL, NULL, NULL, NULL,  9, 0, 2, 'Estante P-Bloch 2'),
    ('STK_PT_BLOCH_ETU_70_XX',         'PT-BLO-ETU-7.0-XX',       '7790004004', 'POINTE', 'PUNTA_BLOCH', 'POINTE', 'Bloch', 'ETU',     NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '7',   'XX',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, 11, 0, 2, 'Estante P-Bloch 2'),

-- 5.E Puntas Sansha Duval (Modelos 4.0 Americanas: Durezas Flex, Regular y Strong en anchos X y W)
    ('STK_PT_DUVAL_40_8_X_FLEX',       'PT-DUV-4.0-8-X-FLEX',     '7790005001', 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8',   'X', NULL, NULL, NULL, NULL, NULL, NULL, 'FLEX',    8, 0, 2, 'Estante P-Duval 1'),
    ('STK_PT_DUVAL_40_85_W_REG',       'PT-DUV-4.0-8.5-W-REG',    '7790005002', 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '8.5', 'W', NULL, NULL, NULL, NULL, NULL, NULL, 'REGULAR',10, 0, 2, 'Estante P-Duval 1'),
    ('STK_PT_DUVAL_40_9_W_STR',        'PT-DUV-4.0-9-W-STR',      '7790005003', 'POINTE', 'PUNTA_SANSHA_DUVAL', 'POINTE', 'Sansha Duval', '4.0 Americanas', NULL, 'Rosa', 'SATEN', 'ORIGINAL', NULL, 'US', '9',   'W', NULL, NULL, NULL, NULL, NULL, NULL, 'STRONG',  7, 0, 2, 'Estante P-Duval 1'),

-- 5.F Indumentaria Técnica y Medias (Suspensores, Media Mallas, Cancán, Bombachas)
    ('STK_IND_SUSP_BLOCH_M',           'IND-SUS-BLO-NG-M',        '7790006001', 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Bloch',   'Dance Belt',  NULL, 'Negro',  'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'M',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 15, 0, 3, 'Gaveta Ind-1'),
    ('STK_IND_SUSP_CAPEZIO_L',         'IND-SUS-CAP-PL-L',        '7790006002', 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Capezio', 'Quilted Belt',NULL, 'Piel',   'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'L',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 12, 0, 3, 'Gaveta Ind-1'),
    ('STK_IND_SUSP_BLOCH_XL',          'IND-SUS-BLO-PL-XL',       '7790006003', 'APPAREL', 'SUSPENSOR_MASCULINO', 'SUPPORTER', 'Bloch',   'Dance Belt',  NULL, 'Piel',   'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'XL',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,  8, 0, 2, 'Gaveta Ind-1'),

    ('STK_IND_MMALLA_FOUETTE_PL_M',    'IND-MM-FOU-PL-M',         '7790006004', 'APPAREL', 'MEDIA_MALLA',         'TIGHTS',    'Fouette', 'Clásica',     NULL, 'Piel',   'MICROFIBRA',    'ORIGINAL', NULL, 'ALPHA', 'M',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 25, 0, 5, 'Gaveta Ind-2'),
    ('STK_IND_MMALLA_FOUETTE_NG_L',    'IND-MM-FOU-NG-L',         '7790006005', 'APPAREL', 'MEDIA_MALLA',         'TIGHTS',    'Fouette', 'Clásica',     NULL, 'Negro',  'MICROFIBRA',    'ORIGINAL', NULL, 'ALPHA', 'L',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 20, 0, 5, 'Gaveta Ind-2'),
    ('STK_IND_MMALLA_CAPEZIO_BL_S',    'IND-MM-CAP-BL-S',         '7790006006', 'APPAREL', 'MEDIA_MALLA',         'TIGHTS',    'Capezio', 'Ultra Soft',  NULL, 'Blanco', 'MICROFIBRA',    'ORIGINAL', NULL, 'ALPHA', 'S',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 18, 0, 4, 'Gaveta Ind-2'),

    ('STK_IND_CANCAN_SILVANA_A',       'IND-CAN-SIL-RS-A',        '7790006007', 'APPAREL', 'CANCAN',              'TIGHTS',    'Silvana', 'Ballet Talle A', NULL, 'Rosa','MICROFIBRA',    'ORIGINAL', NULL, 'ALPHA', 'A',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 30, 0, 6, 'Gaveta Ind-3'),
    ('STK_IND_CANCAN_HD_M',            'IND-CAN-HD-PL-M',         '7790006008', 'APPAREL', 'CANCAN',              'TIGHTS',    'HD',      'Convertible', NULL, 'Piel',   'MICROFIBRA',    'ORIGINAL', NULL, 'ALPHA', 'M',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 22, 0, 5, 'Gaveta Ind-3'),
    ('STK_IND_CANCAN_CAP_RED_PL_SM',   'IND-CAN-CAP-RED-PL-SM',   '7790006009', 'APPAREL', 'CANCAN',              'TIGHTS',    'Capezio', 'Red',         NULL, 'Piel',   'RED_FISHNET',   'ORIGINAL', NULL, 'ALPHA', 'S/M', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 16, 0, 4, 'Gaveta Ind-3'),
    ('STK_IND_CANCAN_CAP_RED_RS_XL',   'IND-CAN-CAP-RED-RS-XL',   '7790006010', 'APPAREL', 'CANCAN',              'TIGHTS',    'Capezio', 'Red',         NULL, 'Rosa',   'RED_FISHNET',   'ORIGINAL', NULL, 'ALPHA', 'XL',  NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 12, 0, 3, 'Gaveta Ind-3'),

    ('STK_IND_BOMB_FOUETTE_RJ_S',      'IND-BOM-FOU-RJ-S',        '7790006011', 'APPAREL', 'BOMBACHA_ESCENA',     'BRIEFS',    'Fouette', 'Escena',      NULL, 'Rojas',  'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'S',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 20, 0, 4, 'Gaveta Ind-4'),
    ('STK_IND_BOMB_FOUETTE_PL_M',      'IND-BOM-FOU-PL-M',        '7790006012', 'APPAREL', 'BOMBACHA_ESCENA',     'BRIEFS',    'Fouette', 'Escena',      NULL, 'Piel',   'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'M',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 24, 0, 4, 'Gaveta Ind-4'),
    ('STK_IND_BOMB_FOUETTE_PL_L',      'IND-BOM-FOU-PL-L',        '7790006013', 'APPAREL', 'BOMBACHA_ESCENA',     'BRIEFS',    'Fouette', 'Escena',      NULL, 'Piel',   'ALGODON_LYCRA', 'ORIGINAL', NULL, 'ALPHA', 'L',   NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 18, 0, 4, 'Gaveta Ind-4');

-- -----------------------------------------------------------------------------
-- 6. RESERVAS NOMINATIVAS ACTIVAS DE EJEMPLO ('Popi' y 'Brenda')
--    (El trigger trg_stock_reservation_after_insert incrementa reserved_qty automáticamente)
-- -----------------------------------------------------------------------------
INSERT INTO stock_reservations (
    id, stock_item_id, personnel_id, reserved_for_label, quantity, status, work_id, notes, created_by
) VALUES
    ('RES_INIT_POPI_01',   'STK_GM_EUR_SC_75W4PDH', 'PER_DANCER_POPI',   'Popi',   2, 'ACTIVE', 'WORK_LAGO_CISNES', 'Separadas en Gaveta GM-▲1 para Popi (El Lago de los Cisnes)', 'PER_INSPECTOR_MAIN'),
    ('RES_INIT_BRENDA_01', 'STK_GM_LYR_SK_85M4HDL', 'PER_DANCER_BRENDA', 'Brenda', 2, 'ACTIVE', 'WORK_LAGO_CISNES', 'Separadas en Gaveta GM-●1 para Brenda (Cisne Negro)',         'PER_INSPECTOR_MAIN');
