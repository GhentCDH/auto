use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Note entity - documentation links, changelog, issues
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNote {
    #[serde(deserialize_with = "super::trim_str")]
    pub entity_type: String,
    #[serde(deserialize_with = "super::trim_str")]
    pub entity_id: String,
    #[serde(deserialize_with = "super::trim_str")]
    pub title: String,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub content: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub note_type: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub url: Option<String>,
    #[serde(default)]
    pub is_pinned: bool,
}

impl CreateNote {
    /// Fill omitted defaultable fields from configured defaults.
    pub fn apply_defaults(&mut self, d: &crate::config::NoteDefaults) {
        self.note_type.get_or_insert_with(|| d.note_type.clone());
    }
}

/// DTO for updating a note
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNote {
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub content: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub note_type: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub url: Option<String>,
    pub is_pinned: Option<bool>,
}
