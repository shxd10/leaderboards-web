use sqlx::FromRow;
use serde::Serialize;

// DB struct models

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: String,
}
