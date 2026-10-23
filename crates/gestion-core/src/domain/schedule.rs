use crate::domain::personnel::GenderGroup;
use serde::{Deserialize, Serialize};

/// Cláusula legal invariante impresa al pie de cada Parte Diario de Ensayo
pub const OFFICIAL_LEGAL_FOOTER_CLAUSE: &str =
    "REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.";

/// Horario obligatorio de fichaje / marcada diaria (Lunes a Viernes)
pub const MANDATORY_CHECKIN_TIME_STR: &str = "08:45";
/// Horario invariable de inicio de la Clase Técnica Matutina
pub const MORNING_CLASS_START_TIME_STR: &str = "09:00";
/// Horario invariable de finalización de la Clase Técnica Matutina
pub const MORNING_CLASS_END_TIME_STR: &str = "10:15";
/// Horario estándar de inicio del bloque de ensayos matutinos
pub const STANDARD_REHEARSAL_START_TIME_STR: &str = "10:30";
/// Tope de jornada ordinaria de 5 horas (08:45 a 13:45 h)
pub const ORDINARY_SHIFT_END_LIMIT_STR: &str = "13:45";
/// Tope de jornada en régimen Pre-Función (+1 hora máxima: salida hasta las 14:45 h)
pub const PRE_FUNCTION_SHIFT_END_LIMIT_STR: &str = "14:45";

/// Maestros titulares en rotación de Clases Matutinas
pub const ROTATION_MASTERS: [&str; 3] = ["Ananiev", "Benites", "Pino"];
/// Pianistas acompañantes titulares en rotación de Clases Matutinas
pub const ROTATION_PIANISTS: [&str; 3] = ["Romero", "Villegas", "Brusutti"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkRegime {
    /// Jornada ordinaria de 5 horas (08:45 a 13:45 h)
    Ordinary,
    /// Régimen Pre-Función con extensión habilitada de hasta +1h máxima (hasta 14:45 h)
    PreFunction,
    /// Régimen Extraordinario: ensayos en escenario (técnica, puesta, luces, orquesta) y funciones tarde/noche o fines de semana
    Extraordinary,
}

impl WorkRegime {
    pub const MANDATORY_CHECKIN_MINUTES: u16 = 8 * 60 + 45;     // 08:45 = 525 min
    pub const MORNING_CLASS_START_MINUTES: u16 = 9 * 60;        // 09:00 = 540 min
    pub const MORNING_CLASS_END_MINUTES: u16 = 10 * 60 + 15;    // 10:15 = 615 min
    pub const REHEARSAL_STD_START_MINUTES: u16 = 10 * 60 + 30;  // 10:30 = 630 min
    pub const ORDINARY_END_LIMIT_MINUTES: u16 = 13 * 60 + 45;   // 13:45 = 825 min
    pub const PRE_FUNCTION_END_LIMIT_MINUTES: u16 = 14 * 60 + 45; // 14:45 = 885 min

    pub fn max_daytime_end_minutes(&self) -> Option<u16> {
        match self {
            Self::Ordinary => Some(Self::ORDINARY_END_LIMIT_MINUTES),
            Self::PreFunction => Some(Self::PRE_FUNCTION_END_LIMIT_MINUTES),
            Self::Extraordinary => None,
        }
    }

    pub fn nominal_end_time_str(&self) -> &'static str {
        match self {
            Self::Ordinary => ORDINARY_SHIFT_END_LIMIT_STR,
            Self::PreFunction => PRE_FUNCTION_SHIFT_END_LIMIT_STR,
            Self::Extraordinary => "23:59",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomCode {
    #[serde(rename = "SALA_1")]
    Sala1,
    #[serde(rename = "SALA_2")]
    Sala2,
    #[serde(rename = "SALA_3")]
    Sala3,
    #[serde(rename = "ESCENARIO")]
    Escenario,
    #[serde(rename = "GIRA_TRASLADO")]
    GiraTraslado,
    #[serde(rename = "GIRA_ESCENARIO")]
    GiraEscenario,
    #[serde(rename = "GIRA_SALA")]
    GiraSala,
    #[serde(rename = "SALA_AUX")]
    SalaAux,
}

impl RoomCode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sala1 => "Sala 1",
            Self::Sala2 => "Sala 2",
            Self::Sala3 => "Sala 3",
            Self::Escenario => "Escenario Principal",
            Self::GiraTraslado => "Traslado / Salida de Gira",
            Self::GiraEscenario => "Escenario de Gira",
            Self::GiraSala => "Sala de Gira",
            Self::SalaAux => "Sala Auxiliar",
        }
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Sala1 => "SALA_1",
            Self::Sala2 => "SALA_2",
            Self::Sala3 => "SALA_3",
            Self::Escenario => "ESCENARIO",
            Self::GiraTraslado => "GIRA_TRASLADO",
            Self::GiraEscenario => "GIRA_ESCENARIO",
            Self::GiraSala => "GIRA_SALA",
            Self::SalaAux => "SALA_AUX",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityType {
    StudioRehearsal,
    StageTechRehearsal,
    StageLightingRehearsal,
    StageOrchestraRehearsal,
    DressRehearsal,
    Performance,
    TourTravel,
    TourStageRehearsal,
    ClassMorning,
    CompanyBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConfirmationState {
    Confirmed,
    RehearsalToConfirm, // "Ensayo a confirmar"
    CastToConfirm,      // "Reparto a confirmar"
}

impl ConfirmationState {
    pub fn display_badge(&self) -> Option<&'static str> {
        match self {
            Self::Confirmed => None,
            Self::RehearsalToConfirm => Some("Ensayo a confirmar"),
            Self::CastToConfirm => Some("Reparto a confirmar"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CastAssignmentCondition {
    Titular,
    AlternateSlash, // Pareja/elenco en barra ej. "Álvarez/Marcato"
    Reinforcement,  // Refuerzo
    Cover,
    ToConfirm,      // A confirmar
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MorningClassSlot {
    pub id: String,
    pub room: RoomCode,               // Sala 1 o Sala 3
    pub gender_division: GenderGroup, // Mujeres vs Varones
    pub start_time: String,           // Invariable "09:00"
    pub end_time: String,             // Invariable "10:15"
    pub master_id: String,            // Ananiev, Benites, Pino
    pub master_name: String,
    pub pianist_id: String,           // Romero, Villegas, Brusutti
    pub pianist_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastMemberEntry {
    pub id: String,
    pub personnel_id: String,
    pub display_name: String,
    pub character_role: Option<String>,
    pub partner_personnel_id: Option<String>,
    pub partner_display_name: Option<String>,
    pub condition: CastAssignmentCondition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RehearsalCastGroup {
    pub id: String,
    pub group_order: i32,
    pub group_label: Option<String>,
    pub formatted_line: String, // Ej: "Álvarez/Marcato - Morchio C/Gil"
    pub is_to_confirm: bool,
    pub members: Vec<CastMemberEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RehearsalBlock {
    pub id: String,
    pub schedule_id: String,
    pub room: RoomCode,
    pub start_time: String, // "HH:MM"
    pub end_time: String,   // "HH:MM"
    pub activity_type: ActivityType,
    pub work_id: Option<String>,
    pub work_title: Option<String>,
    pub passage_id: Option<String>,
    pub passage_name: String, // Ej: "Princesas y Bufón", "Danza Española", "Pas de deux Odette y Sigfrido", "Diana y Acteón", "Cisne Negro"
    pub confirmation_state: ConfirmationState,
    pub sound_requirement: String, // Columna 'SONIDO' para Cabina de TeatroCue
    pub teatrocue_cue_ref: Option<String>,
    pub master_id: Option<String>,
    pub master_name: Option<String>,
    pub pianist_id: Option<String>,
    pub pianist_name: Option<String>,
    pub allow_shared_room: bool,
    pub cast_groups: Vec<RehearsalCastGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyScheduleAggregate {
    pub id: String,
    pub schedule_date: String, // "YYYY-MM-DD"
    pub regime: WorkRegime,
    pub mandatory_checkin_time: String, // "08:45"
    pub nominal_end_time: String,       // "13:45" o "14:45"
    pub minimum_intershift_rest_hours: u8,
    pub class_to_rehearsal_rest_minutes: u16,
    pub is_compensatory_rest_day: bool,
    pub compensatory_rest_days: u8,
    pub compensatory_rest_policy: String,
    pub footer_legal_clause: String,    // OFFICIAL_LEGAL_FOOTER_CLAUSE
    pub morning_classes: Vec<MorningClassSlot>,
    pub rehearsal_blocks: Vec<RehearsalBlock>,
}

impl DailyScheduleAggregate {
    pub fn new_default(id: String, schedule_date: String, regime: WorkRegime) -> Self {
        Self {
            id,
            schedule_date,
            regime,
            mandatory_checkin_time: MANDATORY_CHECKIN_TIME_STR.to_string(),
            nominal_end_time: regime.nominal_end_time_str().to_string(),
            minimum_intershift_rest_hours: 12,
            class_to_rehearsal_rest_minutes: 15,
            is_compensatory_rest_day: false,
            compensatory_rest_days: 1,
            compensatory_rest_policy: "NEXT_DAY".to_string(),
            footer_legal_clause: OFFICIAL_LEGAL_FOOTER_CLAUSE.to_string(),
            morning_classes: Vec::new(),
            rehearsal_blocks: Vec::new(),
        }
    }
}
