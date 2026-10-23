use serde::{Deserialize, Serialize};

/// Reloj Lógico Híbrido (HLC) para ordenamiento causal determinista en sincronización P2P LAN
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HybridLogicalTimestamp {
    pub wall_clock_ms: u64,
    pub logical_counter: u32,
    pub node_id: String,
}

impl HybridLogicalTimestamp {
    pub fn to_sortable_string(&self) -> String {
        format!(
            "{:016x}-{:08x}-{}",
            self.wall_clock_ms, self.logical_counter, self.node_id
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeatroDataManifest {
    pub format_version: u8,
    pub exported_at_iso: String,
    pub source_device_name: String,
    pub schema_version: u32,
    pub last_hlc: Option<String>,
    pub sqlite_blake3_checksum: String,
}
