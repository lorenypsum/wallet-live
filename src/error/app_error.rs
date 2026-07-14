use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
//TODO: censurar erros relevantes para não vazar informações sensíveis, como erros de banco de dados
pub enum AppError {
    // Implementa display sozinho
    #[error("Missing authorization header.")]
    MissingAuthorization,
    #[error("Invalid credentials.")]
    InvalidCredentials,
    #[error("Asset not found.")]
    AssetNotFound,
    #[error("Asset already exists.")]
    AssetAlreadyExists,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("This username already exists.")]
    UsernameTaken,
    #[error("User does not exist.")]
    //TODO: chamar esse erro no lugar certo
    UserNotFound,
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    TemplateError(#[from] askama::Error),
    #[error(transparent)]
    JwtError(#[from] jwt_simple::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let error_response = ErrorResponse {
            error: self.to_string(),
        };

        let status = match self {
            AppError::MissingAuthorization => StatusCode::BAD_REQUEST,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::AssetNotFound | AppError::UserNotFound => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::AssetAlreadyExists => StatusCode::CONFLICT,
            AppError::Database(_) | AppError::TemplateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UsernameTaken => StatusCode::CONFLICT,
            AppError::JwtError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(error_response)).into_response()
    }
}
