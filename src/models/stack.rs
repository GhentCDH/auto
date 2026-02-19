use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Stack entity - programming languages, frameworks, technologies
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Stack {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// DTO for creating a new stack
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateStack {
    pub name: String,
    pub notes: Option<String>,
}

/// DTO for updating a stack
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStack {
    pub name: Option<String>,
    pub notes: Option<String>,
}

/// Stack relation for application detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct StackRelation {
    pub id: String,
    pub name: String,
}

/// Stack with related applications
#[derive(Debug, Serialize, ToSchema)]
pub struct StackWithRelations {
    #[serde(flatten)]
    pub stack: Stack,
    pub applications: Vec<ApplicationStackRelation>,
}

/// Application relation for stack detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApplicationStackRelation {
    pub id: String,
    pub name: String,
    pub status: String,
}
