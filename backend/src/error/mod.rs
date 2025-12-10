use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// Structured API error response
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Application error types
#[derive(Debug)]
pub enum ApiError {
    // Client errors (4xx)
    BadRequest(String),
    NotFound(String),

    // Server errors (5xx)
    InternalError(String),
    DatabaseError(String),
    StorageError(String),
    ExternalApiError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ApiError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ApiError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            ApiError::ExternalApiError(msg) => write!(f, "External API error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl ApiError {
    /// Get the appropriate HTTP status code for this error
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::StorageError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ExternalApiError(_) => StatusCode::BAD_GATEWAY,
        }
    }

    /// Get the error type name
    fn error_type(&self) -> &str {
        match self {
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::InternalError(_) => "INTERNAL_ERROR",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::StorageError(_) => "STORAGE_ERROR",
            ApiError::ExternalApiError(_) => "EXTERNAL_API_ERROR",
        }
    }

    /// Get the error message
    fn message(&self) -> &str {
        match self {
            ApiError::BadRequest(msg)
            | ApiError::NotFound(msg)
            | ApiError::InternalError(msg)
            | ApiError::DatabaseError(msg)
            | ApiError::StorageError(msg)
            | ApiError::ExternalApiError(msg) => msg,
        }
    }

    /// Log the error at appropriate level
    fn log(&self) {
        match self {
            // Client errors - log as warnings
            ApiError::BadRequest(_) | ApiError::NotFound(_) => {
                eprintln!("[WARN] {}", self);
            }
            // Server errors - log as errors
            ApiError::InternalError(_)
            | ApiError::DatabaseError(_)
            | ApiError::StorageError(_)
            | ApiError::ExternalApiError(_) => {
                eprintln!("[ERROR] {}", self);
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Log the error before converting to response
        self.log();

        let status = self.status_code();
        let error_response = ErrorResponse {
            error: self.error_type().to_string(),
            message: self.message().to_string(),
            details: None,
        };

        (status, Json(error_response)).into_response()
    }
}

// Conversion from anyhow::Error for easier error handling
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        // Check if it's a known error type by message
        let err_msg = err.to_string();

        if err_msg.contains("not found") || err_msg.contains("Not found") {
            ApiError::NotFound(err_msg)
        } else if err_msg.contains("database") || err_msg.contains("SQL") {
            ApiError::DatabaseError(err_msg)
        } else {
            ApiError::InternalError(err_msg)
        }
    }
}

// Conversion from storage errors
impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => ApiError::NotFound(err.to_string()),
            _ => ApiError::StorageError(err.to_string()),
        }
    }
}
