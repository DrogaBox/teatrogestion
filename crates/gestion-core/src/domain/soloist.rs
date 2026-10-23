use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoloistCertificationItemDto {
    pub id: String,
    pub certification_id: String,
    pub personnel_id: Option<String>,
    pub dancer_name: String,
    pub work_role: String,
    pub performance_dates: String,
    pub performance_count: i32,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoloistCertificationDto {
    pub id: String,
    pub program_title: String,
    pub hall_name: String,
    pub document_date: String,
    pub recipient_name: String,
    pub recipient_title: String,
    pub intro_body: String,
    pub closing_text: String,
    pub status: String,
    pub notes: Option<String>,
    pub total_performances: i32,
    pub items: Vec<SoloistCertificationItemDto>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSoloistItemPayload {
    pub id: Option<String>,
    pub personnel_id: Option<String>,
    pub dancer_name: String,
    pub work_role: String,
    pub performance_dates: String,
    pub performance_count: i32,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSoloistCertificationPayload {
    pub id: Option<String>,
    pub program_title: String,
    pub hall_name: String,
    pub document_date: String,
    pub recipient_name: String,
    pub recipient_title: String,
    pub intro_body: String,
    pub closing_text: String,
    pub status: String,
    pub notes: Option<String>,
    pub items: Vec<SaveSoloistItemPayload>,
}
