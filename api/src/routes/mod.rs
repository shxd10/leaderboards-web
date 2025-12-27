use axum::{Router, extract::State, routing::{get, post, delete}};
use crate::response::{ApiError, ApiResponse, BasicMessage};
use sqlx::SqlitePool;
pub mod users;
pub mod test;

use users::{get_user, get_users, create_user};
use test::test;

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(handler))
        .route("/login", post(create_user))
        .route("/users", get(get_users))
        .route("/user/{id}", get(get_user))
        .route("/test", get(test))
        .with_state(pool)
        .fallback(not_found)
}

async fn handler() -> Result<ApiResponse<BasicMessage>, ApiError> {
    Ok(ApiResponse::JsonData(BasicMessage {
        msg: "Hello World!".to_string(),
    }))
}

async fn not_found() -> Result<ApiResponse<BasicMessage>, ApiError> {
    Err(ApiError::NotFound("Route not found".to_string()))
}
