use std::io;
use axum::{http::StatusCode, response::{IntoResponse, Response}};

/// Custom error types for the wiki application
#[derive(Debug)]
pub enum WikiError {
    Io(io::Error),
    NotFound,
    InvalidPath,
    TemplateError(String),
    SearchError(String),
    NavigationError(String),
    RenderError(String),
}

impl From<io::Error> for WikiError {
    fn from(err: io::Error) -> Self {
        WikiError::Io(err)
    }
}

impl IntoResponse for WikiError {
    fn into_response(self) -> Response {
        match self {
            WikiError::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            WikiError::InvalidPath => (StatusCode::BAD_REQUEST, "Invalid path").into_response(),
            WikiError::Io(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("I/O error: {}", e),
            )
                .into_response(),
            WikiError::TemplateError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            )
                .into_response(),
            WikiError::SearchError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Search error: {}", e),
            )
                .into_response(),
            WikiError::NavigationError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Navigation error: {}", e),
            )
                .into_response(),
            WikiError::RenderError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Render error: {}", e),
            )
                .into_response(),
        }
    }
}
