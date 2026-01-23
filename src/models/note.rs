use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Note entity - documentation links, changelog, issues
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub content: Option<String>,
    pub note_type: String,
    pub url: Option<String>,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new note
#[derive(Debug, Deserialize)]
pub struct CreateNote {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub content: Option<String>,
    #[serde(default = "default_note_type")]
    pub note_type: String,
    pub url: Option<String>,
    #[serde(default)]
    pub is_pinned: bool,
}

/// DTO for updating a note
#[derive(Debug, Deserialize)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub content: Option<String>,
    pub note_type: Option<String>,
    pub url: Option<String>,
    pub is_pinned: Option<bool>,
}

fn default_note_type() -> String {
    "general".to_string()
}
