use sqlx::FromRow;
use serde::Serialize;

// DB struct models

// exposed user
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserLoginData {
    pub id: i64,
    pub password_hash: String,
}