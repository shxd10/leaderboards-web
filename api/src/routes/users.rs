use crate::models::User;
use crate::response::{ApiError, ApiResponse, BasicMessage};
use axum::{extract::{Path, State, Json}, debug_handler};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
}

#[debug_handler]
pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<ApiResponse<User>, ApiError> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO user (username) VALUES (?) RETURNING id, username, created_at"
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(user))
}

pub async fn get_users(State(pool): State<SqlitePool>) -> Result<ApiResponse<Vec<User>>, ApiError> {
    let users: Vec<User> = sqlx::query_as::<_, User>("SELECT id, username, created_at FROM user")
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(users))
}

pub async fn get_user(Path(id): Path<i64>, State(pool): State<SqlitePool>) -> Result<ApiResponse<User>, ApiError> {
    let user: User = sqlx::query_as::<_, User>("SELECT id, username, created_at FROM user WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(user))
}