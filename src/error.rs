use derive_more::Display;

/// All possible Errors returned inside functions of the vink crate
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
    #[error("AutoError: {0}")]
    AutoError(#[from] AutoError),
}

#[derive(Debug, thiserror::Error, Display)]
pub enum AutoError {
    Placeholder,
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            self.to_string().into_response(),
        )
            .into_response()
    }
}
