use axum::{Router, routing::get};
use crate::response::{ApiError, ApiResponse, BasicMessage};
pub mod users;
pub mod test;

use users::{get_user, get_users};
use test::test;

pub fn router() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/users", get(get_users))
        .route("/user/{id}", get(get_user))
        .route("/test", get(test))
        .fallback(not_found)
}

async fn hello_world() -> Result<ApiResponse<BasicMessage>, ApiError> {
    Ok(ApiResponse::JsonData(BasicMessage {
        msg: "Hello, World!".to_string(),
    }))
}

async fn not_found() -> Result<ApiResponse<BasicMessage>, ApiError> {
    Ok(ApiResponse::JsonData(BasicMessage {
        msg: "Route not found".to_string(),
    }))
}