use serde::{Deserialize, Serialize};

/// Estados operativos directos de la inspección diaria del Ballet Oficial
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttendanceStatus {
    /// "?" - Estado dudoso / a confirmar por la inspección
    Pending,
    /// "Está" - Presente en el teatro a horario (08:45 h)
    Present,
    /// "No está en clase" - Presente en el teatro pero no hace la barra/clase matutina
    NotInClass,
    /// "Está en la fisioterapia" - Presente en el teatro realizándose fisioterapia/kinesiología
    InPhysiotherapy,
    /// "Llegó tarde" - Arribo con demora posterior a las 08:45 h
    Late,
    /// "Tiene licencia médica" - Carpeta médica activa con certificado adjunto
    MedicalLeave,
    /// "No está" - Ausente de la jornada
    Absent,
    /// "Estado Jubilatorio" - En trámite previsional / eximido de concurrencia (no concurre)
    RetiredExempt,
    /// "Sin parte" - Sin convocar / no citado al parte diario
    NoPart,
    /// "Inasistencia justificada" (IJ) - Ausencia con justificación; tope de 2 por mes y 10 por año
    JustifiedAbsence,
    /// "Adecuación de tareas" - Trabaja en otra área (no hace clase ni ensayo) pero cumple jornada
    TaskAdjustment,
}

impl AttendanceStatus {
    pub fn as_operational_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Present => "PRESENT",
            Self::NotInClass => "NOT_IN_CLASS",
            Self::InPhysiotherapy => "IN_PHYSIOTHERAPY",
            Self::Late => "LATE",
            Self::MedicalLeave => "MEDICAL_LEAVE",
            Self::Absent => "ABSENT",
            Self::RetiredExempt => "RETIRED_EXEMPT",
            Self::NoPart => "NO_PART",
            Self::TaskAdjustment => "TASK_ADJUSTMENT",
            Self::JustifiedAbsence => "JUSTIFIED_ABSENCE",
        }
    }

    pub fn from_operational_str(s: &str) -> Self {
        match s {
            "PRESENT" | "ON_TIME" => Self::Present,
            "NOT_IN_CLASS" => Self::NotInClass,
            "IN_PHYSIOTHERAPY" => Self::InPhysiotherapy,
            "LATE" => Self::Late,
            "MEDICAL_LEAVE" => Self::MedicalLeave,
            "JUSTIFIED_ABSENCE" | "IJ" => Self::JustifiedAbsence,
            // Códigos de la planilla BOP: se leen igual desde la asistencia diaria
            "EI" | "ART" => Self::MedicalLeave,
            "SP" => Self::Absent,
            "LC" | "LA" | "ES" => Self::NoPart,
            "ABSENT" | "ABSENT_UNJUSTIFIED" => Self::Absent,
            "ABSENT_JUSTIFIED" => Self::JustifiedAbsence,
            "RETIRED_EXEMPT" | "RETIRED" => Self::RetiredExempt,
            "TASK_ADJUSTMENT" | "AT" | "ADECUACION" => Self::TaskAdjustment,
            "NO_PART" | "SIN_PARTE" | "WITHOUT_PART" | "NOT_SUMMONED" | "S/P" => Self::NoPart,
            _ => Self::Pending,
        }
    }

    pub fn to_db_attendance_status(&self) -> &'static str {
        match self {
            Self::Present | Self::NotInClass | Self::InPhysiotherapy | Self::RetiredExempt | Self::TaskAdjustment => "ON_TIME",
            Self::Late => "LATE",
            Self::MedicalLeave => "MEDICAL_LEAVE",
            Self::Absent => "ABSENT_UNJUSTIFIED",
            Self::Pending => "PENDING",
            Self::NoPart | Self::JustifiedAbsence => "ABSENT_JUSTIFIED",
        }
    }

    pub fn is_in_class(&self) -> bool {
        match self {
            Self::Present | Self::Late => true,
            Self::NotInClass | Self::InPhysiotherapy | Self::MedicalLeave | Self::Absent | Self::RetiredExempt | Self::NoPart | Self::TaskAdjustment | Self::JustifiedAbsence => false,
            Self::Pending => true,
        }
    }
}

/// Tipos de licencia registrables en la carpeta médica permanente
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaveType {
    /// Licencia por enfermedad (comportamiento histórico por defecto)
    Enfermedad,
    /// Licencia por estudio
    Estudio,
    /// Licencia sin goce de haberes
    SinGoceDeHaberes,
    /// Otras
    Otras,
}

impl LeaveType {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Enfermedad => "ENFERMEDAD",
            Self::Estudio => "ESTUDIO",
            Self::SinGoceDeHaberes => "SIN_GOCE_DE_HABERES",
            Self::Otras => "OTRAS",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "ESTUDIO" => Self::Estudio,
            "SIN_GOCE_DE_HABERES" => Self::SinGoceDeHaberes,
            "OTRAS" => Self::Otras,
            _ => Self::Enfermedad,
        }
    }

    pub fn label_es(&self) -> &'static str {
        match self {
            Self::Enfermedad => "Licencia por enfermedad",
            Self::Estudio => "Licencia por estudio",
            Self::SinGoceDeHaberes => "Licencia sin goce de haberes",
            Self::Otras => "Otras",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceLogDto {
    pub id: String,
    pub schedule_id: String,
    pub schedule_date: String,
    pub personnel_id: String,
    pub dancer_name: String,
    pub legajo: Option<String>,
    pub role: String,
    pub category_name: String,
    pub expected_checkin_time: String,
    pub actual_checkin_time: Option<String>,
    pub attendance_status: AttendanceStatus,
    pub is_in_class: bool,
    pub minutes_late: i32,
    pub remarks: Option<String>,
    pub certificate_file_name: Option<String>,
    pub certificate_data: Option<String>,
    pub medical_diagnosis: Option<String>,
    pub doctor_details: Option<String>,
    pub leave_start_date: Option<String>,
    pub leave_end_date: Option<String>,
    pub leave_type: Option<LeaveType>,
    pub is_retired: bool,
    pub retirement_notes: Option<String>,
    pub physio_notes: Option<String>,
    pub current_physio_diagnosis: Option<String>,
    pub physio_tracking_notes: Option<String>,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordAttendancePayload {
    pub schedule_date: String,
    pub personnel_id: String,
    pub attendance_status: AttendanceStatus,
    pub actual_checkin_time: Option<String>,
    pub minutes_late: Option<i32>,
    pub remarks: Option<String>,
    pub certificate_file_name: Option<String>,
    pub certificate_data: Option<String>,
    pub medical_diagnosis: Option<String>,
    pub doctor_details: Option<String>,
    pub leave_start_date: Option<String>,
    pub leave_end_date: Option<String>,
    pub leave_type: Option<LeaveType>,
    pub physio_notes: Option<String>,
    pub update_permanent_physio_tracking: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityRestriction {
    Normal,
    LightBarreOnly,
    NoJumps,
    NoPointe,
    TotalRest,
}

impl ActivityRestriction {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::LightBarreOnly => "LIGHT_BARRE_ONLY",
            Self::NoJumps => "NO_JUMPS",
            Self::NoPointe => "NO_POINTE",
            Self::TotalRest => "TOTAL_REST",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "LIGHT_BARRE_ONLY" => Self::LightBarreOnly,
            "NO_JUMPS" => Self::NoJumps,
            "NO_POINTE" => Self::NoPointe,
            "TOTAL_REST" => Self::TotalRest,
            _ => Self::Normal,
        }
    }

    pub fn label_es(&self) -> &'static str {
        match self {
            Self::Normal => "Apto sin restricciones",
            Self::LightBarreOnly => "Solo barra suave",
            Self::NoJumps => "Eximido/a de saltos",
            Self::NoPointe => "Eximida de puntas",
            Self::TotalRest => "Reposo deportivo absoluto",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiotherapySessionDto {
    pub id: String,
    pub personnel_id: String,
    pub dancer_name: String,
    pub session_date: String,
    pub session_time: String,
    pub doctor_name: String,
    pub diagnosis: Option<String>,
    pub treatment_applied: String,
    pub evolution_notes: Option<String>,
    pub activity_restriction: ActivityRestriction,
    pub attachment_name: Option<String>,
    pub attachment_data: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePhysioSessionPayload {
    pub personnel_id: String,
    pub session_date: String,
    pub session_time: Option<String>,
    pub doctor_name: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment_applied: String,
    pub evolution_notes: Option<String>,
    pub activity_restriction: ActivityRestriction,
    pub attachment_name: Option<String>,
    pub attachment_data: Option<String>,
    pub sync_attendance_status: bool,
}
