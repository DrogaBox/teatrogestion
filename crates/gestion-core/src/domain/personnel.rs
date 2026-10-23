use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersonnelRole {
    Dancer,
    Master,
    Pianist,
    Inspector,
    SoundTech,
}

impl PersonnelRole {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Dancer => "DANCER",
            Self::Master => "MASTER",
            Self::Pianist => "PIANIST",
            Self::Inspector => "INSPECTOR",
            Self::SoundTech => "SOUND_TECH",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "DANCER" => Some(Self::Dancer),
            "MASTER" => Some(Self::Master),
            "PIANIST" => Some(Self::Pianist),
            "INSPECTOR" => Some(Self::Inspector),
            "SOUND_TECH" => Some(Self::SoundTech),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GenderGroup {
    Female,
    Male,
    NonBinary,
    NotApplicable,
}

impl GenderGroup {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Female => "FEMALE",
            Self::Male => "MALE",
            Self::NonBinary => "NON_BINARY",
            Self::NotApplicable => "NOT_APPLICABLE",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "FEMALE" => Some(Self::Female),
            "MALE" => Some(Self::Male),
            "NON_BINARY" => Some(Self::NonBinary),
            "NOT_APPLICABLE" => Some(Self::NotApplicable),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonnelCategory {
    pub id: String,
    pub code: String,
    pub name: String,
    pub annual_pointe_quota: i32,
    pub annual_flat_quota: i32,
    pub annual_tights_quota: i32,
    pub annual_supporter_quota: i32,
    pub annual_briefs_quota: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Personnel {
    pub id: String,
    pub legajo: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub nickname: Option<String>, // Ej: Some("Popi"), Some("Brenda")
    pub role: PersonnelRole,
    pub gender_group: GenderGroup,
    pub category_id: String,
    pub phone_whatsapp: Option<String>,
    pub default_pointe_raw_code: Option<String>,
    pub default_flat_size: Option<String>,
    pub is_active: bool,
    pub notes: Option<String>,
}

impl Personnel {
    /// Formato oficial con apodo para búsqueda y depósito (ej. `Gómez, Sofía ("Popi")`)
    pub fn display_full_with_nickname(&self) -> String {
        match &self.nickname {
            Some(nick) if !nick.trim().is_empty() && nick.trim() != self.last_name => {
                format!("{}, {} (\"{}\")", self.last_name, self.first_name, nick.trim())
            }
            _ => format!("{}, {}", self.last_name, self.first_name),
        }
    }

    /// Nombre corto de citación para el Parte de Ensayo (ej. `Álvarez`, `Morchio C.`)
    pub fn stage_citation_name(&self) -> &str {
        &self.last_name
    }
}

/// Formatea una pareja doble de estudio según la notación oficial de Inspección
/// Ejemplo: `format_double_cast_pair(&["Álvarez", "Marcato"], &["Morchio C", "Gil"])`
/// produce `"Álvarez/Marcato - Morchio C/Gil"`.
pub fn format_double_cast_pair(first_role_names: &[&str], second_role_names: &[&str]) -> String {
    let left = first_role_names.join("/");
    let right = second_role_names.join("/");
    if right.is_empty() {
        left
    } else if left.is_empty() {
        right
    } else {
        format!("{left} - {right}")
    }
}
