use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::openrouter::OpenRouterError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("upstream: {0}")]
    Upstream(String),
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(serde::Serialize)]
struct ErrorBody<'a> {
    error: ErrorInner<'a>,
}

#[derive(serde::Serialize)]
struct ErrorInner<'a> {
    code: &'a str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::Upstream(msg) => (StatusCode::BAD_GATEWAY, "UPSTREAM", msg),
            AppError::Database(e) => {
                tracing::error!(error = ?e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL",
                    "database error".to_string(),
                )
            }
            AppError::Internal(msg) => {
                tracing::error!(message = %msg, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL",
                    "internal server error".to_string(),
                )
            }
        };

        (
            status,
            Json(ErrorBody {
                error: ErrorInner { code, message },
            }),
        )
            .into_response()
    }
}

impl From<OpenRouterError> for AppError {
    fn from(err: OpenRouterError) -> Self {
        tracing::warn!(error = ?err, "upstream provider error");
        AppError::Upstream(err.client_message().to_string())
    }
}
