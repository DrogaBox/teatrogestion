-- Migración 0012: Días de Franco Compensatorio y Bóveda Documental de Inspección (Carina Lion)
ALTER TABLE daily_schedules ADD COLUMN compensatory_rest_days INTEGER NOT NULL DEFAULT 1;

CREATE TABLE IF NOT EXISTS inspection_vault_documents (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    document_date TEXT NOT NULL,
    personnel_id TEXT REFERENCES personnel(id) ON DELETE SET NULL,
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL DEFAULT 0,
    file_data TEXT NOT NULL,
    amount_ars REAL,
    supplier_vendor TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_inspection_vault_category ON inspection_vault_documents(category);
CREATE INDEX IF NOT EXISTS idx_inspection_vault_date ON inspection_vault_documents(document_date);
CREATE INDEX IF NOT EXISTS idx_inspection_vault_personnel ON inspection_vault_documents(personnel_id);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

