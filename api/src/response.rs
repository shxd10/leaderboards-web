use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;

#[derive(Serialize)]
pub struct BasicMessage {
    pub msg: String,
}

#[derive(Serialize)]
struct ErrorBody {
    code: u16,
    msg: String,
}

#[allow(dead_code)]
pub enum ApiResponse<T> {
    OK,
    Created,
    JsonData(T),
}

#[allow(dead_code)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    Forbidden(String),
    Unauthorized(String),
    InternalServerError(String),
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        match self {
            Self::OK => (StatusCode::OK).into_response(),
            Self::Created => (StatusCode::CREATED).into_response(),
            Self::JsonData(data) => (StatusCode::OK, Json(data)).into_response(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::BadRequest(s) => (StatusCode::BAD_REQUEST, s),
            ApiError::NotFound(s) => (StatusCode::NOT_FOUND, s),
            ApiError::Forbidden(s) => (StatusCode::FORBIDDEN, s),
            ApiError::Unauthorized(s) => (StatusCode::UNAUTHORIZED, s),
            ApiError::InternalServerError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s),
        };

        let body = ErrorBody {
            code: status.as_u16(),
            msg,
        };

        (status, Json(body)).into_response()
    }
}