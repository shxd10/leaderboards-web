use axum::{Router, routing::{get, post, delete, put, patch}};
use crate::response::{ApiError, ApiResponse, BasicMessage};
use sqlx::SqlitePool;
use serde::Serialize;
pub mod users;

use users::{get_user, get_users, create_user, patch_user, delete_user, verify_user};

#[derive(Serialize)]
struct RouteInfo {
    route: String,
    methods: String,
    function: String,
}

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/", get(endpoint_index))
                .route("/signup", post(create_user))
                .route("/login", post(verify_user))
                .route("/user/{id}", get(get_user).patch(patch_user).delete(delete_user))
                .route("/users", get(get_users))
                .with_state(pool)
                .fallback(not_found)
        )
}

async fn endpoint_index() -> Result<ApiResponse<Vec<RouteInfo>>, ApiError> {
    let routes = vec![
        RouteInfo { route: "/api".to_string(), methods: "GET, HEAD".to_string(), function: "endpoint index (this route)".to_string() },
        RouteInfo { route: "/api/user".to_string(), methods: "POST".to_string(), function: "create user".to_string() },
        RouteInfo { route: "/api/users".to_string(), methods: "GET, HEAD".to_string(), function: "list all users".to_string() },
        RouteInfo { route: "/api/user/{id}".to_string(), methods: "GET, HEAD".to_string(), function: "get user by id".to_string() },
    ];

    Ok(ApiResponse::JsonData(routes))
}

async fn not_found() -> Result<ApiResponse<BasicMessage>, ApiError> {
    Err(ApiError::NotFound("Route not found".to_string()))
}
