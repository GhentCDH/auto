use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// All possible Errors returned inside functions of the auto crate
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("AxumError: {0}")]
    AxumError(#[from] axum::Error),
    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("SqlxError: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("SqlxMigrateError: {0}")]
    SqlxMigrateError(#[from] sqlx::migrate::MigrateError),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    Conflict(String),
}

/// Error response body for API endpoints
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            Error::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            Error::ValidationError(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg.clone()),
            Error::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            Error::SqlxError(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "not_found", "Resource not found".to_string())
            }
            Error::SqlxError(sqlx::Error::Database(db_err)) => {
                // Handle unique constraint violations
                if db_err.is_unique_violation() {
                    (StatusCode::CONFLICT, "conflict", "Resource already exists".to_string())
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "database_error", "Database error".to_string())
                }
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Internal server error".to_string()),
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message,
        };

        (status, Json(body)).into_response()
    }
}
