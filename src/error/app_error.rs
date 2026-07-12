use axum::{Json, http::StatusCode, response::IntoResponse};
use thiserror::Error;
use serde::Serialize;

#[derive(Debug, Error)]
pub enum AppError {
    // Implementa display sozinho
    #[error("Missing authorization header.")]
    MissingAuthorization,
    #[error("Invalid credentials.")]
    InvalidCredentials,
    #[error("Asset not found.")]
    AssetNotFound,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("This username already exists.")]
    UsernameTaken,
    #[error(transparent)]
    TemplateError(#[from] askama::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let error_response =  ErrorResponse {
            error: self.to_string(),
        };

        let status = match self {
            AppError::MissingAuthorization => StatusCode::BAD_REQUEST,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::AssetNotFound => StatusCode::NOT_FOUND,
            AppError::Database(_) | AppError::TemplateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UsernameTaken => StatusCode::CONFLICT,
        };

        (status, Json(error_response)).into_response()
    }
}

