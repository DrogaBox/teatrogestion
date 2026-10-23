use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockFamily {
    MensFlat,
    WomensFlat,
    Pointe,
    Apparel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockSubcategory {
    MediaPuntaVaron,
    MediaPuntaMujerUnisex,
    PuntaGaynorMinden,
    PuntaBloch,
    PuntaSanshaDuval,
    SuspensorMasculino,
    MediaMalla,
    Cancan,
    BombachaEscena,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuotaBucket {
    Pointe,
    Flat,
    Tights,
    Supporter,
    Briefs,
    Exempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Brand {
    Sansha,
    Capezio,
    HD,
    Bloch,
    GaynorMinden,
    SanshaDuval,
    Fouette,
    Silvana,
}

impl Brand {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sansha => "Sansha",
            Self::Capezio => "Capezio",
            Self::HD => "HD",
            Self::Bloch => "Bloch",
            Self::GaynorMinden => "Gaynor Minden",
            Self::SanshaDuval => "Sansha Duval",
            Self::Fouette => "Fouette",
            Self::Silvana => "Silvana",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShoeMaterial {
    Cuero,
    TelaElastica,
    Canvas,
    Saten,
    SatenVeganLyra,
    AlgodonLycra,
    Microfibra,
    RedFishnet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PropDyeState {
    Original,
    /// Estado de utilería: 'Teñida'
    Tenida,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SizeSystem {
    Us,
    Eu,
    Alpha,
}

/// Líneas de manufactura Gaynor Minden con simbología oficial de depósito
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GaynorManufacturingLine {
    /// Europeas ▲
    EuropeanTriangle,
    /// Lyra ●
    LyraCircle,
    /// Americanas ★
    AmericanStar,
}

impl GaynorManufacturingLine {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::EuropeanTriangle => "▲",
            Self::LyraCircle => "●",
            Self::AmericanStar => "★",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            Self::EuropeanTriangle => "Europeas ▲",
            Self::LyraCircle => "Lyra ●",
            Self::AmericanStar => "Americanas ★",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GaynorFit {
    Sculpted, // SC
    Classic,  // CL
    Sleek,    // SK
}

impl GaynorFit {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Sculpted => "SC",
            Self::Classic => "CL",
            Self::Sleek => "SK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GaynorWidth {
    Narrow, // N
    Medium, // M
    Wide,   // W
}

impl GaynorWidth {
    pub fn code(&self) -> char {
        match self {
            Self::Narrow => 'N',
            Self::Medium => 'M',
            Self::Wide => 'W',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GaynorBox {
    Box2 = 2,
    Box3 = 3,
    Box4 = 4,
    Box5 = 5,
}

impl GaynorBox {
    pub fn number(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GaynorShank {
    Extraflex,  // X (Bolsa Amarilla)
    Supple,     // S (Bolsa Rosa)
    Feather,    // F (Bolsa Azul)
    Pianissimo, // P (Bolsa Lavanda - ej. SC - 7.5W4PDH)
    Hard,       // H (Bolsa Verde - ej. SK - 8.5M4HDL)
}

impl GaynorShank {
    pub fn code(&self) -> char {
        match self {
            Self::Extraflex => 'X',
            Self::Supple => 'S',
            Self::Feather => 'F',
            Self::Pianissimo => 'P',
            Self::Hard => 'H',
        }
    }

    pub fn bag_color_es(&self) -> &'static str {
        match self {
            Self::Extraflex => "Bolsa Amarilla (Extraflex)",
            Self::Supple => "Bolsa Rosa (Supple)",
            Self::Feather => "Bolsa Azul (Feather)",
            Self::Pianissimo => "Bolsa Lavanda (Pianissimo)",
            Self::Hard => "Bolsa Verde (Hard)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GaynorVamp {
    Low,              // L
    Deep,             // D
    DeepPlus6Mm,      // D +6mm
    DeepPlusHalfInch, // D +1/2"
}

impl GaynorVamp {
    pub fn base_code(&self) -> char {
        match self {
            Self::Low => 'L',
            Self::Deep | Self::DeepPlus6Mm | Self::DeepPlusHalfInch => 'D',
        }
    }

    pub fn special_extension_label(&self) -> Option<&'static str> {
        match self {
            Self::DeepPlus6Mm => Some("+6mm"),
            Self::DeepPlusHalfInch => Some("+1/2\""),
            Self::Low | Self::Deep => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GaynorHeel {
    Low,  // L
    High, // H
}

impl GaynorHeel {
    pub fn code(&self) -> char {
        match self {
            Self::Low => 'L',
            Self::High => 'H',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GaynorParsedSpec {
    pub raw_code: String,
    pub fit: GaynorFit,
    pub size: String,
    pub width: GaynorWidth,
    pub box_shape: GaynorBox,
    pub shank: GaynorShank,
    pub vamp: GaynorVamp,
    pub heel: GaynorHeel,
}

impl GaynorParsedSpec {
    /// Reconstruye el código canónico normalizado de fábrica (ej. `"SC - 7.5W4PDH"` o `"SC - 7.5W4PDH +6mm"`)
    pub fn canonical_code(&self) -> String {
        let base = format!(
            "{} - {}{}{}{}{}{}",
            self.fit.code(),
            self.size,
            self.width.code(),
            self.box_shape.number(),
            self.shank.code(),
            self.vamp.base_code(),
            self.heel.code()
        );
        match self.vamp.special_extension_label() {
            Some(ext) => format!("{base} {ext}"),
            None => base,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DuvalHardness {
    Flex,
    Regular,
    Strong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockItem {
    pub id: String,
    pub sku_code: String,
    pub barcode: Option<String>,
    pub family: StockFamily,
    pub subcategory: StockSubcategory,
    pub quota_bucket: QuotaBucket,
    pub brand: Brand,
    pub model_name: String,
    pub raw_factory_code: Option<String>,
    pub color: String,
    pub material: ShoeMaterial,
    pub prop_dye_state: PropDyeState,
    pub dyed_for_work_id: Option<String>,
    pub size_system: SizeSystem,
    pub size_value: String,
    pub width_value: Option<String>,
    pub gm_line: Option<GaynorManufacturingLine>,
    pub gm_spec: Option<GaynorParsedSpec>,
    pub duval_hardness: Option<DuvalHardness>,
    pub physical_qty: u32,
    pub reserved_qty: u32,
    pub min_alert_qty: u32,
    pub storage_bin_location: Option<String>,
}

impl StockItem {
    /// Calcula el stock disponible libre (`stock_fisico - stock_reservado`)
    pub fn free_available_qty(&self) -> u32 {
        self.physical_qty.saturating_sub(self.reserved_qty)
    }

    pub fn is_low_stock(&self) -> bool {
        self.free_available_qty() <= self.min_alert_qty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReservationStatus {
    Active,
    Delivered,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockReservation {
    pub id: String,
    pub stock_item_id: String,
    pub personnel_id: String,
    pub reserved_for_label: String, // Ej: "Popi", "Brenda"
    pub quantity: u32,
    pub status: ReservationStatus,
    pub work_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoucherDeliveryItem {
    pub id: String,
    pub voucher_id: String,
    pub stock_item_id: String,
    pub reservation_id: Option<String>,
    pub quantity: u32,
    pub quota_bucket: QuotaBucket,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoucherDelivery {
    pub id: String,
    pub voucher_number: String,
    pub personnel_id: String,
    pub season_year: i32,
    pub delivered_by_inspector_id: String,
    pub delivery_timestamp: String,
    pub is_quota_exception: bool,
    pub exception_reason: Option<String>,
    pub device_id: String,
    pub signature_svg_strokes: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<VoucherDeliveryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DancerQuotaStatus {
    pub personnel_id: String,
    pub full_name: String,
    pub nickname: Option<String>,
    pub category_name: String,
    pub season_year: i32,
    pub effective_pointe_quota: i32,
    pub consumed_pointe_qty: i32,
    pub effective_flat_quota: i32,
    pub consumed_flat_qty: i32,
    pub effective_tights_quota: i32,
    pub consumed_tights_qty: i32,
    pub effective_supporter_quota: i32,
    pub consumed_supporter_qty: i32,
    pub effective_briefs_quota: i32,
    pub consumed_briefs_qty: i32,
}

impl DancerQuotaStatus {
    pub fn remaining_pointe_qty(&self) -> i32 {
        self.effective_pointe_quota - self.consumed_pointe_qty
    }

    pub fn remaining_flat_qty(&self) -> i32 {
        self.effective_flat_quota - self.consumed_flat_qty
    }

    pub fn remaining_tights_qty(&self) -> i32 {
        self.effective_tights_quota - self.consumed_tights_qty
    }

    pub fn remaining_supporter_qty(&self) -> i32 {
        self.effective_supporter_quota - self.consumed_supporter_qty
    }

    pub fn remaining_briefs_qty(&self) -> i32 {
        self.effective_briefs_quota - self.consumed_briefs_qty
    }
}
