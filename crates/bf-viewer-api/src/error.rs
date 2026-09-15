use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    NotFoud,
    BadRequest,
    Busy,
    Internal(bf_viewer_core::Error),
}

impl From<bf_viewer_core::Error> for AppError {
    fn from(e: bf_viewer_core::Error) -> Self {
        AppError::Internal(e)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.into())
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        AppError::Internal(e.into())
    }
}

#[derive(Serialize)]
struct Problem {
    #[serde(rename = "type")]
    kind: &'static str,
    title: &'static str,
    status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title, detail) = match self {
            AppError::NotFoud => (StatusCode::NOT_FOUND, "Record not foud", None),
            AppError::BadRequest => (
                StatusCode::BAD_REQUEST,
                "Invalid query",
                Some("Provide either record_index or key, not both".into()),
            ),
            AppError::Busy => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Conversion capacity exhausted",
                None,
            ),
            AppError::Internal(e) => {
                tracing::error!(error = %e, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error",
                    None,
                )
            }
        };

        let body = Problem {
            kind: "about:blank",
            title,
            status: status.as_u16(),
            detail,
        };

        let mut response = (status, Json(body)).into_response();

        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );

        response
    }
}
